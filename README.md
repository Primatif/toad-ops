# toad-ops

Batch operations, analytics, and safety engine for the
[Primatif Toad](https://github.com/Primatif/Primatif_Toad) ecosystem.

## What It Does

`toad-ops` handles **bulk operations and ecosystem analytics**. It provides the
execution engine for running commands across multiple projects, cleaning build
artifacts, computing disk usage statistics, and managing audit trails.

- **Batch Execution** — `execute_batch_operation()` runs shell commands across
  multiple projects in parallel using `rayon`, with fail-fast support and
  timeout protection.
- **Artifact Cleaning** — `clean::execute_batch_clean()` removes build
  artifacts (`target/`, `node_modules/`, etc.) across projects, returning a
  structured `BatchCleanReport` with bytes reclaimed.
- **Analytics** — `stats::generate_analytics_report()` computes per-project
  disk usage, bloat index, and ecosystem totals — filterable by query and tag.
- **Safety** — `safety` module provides pre-flight checks and stack-mismatch
  detection before batch operations (e.g., warning before running `cargo` on a
  non-Rust project).
- **Audit Logging** — `audit` module records batch operations with timestamps,
  user, target counts, and success/fail metrics.
- **Shell Execution** — `shell` module provides sandboxed command execution
  with timeout and output capture.
- **Workflows** — `workflow` module manages custom workflow definitions.

## Role in the Ecosystem

`toad-ops` is the operations layer. It depends only on `toad-core` for data
models and report types. It is consumed by `toad-discovery` (for stats during
scanning), the CLI (`toad do`, `toad clean`, `toad stats`), and the MCP server
(`get_project_stats` tool).

```text
toad-core ── toad-ops ──┬── toad-discovery
                        ├── bin/toad (toad do, clean, stats)
                        └── bin/toad-mcp (get_project_stats)
```

## License

BUSL-1.1
