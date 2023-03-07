use anyhow::Result;
use reqwest::Url;
use slog_scope::info;
use std::{ffi::OsString, fs};

pub struct ContainerInfoDownload {
    url: Url,
}

impl ContainerInfoDownload {
    pub fn new(url: Url) -> Self {
        Self { url }
    }

    pub fn download_to(self, dest_path: OsString) -> Result<OsString> {
        let url = &self.url.as_str();
        info!("Download from {} started.", url);
        let contents = reqwest::blocking::get(url.to_owned())?.bytes()?;
        fs::write(&dest_path, contents)?;
        info!("Downloading from {} done.", url);
        Ok(dest_path)
    }
}
