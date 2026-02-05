use super::shell::*;
use anyhow::Result;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_run_in_dir_success() -> Result<()> {
    let dir = tempdir()?;
    let result = run_in_dir(dir.path(), "echo hello", Duration::from_secs(5))?;
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout.trim(), "hello");
    assert!(!result.timed_out);
    Ok(())
}

#[test]
fn test_run_in_dir_failure() -> Result<()> {
    let dir = tempdir()?;
    let result = run_in_dir(dir.path(), "false", Duration::from_secs(5))?;
    assert_ne!(result.exit_code, 0);
    assert!(!result.timed_out);
    Ok(())
}

#[test]
fn test_run_in_dir_timeout() -> Result<()> {
    let dir = tempdir()?;
    let result = run_in_dir(dir.path(), "sleep 2", Duration::from_millis(100))?;
    assert!(result.timed_out);
    assert_eq!(result.exit_code, -1);
    Ok(())
}
