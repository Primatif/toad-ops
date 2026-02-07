use super::shell::*;
use super::stats::*;
use anyhow::Result;
use std::fs;
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

#[test]
fn test_calculate_project_stats() -> Result<()> {
    let dir = tempdir()?;
    let p = dir.path();

    // Create source file
    fs::write(p.join("main.rs"), "fn main() {}")?; // ~12 bytes

    // Create artifact file
    let target_dir = p.join("target");
    fs::create_dir(&target_dir)?;
    fs::write(target_dir.join("binary"), "0".repeat(1000))?; // 1000 bytes

    let mut artifact_dirs = std::collections::HashSet::new();
    artifact_dirs.insert("target");
    let stats = calculate_project_stats(p, &artifact_dirs);

    assert!(stats.total_bytes >= 1012);
    assert!(stats.artifact_bytes >= 1000);
    assert!(stats.source_bytes >= 12);
    assert!(stats.bloat_index > 90.0);

    Ok(())
}

#[test]
fn test_clean_project() -> Result<()> {
    let dir = tempdir()?;
    let p = dir.path();

    // Create source file
    fs::write(p.join("README.md"), "hello")?;

    // Create artifact directory
    let target_dir = p.join("target");
    fs::create_dir(&target_dir)?;
    fs::write(target_dir.join("artifact"), "0".repeat(100))?;

    // 1. Dry run
    let artifacts = vec!["target".to_string()];
    let res = crate::clean::clean_project(p, &artifacts, true)?;
    assert!(res.bytes_reclaimed >= 100);
    assert!(target_dir.exists());

    // 2. Real clean
    let res = crate::clean::clean_project(p, &artifacts, false)?;
    assert!(res.bytes_reclaimed >= 100);
    assert!(!target_dir.exists());
    assert!(p.join("README.md").exists());

    Ok(())
}

#[test]
fn test_clean_project_safety() -> Result<()> {
    let dir = tempdir()?;
    let p = dir.path();

    // Try to clean a reserved path
    let artifacts = vec![".git".to_string()];
    let res = crate::clean::clean_project(p, &artifacts, false)?;

    assert_eq!(res.bytes_reclaimed, 0);
    assert!(res.errors[0].contains("Skipping reserved path"));

    Ok(())
}

#[test]
fn test_format_size() {
    assert_eq!(format_size(500), "500 B");
    assert_eq!(format_size(1024), "1.00 KB");
    assert_eq!(format_size(1024 * 1024), "1.00 MB");
    assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
}
