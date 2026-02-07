use crate::stats::calculate_project_stats;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub struct CleanResult {
    pub bytes_reclaimed: u64,
    pub files_removed: usize,
    pub errors: Vec<String>,
}

/// Reserved paths that should NEVER be cleaned, even if a strategy claims them.
pub const RESERVED_PATHS: &[&str] = &[
    ".git",
    ".gitignore",
    ".github",
    "src",
    "lib",
    "bin",
    "main.rs",
    "README.md",
    "README.markdown",
    "readme.md",
    "LICENSE",
    "Cargo.toml",
    "package.json",
    "go.mod",
    "pyproject.toml",
    "requirements.txt",
    "Justfile",
    "Makefile",
];

/// Cleans build artifacts from a project directory based on the provided list of artifact directories.
pub fn clean_project(path: &Path, artifact_dirs: &[String], dry_run: bool) -> Result<CleanResult> {
    let mut result = CleanResult {
        bytes_reclaimed: 0,
        files_removed: 0,
        errors: Vec::new(),
    };

    for artifact in artifact_dirs {
        // Safety check: Don't allow cleaning reserved paths
        if RESERVED_PATHS.contains(&artifact.as_str()) {
            result
                .errors
                .push(format!("Skipping reserved path: {}", artifact));
            continue;
        }

        let target_path = path.join(artifact);
        if !target_path.exists() {
            continue;
        }

        // Calculate size before deletion
        let stats = calculate_project_stats(&target_path, &[]); // Pass empty artifact list to count everything as "source" for size calc
        let size = stats.total_bytes;

        if dry_run {
            result.bytes_reclaimed += size;
            result.files_removed += 1; // Representing the directory as one "unit" for now
        } else {
            match fs::remove_dir_all(&target_path) {
                Ok(_) => {
                    result.bytes_reclaimed += size;
                    result.files_removed += 1;
                }
                Err(e) => {
                    result
                        .errors
                        .push(format!("Failed to remove {}: {}", artifact, e));
                }
            }
        }
    }

    Ok(result)
}
