use crate::stats::calculate_project_stats;
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use toad_core::{BatchCleanReport, CleanResult, ProgressReporter, ProjectDetail, ToadResult};

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

pub fn clean_project(
    path: &Path,
    artifact_dirs: &[String],
    dry_run: bool,
) -> ToadResult<CleanResult> {
    let mut result = CleanResult {
        bytes_reclaimed: 0,
        files_removed: 0,
        errors: Vec::new(),
    };

    for artifact in artifact_dirs {
        let artifact_lower = artifact.to_lowercase();
        if RESERVED_PATHS
            .iter()
            .any(|p| p.to_lowercase() == artifact_lower)
        {
            result
                .errors
                .push(format!("Skipping reserved path: {}", artifact));
            continue;
        }

        let target_path = path.join(artifact);
        if !target_path.exists() {
            continue;
        }

        let empty_set = std::collections::HashSet::new();
        let stats = calculate_project_stats(&target_path, &empty_set);
        let size = stats.total_bytes;

        if dry_run {
            result.bytes_reclaimed += size;
            result.files_removed += 1;
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

pub fn execute_batch_clean(
    targets: &[ProjectDetail],
    reporter: &dyn ProgressReporter,
) -> BatchCleanReport {
    reporter.set_length(targets.len() as u64);

    let results: Vec<(String, ToadResult<CleanResult>)> = targets
        .par_iter()
        .map(|project| {
            let res = clean_project(&project.path, &project.artifact_dirs, false);
            reporter.inc(1);
            (project.name.clone(), res)
        })
        .collect();

    let mut total_reclaimed = 0;
    let mut success_count = 0;
    let mut fail_count = 0;

    for (_, outcome) in &results {
        match outcome {
            Ok(res) => {
                if res.errors.is_empty() {
                    success_count += 1;
                } else {
                    fail_count += 1;
                }
                total_reclaimed += res.bytes_reclaimed;
            }
            Err(_) => {
                fail_count += 1;
            }
        }
    }

    BatchCleanReport {
        results,
        total_reclaimed,
        success_count,
        fail_count,
    }
}
