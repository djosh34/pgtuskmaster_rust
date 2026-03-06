## Task: Make `pgtm.postgresql.conf` the only startup config entrypoint and remove generic `-c` GUC injection <status>done</status> <passes>true</passes> <priority>high</priority>

<blocked_by>01-task-remove-backup-config-and-process-surface,02-task-remove-runtime-restore-bootstrap-and-archive-helper-wiring,04-task-remove-backup-harness-installers-and-gate-selection,05-task-remove-backup-docs-and-obsolete-task-artifacts</blocked_by>
<blocked_by>01-task-introduce-a-typed-managed-postgres-conf-model-and-serializer</blocked_by>

<description>
**Goal:** Make Postgres always start through `config_file=<abs path to PGDATA/pgtm.postgresql.conf>` and remove the generic startup path that injects arbitrary repeated `-c key=value` settings through `pg_ctl -o`.
This task establishes the actual runtime ownership boundary promised by the new managed-config model. The higher-order goal is to make `SHOW config_file;` point to one authoritative pgtuskmaster-owned file on every managed start path.

**Scope:**
- Change normal startup, resume, and HA-driven starts so `pgtm.postgresql.conf` is always the authoritative config file.
- Remove generic `StartPostgresSpec.extra_postgres_settings`.
- Remove generic `host`, `port`, and `socket_dir` startup flag injection as a configuration mechanism; these must come from the typed managed config and its serializer.
- Narrow the process-layer startup contract so it carries only what the process runner actually needs to start Postgres under pgtuskmaster ownership.
- Preserve `pg_ctl -l <log_file>` and other non-config operational arguments where still needed, but do not use `pg_ctl -o` as a generic config transport anymore.

**Execution reconciliation on 2026-03-07:**
- The repository already implements the core startup-contract migration this task describes.
- `src/process/jobs.rs` now carries a narrowed `StartPostgresSpec` with `data_dir`, `config_file`, `log_file`, `wait_seconds`, and `timeout_ms`.
- `src/process/worker.rs` now renders managed startup as `pg_ctl ... -o "-c config_file=<abs path>" ...` instead of transporting arbitrary repeated startup GUCs.
- `src/runtime/node.rs` and `src/ha/process_dispatch.rs` both materialize managed config immediately before start and pass the authoritative `pgtm.postgresql.conf` path into `StartPostgresSpec`.
- `src/postgres_managed.rs` and `src/postgres_managed_conf.rs` already own `pgtm.postgresql.conf` plus the managed HBA, ident, TLS, and standby-signal side files.
- `src/test_harness/ha_e2e/startup.rs` already proves `SHOW config_file;`, `SHOW hba_file;`, and `SHOW ident_file;` point at the managed files.
- Direct PostgreSQL harness helpers that intentionally append to `postgresql.conf` outside pgtuskmaster ownership remain a separate task-boundary concern and should be handled under task 04 rather than silently broadening task 02.

**Expected outcome:**
- Every pgtuskmaster-managed Postgres start path uses `config_file=<abs path to pgtm.postgresql.conf>`.
- The process layer no longer transports a generic settings map for Postgres configuration.
- Postgres listen host, port, socket dir, HBA file, ident file, and TLS file paths come from the managed config file, not startup flag injection.
- This task's artifact no longer misstates the live repository state or conflates managed startup ownership with direct-Postgres harness helpers.

</description>

<acceptance_criteria>
- [x] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [x] `StartPostgresSpec` no longer contains a generic config bag for arbitrary Postgres GUC injection
- [x] Runtime startup and HA start paths always materialize and use `config_file=<abs path to pgtm.postgresql.conf>`
- [x] Generic repeated `-c key=value` injection is removed as the main configuration path for Postgres startup
- [x] `SHOW config_file;` real-binary tests prove the active config file is `PGDATA/pgtm.postgresql.conf`
- [x] No pgtuskmaster-managed start path still depends on the default `postgresql.conf` as the active parent config
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

---

### Exhaustive checklist (must be treated as part of acceptance)

- [x] `src/process/jobs.rs`
  - [x] Replace `StartPostgresSpec.extra_postgres_settings`.
  - [x] Remove any config-transport fields that only existed to support startup flag injection.
  - [x] Keep only the minimal operational fields needed once config ownership moves to `pgtm.postgresql.conf`.
- [x] `src/process/worker.rs`
  - [x] Remove the loop that renders repeated `-c key=value` from a generic settings map.
  - [x] Replace it with the narrow startup contract that points Postgres at the authoritative managed config file.
  - [x] Update command-rendering tests accordingly.
