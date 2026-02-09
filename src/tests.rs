use crate::clean::clean_project;
use crate::shell;
use crate::stats::calculate_project_stats;
use std::fs;
use std::time::Duration;
use tempfile::tempdir;
use toad_core::ToadResult;

#[test]
fn test_run_in_dir_success() -> ToadResult<()> {
    let dir = tempdir()?;
    let res = shell::run_in_dir(dir.path(), "echo 'hello'", Duration::from_secs(5))?;
    assert_eq!(res.exit_code, 0);
    assert!(res.stdout.contains("hello"));
    Ok(())
}

#[test]
fn test_run_in_dir_failure() -> ToadResult<()> {
    let dir = tempdir()?;
    let res = shell::run_in_dir(dir.path(), "false", Duration::from_secs(5))?;
    assert_ne!(res.exit_code, 0);
    Ok(())
}

#[test]
fn test_run_in_dir_timeout() -> ToadResult<()> {
    let dir = tempdir()?;
    let res = shell::run_in_dir(dir.path(), "sleep 10", Duration::from_secs(1))?;
    assert!(res.timed_out);
    Ok(())
}

#[test]
fn test_calculate_project_stats() -> ToadResult<()> {
    let dir = tempdir()?;
    let p = dir.path();
    fs::write(p.join("a.txt"), "hello")?;
    fs::create_dir(p.join("target"))?;
    fs::write(p.join("target/b.txt"), "world")?;

    let mut artifacts = std::collections::HashSet::new();
    artifacts.insert("target");

    let stats = calculate_project_stats(p, &artifacts);
    assert!(stats.total_bytes >= 10);
    assert!(stats.artifact_bytes >= 5);
    Ok(())
}

#[test]
fn test_clean_project() -> ToadResult<()> {
    let dir = tempdir()?;
    let p = dir.path();
    fs::create_dir(p.join("target"))?;
    fs::write(p.join("target/a.txt"), "data")?;

    let res = clean_project(p, &vec!["target".to_string()], false)?;
    assert_eq!(res.files_removed, 1);
    assert!(!p.join("target").exists());
    Ok(())
}

#[test]
fn test_clean_project_safety() -> ToadResult<()> {
    let dir = tempdir()?;
    let p = dir.path();
    fs::write(p.join("Cargo.toml"), "data")?;

    let res = clean_project(p, &vec!["Cargo.toml".to_string()], false)?;
    assert!(res.errors.len() > 0);
    assert!(p.join("Cargo.toml").exists());
    Ok(())
}
