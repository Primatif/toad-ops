use serde::{Deserialize, Serialize};
use std::fs;
use toad_core::{GlobalConfig, ToadResult};

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

pub fn log_operation(entry: AuditEntry) -> ToadResult<()> {
    let log_dir = GlobalConfig::config_dir(None)?.join("audit");
    if !log_dir.exists() {
        fs::create_dir_all(&log_dir)?;
    }

    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let log_path = log_dir.join(format!("{}.jsonl", date));

    let json = serde_json::to_string(&entry)?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    use std::io::Write;
    writeln!(file, "{}", json)?;

    Ok(())
}
