use anyhow::Result;
use reqwest::Url;
use std::{ffi::OsString, fs};

pub struct ContainerInfoDownload {
    url: Url,
}

impl ContainerInfoDownload {
    pub fn new(url: Url) -> Self {
        Self { url }
    }

    pub fn download_to(self, dest_path: OsString) -> Result<OsString> {
        let contents = reqwest::blocking::get(self.url)?.bytes()?;
        fs::write(&dest_path, contents)?;
        Ok(dest_path)
    }
}
