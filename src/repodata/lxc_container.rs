use anyhow::{anyhow, Result};

use crate::config::LXCContainer;

const CONTAINER_PARTS_COUNT: u8 = 6;

pub struct LXCContainers;

impl LXCContainers {
    pub fn build_collection(file_as_string: String) -> Result<Vec<LXCContainer>> {
        file_as_string
            .lines()
            .filter_map(|line| {
                let container_info: Vec<_> = line.split(";").collect();
                let container = match container_info.len() as u8 {
                    CONTAINER_PARTS_COUNT => Ok(LXCContainer {
                        dist: container_info[0].trim().to_string(),
                        release: container_info[1].trim().to_string(),
                        arch: container_info[2].trim().to_string(),
                        type_: container_info[3].trim().to_string(),
                        name: container_info[4].trim().to_string(),
                        path: {
                            let path = container_info[5].trim().to_string();

                            if path.ends_with("/") {
                                path
                            } else {
                                path + "/"
                            }
                        },
                        post_process: "".to_string(),
                    }),
                    // TODO: create error module
                    _ => Err(anyhow!("Do not grab container info")),
                };

                Some(container)
            })
            .collect()
    }
}

pub trait FilterBy {
    fn filter_by(&self, container_filter: Vec<LXCContainer>) -> Result<Vec<LXCContainer>>;
}

impl FilterBy for Vec<LXCContainer> {
    fn filter_by(&self, container_filter: Vec<LXCContainer>) -> Result<Vec<LXCContainer>> {
        let filtered_containers: Vec<_> = self
            .into_iter()
            .flat_map(|lxc_container| {
                container_filter
                    .iter()
                    .filter_map(|search_container| {
                        let search_container = search_container.to_owned();
                        let lxc_container = lxc_container.to_owned();

                        if search_container == lxc_container {
                            Some(LXCContainer {
                                post_process: search_container.post_process,
                                ..lxc_container
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

impl PartialEq for LXCContainer {
    fn eq(&self, other: &Self) -> bool {
        self.dist == other.dist
            && self.release == other.release
            && self.type_ == other.type_
            && self.arch == other.arch
    }
}
impl Eq for LXCContainer {}
