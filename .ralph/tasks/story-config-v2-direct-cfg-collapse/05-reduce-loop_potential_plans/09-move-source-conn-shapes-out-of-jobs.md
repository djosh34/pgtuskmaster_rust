# Move source connection shapes out of jobs

- Smell: `MandatoryRoleSourceConn` and `MandatorySourceRole` live in `jobs.rs`, but they are constructed from DCS and config in `source.rs`.
- Files: `src/process/jobs.rs:113`, `src/process/jobs.rs:120`, `src/process/source.rs:21`, `src/process/planner.rs:137`
- Collapse: move the source connection DTO next to `source_from_member`, or reuse a more general postgres-source type there.
- Win: job specs stop owning DCS/config materialization details.
