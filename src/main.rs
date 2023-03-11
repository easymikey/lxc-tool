use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use slog::{o, Drain};
use slog_scope::{error, info};
use url::Url;

use crate::repodata::{
    container_info_download::LXCContainerMetadataCollection, lxc_container_metadata::FilterBy,
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

        let url = Url::parse(&config.repodata.target_url.origin)?
            .join(&config.repodata.target_url.index_uri)?;

        info!("Download LXC container metadata from {} started.", &url);

        let lxc_container_metadata_collection = LXCContainerMetadataCollection::of(url.clone())
            .get()?
            .filter_by(config.repodata.container_filters)?;

        info!("Download LXC container metadata from {} done.", &url);

        println!("{:#?}", lxc_container_metadata_collection);
        println!(
            "Number of containers is {}",
            lxc_container_metadata_collection.len()
        );

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
