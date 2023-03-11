use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetUrl {
    pub origin: String,
    pub index_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerFilter {
    #[serde(default)]
    pub dist: Option<String>,
    #[serde(default)]
    pub release: Option<String>,
    #[serde(default)]
    pub arch: Option<String>,
    #[serde(rename = "type", default)]
    pub type_: Option<String>,
    #[serde(default)]
    pub post_process: Option<String>,
}

impl IntoIterator for ContainerFilter {
    type Item = (String, String);
    type IntoIter = ContainerFilterIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        ContainerFilterIntoIterator {
            container_filter: self,
            index: 0,
        }
    }
}

pub struct ContainerFilterIntoIterator {
    container_filter: ContainerFilter,
    index: usize,
}

impl Iterator for ContainerFilterIntoIterator {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        let container_filter_slice = [
            ("dist", &self.container_filter.dist),
            ("release", &self.container_filter.release),
            ("arch", &self.container_filter.arch),
            ("type", &self.container_filter.type_),
        ];

        let result = match container_filter_slice.get(self.index) {
            Some((key, value)) => match value {
                Some(value) => (key.to_string(), value.to_string()),
                None => return None,
            },
            _ => return None,
        };

        self.index += 1;
        Some(result)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostScript {
    pub path: String,
    pub timeout: Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoData {
    pub host_root_dir: String,
    pub target_url: TargetUrl,
    pub container_filters: Vec<ContainerFilter>,
    pub download_files: Vec<String>,
    pub number_of_container_to_backup: i16,
    pub post_script: PostScript,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
