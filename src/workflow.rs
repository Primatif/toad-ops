use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use toad_core::{CustomWorkflow, WorkflowRegistry};

/// Returns the list of command names reserved by the built-in CLI.
pub fn reserved_command_names() -> Vec<&'static str> {
    vec![
        "create", "reveal", "status", "stats", "home", "do", "tag", "untag",
        "manifest", "sync", "strategy", "clean", "docs", "project", "ggit",
        "cw", "list", "version", "help",
    ]
}

/// Registers a new custom workflow.
pub fn register_workflow(
    registry: &mut WorkflowRegistry,
    name: String,
    script_path: PathBuf,
    description: Option<String>,
) -> Result<()> {
    let name_lower = name.to_lowercase();
    
    // 1. Check reserved namespace
    if reserved_command_names().contains(&name_lower.as_str()) {
        bail!("Command name '{}' is reserved by Toad built-ins.", name_lower);
    }

    // 2. Validate script path
    if !script_path.exists() {
        bail!("Script path does not exist: {:?}", script_path);
    }
    
    // 3. Register
    let workflow = CustomWorkflow {
        name: name_lower.clone(),
        description,
        script_path,
        registered_at: std::time::SystemTime::now(),
    };

    registry.workflows.insert(name_lower, workflow);
    
    // Update reserved cache
    registry.reserved_namespaces = reserved_command_names()
        .iter()
        .map(|s| s.to_string())
        .collect();

    Ok(())
}

/// Executes a custom workflow.
pub fn run_workflow(workflow: &CustomWorkflow, args: &[String]) -> Result<i32> {
    let mut cmd = Command::new(&workflow.script_path);
    cmd.args(args);
    
    let mut child = cmd.spawn()?;
    let status = child.wait()?;
    
    Ok(status.code().unwrap_or(0))
}
