use crate::config::ImageFilter;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LXCImageMetadata {
    #[serde(default)]
    pub dist: String,
    #[serde(default)]
    pub release: String,
    #[serde(default)]
    pub arch: String,
    #[serde(rename = "type", default)]
    pub type_: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub path: String,
}

impl LXCImageMetadata {
    pub fn of_metadata(input: &str) -> Result<LXCImageMetadata> {
        match input.split(";").collect::<Vec<_>>().as_slice() {
            &[dist, release, arch, type_, name, path] => Ok(LXCImageMetadata {
                dist: dist.trim().to_string(),
                release: release.trim().to_string(),
                arch: arch.trim().to_string(),
                type_: type_.trim().to_string(),
                name: name.trim().to_string(),
                path: {
                    let path = path.trim().to_string();

                    if path.ends_with("/") {
                        path
                    } else {
                        path + "/"
                    }
                },
            }),
            _ => bail!("Do not grab LXC container info"),
        }
    }
}

impl LXCImageMetadata {
    pub fn get(&self, idx: &str) -> Option<String> {
        match idx {
            "dist" => Some(self.dist.clone()),
            "release" => Some(self.release.clone()),
            "arch" => Some(self.arch.clone()),
            "type" => Some(self.type_.clone()),
            _ => None,
        }
    }
}

pub trait FilterBy {
    fn filter_by(
        &self,
        image_filters: Vec<ImageFilter>,
    ) -> Result<Vec<(LXCImageMetadata, Option<PathBuf>)>>;
}

impl FilterBy for Vec<LXCImageMetadata> {
    fn filter_by(
        &self,
        image_filters: Vec<ImageFilter>,
    ) -> Result<Vec<(LXCImageMetadata, Option<PathBuf>)>> {
        let filtered_containers: Vec<_> = self
            .into_iter()
            .flat_map(|lxc_container_metadata| {
                image_filters
                    .iter()
                    .filter_map(|image_filter| {
                        let is_match = image_filter.clone().into_iter().all(|(key, value)| {
                            value == lxc_container_metadata.get(&key).unwrap_or_default()
                        });

                        if is_match {
                            Some((
                                lxc_container_metadata.clone(),
                                image_filter.post_process.clone(),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        if filtered_containers.len() != 0 {
            Ok(filtered_containers)
        } else {
            bail!("Do not LXC grab container info")
        }
    }
}
