use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest;
use slog_scope::info;
use std::{cmp::min, io::Write};
use tempfile::{Builder, NamedTempFile};
use url::Url;

pub async fn download_image(url: Url) -> Result<NamedTempFile> {
    let response = reqwest::get(url.as_str()).await?;
    let total_size = response
        .content_length()
        .ok_or_else(|| anyhow!("Failed to get content length from {}", &url))?;
    let progress_bar = ProgressBar::new(total_size);
    progress_bar.set_style(ProgressStyle::default_bar()
			.template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/green}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")?
			.progress_chars("#>-"));

    info!("Download LXC image file '{}' started.", &url);

    let mut tempfile = Builder::new().tempfile()?;
    let mut downloaded_bytes: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        tempfile.write_all(&chunk)?;
        let new_downloaded_bytes = min(downloaded_bytes + (chunk.len() as u64), total_size);
        downloaded_bytes = new_downloaded_bytes;
        progress_bar.set_position(new_downloaded_bytes);
    }

    info!("Download LXC image file '{}' done.", &url);

    Ok(tempfile)
}
