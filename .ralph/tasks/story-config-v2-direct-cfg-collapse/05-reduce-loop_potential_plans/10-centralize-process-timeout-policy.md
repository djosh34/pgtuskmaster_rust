# Centralize process timeout policy

- Smell: timeout policy leaks into every spec as `timeout_ms`, is partly filled in planner code, and is partly defaulted again in worker code.
- Files: `src/process/jobs.rs:107`, `src/process/jobs.rs:126`, `src/process/jobs.rs:140`, `src/process/planner.rs:67`, `src/process/worker.rs:847`
- Collapse: keep timeout policy in one place, preferably by action kind plus runtime config, and delete per-spec `timeout_ms` fields unless a caller truly overrides them.
- Win: fewer duplicated defaults and less policy mixed into transport structs.
