mod lxc_image_download;
mod lxc_image_entries_cleanup;
mod lxc_image_metadata;
mod lxc_image_metadata_collection;
mod lxc_image_metadata_entries_create;
mod lxc_image_metadata_save;
mod lxc_image_patch;

use crate::{
    config, repodata::lxc_image_download::download_image,
    repodata::lxc_image_entries_cleanup::cleanup_image_entries,
    repodata::lxc_image_metadata::FilterBy,
    repodata::lxc_image_metadata_collection::LXCImageMetadataCollection,
    repodata::lxc_image_metadata_entries_create::create_image_metadata_entries,
    repodata::lxc_image_metadata_save::save_image_metadata, repodata::lxc_image_patch::patch_image,
};

use anyhow::{anyhow, Result};
use std::{fs, os::unix::prelude::PermissionsExt};
use tempfile::Builder;

pub async fn download_images(config: config::Config) -> Result<()> {
    let meta_data_url = config
        .repodata
        .target_url
        .origin
        .join(&config.repodata.target_url.index_uri)?;

    let lxc_image_metadata_collection = LXCImageMetadataCollection::of(&meta_data_url)
        .get()
        .await?
        .filter_by(config.repodata.image_filters)?;

    for (lxc_image_metadata, post_process) in lxc_image_metadata_collection {
        let image_tempdir_path = Builder::new().prefix(".repodata_").tempdir()?;
        let image_dir_path = &config.repodata.host_root_dir.join(&lxc_image_metadata.path);

        if image_dir_path.exists() {
            continue;
        }

        fs::create_dir_all(image_dir_path)?;

        for image_file in &config.repodata.image_files {
            let image_dir = lxc_image_metadata.path.to_str().ok_or_else(|| {
                anyhow!(
                    "Download LXC image failed. Convert path to string error. Path: {:?}",
                    lxc_image_metadata.path
                )
            })?;
            let download_url = config
                .repodata
                .target_url
                .origin
                .join(image_dir)?
                .join(image_file)?;
            let tempfile = download_image(download_url).await?;

            if image_file == "rootfs.tar.xz" {
                if let Some(post_process) = &post_process {
                    patch_image(
                        post_process,
                        &tempfile,
                        config.repodata.patcher_timeout,
                        &lxc_image_metadata,
                    )?;
                }
            }

            fs::rename(&tempfile, &image_tempdir_path.path().join(image_file))?;
            tempfile.as_file().metadata()?.permissions().set_mode(0o644);
        }

        fs::rename(&image_tempdir_path, image_dir_path)?;
        image_dir_path.metadata()?.permissions().set_mode(0o755);
    }

    cleanup_image_entries(
        &config.repodata.host_root_dir,
        config.repodata.number_of_container_to_backup,
        create_image_metadata_entries(&config.repodata.host_root_dir)?,
    )?;

    save_image_metadata(
        &config.repodata.host_root_dir,
        config.repodata.target_url.index_uri,
        config.repodata.username,
        create_image_metadata_entries(&config.repodata.host_root_dir)?,
    )?;

    Ok(())
}
