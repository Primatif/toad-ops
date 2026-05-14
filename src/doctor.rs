use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use toad_core::{ProjectRegistry, ToadResult, Workspace};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthReport {
    pub version: String,
    pub mcp_installed: bool,
    pub workspace_discovered: bool,
    pub workspace_path: Option<PathBuf>,
    pub registry_fresh: bool,
    pub project_count: usize,
    pub git_remotes_reachable: bool,
    pub uninitialized_submodules: usize,
    pub dirty_submodules: usize,
    pub manifest_exists: bool,
    pub atlas_exists: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub diagnostics: toad_core::DiagnosticReport,
}

pub fn run_health_check(workspace: &Workspace) -> ToadResult<HealthReport> {
    let mut issues = Vec::new();
    let mut warnings = Vec::new();

    let mcp_installed = std::process::Command::new("toad-mcp")
        .arg("--version")
        .output()
        .is_ok();

    if !mcp_installed {
        warnings.push("Install toad-mcp: cargo install --path bin/toad-mcp".to_string());
    }

    let stored_fp = workspace.stored_fingerprint();
    let current_fp = workspace.get_fingerprint().unwrap_or(0);
    let registry_fresh = current_fp == stored_fp;

    if !registry_fresh {
        warnings.push("Run 'toad manifest' to refresh context".to_string());
    }

    let registry = ProjectRegistry::load(workspace.active_context.as_deref(), None);
    let project_count = match &registry {
        Ok(r) => r.projects.len(),
        Err(e) => {
            issues.push(format!("Registry load failed: {}", e));
            issues.push("Run 'toad sync' to build registry".to_string());
            0
        }
    };

    let git_remotes_reachable = std::process::Command::new("git")
        .args(["ls-remote", "--exit-code", "origin"])
        .current_dir(&workspace.projects_dir)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !git_remotes_reachable {
        warnings.push("Check network connection or git remote configuration".to_string());
    }

    let mut uninitialized_submodules = 0;
    let mut dirty_submodules = 0;

    if let Ok(output) = std::process::Command::new("git")
        .args(["submodule", "status"])
        .current_dir(&workspace.projects_dir)
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        uninitialized_submodules = stdout.lines().filter(|l| l.starts_with('-')).count();
        dirty_submodules = stdout.lines().filter(|l| l.starts_with('+')).count();

        if uninitialized_submodules > 0 {
            warnings.push(format!(
                "Run 'git submodule update --init' to initialize {} submodules",
                uninitialized_submodules
            ));
        }
    }

    let manifest_exists = workspace.manifest_path().exists();
    if !manifest_exists {
        warnings.push("Run 'toad manifest' to generate manifest".to_string());
    }

    let atlas_exists = workspace.atlas_path().exists();
    if !atlas_exists {
        warnings.push("Run 'toad manifest' to generate atlas".to_string());
    }

    Ok(HealthReport {
        version: env!("CARGO_PKG_VERSION").to_string(),
        mcp_installed,
        workspace_discovered: true,
        workspace_path: Some(workspace.projects_dir.clone()),
        registry_fresh,
        project_count,
        git_remotes_reachable,
        uninitialized_submodules,
        dirty_submodules,
        manifest_exists,
        atlas_exists,
        issues,
        warnings,
        diagnostics: toad_core::DiagnosticReport::new(),
    })
}