- [x] `src/runtime/node.rs`
  - [x] Make every managed start path materialize the authoritative managed config and pass only the new startup contract.
  - [x] Remove reliance on default `postgresql.conf` as an active configuration parent for pgtuskmaster-managed starts.
- [x] `src/ha/process_dispatch.rs`
  - [x] Make HA-driven `StartPostgres` requests use the same authoritative managed config path as runtime startup.
  - [x] Remove any remaining per-start config-bag plumbing.
- [x] `src/postgres_managed.rs`
  - [x] Ensure the authoritative managed config path is always available for managed starts.
  - [x] Do not leave backup-era conditional ownership around `config_file`.
- [x] `src/test_harness/ha_e2e/startup.rs`
  - [x] Add or update real-binary assertions for `SHOW config_file;`.
  - [x] Keep existing `SHOW hba_file;` and `SHOW ident_file;` assertions aligned with the new ownership model.
- [x] `src/process/worker.rs` tests
  - [x] Remove tests that assert generic `-c hba_file=...` / `-c ident_file=...` rendering.
  - [x] Add tests that assert the narrowed startup contract and the exact `config_file` entrypoint.

---

### Execution plan written on 2026-03-07 by Codex

This task file is stale relative to the current repository state. Before changing code, the execution pass must treat this as a skeptical verification-and-reconciliation task, not as a blind reimplementation task.

Current repo evidence from initial audit:
- `src/process/jobs.rs` already defines `StartPostgresSpec` with only `data_dir`, `config_file`, `log_file`, `wait_seconds`, and `timeout_ms`.
- `src/process/worker.rs` already renders startup as `pg_ctl ... -o "-c config_file=<abs path>" ...` and no longer carries a generic repeated `-c key=value` settings map.
- `src/runtime/node.rs` already materializes managed config before startup and passes `managed.postgresql_conf_path` into `StartPostgresSpec`.
- `src/ha/process_dispatch.rs` already materializes managed config and passes the same authoritative config path for HA-driven starts.
- `src/postgres_managed.rs` already writes `PGDATA/pgtm.postgresql.conf` plus managed HBA/ident/TLS side files.
- `src/test_harness/ha_e2e/startup.rs` already asserts `SHOW config_file;` equals `PGDATA/pgtm.postgresql.conf`.
- Task `01-task-introduce-a-typed-managed-postgres-conf-model-and-serializer.md` already claims a large portion of the implementation that this task description still describes as future work.

Because of that mismatch, the execution pass must first determine whether task 02 still has any remaining code/doc work, or whether it is now primarily a closure and verification task.

### Detailed execution plan for the later `NOW EXECUTE` pass

1. Audit the task boundary before changing anything.
   - Re-read this task's goal and distinguish pgtuskmaster-managed PostgreSQL start paths from direct-Postgres harness helpers.
   - Treat runtime startup, resume, and HA-dispatched `StartPostgres` as in scope for task 02.
   - Treat direct test harness helpers that intentionally boot vanilla PostgreSQL outside pgtuskmaster ownership, such as `src/test_harness/pg16.rs`, as out of scope for task 02 unless they are proven to be exercising a managed start path by mistake.
   - If repo search finds stale docs or harness surfaces that still describe or mutate `postgresql.conf`, decide whether they belong in this task or in task 04 instead of silently broadening task 02.
   - Reconcile any overlap with task 01 and task 04 explicitly in this task file so the next engineer can see which remaining gaps genuinely belong here.

2. Audit the startup contract end-to-end before changing anything.
   - Re-read `src/process/jobs.rs`, `src/process/worker.rs`, `src/runtime/node.rs`, `src/ha/process_dispatch.rs`, `src/postgres_managed.rs`, and `src/postgres_managed_conf.rs`.
   - Confirm there is no remaining `extra_postgres_settings`, no generic startup GUC bag, and no start-path-specific `-h` / `-p` / `-k` injection path still used for managed PostgreSQL startup.
   - Confirm the only `pg_ctl -o` usage for managed `StartPostgres` is the narrow `-c config_file=<abs path>` override; treat `-l`, `-w`, and timeout handling as separate operational flags outside the config-ownership question.
   - Search the repo for stale references to the old contract such as `extra_postgres_settings`, generic repeated `-c key=value`, or claims that startup still relies on default `postgresql.conf`.

