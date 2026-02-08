use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use toad_core::{CustomWorkflow, WorkflowRegistry};

/// Returns the list of command names reserved by the built-in CLI.
pub fn reserved_command_names() -> Vec<&'static str> {
    vec![
        "create", "reveal", "status", "stats", "home", "do", "tag", "untag", "manifest", "sync",
        "strategy", "clean", "docs", "project", "ggit", "cw", "list", "version", "help",
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
        bail!(
            "Command name '{}' is reserved by Toad built-ins.",
            name_lower
        );
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

/// Distributes multiple skills to the specific memory slots of the registered AI vendors.
///
/// Each skill is a tuple of (file_stem, content).
/// Example: ("toad-blueprint", "...")
pub fn distribute_skills(
    root_path: &Path,
    vendors: &[String],
    skills: Vec<(String, String)>,
) -> Result<Vec<PathBuf>> {
    let mut synced_paths = Vec::new();

    for vendor_entry in vendors {
        for (stem, content) in &skills {
            let (target_path, formatted_content) = if vendor_entry.contains(':') {
                // Custom mapping: "name:path/to/dir"
                let parts: Vec<&str> = vendor_entry.splitn(2, ':').collect();
                let dir = root_path.join(parts[1]);
                (
                    Some(dir.join(format!("{}.md", stem))),
                    format_as_skill(parts[0], content),
                )
            } else {
                // Built-in vendors
                let vendor_lower = vendor_entry.to_lowercase();
                match vendor_lower.as_str() {
                    "windsurf" => (
                        Some(root_path.join(format!(".windsurf/rules/{}.md", stem))),
                        format_as_skill("Windsurf", content),
                    ),
                    "gemini" => (
                        Some(root_path.join(format!(".gemini/skills/{}.md", stem))),
                        format_as_gemini_skill(stem, content),
                    ),
                    "cursor" => {
                        // Cursor only has one authoritative file, so we append/merge
                        // For now, we'll use .cursorrules as the primary slot for the blueprint
                        if stem == "toad-blueprint" {
                            (
                                Some(root_path.join(".cursorrules")),
                                format_as_system_prompt("Cursor", content),
                            )
                        } else {
                            (None, String::new())
                        }
                    }
                    "claude" => {
                        if stem == "toad-blueprint" {
                            (
                                Some(root_path.join(".clauderules")),
                                format_as_system_prompt("Claude", content),
                            )
                        } else {
                            (None, String::new())
                        }
                    }
                    "copilot" | "github" | "codex" => (
                        Some(root_path.join(format!(".github/{}.md", stem))),
                        format_as_system_prompt("GitHub Copilot", content),
                    ),
                    "roo" | "cline" => (
                        Some(root_path.join(format!(".{}.md", stem))),
                        format_as_system_prompt("Roo/Cline", content),
                    ),
                    "continue" => (
                        Some(root_path.join(format!(".continue/{}.md", stem))),
                        format_as_system_prompt("Continue", content),
                    ),
                    "cody" | "sourcegraph" => (
                        Some(root_path.join(format!(".cody/{}.md", stem))),
                        format_as_system_prompt("Cody", content),
                    ),
                    "pear" | "pearai" => (
                        Some(root_path.join(format!(".{}.md", stem))),
                        format_as_system_prompt("PearAI", content),
                    ),
                    "supermaven" => (
                        Some(root_path.join(format!(".supermaven/{}.md", stem))),
                        format_as_system_prompt("Supermaven", content),
                    ),
                    "agnostic" | "ai" => (
                        Some(root_path.join(format!(".ai/{}.md", stem))),
                        content.to_string(),
                    ),
                    _ => (None, String::new()),
                }
            };

            if let Some(path) = target_path {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&path, formatted_content)?;
                synced_paths.push(path);
            }
        }
    }

    Ok(synced_paths)
}

/// Formats content as a generic System Prompt Skill
fn format_as_skill(vendor: &str, content: &str) -> String {
    format!(
        "# Skill: {} Awareness ({})\n\n> **Auto-Generated by Toad**\n\n{}\n",
        vendor,
        vendor,
        content.trim()
    )
}

/// Formats content specifically for Gemini Skills (XML-style)
fn format_as_gemini_skill(name: &str, content: &str) -> String {
    format!(
        "<skill>\n  <name>{}</name>\n  <description>Auto-generated architectural context from Toad.</description>\n  <instructions>\n{}\n  </instructions>\n</skill>\n",
        name,
        content.trim()
    )
}

/// Formats content as a System Prompt for rule-based agents
fn format_as_system_prompt(agent_name: &str, content: &str) -> String {
    format!(
        "You are an expert software architect using {}.\n\n# Project Blueprint\n\nUse this architectural context to understand dependencies and data flow:\n\n{}\n",
        agent_name,
        content.trim()
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
