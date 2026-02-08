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

/// Distributes the blueprint as a "Skill" or "System Context" to registered AI vendors.
///
/// This wraps the raw blueprint in vendor-specific instructions (e.g., <skill> tags for Gemini,
/// System Prompts for Cursor) to ensure the agent actively "knows" this information.
pub fn distribute_blueprint(
    root_path: &Path,
    vendors: &[String],
    blueprint_content: &str,
) -> Result<Vec<PathBuf>> {
    let mut synced_paths = Vec::new();

    for vendor_entry in vendors {
        let (target_path, formatted_content) = if vendor_entry.contains(':') {
            // Custom mapping: "name:path"
            let parts: Vec<&str> = vendor_entry.splitn(2, ':').collect();
            (
                Some(root_path.join(parts[1])),
                format_as_skill(parts[0], blueprint_content),
            )
        } else {
            // Built-in vendors
            let vendor_lower = vendor_entry.to_lowercase();
            match vendor_lower.as_str() {
                "windsurf" => (
                    Some(root_path.join(".windsurf/rules/toad-blueprint.md")),
                    format_as_skill("Windsurf", blueprint_content),
                ),
                "gemini" => (
                    // Gemini Skills convention
                    Some(root_path.join(".gemini/skills/toad-blueprint.md")),
                    format_as_gemini_skill(blueprint_content),
                ),
                "cursor" => (
                    Some(root_path.join(".cursorrules")),
                    format_as_system_prompt("Cursor", blueprint_content),
                ),
                "claude" => (
                    Some(root_path.join(".clauderules")),
                    format_as_system_prompt("Claude", blueprint_content),
                ),
                "copilot" | "github" | "codex" => (
                    Some(root_path.join(".github/copilot-instructions.md")),
                    format_as_system_prompt("GitHub Copilot", blueprint_content),
                ),
                "roo" | "cline" => (
                    Some(root_path.join(".clinerules")),
                    format_as_system_prompt("Roo/Cline", blueprint_content),
                ),
                "continue" => (
                    Some(root_path.join(".continue/toad-blueprint.md")),
                    format_as_system_prompt("Continue", blueprint_content),
                ),
                "cody" | "sourcegraph" => (
                    Some(root_path.join(".cody/toad-blueprint.md")),
                    format_as_system_prompt("Cody", blueprint_content),
                ),
                "pear" | "pearai" => (
                    Some(root_path.join(".pearrules")),
                    format_as_system_prompt("PearAI", blueprint_content),
                ),
                "supermaven" => (
                    Some(root_path.join(".supermaven/toad-blueprint.md")),
                    format_as_system_prompt("Supermaven", blueprint_content),
                ),
                "agnostic" | "ai" => (
                    Some(root_path.join(".ai/toad-blueprint.md")),
                    blueprint_content.to_string(),
                ),
                _ => (None, String::new()),
            }
        };

        if let Some(path) = target_path {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            // For single-file rules (like .cursorrules), we might want to append or merge
            // But for a Blueprint sync, we largely want to ensure the architectural truth is fresh.
            // Current strategy: Overwrite to prevent stale context accumulation.
            std::fs::write(&path, formatted_content)?;
            synced_paths.push(path);
        }
    }

    Ok(synced_paths)
}

/// Formats content as a generic System Prompt Skill
fn format_as_skill(vendor: &str, content: &str) -> String {
    format!(
        "# Skill: Architectural Awareness ({})\n\n> **Auto-Generated by Toad**\n\n{}\n",
        vendor, content
    )
}

/// Formats content specifically for Gemini Skills (XML-style)
fn format_as_gemini_skill(content: &str) -> String {
    format!(
        "<skill>\n  <name>architectural-blueprint</name>\n  <description>Provides a deep dependency graph and type flow map of the project.</description>\n  <instructions>\n{}\n  </instructions>\n</skill>\n",
        content
    )
}

/// Formats content as a System Prompt for rule-based agents
fn format_as_system_prompt(agent_name: &str, content: &str) -> String {
    format!(
        "You are an expert software architect using {}.\n\n# Project Blueprint\n\nUse this architectural context to understand dependencies and data flow:\n\n{}\n",
        agent_name, content
    )
}

/// Executes a custom workflow.
pub fn run_workflow(workflow: &CustomWorkflow, args: &[String]) -> Result<i32> {
    let mut cmd = Command::new(&workflow.script_path);
    cmd.args(args);
    
    let mut child = cmd.spawn()?;
    let status = child.wait()?;
    
    Ok(status.code().unwrap_or(0))
}
