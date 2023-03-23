use super::lxc_image_metadata::LXCImageMetadata;

use anyhow::Result;
use slog_scope::info;
use std::{collections::HashMap, fs, time::Duration};

pub fn cleanup_entries(
    entry: Vec<(LXCImageMetadata, Duration)>,
    number_of_container_to_backup: i16,
) -> Result<()> {
    info!("Cleanup LXC images started.");

    let hmap: HashMap<&str, Vec<(LXCImageMetadata, Duration)>> =
        entry
            .iter()
            .fold(HashMap::new(), |mut acc, (image_meta, mtime)| {
                let path_as_str = match image_meta.path.to_str() {
                    Some(v) => v,
                    _ => return acc,
                };
                let pos = match path_as_str.rfind("/") {
                    Some(v) => v,
                    _ => return acc,
                };

                acc.entry(&path_as_str[..pos])
                    .and_modify(|x| {
                        x.push((image_meta.to_owned(), mtime.to_owned()));
                    })
                    .or_insert(vec![]);
                acc
            });

    for mut image in hmap.into_values() {
        image.sort_by(|a, b| a.1.cmp(&b.1));

        while image.len() > number_of_container_to_backup as usize {
            let delete_dir = &image[0].0.path;
            fs::remove_dir_all(delete_dir)?;
            info!("Delete LXC image directory. Dir: {:?}", delete_dir);
            image.remove(0);
        }
    }

    info!("Cleanup LXC images done.");

    Ok(())
}
