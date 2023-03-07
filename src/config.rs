use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum LogLevel {
    Critical,
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for slog::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Critical => slog::Level::Critical,
            LogLevel::Error => slog::Level::Error,
            LogLevel::Warning => slog::Level::Warning,
            LogLevel::Info => slog::Level::Info,
            LogLevel::Debug => slog::Level::Debug,
            LogLevel::Trace => slog::Level::Trace,
        }
    }
}

pub type Timeout = u64;

#[derive(Serialize, Deserialize)]
pub struct TargetUrl {
    pub origin: String,
    pub index_uri: String,
}

#[derive(Serialize, Deserialize)]
pub struct LXCContainer {
    pub dist: String,
    pub release: String,
    pub arch: String,
    #[serde(rename = "type", default)]
    pub type_: String,
    #[serde(default)]
    pub post_process: String,
}

#[derive(Serialize, Deserialize)]
pub struct PostScript {
    pub path: String,
    pub timeout: Timeout,
}

#[derive(Serialize, Deserialize)]
pub struct RepoData {
    pub host_root_dir: String,
    pub target_url: TargetUrl,
    pub container_filter: Vec<LXCContainer>,
    pub download_files: Vec<String>,
    pub number_of_container_to_backup: i16,
    pub post_script: PostScript,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub log_level: LogLevel,
    pub repodata: RepoData,
}

impl Config {
    fn validate(&self) -> Result<()> {
        Ok(())
    }

    pub fn read(file: &str) -> Result<Self> {
        let config = std::fs::read_to_string(file)
            .with_context(|| format!("Failed to load config file {:?}", file))?;
        let config: Self = serde_yaml::from_str(&config)
            .with_context(|| format!("Failed to parse config file {:?}", file))?;

        config.validate()?;
        Ok(config)
    }
}
