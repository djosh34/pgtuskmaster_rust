# Merge worker and postmaster process inspection

- Smell: `worker.rs` and `postmaster.rs` both inspect `/proc`, parse pid-related state, and identify postgres processes with separate code paths.
- Files: `src/process/worker.rs:320`, `src/process/postmaster.rs:127`, `src/process/postmaster.rs:197`
- Collapse: centralize process inspection in one module and make preflight reuse the same verification path as postmaster lookup.
- Win: removes duplicated Linux process probing logic and reduces mismatch risk.
