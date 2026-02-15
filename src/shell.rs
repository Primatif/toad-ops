use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use toad_core::{ToadError, ToadResult};
use wait_timeout::ChildExt;

#[derive(Debug)]
pub struct OpResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
}

pub fn run_in_dir(dir: &Path, command: &str, timeout: Duration) -> ToadResult<OpResult> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| ToadError::OperationFailed(format!("Failed to spawn process: {}", e)))?;

    match child
        .wait_timeout(timeout)
        .map_err(|e| ToadError::OperationFailed(format!("Error waiting for process: {}", e)))?
    {
        Some(status) => {
            let output = child
                .wait_with_output()
                .map_err(|e| ToadError::OperationFailed(format!("Failed to read output: {}", e)))?;
            Ok(OpResult {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: status.code().unwrap_or(-1),
                timed_out: false,
            })
        }
        None => {
            let _ = child.kill();
            let _ = child.wait(); // Cleanup
            Ok(OpResult {
                stdout: String::new(),
                stderr: "Operation timed out".to_string(),
                exit_code: -1,
                timed_out: true,
            })
        }
    }
}
