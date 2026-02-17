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

    for vendor_spec in vendors {
        let (vendor_name, vendor_dir) = if vendor_spec.contains(':') {
            let parts: Vec<&str> = vendor_spec.splitn(2, ':').collect();
            (parts[0], root.join(parts[1]))
        } else {
            let dir = match vendor_spec.to_lowercase().as_str() {
                "windsurf" => root.join(".windsurf"),
                "cursor" => root.join(".cursor"),
                "gemini" => root.join(".gemini"),
                "copilot" => root.join(".github/copilot"),
                "continue" => root.join(".continue"),
                "aider" => root.join(".aider"),
                "supermaven" => root.join(".supermaven"),
                "trae" => root.join(".trae"),
                "cline" => root.join(".cline"),
                "pearai" => root.join(".pearai"),
                "bolt" => root.join(".bolt"),
                "lovable" => root.join(".lovable"),
                "v0" => root.join(".v0"),
                _ => continue,
            };
            (vendor_spec.as_str(), dir)
        };

        if !vendor_dir.exists() {
            fs::create_dir_all(&vendor_dir)?;
        }

        for (name, content) in &skills {
            let filename = format!("{}.md", name);
            let path = vendor_dir.join(filename);

            let final_content = match vendor_name.to_lowercase().as_str() {
                "gemini" => {
                    let description = match name.as_str() {
                        "toad-blueprint" => "Architectural blueprint and dependency map.",
                        "toad-cli" => "Toad Control CLI command reference.",
                        "toad-mcp" => "Model Context Protocol tool reference.",
                        _ => "Auto-generated ecosystem context.",
                    };
                    format!(
                        "<skill>\n  <name>{}</name>\n  <description>{}</description>\n  <instructions>\n{}\n  </instructions>\n</skill>\n",
                        name, description, content
                    )
                }
                _ => {
                    if content.ends_with('\n') {
                        content.clone()
                    } else {
                        format!("{}\n", content)
                    }
                }
            };

            fs::write(&path, final_content)?;
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
