use crate::repodata::lxc_image_metadata::LXCImageMetadata;

use anyhow::Result;
use reqwest::Url;
use slog_scope::info;

pub struct LXCImageMetadataCollection {
    url: Url,
}

impl LXCImageMetadataCollection {
    pub fn of(url: &Url) -> Self {
        Self { url: url.clone() }
    }

    pub async fn get(self) -> Result<Vec<LXCImageMetadata>> {
        info!("Download LXC images metadata from '{}' started.", &self.url);

        let r = reqwest::get(self.url.clone())
            .await?
            .text()
            .await?
            .lines()
            .filter_map(|line| Some(LXCImageMetadata::of_metadata(line)))
            .collect::<Result<Vec<_>>>()?;

        info!("Download LXC images metadata from '{}' done.", self.url);

        Ok(r)
    }
}
