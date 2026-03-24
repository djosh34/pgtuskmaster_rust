## Plan: Delete Managed Postgres Path Bundle DTO

### Why this reduction target

`src/postgres_managed.rs` still projects deterministic file paths into `ManagedPostgresConfig` even though the production caller only needs the side effects:

- `src/process/cluster.rs` calls `materialize_managed_postgres_config(...)` from `materialize_start_config(...)` and immediately discards the returned bundle with `.map(|_| ())`.
- The bundle fields are all derivable from `cfg.postgres.*`, `cfg.postgres.data_dir`, and the existing managed-file constants.
- Most remaining field reads are inside `src/postgres_managed.rs` tests, which live in the same module and can derive the same paths directly instead of forcing production code to return a courier struct.

That is classic overabstraction. The module owns file materialization and signal mutations, but it still exports a DTO that mostly restates where those side effects landed.

### Current overlap already verified

- `ManagedPostgresConfig` contains only file paths: `postgresql_conf_path`, `hba_path`, `ident_path`, `standby_passfile_path`, `standby_signal_path`, `recovery_signal_path`, `postgresql_auto_conf_path`, and `quarantined_postgresql_auto_conf_path`.
- `materialize_managed_postgres_config(...)` already knows every one of those paths before it performs any writes.
- The only non-test runtime caller is `src/process/cluster.rs`, and it does not use any field from the returned struct.
- There is already one canonical artifact-path helper, `managed_standby_passfile_path(...)`; the rest of the managed artifact paths are still inlined as repeated joins inside the materializer and tests.

### Execution plan

1. Collapse the runtime API to side effects only.
   - Change `materialize_managed_postgres_config(...)` to return `Result<(), ManagedPostgresError>`.
   - Delete `ManagedPostgresConfig` entirely.
   - Remove the `map(|_| ())` call in `src/process/cluster.rs` and propagate the simplified result directly.

2. Keep canonical managed-artifact paths in one place.
   - Add small path helpers in `src/postgres_managed.rs` for the managed files that are currently recomputed inline: managed `postgresql.conf`, active and quarantined `postgresql.auto.conf`, and the recovery signal files.
   - Reuse those helpers inside the materializer so path ownership stays in the same module.
   - Keep helper visibility as narrow as possible; do not create a new public test API unless an external runtime caller truly needs it.

3. Rewrite module tests around canonical paths instead of the deleted DTO.
   - In `src/postgres_managed.rs` tests, read managed files back through the helper-derived paths or direct `data_dir.join(...)` lookups instead of `managed.*_path` fields.
   - Replace `standby_passfile_path: Option<PathBuf>` assertions with existence checks against `managed_standby_passfile_path(...)`.
   - Update the real clone/start integration test to pass the derived managed config path into `postgres -c config_file=...` and to inspect the derived standby passfile path directly.

4. Validate the reduction.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current `+7202 -10007 diff: -2805` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.

### Guardrails

- Do not replace `ManagedPostgresConfig` with another bundle or test-only wrapper.
- Do not broaden visibility just to keep tests convenient; the tests live in the same module and can use private helpers.
- If deleting the DTO exposes a second boundary that still forces duplicate path logic across callers, stop and switch the plan back to `TO BE VERIFIED` instead of inventing another courier type.

NOW EXECUTE
