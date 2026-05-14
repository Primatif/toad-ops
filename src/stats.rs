use std::collections::HashSet;
use std::path::Path;
use toad_core::{AnalyticsReport, ProjectAnalytics, ProjectDetail};
use walkdir::WalkDir;

#[derive(Debug, Clone, Default)]
pub struct ProjectStats {
    pub total_bytes: u64,
    pub artifact_bytes: u64,
    pub source_bytes: u64,
    pub bloat_index: f64,
}

/// Generates a structured analytics report for a set of projects.
pub fn generate_analytics_report(
    projects: &[ProjectDetail],
    query: Option<&str>,
    tag: Option<&str>,
) -> AnalyticsReport {
    let mut offenders = Vec::new();
    let mut total_usage = 0;
    let mut total_artifacts = 0;

    for p in projects {
        if let Some(q) = query {
            if !p.name.to_lowercase().contains(&q.to_lowercase()) {
                continue;
            }
        }

        if let Some(t) = tag {
            let target = if t.starts_with('#') {
                t.to_string()
            } else {
                format!("#{}", t)
            };
            if !p.tags.contains(&target) {
                continue;
            }
        }

        let stats = calculate_project_stats(&p.path, &artifact_set_to_hashset(&p.artifact_dirs));

        total_usage += stats.total_bytes;
        total_artifacts += stats.artifact_bytes;

        offenders.push(ProjectAnalytics {
            name: p.name.clone(),
            total_size: stats.total_bytes,
            artifact_size: stats.artifact_bytes,
            bloat_percentage: stats.bloat_index,
            activity: p.activity.clone(),
        });
    }

    // Sort offenders by size (descending)
    offenders.sort_by(|a, b| b.total_size.cmp(&a.total_size));

    AnalyticsReport {
        total_usage,
        total_artifacts,
        offenders,
    }
}

fn artifact_set_to_hashset(dirs: &[String]) -> std::collections::HashSet<&str> {
    dirs.iter().map(|s| s.as_str()).collect()
}

/// Calculates disk usage statistics for a project.
pub fn calculate_project_stats(path: &Path, artifact_dirs: &HashSet<&str>) -> ProjectStats {
    let mut stats = ProjectStats::default();

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_file() {
                let size = metadata.len();
                stats.total_bytes += size;

                // Check if this file is inside an artifact directory
                if is_artifact(entry.path(), path, artifact_dirs) {
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

fn is_artifact(file_path: &Path, project_root: &Path, artifact_dirs: &HashSet<&str>) -> bool {
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
