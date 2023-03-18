mod lxc_image_download;
mod lxc_image_metadata;
mod lxc_image_metadata_collection;
mod lxc_image_patch;

use self::{
    lxc_image_download::download_image, lxc_image_metadata::FilterBy,
    lxc_image_metadata_collection::LXCImageMetadataCollection, lxc_image_patch::patch_image,
};

use crate::config;

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
        let out_tempdir_path = Builder::new().prefix(".repodata_").tempdir()?;
        let out_dir_path = &config.repodata.host_root_dir.join(&lxc_image_metadata.path);

        if out_dir_path.exists() {
            continue;
        }

        fs::create_dir_all(&out_dir_path)?;

        for image_file in &config.repodata.image_files {
            let download_url = config
                .repodata
                .target_url
                .origin
                .join(&lxc_image_metadata.path.to_str().ok_or_else(|| {
                    anyhow!(
                        "Download LXC image failed. Convert path to string error. Path: {:?}",
                        lxc_image_metadata.path
                    )
                })?)?
                .join(&image_file)?;
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

            fs::rename(&tempfile, &out_tempdir_path.path().join(image_file))?;
            tempfile.as_file().metadata()?.permissions().set_mode(0o644);
        }

        fs::rename(&out_tempdir_path, &out_dir_path)?;
        out_dir_path.metadata()?.permissions().set_mode(0o755);
    }

    Ok(())
}
