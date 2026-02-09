use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use toad_core::{BatchOperationReport, OperationResult, ProjectDetail};

pub mod audit;
pub mod clean;
pub mod safety;
pub mod shell;
pub mod stats;
pub mod workflow;

/// Executes a shell command across a list of projects in parallel.
pub fn execute_batch_operation(
    projects: &[ProjectDetail],
    command: &str,
    fail_fast: bool,
) -> BatchOperationReport {
    let failed = AtomicBool::new(false);

    let results: Vec<OperationResult> = projects
        .par_iter()
        .map(|p| {
            if fail_fast && failed.load(Ordering::Relaxed) {
                return OperationResult {
                    project_name: p.name.clone(),
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: "Skipped due to previous failure".to_string(),
                    timed_out: false,
                };
            }

            let res = shell::run_in_dir(&p.path, command, Duration::from_secs(30));

            let outcome = match res {
                Ok(op_res) => {
                    if op_res.exit_code != 0 {
                        failed.store(true, Ordering::Relaxed);
                    }
                    OperationResult {
                        project_name: p.name.clone(),
                        exit_code: op_res.exit_code,
                        stdout: op_res.stdout,
                        stderr: op_res.stderr,
                        timed_out: op_res.timed_out,
                    }
                }
                Err(e) => {
                    failed.store(true, Ordering::Relaxed);
                    OperationResult {
                        project_name: p.name.clone(),
                        exit_code: -1,
                        stdout: String::new(),
                        stderr: format!("Error: {}", e),
                        timed_out: false,
                    }
                }
            };

            outcome
        })
        .collect();

    let mut success_count = 0;
    let mut fail_count = 0;
    let mut skip_count = 0;

    for r in &results {
        if r.exit_code == 0 {
            success_count += 1;
        } else if r.stderr == "Skipped due to previous failure" {
            skip_count += 1;
        } else {
            fail_count += 1;
        }
    }

    BatchOperationReport {
        command: command.to_string(),
        results,
        success_count,
        fail_count,
        skip_count,
    }
}

#[cfg(test)]
mod tests;
