use super::lxc_image_metadata::LXCImageMetadata;

use anyhow::Result;
use slog_scope::info;
use std::{collections::HashMap, fs, path::PathBuf, time::Duration};

pub fn cleanup_image_entries(
    root_dir: &PathBuf,
    number_of_container_to_backup: i16,
    image_entries: Vec<(LXCImageMetadata, Duration)>,
) -> Result<()> {
    info!("Cleanup LXC images started.");

    let hashed_image_entries: HashMap<_, Vec<_>> =
        image_entries
            .into_iter()
            .fold(HashMap::new(), |mut acc, (image_meta, mtime)| {
                acc.entry((
                    image_meta.dist.clone(),
                    image_meta.release.clone(),
                    image_meta.arch.clone(),
                    image_meta.type_.clone(),
                ))
                .and_modify(|entries| {
                    entries.push((image_meta.path.to_owned(), mtime.to_owned()));
                })
                .or_insert(Vec::new());

                acc
            });

    for mut image_entries in hashed_image_entries.into_values() {
        image_entries.sort_by(|a, b| a.1.cmp(&b.1));

        while image_entries.len() > number_of_container_to_backup as usize {
            let (removed_dir, _) = &image_entries[0];
            if removed_dir.canonicalize()?.starts_with(root_dir) {
                fs::remove_dir_all(removed_dir)?;
                info!(
                    "Remove LXC image directory. Directory path: {:?}",
                    removed_dir
                );
                image_entries.remove(0);
            }
        }
    }

    info!("Cleanup LXC images done.");

    Ok(())
}
