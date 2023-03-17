mod config;
mod repodata;

use crate::repodata::{
    lxc_image_download::download_image, lxc_image_metadata::FilterBy,
    lxc_image_metadata_collection::LXCImageMetadataCollection, lxc_image_patch::patch_image,
};

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use slog::{o, Drain};
use slog_scope::{error, info};
use std::{fs, os::unix::prelude::PermissionsExt};
use tempfile::Builder;
use url::Url;

const CONFIG_DEFAULT_PATH: &str = "/etc/lxc-tool.yaml";

/// Download
#[derive(Args)]
struct CmdDownloadImages;

impl CmdDownloadImages {
    async fn run(config: config::Config) -> Result<()> {
        info!("Download LXC images started.");

        let target_url_origin = Url::parse(&config.repodata.target_url.origin)?;
        let meta_data_url = target_url_origin.join(&config.repodata.target_url.index_uri)?;

        let lxc_image_metadata_collection = LXCImageMetadataCollection::of(&meta_data_url)
            .get()
            .await?
            .filter_by(config.repodata.image_filters)?;

        for (lxc_container_metadata, post_process) in lxc_image_metadata_collection {
            let out_tempdir_path = Builder::new().prefix(".repodata_").tempdir()?;
            let path = lxc_container_metadata.path.trim_start_matches("/");
            let out_dir_path = &config.repodata.host_root_dir.join(&path);

            if out_dir_path.exists() {
                continue;
            }

            fs::create_dir_all(&out_dir_path)?;

            for image_file in &config.repodata.image_files {
                let download_url = target_url_origin.join(&path)?.join(&image_file)?;
                let tempfile = download_image(download_url).await?;

                let is_filename_rootfs = image_file == "rootfs.tar.xz";

                if is_filename_rootfs {
                    if let Some(post_process) = &post_process {
                        patch_image(
                            post_process,
                            &tempfile,
                            config.repodata.patcher_timeout,
                            &lxc_container_metadata,
                        )?;
                    }

                    if let Some(post_script) = &config.repodata.post_script_path {
                        patch_image(
                            post_script,
                            &tempfile,
                            config.repodata.patcher_timeout,
                            &lxc_container_metadata,
                        )?;
                    }
                }

                fs::rename(&tempfile, &out_tempdir_path.path().join(image_file))?;
                tempfile.as_file().metadata()?.permissions().set_mode(0o644);
            }

            fs::rename(&out_tempdir_path, &out_dir_path)?;
            out_dir_path.metadata()?.permissions().set_mode(0o755);
        }

        info!("Download LXС images done.");

        Ok(())
    }
}

#[derive(Subcommand)]
enum CommandLine {
    /// Dump parsed config file. Helps to find typos
    DumpConfig,
    /// Download LXC containers
    DownloadImages,
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
            CommandLine::DownloadImages => CmdDownloadImages::run(config).await,
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
