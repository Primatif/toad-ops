use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub struct OpResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

/// Executes a shell command in the specified directory.
pub fn run_in_dir(dir: &Path, command: &str) -> Result<OpResult> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(dir)
        .output()?;

    Ok(OpResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}
