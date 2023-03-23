use super::lxc_image_metadata::LXCImageMetadata;

use anyhow::{bail, Result};
use regex::Regex;
use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};
use walkdir::WalkDir;

pub fn create_entries(root_dir: &PathBuf) -> Result<Vec<(LXCImageMetadata, Duration)>> {
    let re = Regex::new(
        r"/images/(?P<dist>.+)/(?P<release>.+)/(?P<arch>.+)/(?P<type>.+)/(?P<name>\d\d\d\d\d\d\d\d_\d\d:\d\d)",
    )?;
    let entry: Vec<_> = WalkDir::new(root_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let path = e.path().to_path_buf();
            let path_as_str = path.to_str()?;
            let caps = re.captures(path_as_str)?;
            let mtime = e
                .metadata()
                .ok()?
                .modified()
                .ok()?
                .duration_since(SystemTime::UNIX_EPOCH)
                .ok()?;

            Some((
                LXCImageMetadata {
                    dist: caps["dist"].to_string(),
                    release: caps["release"].to_string(),
                    arch: caps["arch"].to_string(),
                    type_: caps["type"].to_string(),
                    name: caps["name"].to_string(),
                    path,
                },
                mtime,
            ))
        })
        .collect();

    if entry.len() != 0 {
        Ok(entry)
    } else {
        bail!("Get LXC images from FS failed. Get images from FS error. Images is equal 0.")
    }
}