3. Identify the true remaining gap list and update this task's implementation scope accordingly.
   - If code already satisfies a checklist item, convert the execution work for that item into verification plus task-file reconciliation rather than rewriting the same code.
   - If any real gap remains, record the exact file/function still violating the authoritative-config contract before editing code.
   - Pay special attention to hidden start surfaces outside the obvious modules, including tests, helper paths, or worker-local startup helpers that could still mutate or rely on default `postgresql.conf`.

4. Verify the authoritative config ownership boundary at code level.
   - In `src/postgres_managed.rs`, verify that managed config materialization always yields an absolute `pgtm.postgresql.conf` path for every managed start intent and that startup-critical settings are rendered into that file, not transported through process-layer flags.
   - In `src/postgres_managed_conf.rs`, verify that listen host, listen port, socket dir, HBA file, ident file, TLS files, and replica settings are rendered deterministically into the managed config file.
   - Confirm there is no conditional backup-era ownership around `config_file` and that reserved extra GUC validation still prevents user override of owned settings.

5. Verify runtime and HA start paths are fully aligned.
   - In `src/runtime/node.rs`, verify normal start, resume, and clone-to-replica flows all materialize managed config immediately before starting PostgreSQL and all use the same `StartPostgresSpec` contract.
   - In `src/ha/process_dispatch.rs`, verify `HaAction::StartPostgres` uses the same managed config materialization path and does not reconstruct config through ad hoc process arguments.
   - If runtime and HA differ in materialization timing, intent derivation, or file ownership behavior, fix that divergence instead of papering over it in tests.

6. Reconcile tests with the actual contract and add any missing coverage.
   - Re-read `src/process/worker.rs` unit tests around `build_command_start_postgres_uses_managed_config_file_override`.
   - Confirm there are no stale tests that still encode `-c hba_file=...` / `-c ident_file=...` startup injection expectations.
   - Re-read `src/test_harness/ha_e2e/startup.rs` and confirm the real-binary assertions cover `SHOW config_file;`, `SHOW hba_file;`, and `SHOW ident_file;` against pgtuskmaster-managed paths.
   - If repo search finds raw `postgresql.conf` mutation in direct-Postgres harnesses, do not automatically rewrite those helpers in task 02; either prove they are actually part of a managed start path or record them as task-04-owned follow-up.
   - Add or tighten tests only where the audit finds a real missing proof surface for managed startup ownership.

7. Update documentation and task artifacts to remove stale claims.
   - Search docs and task files for statements that still describe the old startup model.
   - Update real docs only where they directly document the managed startup contract owned by this task; push broader harness/documentation migration work that belongs to task 04 back to that task instead of duplicating it here.
   - Remove or rewrite stale wording that says startup still depends on default `postgresql.conf` or a generic `-c` settings bag.
   - If task 02 is now mostly subsumed by task 01 plus later docs work, make that relationship explicit in this task file rather than leaving contradictory historical text.

8. Reconcile this task file itself during execution.
   - Tick checklist items only after the repo audit or code/test/doc changes justify them.
   - Replace stale research notes in this task with accurate findings from the current codebase so the task does not mislead the next engineer.
   - Preserve a short explanation of what was already implemented before this pass versus what this pass actually changed.

9. Run the full required gates after all code/doc/task-file edits are complete.
   - `make check`
   - `make test`
   - `make test-long`
   - `make lint`
   - Do not mark the task complete unless all four pass.

10. Only after the gates pass cleanly, perform closure steps.
   - Set `<passes>true</passes>` in this task file.
   - Run `/bin/bash .ralph/task_switch.sh`.
   - Commit all changes, including `.ralph` files, with the required `task finished [task name]: ...` format and include evidence of the completed gates.
   - Push with `git push`.

### Specific skepticism the verification pass must apply

- Do not trust task 02's original research notes; they are already contradicted by the repository.
- Do not trust task 01's completion notes blindly either; verify the live code paths and proof surfaces still match those claims.
- Challenge whether any test-only helper or worker-local startup harness still mutates default `postgresql.conf` in a way that actually bypasses a managed start path, and separate those findings from direct-Postgres harnesses that belong to task 04 rather than this task.
- Challenge whether there are still docs or task artifacts that would send the next engineer toward the wrong startup model.
- Challenge whether `pg_ctl -o "-c config_file=..."` should remain the final narrow contract or whether any remaining non-config startup flags are still leaking configuration ownership.

NOW EXECUTE
