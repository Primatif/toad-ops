use std::path::Path;
use toad_core::ProjectStack;
use walkdir::WalkDir;

#[derive(Debug, Clone, Default)]
pub struct ProjectStats {
    pub total_bytes: u64,
    pub artifact_bytes: u64,
    pub source_bytes: u64,
    pub bloat_index: f64,
}

/// Calculates disk usage statistics for a project.
pub fn calculate_project_stats(path: &Path, stack: &ProjectStack) -> ProjectStats {
    let mut stats = ProjectStats::default();

    let artifact_dirs = get_artifact_dirs(stack);

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_file() {
                let size = metadata.len();
                stats.total_bytes += size;

                // Check if this file is inside an artifact directory
                if is_artifact(entry.path(), path, &artifact_dirs) {
                    stats.artifact_bytes += size;
                } else {
                    stats.source_bytes += size;
                }
            }
        }
    }

    if stats.total_bytes > 0 {
        stats.bloat_index = (stats.artifact_bytes as f64 / stats.total_bytes as f64) * 100.0;
    }

    stats
}

fn get_artifact_dirs(stack: &ProjectStack) -> Vec<&'static str> {
    match stack {
        ProjectStack::Rust => vec!["target"],
        ProjectStack::NodeJS => vec!["node_modules", ".next", "dist", "build", "out"],
        ProjectStack::Python => vec![
            "__pycache__",
            ".venv",
            "venv",
            ".pytest_cache",
            "build",
            "dist",
        ],
        ProjectStack::Go => vec!["bin", "vendor"], // vendor is controversial but often large
        ProjectStack::Monorepo => vec!["node_modules", "target", ".turbo", "dist"],
        ProjectStack::Generic => vec!["node_modules", "target", "build", "dist"],
    }
}

fn is_artifact(file_path: &Path, project_root: &Path, artifact_dirs: &[&str]) -> bool {
    // Relative path from project root
    if let Ok(rel_path) = file_path.strip_prefix(project_root) {
        for component in rel_path.components() {
            if let Some(comp_str) = component.as_os_str().to_str() {
                if artifact_dirs.contains(&comp_str) {
                    return true;
                }
            }
        }
    }
    false
}

/// Formats bytes into a human-readable string.
pub fn format_size(bytes: u64) -> String {
    let kb = 1024.0;
    let mb = kb * 1024.0;
    let gb = mb * 1024.0;
    let tb = gb * 1024.0;

    let b = bytes as f64;

    if b < kb {
        format!("{} B", bytes)
    } else if b < mb {
        format!("{:.2} KB", b / kb)
    } else if b < gb {
        format!("{:.2} MB", b / mb)
    } else if b < tb {
        format!("{:.2} GB", b / gb)
    } else {
        format!("{:.2} TB", b / tb)
    }
}
