use anyhow::Result;
use reqwest::Url;

use crate::repodata::lxc_container_metadata::LXCContainerMetadata;

pub struct LXCContainerMetadataCollection {
    url: Url,
}

impl LXCContainerMetadataCollection {
    pub fn of(url: &Url) -> Self {
        Self { url: url.clone() }
    }

    pub async fn get(self) -> Result<Vec<LXCContainerMetadata>> {
        Ok(reqwest::get(self.url)
            .await?
            .text()
            .await?
            .lines()
            .filter_map(|line| Some(LXCContainerMetadata::of_metadata(line)))
            .collect::<Result<Vec<_>>>()?)
    }
}
