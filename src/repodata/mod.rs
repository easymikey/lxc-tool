mod lxc_image_download;
mod lxc_image_metadata;
mod lxc_image_metadata_collection;
mod lxc_image_patch;

use self::{
    lxc_image_download::download_image, lxc_image_metadata::FilterBy,
    lxc_image_metadata_collection::LXCImageMetadataCollection, lxc_image_patch::patch_image,
};

use crate::config;

use anyhow::Result;
use std::{fs, os::unix::prelude::PermissionsExt};
use tempfile::Builder;
use url::Url;

pub async fn download_images(config: config::Config) -> Result<()> {
    let target_url_origin = Url::parse(&config.repodata.target_url.origin)?;
    let meta_data_url = target_url_origin.join(&config.repodata.target_url.index_uri)?;

    let lxc_image_metadata_collection = LXCImageMetadataCollection::of(&meta_data_url)
        .get()
        .await?
        .filter_by(config.repodata.image_filters)?;

    for (lxc_container_metadata, post_process) in lxc_image_metadata_collection {
        let out_tempdir_path = Builder::new().prefix(".repodata_").tempdir()?;
        let path = lxc_container_metadata.path.trim_start_matches("/");
        let out_dir_path = &config.repodata.host_root_dir.join(&path);

        if out_dir_path.exists() {
            continue;
        }

        fs::create_dir_all(&out_dir_path)?;

        for image_file in &config.repodata.image_files {
            let download_url = target_url_origin.join(&path)?.join(&image_file)?;
            let tempfile = download_image(download_url).await?;

            let is_filename_rootfs = image_file == "rootfs.tar.xz";

            if is_filename_rootfs {
                if let Some(post_process) = &post_process {
                    patch_image(
                        post_process,
                        &tempfile,
                        config.repodata.patcher_timeout,
                        &lxc_container_metadata,
                    )?;
                }

                if let Some(post_script) = &config.repodata.post_script_path {
                    patch_image(
                        post_script,
                        &tempfile,
                        config.repodata.patcher_timeout,
                        &lxc_container_metadata,
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
