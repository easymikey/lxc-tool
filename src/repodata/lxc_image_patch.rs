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
    info!("Patcher execution started.");

    let tempfile = tempfile.path().to_str().ok_or_else(|| {
        anyhow!(
            "Patcher execution failed. Converting tempfile to string error. Tempfile: {:?}.",
            tempfile
        )
    })?;

    if !&path_to_script.exists() {
        bail!(
            "Patcher execution failed. Path to script do not exists. Path: {:?}.",
            &path_to_script
        );
    }

    let mut child = Command::new(&path_to_script)
        .args(["-path", tempfile])
        .args(["-d", &metadata.dist])
        .args(["-r", &metadata.release])
        .args(["-a", &metadata.arch])
        .spawn()
        .map_err(|err| {
            anyhow!(
                "Patcher execution failed. Failed to create child process. Error: {:?}",
                err
            )
        })?;

    let timeout_sec = Duration::from_secs(timeout);
    let is_status_success = match child.wait_timeout(timeout_sec) {
        Ok(Some(status)) => status.success(),
        _ => {
            child.kill()?;
            child.wait()?.success()
        }
    };

    if is_status_success {
        info!("Patcher execution done.");
    } else {
        info!("Patcher execution failed. Timeout.")
    }

    Ok(())
}
