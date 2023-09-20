use super::lxc_image_metadata::LXCImageMetadata;

use anyhow::{anyhow, bail, Result};
use nix::unistd;
use pwd::Passwd;
use slog_scope::info;
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
    time::Duration,
};

pub fn save_image_metadata(
    root_dir: &PathBuf,
    metadata_path: String,
    username: String,
    mut image_entries: Vec<(LXCImageMetadata, Duration)>,
) -> Result<()> {
    info!("Save LXC image metadata started.");

    let image_metadata_path = root_dir.join(&metadata_path);
    let user_image_metadata_path: PathBuf = root_dir.join("meta/1.0/index-user");

    if let Some(parent_dir_path) = &image_metadata_path.parent() {
        if !parent_dir_path.exists() {
            fs::create_dir_all(parent_dir_path)?;
        }
    }

    let mut file = File::create(&image_metadata_path)?;

    image_entries.sort_by(|a, b| a.0.name.cmp(&b.0.name).reverse());

    for (image_metadata, _) in image_entries {
        let path = image_metadata.path;
        let root_dir_path = root_dir.to_str().ok_or_else(|| {
            anyhow!(
                "Convert directory path to string error. Directory path: {:?}.",
                root_dir
            )
        })?;

        file.write_all(
            (format!(
                "{};{};{};{};{};/{:?}",
                image_metadata.dist,
                image_metadata.release,
                image_metadata.arch,
                image_metadata.type_,
                image_metadata.name,
                path.strip_prefix(root_dir_path)?,
            )
            .replace('\"', "")
                + "\n")
                .as_bytes(),
        )?;
    }

    let copy_image_metadata_path = image_metadata_path.with_extension("7");

    fs::copy(&image_metadata_path, &copy_image_metadata_path)?;

    fs::copy(&image_metadata_path, &user_image_metadata_path)?;

    match Passwd::from_name(&username)? {
        Some(passwd) => {
            for filepath in [image_metadata_path, copy_image_metadata_path] {
                unistd::chown(&filepath, Some(passwd.uid.into()), Some(passwd.gid.into()))?;
            }
        }
        _ => bail!(
            "Save LXC image metadata failed. Invalid username error. Username: {}",
            username
        ),
    }

    info!("Save LXC image metadata done.");

    Ok(())
}
