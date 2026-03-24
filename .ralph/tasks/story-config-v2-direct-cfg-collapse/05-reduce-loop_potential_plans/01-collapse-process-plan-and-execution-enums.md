# Collapse process plan and execution enums

- Smell: `ClusterProcessPlan` and `ProcessExecutionKind` carry almost the same variants and payloads; `tools.rs` mostly clones one into the other.
- Files: `src/process/planner.rs:18`, `src/process/state.rs:38`, `src/process/tools.rs:21`
- Collapse: keep one enum for prepared process work and build the command from that single shape.
- Win: deletes the clone-only lowering layer and the repeated job-kind mapping tied to it.
