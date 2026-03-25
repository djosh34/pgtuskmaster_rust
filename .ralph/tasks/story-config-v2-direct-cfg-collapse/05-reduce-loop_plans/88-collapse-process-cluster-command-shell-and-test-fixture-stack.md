## Plan: Collapse Process Cluster Command Shell And Test Fixture Stack

### Why this is the next verified reduction target

`src/process/cluster.rs` already had its larger launch/bookkeeping couriers collapsed in earlier slices, but the file still carries one more local duplication seam in both production and test code.

- The production command builders still rebuild the same `ProcessCommandSpec` shell over and over.
- The test module still open-codes the same config, observed snapshot, and request scaffolding across most cases.

This is a good next slice because it stays inside one owner file, reuses the existing `ProcessCommandSpec`, `ProcessObservedSnapshot`, and `ProcessIntentRequest` types, and should delete lines without inventing another DTO layer.

### Verified overlap in the current code

1. The command builders still duplicate one local shell.
   - `build_bootstrap_command(...)`
   - `build_basebackup_command(...)`
   - `build_pg_rewind_command(...)`
   - `build_promote_command(...)`
   - `build_demote_command(...)`
   - `build_start_postgres_command(...)`
   - All six rebuild the same `ProcessCommandSpec` fields:
     - binary path from `cfg.process.binaries`
     - `args`
     - `env`
     - `capture_output: cfg.logging.capture_subprocess_output`

2. The three `pg_ctl` command builders still repeat one narrower shape.
   - `build_promote_command(...)`, `build_demote_command(...)`, and `build_start_postgres_command(...)` all emit a `pg_ctl` command with:
     - `-D <data_dir>`
     - no environment variables
     - shared capture policy
   - Their only real variation is the verb-specific tail (`promote`, `stop`, `start`) plus small option differences.

3. The remote-source command builders still repeat the same source projection.
   - `build_basebackup_command(...)` and `build_pg_rewind_command(...)` both:
     - accept `&SourceConn`
     - render `source.conninfo.to_string()`
     - use `role_auth_env(&source.auth)`
     - differ only in executable path and flag names

4. The test module still open-codes the same setup stack many times.
   - `runtime_test_config_with_data_dir(...).map_err(|err| err.to_string())?` appears repeatedly across the test block.
   - `ProcessObservedSnapshot { ... managed_recovery_state: ManagedRecoverySignal::None }` is rebuilt in several tests with only the DCS payload changed.
   - `ProcessIntentRequest { id: JobId(...), intent: ... }` is rebuilt in several tests with only job-id text and intent changed.
   - The repeated `leader = MemberId("node-b".to_string())` plus quorum DCS setup also appears multiple times in the replica/basebackup cases.

### Execution plan

1. Collapse the repeated `ProcessCommandSpec` shell onto one local constructor.
   - Add one small helper in `src/process/cluster.rs` that accepts:
     - `program`
     - `args`
     - `env`
   - Keep `capture_output` ownership in that helper.
   - Rebuild all existing command builders on top of that helper.
   - Do not add a new wrapper struct or enum.

2. Collapse the `pg_ctl` command stack only if it stays smaller than the current call sites.
   - Add at most one small `pg_ctl`-specific helper for the shared `-D <data_dir>` plus common shell.
   - Use it from `build_promote_command(...)`, `build_demote_command(...)`, and `build_start_postgres_command(...)`.
   - Keep verb-specific argument order and semantics unchanged.
   - If the helper grows branchy or requires placeholder arguments, keep only step 1 and leave the pg_ctl builders direct.

3. Collapse the remote-source builder duplication.
   - Rebuild `build_basebackup_command(...)` and `build_pg_rewind_command(...)` from one small local source-command helper or one shared `conninfo`/`env` preparation path.
   - Reuse `SourceConn` and `role_auth_env(...)` as the only source-auth boundary.
   - Do not widen `SourceConn` or introduce a new source-command DTO.

4. Collapse the repeated test fixture scaffolding in the same file.
   - Add a few tiny test-only helpers in `src/process/cluster.rs` for the repeated owners:
     - runtime config creation from a data dir
     - common observed snapshot construction for `ManagedRecoverySignal::None`
     - request creation from `job id` text and `ProcessIntent`
     - leader quorum setup where the only moving part is host/port
   - Rewrite the existing tests to use those helpers instead of repeating the same struct literals.
   - Keep the helpers local to the test module and delete any helper that ends up with only one real caller after the rewrite.

5. Keep the slice flat.
   - Stay inside `src/process/cluster.rs`.
   - Do not add new public APIs.
   - Do not create a generic planner/builder abstraction; these helpers should only delete local duplication in this file.

6. Validate the slice.
   - `cargo fmt`
   - `bash .ralph/git_current_lines.sh` and verify total drops below `34131`
   - `bash .ralph/git_diff_lines_since.sh` and verify the diff improves beyond the current `+10233 -14177 diff: -3944` baseline
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`

### Guardrails

- No new structs, enums, builder objects, or callback-heavy helper layers.
- Reuse `ProcessCommandSpec`, `ProcessObservedSnapshot`, and `ProcessIntentRequest`; do not replace them.
- If the `pg_ctl` helper or remote-source helper becomes more abstract than the duplication it removes, keep the simpler direct builders and only collapse the clearly net-negative shell.
- If test helper extraction adds more lines than it removes after `cargo fmt`, switch this plan back to `TO BE VERIFIED`.

### Expected yield

- Delete one layer of repeated `ProcessCommandSpec` construction from six production builders.
- Delete one more layer of repeated `pg_ctl`/remote-source argument assembly if that remains net-negative.
- Remove a sizable chunk of repeated test setup in the `src/process/cluster.rs` test module.
- Keep the slice local to one file, which should make reduction easier to validate and easier to revert if the line count does not improve.

NOW EXECUTE
