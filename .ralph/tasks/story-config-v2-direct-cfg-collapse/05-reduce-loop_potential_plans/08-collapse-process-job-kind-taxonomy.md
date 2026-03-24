# Collapse process job-kind taxonomy

- Smell: `ProcessIntent`, `PostgresStartIntent`, `PostgresStartMode`, `ActiveJobKind`, and `ProcessJobKind` all encode the same classification table in different forms.
- Files: `src/process/jobs.rs:12`, `src/process/jobs.rs:72`, `src/process/jobs.rs:96`, `src/process/jobs.rs:184`, `src/process/jobs.rs:196`, `src/process/state.rs:47`
- Collapse: define one canonical process/action kind and derive the logging/state view at the edges instead of storing parallel enums.
- Win: removes repeated `match` ladders and the odd split between `StartPostgres` and `StartPrimary/Replica`.
