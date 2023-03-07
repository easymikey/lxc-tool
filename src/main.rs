use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use config::{RepoData, TargetUrl};
use slog::{o, Drain};
use slog_scope::{error, info};
use std::fs::read_to_string;
use tempfile::Builder;
use url::Url;

use crate::repodata::{
    container_info_download::ContainerInfoDownload, lxc_container::FilterBy,
    lxc_container::LXCContainers,
};

mod config;
mod repodata;

const CONFIG_DEFAULT_PATH: &str = "/etc/lxc-tool.yaml";

/// Download
#[derive(Args)]
struct CmdDownloadContainers;

impl CmdDownloadContainers {
    fn run(config: config::Config) -> Result<()> {
        info!("Download containers started.");

        let RepoData {
            target_url: TargetUrl { origin, index_uri },
            container_filters,
            ..
        } = config.repodata;
        let url = Url::parse(&origin)?.join(&index_uri)?;
        let tempfile_name = url.as_str().split("/").last().unwrap_or("tempfile");
        let tempdir = Builder::new().prefix(".repodata_").tempdir()?;
        let tempfile = tempdir.path().join(tempfile_name).into_os_string();
        let tempfile_path = ContainerInfoDownload::new(url).download_to(tempfile)?;
        let tempfile_as_string = read_to_string(tempfile_path)?;

        info!("Create LXC containers data started.");
        let lxc_containers =
            LXCContainers::build_collection(tempfile_as_string)?.filter_by(container_filters)?;

        info!("Create LXC containers data done.");

        println!("{:#?}", lxc_containers);
        println!("Number of containers is {}", lxc_containers.len());

        info!("Download containers done.");

        Ok(())
    }
}

#[derive(Subcommand)]
enum CommandLine {
    /// Dump parsed config file. Helps to find typos
    DumpConfig,
    /// Download LXC containers
    DownloadContainers,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Application {
    /// Path to configuration file
    #[clap(short, default_value = CONFIG_DEFAULT_PATH)]
    config_path: String,
    /// Subcommand
    #[clap(subcommand)]
    command: CommandLine,
}

impl Application {
    fn init_syslog_logger(log_level: slog::Level) -> Result<slog_scope::GlobalLoggerGuard> {
        let logger = slog_syslog::SyslogBuilder::new()
            .facility(slog_syslog::Facility::LOG_USER)
            .level(log_level)
            .unix("/dev/log")
            .start()?;

        let logger = slog::Logger::root(logger.fuse(), o!());
        Ok(slog_scope::set_global_logger(logger))
    }

    fn init_env_logger() -> Result<slog_scope::GlobalLoggerGuard> {
        Ok(slog_envlogger::init()?)
    }

    fn init_logger(&self, config: &config::Config) -> Result<slog_scope::GlobalLoggerGuard> {
        if std::env::var("RUST_LOGRU").is_ok() {
            Self::init_env_logger()
        } else {
            Self::init_syslog_logger(config.log_level.into())
        }
    }

    fn run_command(&self, config: config::Config) -> Result<()> {
        match &self.command {
            CommandLine::DumpConfig => {
                let config =
                    serde_yaml::to_string(&config).with_context(|| "Failed to dump config")?;
                println!("{}", config);
                Ok(())
            }
            CommandLine::DownloadContainers => CmdDownloadContainers::run(config),
        }
    }

    pub fn run(&self) {
        let config = config::Config::read(&self.config_path).expect("Config");
        let _logger_guard = self.init_logger(&config).expect("Logger");

        if let Err(err) = self.run_command(config) {
            error!("Failed with error: {:#}", err);
        }
    }
}
fn main() {
    Application::parse().run();
}
