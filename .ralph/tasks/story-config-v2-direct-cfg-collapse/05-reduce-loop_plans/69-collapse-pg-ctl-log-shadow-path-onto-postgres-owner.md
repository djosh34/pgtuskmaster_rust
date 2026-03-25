## Plan: Collapse pg_ctl Log Shadow Path Onto Postgres Owner

### Why this is the next reduction target

`pg_ctl` startup log ownership is split across two config branches today.

- Process startup writes to `cfg.postgres.log_file`.
- Postgres ingest tails `cfg.logging.postgres_pg_ctl_log`.

Those are the same operational file, but the config model carries them twice and even gives them different defaults. That is a bad boundary: the process owner should own the path it writes, and ingest should read that authoritative path instead of carrying a second shadow field.

### Current overlap already verified

- `src/process/cluster.rs:281-291` builds `pg_ctl ... -l <cfg.postgres.log_file>`.
- `src/logging/postgres_ingest.rs:275-281` initializes the `pg_ctl` tailer from `cfg.logging.postgres_pg_ctl_log`.
- `src/config_v2/parser/load_config.rs:174-186` parses `logging.postgres.pg_ctl_log_file` as a second path and defaults it to `logging.postgres.log_dir.join("pg_ctl.log")`.
- `src/config_v2/parser/load_config.rs:283-344` repeats that split ownership in `load_runtime_test_config_from_paths()`: `postgres.log_file` defaults to `working_root.join("postgres.log")`, while `logging.postgres_pg_ctl_log` defaults to `postgres_log_dir.join("pg_ctl.log")`.
- `src/logging/postgres_ingest.rs:1581-1582` has to manually force the two fields back together in the real `pg_ctl` ingest test.
- `src/config_v2/parser/private_schema.rs:513-526`, `src/config_v2/types.rs:148-161`, and `docs/src/reference/runtime-configuration.md:338` all preserve the extra shadow field as public/runtime surface area.

### Execution plan

1. Delete the shadow path from the config schema and runtime types.
   - Remove `pg_ctl_log_file` from `raw::PostgresLoggingConfig`.
   - Remove `postgres_pg_ctl_log` from `LoggingConfig`.
   - Do not replace them with another alias field.

2. Move `pg_ctl` log ownership fully onto `postgres.paths.log_file`.
   - In `src/logging/postgres_ingest.rs`, construct the `pg_ctl` tailer from `cfg.postgres.log_file`.
   - Keep `logging.postgres.log_dir` as the separate owner for collector files; only the `pg_ctl` path moves.

3. Collapse parsing and fixture defaults onto the single owner.
   - Stop parsing/normalizing `logging.postgres.pg_ctl_log_file` in `src/config_v2/parser/load_config.rs`.
   - Keep `postgres.paths.log_file` as the sole defaulted path for the startup log.
   - Align `load_runtime_test_config_from_paths()` with the same single-path ownership so test fixtures stop starting from split defaults.

4. Delete leaf-level re-glue in tests and docs.
   - Remove the manual `cfg.logging.postgres_pg_ctl_log = ...` plus mirror assignment in the `pg_ctl` ingest test.
   - Update parser tests, fixture renderers, and docs so they no longer mention `logging.postgres.pg_ctl_log_file`.
   - Adjust any assertions that currently observe the removed field.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require the diff to remain net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- If you find a real scenario that requires ingest to tail a different `pg_ctl` path than the one process startup writes, switch this plan back to `TO BE VERIFIED` instead of smuggling the split path back under a new name.
- Do not add compatibility parsing or deprecated aliases; this repo is greenfield and the goal is code deletion.
- Keep `logging.postgres.log_dir` intact unless the implementation proves it can also be collapsed without mixing `pg_ctl` output with collector output semantics.

### Expected yield

- Remove one raw schema field, one runtime field, one parser normalization branch, one duplicated test override, and the matching docs surface.
- Fix the current default mismatch where startup writes one file but ingest defaults to tailing another.
- Reduce config and test code while tightening ownership to the module that actually writes the file.

NOW EXECUTE
