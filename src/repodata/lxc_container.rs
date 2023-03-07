use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

const CONTAINER_PARTS_COUNT: u8 = 6;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LXCContainer {
    dist: String,
    release: String,
    arch: String,
    type_: String,
    name: String,
    path: String,
}

impl LXCContainer {
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
                    }),
                    // TODO: create error module
                    _ => Err(anyhow!("Do not grab container info")),
                };

                Some(container)
            })
            .collect()
    }
}

impl IntoIterator for LXCContainer {
    type Item = String;
    type IntoIter = std::array::IntoIter<String, 6>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIterator::into_iter([
            self.dist,
            self.release,
            self.arch,
            self.type_,
            self.name,
            self.path,
        ])
    }
}
