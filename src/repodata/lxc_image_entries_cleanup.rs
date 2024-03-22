use super::lxc_image_metadata::LXCImageMetadata;

use anyhow::Result;
use slog_scope::info;
use std::{collections::HashMap, fs, path::PathBuf, time::Duration};

pub fn cleanup_image_entries(
    root_dir: &PathBuf,
    number_of_container_to_backup: usize,
    image_entries: Vec<(LXCImageMetadata, Duration)>,
) -> Result<()> {
    info!("Cleanup LXC images started.");

    let root_dir = root_dir.canonicalize()?;
    let hashed_image_entries: HashMap<_, Vec<_>> =
        image_entries
            .into_iter()
            .fold(HashMap::new(), |mut acc, (image_meta, mtime)| {
                let image_entries = acc
                    .entry((
                        image_meta.dist,
                        image_meta.release,
                        image_meta.arch,
                        image_meta.type_,
                    ))
                    .or_default();
                image_entries.push((image_meta.path, mtime));

                acc
            });

    for mut image_entries in hashed_image_entries.into_values() {
        if image_entries.len() <= number_of_container_to_backup {
            continue;
        }

        let number_of_container_to_remove = image_entries.len() - number_of_container_to_backup;
        image_entries.sort_by(|a, b| a.1.cmp(&b.1));

        info!("cleanup_image_entries: {:#?}", image_entries);

        for (removed_dir, _) in image_entries.iter().take(number_of_container_to_remove) {
            if removed_dir.canonicalize()?.starts_with(&root_dir) {
                fs::remove_dir_all(removed_dir)?;
                info!(
                    "Remove LXC image directory. Directory path: {:?}",
                    removed_dir
                );
            }
        }
    }

    info!("Cleanup LXC images done.");

    Ok(())
}
