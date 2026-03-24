# Extract start-postgres preflight from worker

- Smell: `worker.rs` owns postgres pid checks, socket lock parsing, stale file cleanup, and process start orchestration together.
- Files: `src/process/worker.rs:320`, `src/process/worker.rs:363`, `src/process/worker.rs:403`, `src/process/worker.rs:455`
- Collapse: move start preflight into a dedicated module or into postmaster/process-prep code and leave `worker.rs` with orchestration only.
- Win: cuts a large non-worker concern out of the worker loop.
