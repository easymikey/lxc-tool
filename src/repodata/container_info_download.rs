use anyhow::Result;
use reqwest::Url;

use crate::repodata::lxc_container_metadata::LXCContainerMetadata;

pub struct LXCContainerMetadataCollection {
    url: Url,
}

impl LXCContainerMetadataCollection {
    pub fn of(url: Url) -> Self {
        Self { url }
    }

    pub fn get(self) -> Result<Vec<LXCContainerMetadata>> {
        Ok(reqwest::blocking::get(self.url)?
            .text()?
            .lines()
            .filter_map(|line| Some(LXCContainerMetadata::of_metadata(line)))
            .collect::<Result<Vec<_>>>()?)
    }
}
