use crate::config::Timeout;

use super::lxc_image_metadata::LXCImageMetadata;

use anyhow::{anyhow, bail, Result};
use slog_scope::info;
use std::{path::PathBuf, process::Command, time::Duration};
use tempfile::NamedTempFile;
use wait_timeout::ChildExt;

pub fn patch_image(
    path_to_script: &PathBuf,
    tempfile: &NamedTempFile,
    timeout: Timeout,
    metadata: &LXCImageMetadata,
) -> Result<()> {
    info!("Patch LXC image started.");

    let tempfile = tempfile.path().to_str().ok_or_else(|| {
        anyhow!(
            "Patch LXC image failed. Convert tempfile to string error. Tempfile: {:?}.",
            tempfile
        )
    })?;

    if !&path_to_script.exists() {
        bail!(
            "Patch LXC image failed. Path to script do not exists. Path: {:?}.",
            &path_to_script
        );
    }

    let mut child = Command::new(path_to_script)
        .args(["-path", tempfile])
        .args(["-d", &metadata.dist])
        .args(["-r", &metadata.release])
        .args(["-a", &metadata.arch])
        .spawn()
        .map_err(|err| {
            anyhow!(
                "Patch LXC image failed. Create child process error. Error: {:?}",
                err
            )
        })?;

    let timeout_sec = Duration::from_secs(timeout);

    match child.wait_timeout(timeout_sec) {
        Ok(Some(status)) => {
            info!("Patch LXC image done.");
            if !status.success() {
                bail!("Patch LXC image completed with non-zero exit code.")
            }
        }
        _ => {
            child.kill()?;
            child.wait()?;
            bail!("Patch LXC image failed. Timeout error.")
        }
    };

    Ok(())
}
