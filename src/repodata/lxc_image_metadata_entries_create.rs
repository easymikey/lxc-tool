use super::lxc_image_metadata::LXCImageMetadata;

use anyhow::Result;
use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};
use walkdir::WalkDir;

pub fn create_image_metadata_entries(
    root_dir: &PathBuf,
) -> Result<Vec<(LXCImageMetadata, Duration)>> {
    let image_entries: Vec<_> = WalkDir::new(root_dir)
        .min_depth(5)
        .max_depth(5)
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            if !path.is_dir() {
                return None;
            }

            let mtime = entry
                .metadata()
                .ok()?
                .modified()
                .ok()?
                .duration_since(SystemTime::UNIX_EPOCH)
                .ok()?;

            let entry_metadata = match path
                .components()
                .filter_map(|c| c.as_os_str().to_str())
                .collect::<Vec<_>>()
                .as_slice()
            {
                &[.., dist, release, arch, type_, name] => Some((
                    LXCImageMetadata {
                        dist: dist.to_string(),
                        release: release.to_string(),
                        arch: arch.to_string(),
                        type_: type_.to_string(),
                        name: name.to_string(),
                        path: path.to_path_buf(),
                    },
                    mtime,
                )),
                _ => None,
            }?;
            Some(entry_metadata)
        })
        .collect();

    Ok(image_entries)
}
