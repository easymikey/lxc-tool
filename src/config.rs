use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use url::Url;

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
    pub origin: Url,
    pub index_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageFilter {
    #[serde(default)]
    pub dist: Option<String>,
    #[serde(default)]
    pub release: Option<String>,
    #[serde(default)]
    pub arch: Option<String>,
    #[serde(rename = "type", default)]
    pub type_: Option<String>,
    #[serde(default)]
    pub post_process: Option<PathBuf>,
}

impl IntoIterator for ImageFilter {
    type Item = (String, String);
    type IntoIter = ImageFilterIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        ImageFilterIntoIterator {
            image_filter: self,
            index: 0,
        }
    }
}

pub struct ImageFilterIntoIterator {
    image_filter: ImageFilter,
    index: usize,
}

impl Iterator for ImageFilterIntoIterator {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        let image_filter_slice = [
            ("dist", &self.image_filter.dist),
            ("release", &self.image_filter.release),
            ("arch", &self.image_filter.arch),
            ("type", &self.image_filter.type_),
        ];

        let result = match image_filter_slice.get(self.index) {
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

pub type ImageFiles = Vec<String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repodata {
    // Directory where the images will be saved
    pub host_root_dir: PathBuf,
    // Url from which information about images will be received
    pub target_url: TargetUrl,
    // Filters based on which images will be selected
    pub image_filters: Vec<ImageFilter>,
    // Image files to be desired in host_root_dir
    pub image_files: ImageFiles,
    // Numbers of containers to backup
    pub number_of_container_to_backup: i16,
    // Timeout to the post_script or post process (maybe in the image metadata) that will run after the image is loaded
    pub patcher_timeout: Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub log_level: LogLevel,
    pub repodata: Repodata,
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
