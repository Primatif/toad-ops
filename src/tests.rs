use super::shell::*;
use anyhow::Result;
use tempfile::tempdir;

#[test]
fn test_run_in_dir_success() -> Result<()> {
    let dir = tempdir()?;
    let result = run_in_dir(dir.path(), "echo hello")?;
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout.trim(), "hello");
    Ok(())
}

#[test]
fn test_run_in_dir_failure() -> Result<()> {
    let dir = tempdir()?;
    let result = run_in_dir(dir.path(), "false")?;
    assert_ne!(result.exit_code, 0);
    Ok(())
}
