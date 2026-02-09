use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use toad_core::{CustomWorkflow, ToadError, ToadResult, WorkflowRegistry};

pub fn register_workflow(
    registry: &mut WorkflowRegistry,
    name: String,
    script_path: PathBuf,
    description: Option<String>,
) -> ToadResult<()> {
    if registry.is_reserved(&name) {
        return Err(ToadError::Config(format!(
            "Name '{}' is reserved by a built-in command",
            name
        )));
    }

    let workflow = CustomWorkflow {
        name: name.clone(),
        description,
        script_path,
        registered_at: std::time::SystemTime::now(),
    };

    registry.workflows.insert(name.to_lowercase(), workflow);
    Ok(())
}

pub fn distribute_skills(
    root: &Path,
    vendors: &[String],
    skills: Vec<(String, String)>,
) -> ToadResult<Vec<PathBuf>> {
    let mut synced_paths = Vec::new();

    for vendor in vendors {
        let vendor_dir = match vendor.as_str() {
            "windsurf" => root.join(".windsurf"),
            "cursor" => root.join(".cursor"),
            "gemini" => root.join(".gemini"),
            _ => continue,
        };

        if !vendor_dir.exists() {
            fs::create_dir_all(&vendor_dir)?;
        }

        for (name, content) in &skills {
            let filename = format!("{}.md", name);
            let path = vendor_dir.join(filename);
            fs::write(&path, content)?;
            synced_paths.push(path);
        }
    }

    Ok(synced_paths)
}

pub fn run_workflow(workflow: &CustomWorkflow, args: &[String]) -> ToadResult<i32> {
    let mut command = Command::new("bash");
    command.arg(&workflow.script_path);
    for arg in args {
        command.arg(arg);
    }

    let status = command
        .status()
        .map_err(|e| ToadError::OperationFailed(format!("Failed to execute workflow: {}", e)))?;
    Ok(status.code().unwrap_or(-1))
}
