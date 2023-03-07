use crate::config::Timeout;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{process::Command, time::Duration};
use wait_timeout::ChildExt;

#[derive(Debug, Serialize, Deserialize)]
struct ChildProcess {
    path_to_bin: String,
    arg: String,
    timeout: Timeout,
}

// The module do not use yet
#[allow(dead_code)]
impl ChildProcess {
    fn run(self) -> Result<Option<i32>> {
        let mut child = Command::new(self.path_to_bin).arg(self.arg).spawn()?;

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
