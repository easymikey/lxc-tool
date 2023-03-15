use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use slog::{o, Drain};
use slog_scope::{error, info};
use std::{fs, os::unix::prelude::PermissionsExt, path::Path};
use tempfile::Builder;
use url::Url;

use crate::repodata::{
    lxc_container_downloader::LXCContainerDownloader, lxc_container_metadata::FilterBy,
    lxc_container_metadata_collection::LXCContainerMetadataCollection,
    lxc_container_patcher::LCXContainerPatcher,
};

mod config;
mod repodata;

const CONFIG_DEFAULT_PATH: &str = "/etc/lxc-tool.yaml";

/// Download
#[derive(Args)]
struct CmdDownloadContainers;

impl CmdDownloadContainers {
    async fn run(config: config::Config) -> Result<()> {
        info!("Download LXC containers started.");

        let target_url_origin = Url::parse(&config.repodata.target_url.origin)?;
        let meta_data_url = target_url_origin.join(&config.repodata.target_url.index_uri)?;

        info!(
            "Download LXC container metadata from '{}' started.",
            &meta_data_url
        );

        let lxc_container_metadata_collection = LXCContainerMetadataCollection::of(&meta_data_url)
            .get()
            .await?
            .filter_by(config.repodata.container_filters)?;

        info!(
            "Download LXC container metadata from '{}' done.",
            &meta_data_url
        );

        for lxc_container_metadata in lxc_container_metadata_collection {
            let out_tempdir_path = Builder::new().prefix(".repodata_").tempdir()?;
            let path = lxc_container_metadata.path.unwrap_or_default();
            let path = if path.starts_with("/") {
                &path[1..path.len() - 1]
            } else {
                &path
            };
            let out_dir_path = Path::new(&config.repodata.host_root_dir.to_owned()).join(&path);

            if out_dir_path.exists() {
                info!("LCX container directory exists. Resuming.");
                continue;
            }

            info!("LCX container director doesn't exists. Skipping.");

            fs::create_dir_all(&out_dir_path)?;

            for filename in &config.repodata.download_files {
                let download_url = target_url_origin.join(&path)?.join(&filename)?;
                let tempfile = &out_tempdir_path.path().join(&filename);

                LXCContainerDownloader::of(download_url)
                    .download(&tempfile)
                    .await?;

                let is_filename_rootfs = filename == "rootfs.tar.xz";

                if lxc_container_metadata.post_process.is_some() && is_filename_rootfs {
                    info!("Post process start execution. ");

                    if LCXContainerPatcher::of(
                        &lxc_container_metadata.post_process,
                        tempfile,
                        config.repodata.post_script.timeout,
                    )
                    .patch([
                        &lxc_container_metadata.dist,
                        &lxc_container_metadata.release,
                        &lxc_container_metadata.arch,
                    ])? != Some(0)
                    {
                        info!("Post process failed to execution.");
                    }

                    info!("Post process executed.");
                }

                if config.repodata.post_script.path.is_some() && is_filename_rootfs {
                    info!("Post process start execution.");

                    if LCXContainerPatcher::of(
                        &config.repodata.post_script.path,
                        tempfile,
                        config.repodata.post_script.timeout,
                    )
                    .patch([
                        &lxc_container_metadata.dist,
                        &lxc_container_metadata.release,
                        &lxc_container_metadata.arch,
                    ])? != Some(0)
                    {
                        info!("Post script failed to execution.");
                    }

                    info!("Post script executed.");
                }

                tempfile.metadata()?.permissions().set_mode(0o644);
            }

            fs::rename(&out_tempdir_path, &out_dir_path)?;
            out_dir_path.metadata()?.permissions().set_mode(0o755);
        }

        info!("Download LCX containers done.");

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

    async fn run_command(&self, config: config::Config) -> Result<()> {
        match &self.command {
            CommandLine::DumpConfig => {
                let config =
                    serde_yaml::to_string(&config).with_context(|| "Failed to dump config")?;
                println!("{}", config);
                Ok(())
            }
            CommandLine::DownloadContainers => Ok(CmdDownloadContainers::run(config).await?),
        }
    }

    pub async fn run(&self) {
        let config = config::Config::read(&self.config_path).expect("Config");
        let _logger_guard = self.init_logger(&config).expect("Logger");

        if let Err(err) = self.run_command(config).await {
            error!("Failed with error: {:#}", err);
        }
    }
}
#[tokio::main]
async fn main() {
    Application::parse().run().await;
}
