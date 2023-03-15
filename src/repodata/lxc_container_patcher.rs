use crate::config::Timeout;
use anyhow::{anyhow, bail, Result};
use std::{
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};
use wait_timeout::ChildExt;

pub struct LCXContainerPatcher<'a> {
    pub path_to_script: Option<&'a String>,
    pub tempfile: &'a PathBuf,
    pub timeout: Timeout,
}

impl<'a> LCXContainerPatcher<'a> {
    pub fn of(path_to_script: &'a Option<String>, tempfile: &'a PathBuf, timeout: Timeout) -> Self {
        LCXContainerPatcher {
            path_to_script: path_to_script.as_ref(),
            tempfile,
            timeout,
        }
    }

    pub fn patch(self, args: [&Option<String>; 3]) -> Result<Option<i32>> {
        let args = args
            .into_iter()
            .filter_map(|x| x.as_ref())
            .collect::<Vec<_>>();

        let path_to_script = self
            .path_to_script
            .ok_or_else(|| anyhow!("Path to script do not pass."))?;

        let tempfile = self
            .tempfile
            .to_str()
            .ok_or_else(|| anyhow!("Converting tempfile to string error. {:?}", self.tempfile))?;

        if args.len() != 3 {
            bail!("Patcher arguments do not correct. Args: {:?}", &args);
        }

        if !Path::new(&path_to_script).exists() {
            bail!("Path to script do not exists. Path: {}", &path_to_script)
        }

        let mut child = Command::new(&path_to_script)
            .args(["-path", tempfile])
            .args(["-d", args[0]])
            .args(["-r", args[1]])
            .args(["-a", args[2]])
            .spawn()?;
        let timeout_sec = Duration::from_secs(self.timeout);
        let status_code = match child.wait_timeout(timeout_sec)? {
            Some(status) => status.code(),
            None => {
                child.kill()?;
                child.wait()?.code()
            }
        };

        Ok(status_code)
    }
}
