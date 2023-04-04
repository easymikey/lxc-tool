use crate::config::ImageFilter;

use anyhow::{bail, Result};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LXCImageMetadata {
    pub dist: String,
    pub release: String,
    pub arch: String,
    pub type_: String,
    pub name: String,
    pub path: PathBuf,
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
                path: PathBuf::from(
                    path.trim()
                        .trim_start_matches("/")
                        .trim_end_matches("/")
                        .to_string()
                        + "/",
                ),
            }),
            _ => bail!("Create LXC image metadata. Receive data error."),
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
            bail!("Filter LXC images failed. Filter images error. Images is equal 0.")
        }
    }
}
