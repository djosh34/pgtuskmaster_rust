# Delete process cluster shell

- Smell: `ProcessCluster` mainly pipes planner -> session -> tools and wraps the same `ProcessError` three times.
- Files: `src/process/cluster.rs:25`, `src/process/cluster.rs:53`, `src/process/cluster.rs:89`
- Collapse: replace it with one `prepare_process_launch(...)` function and inline stage context into the returned error.
- Win: likely deletes most of `src/process/cluster.rs` without losing behavior.
