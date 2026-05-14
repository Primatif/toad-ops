use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use toad_core::{
    AiReadiness, DebtIndicators, DependencyGraph, DependencyNode, HealthScore, PatternMetrics,
    ProjectDetail, ProjectInsight, ToadResult, TrendReport, VelocityMetrics,
};

pub fn analyze_dependencies(projects: &[ProjectDetail]) -> ToadResult<DependencyGraph> {
    let mut nodes = HashMap::new();
    let mut crate_to_project = HashMap::new();

    // 1. Initialize nodes and mapping
    for p in projects {
        nodes.insert(
            p.name.clone(),
            DependencyNode {
                name: p.name.clone(),
                version: None,
                depth: 0,
                dependents: Vec::new(),
                dependencies: Vec::new(),
            },
        );

        // Heuristic: Read Cargo.toml to get the actual crate name
        let cargo_path = p.path.join("Cargo.toml");
        if cargo_path.exists() {
            if let Ok(content) = fs::read_to_string(&cargo_path) {
                if let Some(line) = content.lines().find(|l| l.trim().starts_with("name =")) {
                    if let Some(crate_name) = line.split('"').nth(1) {
                        crate_to_project.insert(crate_name.to_string(), p.name.clone());
                    }
                }
            }
        }
    }

    // 2. Extract dependencies
    for p in projects {
        let cargo_path = p.path.join("Cargo.toml");
        if cargo_path.exists() {
            // A. Try cargo_metadata for precision
            if let Ok(metadata) = cargo_metadata::MetadataCommand::new()
                .manifest_path(&cargo_path)
                .no_deps()
                .exec()
            {
                if let Some(package) = metadata.packages.iter().find(|pkg| {
                    crate_to_project.get(pkg.name.as_str()) == Some(&p.name)
                        || pkg
                            .manifest_path
                            .parent()
                            .map(|parent| p.path.ends_with(parent.as_std_path()))
                            .unwrap_or(false)
                }) {
                    if let Some(node) = nodes.get_mut(&p.name) {
                        node.version = Some(package.version.to_string());

                        for dep in &package.dependencies {
                            // Map crate name back to project name
                            if let Some(dep_proj_name) = crate_to_project.get(dep.name.as_str()) {
                                if !node.dependencies.contains(dep_proj_name) {
                                    node.dependencies.push(dep_proj_name.clone());
                                }
                            }
                        }
                    }
                }
            } else {
                // B. Fallback to naive string matching if cargo_metadata fails
                if let Ok(content) = fs::read_to_string(&cargo_path) {
                    for (crate_name, proj_name) in &crate_to_project {
                        if proj_name == &p.name {
                            continue;
                        }
                        let search_pattern = format!("{} =", crate_name);
                        if content.contains(&search_pattern) {
                            if let Some(node) = nodes.get_mut(&p.name) {
                                if !node.dependencies.contains(proj_name) {
                                    node.dependencies.push(proj_name.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Populate dependents
    let mut dependents_map: HashMap<String, Vec<String>> = HashMap::new();
    let nodes_clone = nodes.clone();
    for (name, node) in &nodes_clone {
        for dep in &node.dependencies {
            dependents_map
                .entry(dep.clone())
                .or_default()
                .push(name.clone());
        }
    }

    for (name, deps) in dependents_map {
        if let Some(node) = nodes.get_mut(&name) {
            node.dependents = deps;
        }
    }

    // 4. Calculate critical path (most dependents)
    let mut critical_path: Vec<String> = nodes.keys().cloned().collect();
    critical_path.sort_by(|a, b| {
        let count_a = nodes.get(a).map(|n| n.dependents.len()).unwrap_or(0);
        let count_b = nodes.get(b).map(|n| n.dependents.len()).unwrap_or(0);
        count_b.cmp(&count_a)
    });

    // 5. Orphaned projects (no dependents AND no dependencies)
    let orphaned_projects = nodes
        .values()
        .filter(|n| n.dependents.is_empty() && n.dependencies.is_empty())
        .map(|n| n.name.clone())
        .collect();

    // 6. Detect cycles
    let circular_dependencies = detect_cycles(&nodes);

    Ok(DependencyGraph {
        nodes,
        critical_path,
        orphaned_projects,
        circular_dependencies,
    })
}

fn detect_cycles(nodes: &HashMap<String, DependencyNode>) -> Vec<Vec<String>> {
    let mut cycles = Vec::new();
    let mut visited = HashSet::new();
    let mut on_stack = HashSet::new();
    let mut stack = Vec::new();

    for name in nodes.keys() {
        if !visited.contains(name) {
            dfs_cycle(
                name,
                nodes,
                &mut visited,
                &mut on_stack,
                &mut stack,
                &mut cycles,
            );
        }
    }

    cycles
}

fn dfs_cycle(
    current: &String,
    nodes: &HashMap<String, DependencyNode>,
    visited: &mut HashSet<String>,
    on_stack: &mut HashSet<String>,
    stack: &mut Vec<String>,
    cycles: &mut Vec<Vec<String>>,
) {
    visited.insert(current.clone());
    on_stack.insert(current.clone());
    stack.push(current.clone());

    if let Some(node) = nodes.get(current) {
        for neighbor in &node.dependencies {
            if on_stack.contains(neighbor) {
                // Found a cycle
                if let Some(pos) = stack.iter().position(|x| x == neighbor) {
                    cycles.push(stack[pos..].to_vec());
                }
            } else if !visited.contains(neighbor) {
                dfs_cycle(neighbor, nodes, visited, on_stack, stack, cycles);
            }
        }
    }

    on_stack.remove(current);
    stack.pop();
}

pub fn analyze_velocity(path: &Path, days: u32) -> ToadResult<VelocityMetrics> {
    // This will need git history
    // For now, return mock data or basic stats
    let commit_res = toad_git::run_git(
        path,
        &[
            "log",
            &format!("--since={} days ago", days),
            "--pretty=format:%an",
        ],
        "internal",
    )?;
    let authors: HashSet<String> = commit_res.stdout.lines().map(|s| s.to_string()).collect();
    let commit_count = commit_res.stdout.lines().count();

    let stats_res = toad_git::run_git(
        path,
        &["log", &format!("--since={} days ago", days), "--shortstat"],
        "internal",
    )?;
    // Parse "+1,234 lines, -567 lines"
    let mut lines_added = 0;
    let mut lines_removed = 0;

    for line in stats_res.stdout.lines() {
        if line.contains("insertion") {
            // Very naive parsing
            let parts: Vec<&str> = line.split(',').collect();
            for p in parts {
                if p.contains("insertion") {
                    lines_added += p
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(0);
                } else if p.contains("deletion") {
                    lines_removed += p
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(0);
                }
            }
        }
    }

    Ok(VelocityMetrics {
        commit_count,
        lines_added,
        lines_removed,
        active_contributors: authors.into_iter().collect(),
        local_deployment_frequency: 0.0,
        local_lead_time_for_changes: 0.0,
        trend: "Stable".to_string(),
    })
}

pub fn analyze_debt(path: &Path) -> ToadResult<DebtIndicators> {
    let mut todo_count = 0;
    let mut fixme_count = 0;
    let mut hack_count = 0;
    let mut large_files = Vec::new();

    // Naive scan for comments
    let walker = ignore::WalkBuilder::new(path)
        .hidden(false)
        .git_ignore(true)
        .build();

    let noise_extensions = [
        "png", "jpg", "jpeg", "gif", "mov", "mp4", "lock", "sum", "bin", "exe", "wasm", "pdf",
    ];

    for entry in walker.flatten() {
        let p = entry.path();
        if p.is_file() {
            // Skip binary and noise
            if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                if noise_extensions.contains(&ext.to_lowercase().as_str()) {
                    continue;
                }
            }

            // Read as string, skip if not valid UTF-8
            if let Ok(content) = fs::read_to_string(p) {
                todo_count += content.to_lowercase().matches("todo").count();
                fixme_count += content.to_lowercase().matches("fixme").count();
                hack_count += content.to_lowercase().matches("hack").count();

                let line_count = content.lines().count();
                if line_count > 700 {
                    large_files.push(p.to_string_lossy().into_owned());
                }
            }
        }
    }

    Ok(DebtIndicators {
        todo_count,
        fixme_count,
        hack_count,
        debt_score: calculate_debt_score(todo_count, fixme_count, large_files.len()),
        large_files,
        test_coverage: None,
        outdated_dependencies: Vec::new(),
        churn_complexity_risk: Vec::new(),
    })
}

fn calculate_debt_score(todo: usize, fixme: usize, large: usize) -> f32 {
    let mut score = 10.0;
    score -= (todo as f32 * 0.1).min(2.0);
    score -= (fixme as f32 * 0.5).min(3.0);
    score -= (large as f32 * 1.0).min(4.0);
    score.max(0.0)
}

pub fn calculate_health_score(p: &ProjectDetail) -> ToadResult<HealthScore> {
    let vcs_score = if p.vcs_status == toad_core::VcsStatus::Clean {
        20
    } else {
        10
    };
    let activity_score = if p.activity == toad_core::ActivityTier::Active {
        15
    } else {
        5
    };
    let doc_score = if p.essence.is_some() { 15 } else { 0 };

    // Naive quality score
    let mut quality_score = 15;
    if p.total_size > 0 && p.bloat_index > 0.5 {
        quality_score -= 5;
    }

    let total = vcs_score + activity_score + doc_score + quality_score;

    Ok(HealthScore {
        total: total as u32,
        vcs_cleanliness: vcs_score as u32,
        test_coverage: 0,
        documentation: doc_score as u32,
        activity: activity_score as u32,
        dependencies: 10,
        code_quality: quality_score as u32,
        ai_readiness: 80,
    })
}

pub fn analyze_ai_readiness(p: &ProjectDetail) -> ToadResult<AiReadiness> {
    let mut score = 70.0;
    let mut factors = HashMap::new();
    let mut recommendations = Vec::new();

    if p.essence.is_some() {
        score += 10.0;
        factors.insert("Documentation".to_string(), 10.0);
    } else {
        recommendations.push("Add a README.md with clear project purpose.".to_string());
    }

    if !p.dna.roles.is_empty() {
        score += 10.0;
        factors.insert("Clear Roles".to_string(), 10.0);
    } else {
        recommendations.push("Structure directories to reflect clear architectural roles (e.g. models, api, services).".to_string());
    }

    Ok(AiReadiness {
        score,
        factors,
        recommendations,
    })
}

pub fn analyze_trends(_path: &Path, _days: u32) -> ToadResult<TrendReport> {
    // v1.2.0 planned
    Ok(TrendReport {
        points: Vec::new(),
        health_trend: "Historical trends analysis requires persistent state (v1.2.0 planned)."
            .to_string(),
        disk_trend: "N/A".to_string(),
        activity_trend: "N/A".to_string(),
    })
}

pub fn analyze_patterns(_projects: &[ProjectDetail]) -> ToadResult<PatternMetrics> {
    // v1.2.0 planned
    Ok(PatternMetrics {
        common_dependencies: Vec::new(),
        error_handling_consistency: 0.0,
        naming_convention_compliance: 0.0,
        architectural_violations: Vec::new(),
    })
}

pub fn generate_insights(projects: &[ProjectDetail]) -> ToadResult<Vec<ProjectInsight>> {
    let mut insights = Vec::new();

    for p in projects {
        if p.vcs_status == toad_core::VcsStatus::Dirty {
            insights.push(ProjectInsight {
                title: format!("Dirty VCS in {}", p.name),
                description: "Uncommitted changes can lead to context drift.".to_string(),
                severity: "Medium".to_string(),
                action_item: "Commit or stash changes.".to_string(),
            });
        }

        if p.total_size > 1_000_000_000 {
            // > 1GB
            insights.push(ProjectInsight {
                title: format!("Large Storage Footprint: {}", p.name),
                description: "Project exceeds 1GB, check for build artifacts or logs.".to_string(),
                severity: "Low".to_string(),
                action_item: "Run 'toad clean' on this project.".to_string(),
            });
        }
    }

    Ok(insights)
}
