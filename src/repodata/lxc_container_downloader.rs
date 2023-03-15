use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest;
use slog_scope::info;
use std::{
    cmp::min,
    fs::{File, OpenOptions},
    io::Seek,
    io::SeekFrom,
    io::Write,
    path::{Path, PathBuf},
};
use url::Url;

pub struct LXCContainerDownloader {
    url: Url,
}

impl LXCContainerDownloader {
    pub fn of(url: Url) -> Self {
        Self { url }
    }

    pub async fn download(self, filepath: &PathBuf) -> Result<()> {
        let response = reqwest::get(self.url.as_str()).await?;

        let total_size = response
            .content_length()
            .ok_or_else(|| anyhow!("Failed to get content length from {}", &self.url))?;

        let progress_bar = ProgressBar::new(total_size);
        progress_bar.set_style(ProgressStyle::default_bar()
			.template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/green}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
			.progress_chars("#>-"));

        info!("Download LCX container file '{}' started.", &self.url);

        let mut file: File;
        let mut downloaded_bytes: u64 = 0;
        let mut stream = response.bytes_stream();

        info!("Check for LCX container file existence");

        if Path::new(filepath).exists() {
            info!("LCX container file exists. Resuming.");

            file = OpenOptions::new().read(true).append(true).open(filepath)?;
            let file_size = std::fs::metadata(filepath)?.len();
            file.seek(SeekFrom::Start(file_size))?;
            downloaded_bytes = file_size;
        } else {
            info!("LCX container file doesn't exists. Freshing.");

            file = File::create(filepath)?;
        }

        while let Some(item) = stream.next().await {
            let chunk = item?;
            file.write_all(&chunk)?;
            let new_downloaded_bytes = min(downloaded_bytes + (chunk.len() as u64), total_size);
            downloaded_bytes = new_downloaded_bytes;
            progress_bar.set_position(new_downloaded_bytes);
        }

        info!("Download LCX container file '{}' done.", &self.url);

        Ok(())
    }
}
