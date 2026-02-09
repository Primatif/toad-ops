use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use toad_core::{ToadError, ToadResult};

#[derive(Debug)]
pub struct OpResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
}

pub fn run_in_dir(dir: &Path, command: &str, timeout: Duration) -> ToadResult<OpResult> {
    let start = Instant::now();

    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| ToadError::OperationFailed(format!("Failed to spawn process: {}", e)))?;

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = child.wait_with_output().map_err(|e| {
                    ToadError::OperationFailed(format!("Failed to read output: {}", e))
                })?;
                return Ok(OpResult {
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    exit_code: status.code().unwrap_or(-1),
                    timed_out: false,
                });
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    return Ok(OpResult {
                        stdout: String::new(),
                        stderr: "Operation timed out".to_string(),
                        exit_code: -1,
                        timed_out: true,
                    });
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                return Err(ToadError::OperationFailed(format!(
                    "Error waiting for process: {}",
                    e
                )));
            }
        }
    }
}
