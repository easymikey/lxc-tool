use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest;
use slog_scope::info;
use std::fs::File;
use std::{cmp::min, io::Write};
use url::Url;

pub struct LXCContainerDownload {
    url: Url,
}

impl LXCContainerDownload {
    pub fn of(url: Url) -> Self {
        Self { url }
    }
    pub async fn download(self, path: &str) -> Result<()> {
        let res = reqwest::get(self.url.clone()).await?;

        let total_size = res
            .content_length()
            .ok_or(anyhow!("Failed to get content length from {}", &self.url))?;

        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
			.template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
			.progress_chars("#>-"));

        info!("Download LXC container from {} started.", self.url);

        // download chunks
        let mut file: File = File::create(path)?;
        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item?;
            file.write_all(&chunk)?;
            let new = min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            pb.set_position(new);
        }

        info!("Download LXC container from {} done.", self.url);

        Ok(())
    }
}
