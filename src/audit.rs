use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use toad_core::GlobalConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub command: String,
    pub target_count: usize,
    pub success_count: usize,
    pub fail_count: usize,
    pub skip_count: usize,
    pub user: String,
}

pub fn log_operation(entry: AuditEntry) -> Result<()> {
    let log_path = GlobalConfig::config_dir()?.join("ops.log");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    let json = serde_json::to_string(&entry)?;
    writeln!(file, "{}", json)?;
    Ok(())
}
