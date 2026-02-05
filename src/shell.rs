use anyhow::Result;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use wait_timeout::ChildExt;

pub struct OpResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
}

/// Executes a shell command in the specified directory with a timeout.
pub fn run_in_dir(dir: &Path, command: &str, timeout: Duration) -> Result<OpResult> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    match child.wait_timeout(timeout)? {
        Some(status) => {
            let output = child.wait_with_output()?;
            Ok(OpResult {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: status.code().unwrap_or(-1),
                timed_out: false,
            })
        }
        None => {
            child.kill()?;
            let _ = child.wait();
            Ok(OpResult {
                stdout: String::new(),
                stderr: format!("Command timed out after {:?}", timeout),
                exit_code: -1,
                timed_out: true,
            })
        }
    }
}
