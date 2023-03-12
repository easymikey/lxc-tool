use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::config::ContainerFilter;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LXCContainerMetadata {
    #[serde(default)]
    pub dist: Option<String>,
    #[serde(default)]
    pub release: Option<String>,
    #[serde(default)]
    pub arch: Option<String>,
    #[serde(rename = "type", default)]
    pub type_: Option<String>,
    #[serde(default)]
    pub post_process: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
}

impl LXCContainerMetadata {
    pub fn of_metadata(input: &str) -> Result<LXCContainerMetadata> {
        match input.split(";").collect::<Vec<_>>().as_slice() {
            &[dist, release, arch, type_, name, path] => Ok(LXCContainerMetadata {
                dist: Some(dist.trim().to_string()),
                release: Some(release.trim().to_string()),
                arch: Some(arch.trim().to_string()),
                type_: Some(type_.trim().to_string()),
                name: Some(name.trim().to_string()),
                path: Some({
                    let path = path.trim().to_string();

                    if path.ends_with("/") {
                        path
                    } else {
                        path + "/"
                    }
                }),
                post_process: None,
            }),
            // TODO: create error module
            _ => Err(anyhow!("Do not grab container info")),
        }
    }
}

impl LXCContainerMetadata {
    pub fn get(&self, s: &str) -> Option<String> {
        match s {
            "dist" => self.dist.clone(),
            "release" => self.release.clone(),
            "arch" => self.arch.clone(),
            "type" => self.type_.clone(),
            _ => None,
        }
    }
}

pub trait FilterBy {
    fn filter_by(
        &self,
        container_filter: Vec<ContainerFilter>,
    ) -> Result<Vec<LXCContainerMetadata>>;
}

impl FilterBy for Vec<LXCContainerMetadata> {
    fn filter_by(
        &self,
        container_filter: Vec<ContainerFilter>,
    ) -> Result<Vec<LXCContainerMetadata>> {
        let filtered_containers: Vec<_> = self
            .into_iter()
            .flat_map(|lxc_container_metadata| {
                container_filter
                    .iter()
                    .filter_map(|search_container| {
                        let is_match = search_container.clone().into_iter().all(|(key, value)| {
                            value == lxc_container_metadata.get(&key).unwrap_or_default()
                        });

                        if is_match {
                            Some(LXCContainerMetadata {
                                post_process: search_container.post_process.clone(),
                                ..lxc_container_metadata.clone()
                            })
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
            // TODO: create error module
            Err(anyhow!("Do not grab container info"))
        }
    }
}
