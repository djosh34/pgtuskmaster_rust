## Task: Remove runtime restore bootstrap and the archive_command helper/proxy wiring <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Delete the runtime-owned restore bootstrap path and the hacky archive/restore helper stack, including the local event-ingest API used only for archive_command/restore_command passthrough logging.
This is now a top-priority blocker inside backup removal, because the surviving `archive_command`, `restore_command`, helper JSON sidecar, and WAL passthrough path are the most disruptive remaining pieces for debugging and further refactoring.

**Scope:**
- Remove startup-time restore bootstrap selection and execution from the runtime.
- Remove managed Postgres ownership of `archive_mode`, `archive_command`, `restore_command`, helper JSON files, recovery takeover files, and self-executable helper lookup.
- Remove the `pgtuskmaster wal ...` subcommand, `wal_passthrough`, `wal`, and the `/events/wal` ingest endpoint.
- Keep `PGDATA/pgtm.postgresql.conf`, but remove all backup-specific ownership and behavior from it.

**Context from research:**
- The archive/restore helper stack was added in commits `6be6c5d` and `fafbc5e` on 2026-03-05.
- Current runtime restore bootstrap selection is in `src/runtime/node.rs`, especially the `cfg.backup.enabled` validation gate, `RestoreBootstrap` startup mode, and restore startup actions.
- Current managed wiring is in `src/postgres_managed.rs`, where backup mode injects `archive_mode`, `archive_command`, `restore_command`, writes `pgtm.pgbackrest.archive.json`, and owns restore-specific takeover cleanup.
- Current helper path is spread across `src/bin/pgtuskmaster.rs`, `src/wal.rs`, `src/wal_passthrough.rs`, `src/self_exe.rs`, `src/backup/archive_command.rs`, `src/api/events.rs`, and the `/events/wal` route in `src/api/worker.rs`.
- `pgtm.postgresql.conf` currently does two backup-related things:
  - when `backup.bootstrap.enabled=true`, `materialize_managed_postgres_config()` adds `config_file=PGDATA/pgtm.postgresql.conf` to startup extra settings
  - when `backup.enabled=true`, `write_managed_postgresql_conf()` writes backup-owned contents into that file: `archive_mode = on`, `archive_command = '...'`, `restore_command = '...'`, and helper-config comments referencing `pgtm.pgbackrest.archive.json`
- `takeover_restored_data_dir()` also recreates `pgtm.postgresql.conf` specifically so restore-bootstrap startup can point Postgres at a pgtuskmaster-owned config file after pgBackRest restore.
- This task must not remove normal managed HBA/ident/TLS materialization or normal Postgres start behavior.
- The user has explicitly decided that `.conf` stays in this task. Do not broaden this task into a redesign of general config-file ownership.

**Definite removal boundary for this task:**
- `RuntimeConfig.backup` runtime gating in `src/runtime/node.rs`
- `StartupMode::RestoreBootstrap`
- `StartupAction::TakeoverRestoredDataDir`
- pgBackRest restore startup action planning/execution
- archive helper JSON sidecar `pgtm.pgbackrest.archive.json`
- `archive_mode`, `archive_command`, `restore_command` ownership
- `pgtuskmaster wal ...` helper mode
- WAL passthrough runner and `/events/wal` API ingest

**Preserve boundary for this task:**
- `InitializePrimary`, `CloneReplica`, and `ResumeExisting`
- managed `pgtm.pg_hba.conf`, `pgtm.pg_ident.conf`, TLS artifacts, and non-backup `extra_settings`
- managed `pgtm.postgresql.conf`
- normal `StartPostgres` path and `pg_basebackup` clone path

**Explicit constraint for this task:**
- `pgtm.postgresql.conf` must stay.
- This task removes only backup-era contents and backup-era uses of that file.
- Do not redesign general `.conf` ownership policy here; that belongs to a separate later story/task.

**Expected outcome:**
- Startup can initialize a primary, clone a replica via `pg_basebackup`, or resume an existing node, but it can no longer perform a repository-based restore bootstrap.
- Postgres managed config no longer emits `archive_command` or `restore_command`, and no helper JSON sidecar is written into PGDATA.
- The node binary no longer exposes a `wal` helper mode and the API no longer exposes `/events/wal`.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Remove the `cfg.backup.enabled` validation gate from `src/runtime/node.rs`.
- [ ] Remove `StartupMode::RestoreBootstrap`, `StartupAction::TakeoverRestoredDataDir`, and all restore-bootstrap planning/execution branches from `src/runtime/node.rs`.
- [ ] Keep `InitializePrimary`, `CloneReplica`, and `ResumeExisting` startup modes working.
- [ ] Keep the `BaseBackup` replica-clone startup path intact in `src/runtime/node.rs`.
- [ ] Delete `src/backup/archive_command.rs`.
- [ ] Remove helper-config generation, archive/restore command rendering, recovery-signal takeover cleanup, and backup-specific `pgtm.postgresql.conf` ownership from `src/postgres_managed.rs`.
- [ ] Preserve non-backup managed config responsibilities in `src/postgres_managed.rs`, especially HBA/ident/TLS file ownership and `extra_settings` needed for normal startup.
- [ ] Keep `pgtm.postgresql.conf` in `src/postgres_managed.rs`, but remove all backup-owned contents and behavior from it:
  - no `archive_mode = on`
  - no `archive_command = '...'`
  - no `restore_command = '...'`
  - no helper-config comments referencing `pgtm.pgbackrest.archive.json`
  - no restore-bootstrap-only behavior tied specifically to pgBackRest
- [ ] Keep any remaining non-backup `config_file=pgtm.postgresql.conf` startup wiring intact in this task.
- [ ] Remove `src/wal.rs`, `src/wal_passthrough.rs`, and all references to them.
- [ ] Remove the `Wal` subcommand and helper execution mode from `src/bin/pgtuskmaster.rs`.
- [ ] Remove `src/api/events.rs` and the `/events/wal` route/authorization/serialization logic from `src/api/worker.rs`.
- [ ] Remove `src/self_exe.rs` if it becomes unused after helper deletion; otherwise reduce it to only the remaining legitimate use and document why it still exists.
- [ ] Delete all references to `archive_command`, `restore_command`, `archive_mode`, `pgtm.pgbackrest.archive.json`, `backup.wal_passthrough`, and `wal passthrough invocation` from `src/`.
- [ ] Verify that `materialize_managed_postgres_config` and startup tests still pass for non-backup flows.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Detailed Execution Plan (Draft 1, 2026-03-06)

### 1. Scope lock against current HEAD, not against stale assumptions

- Research pass completed against current HEAD plus historical snapshots from before Task 01 (`git show HEAD~2:src/postgres_managed.rs` and `git show HEAD~2:src/runtime/node.rs`).
- Current HEAD already appears to have removed most of the originally-described helper surface:
  - no `StartupMode::RestoreBootstrap`
  - no `StartupAction::TakeoverRestoredDataDir`
  - no `src/wal.rs`
  - no `src/wal_passthrough.rs`
  - no `src/api/events.rs`
  - no `wal` CLI subcommand in `src/bin/pgtuskmaster.rs`
  - no `archive_command` / `restore_command` / `archive_mode` / `pgtm.pgbackrest.archive.json` references in `src/`
- Because of that, the execution pass for this task must not waste time trying to rediscover or re-delete already-removed surfaces. Treat "verify deletion by search and keep it deleted" as a real completion step.
- Important drift discovered during research: the preserved `pgtm.postgresql.conf` boundary also appears to have been deleted from current HEAD, even though this task explicitly says it must stay. The execution pass must correct that over-deletion, but it must do so without reintroducing backup-era `config_file` startup wiring.
- Skeptical review result: Draft 1's assumption that the task should restore `config_file=pgtm.postgresql.conf` was too strong. Pointing PostgreSQL at a new minimal managed file would make that file the primary config root and could silently bypass the normal `PGDATA/postgresql.conf` path.

### 2. Product intent that execution must preserve

- Keep the backup feature deleted.
- Keep the surviving startup modes exactly as they are now:
  - `InitializePrimary`
  - `CloneReplica`
  - `ResumeExisting`
- Keep replica cloning via `pg_basebackup`.
- Keep managed HBA/ident/TLS ownership in `src/postgres_managed.rs`.
- Restore or preserve a minimal non-backup-managed `PGDATA/pgtm.postgresql.conf` artifact on disk so this task honors its stated preserve boundary.
- Keep PostgreSQL startup on the existing default `PGDATA/postgresql.conf` path unless current HEAD already has a non-backup `config_file` override somewhere else. Current research indicates it does not.
- Do not reintroduce any backup vocabulary, restore takeover, helper JSON sidecars, or helper subprocess proxy behavior.

### 3. Execution ownership split for the later `NOW EXECUTE` pass

- Use subagents in parallel during execution.
- Worker A ownership:
  - `src/postgres_managed.rs`
  - any tests colocated there or newly added for managed config materialization
  - responsibility: restore the minimal `pgtm.postgresql.conf` artifact without reintroducing backup wiring or `config_file` startup ownership
- Worker B ownership:
  - `src/runtime/node.rs`
  - `src/self_exe.rs`
  - `src/lib.rs`
  - runtime/startup tests
  - responsibility: remove the dead self-exe initialization path and keep startup behavior green
- Main agent ownership:
  - integrate both workers
  - run repository-wide searches
  - update this task file checkboxes
  - update docs only if an actual user-facing or contributor-facing statement becomes stale
  - run `make check`, `make test`, `make test-long`, `make lint`

### 4. `src/postgres_managed.rs`: restore only the minimal surviving `.conf` ownership

- Reintroduce a constant for `pgtm.postgresql.conf` in `src/postgres_managed.rs`.
- Materialize that file during normal managed config generation so the preserved artifact still exists in `PGDATA`.
- The file contents must be intentionally minimal:
  - managed header/comment only
  - no `archive_mode`
  - no `archive_command`
  - no `restore_command`
  - no comments referencing `pgtm.pgbackrest.archive.json`
- Do not inject `config_file=<absolute path to PGDATA/pgtm.postgresql.conf>` into `ManagedPostgresConfig.extra_settings`.
- Reason: with the current startup model, a restored `config_file` override would make the minimal managed file the primary PostgreSQL config and would change runtime behavior far beyond this task's removal boundary.
- Keep `hba_file`, `ident_file`, TLS file settings, and `ssl` ownership in `extra_settings`.
- Do not move HBA/ident/TLS settings into the file. Keep the current `pg_ctl -o ... -c key=value` startup path unchanged aside from the restored inert artifact.
- Keep all error handling explicit. No unwrap/expect/panic shortcuts.

### 5. `src/runtime/node.rs`, `src/self_exe.rs`, and `src/lib.rs`: remove the last dead helper residue

- Remove `crate::self_exe::init_from_current_exe()` from `run_node_from_config`.
- Delete `src/self_exe.rs` if no other legitimate caller exists after that removal.
- Remove `pub(crate) mod self_exe;` from `src/lib.rs`.
- Confirm by search that `PGTM_SELF_EXE_OVERRIDE`, `self_exe`, and related helper-only vocabulary no longer exist anywhere under `src/` or `tests/`, unless a surviving non-backup use is discovered during execution. If such a use unexpectedly exists, document it in this task file before keeping the module.
- Do not broaden this into CLI redesign. `src/bin/pgtuskmaster.rs` is already the simple node-entry binary the task wants.

### 6. Tests to add or update during execution

- Add or restore unit coverage in `src/postgres_managed.rs` that directly proves the intended preserved boundary:
  - `materialize_managed_postgres_config(...)` creates `pgtm.postgresql.conf`
  - the file does not contain backup-owned settings (`archive_mode`, `archive_command`, `restore_command`)
-  `extra_settings` does not gain a `config_file` override as part of this task
- Prefer extending existing HA startup coverage instead of inventing a brand-new startup test:
  - assert `SHOW config_file;` still resolves to the default `PGDATA/postgresql.conf`
  - assert `PGDATA/pgtm.postgresql.conf` exists on disk and is backup-free
- Keep the existing runtime startup-mode tests green; do not re-open startup planning semantics unless the `.conf` reintroduction forces a direct change.
- Use the existing `build_command_start_postgres_includes_extra_settings_deterministically` coverage as the process-layer proof that HBA/ident/TLS command-line propagation remains ordered and intact.

### 7. Search-driven completion checks for execution

- Run and clear repository searches for:
  - `RestoreBootstrap`
  - `TakeoverRestoredDataDir`
  - `archive_command`
  - `restore_command`
  - `archive_mode`
  - `pgtm.pgbackrest.archive.json`
  - `/events/wal`
  - `wal passthrough`
  - `PGTM_SELF_EXE_OVERRIDE`
  - `self_exe`
- Treat search hits in this task file and in later-story task descriptions as expected, but clear them from `src/`, `tests/`, `docs/`, and build scripts unless they are accurate documentation of removed behavior.
- Specifically verify the restored `pgtm.postgresql.conf` artifact exists in `src/postgres_managed.rs` while `config_file` does not reappear in runtime-managed startup settings.

### 8. Documentation expectations for this task

- First search docs for any statement that would become false after reintroducing the minimal managed `pgtm.postgresql.conf` path.
- If no docs mention that internal file boundary, do not invent documentation churn just to satisfy the process.
- If docs still mention removed backup restore/bootstrap or WAL passthrough behavior outside Task 05's explicit cleanup scope, update them now only when they are directly made stale by this task's execution.

### 9. Order of operations for the later `NOW EXECUTE` pass

1. Spawn Worker A on `src/postgres_managed.rs` and its tests.
2. Spawn Worker B on `src/runtime/node.rs`, `src/self_exe.rs`, and `src/lib.rs`.
3. Integrate Worker A first, because the `.conf` preservation decision is the main product-correctness change still missing from HEAD and now has a narrower, safer boundary.
4. Integrate Worker B second, keeping runtime startup green after dead helper removal.
5. Run targeted tests for touched modules before the full gates.
6. Run repository searches for deleted helper vocabulary and for the restored `.conf` boundary.
7. Update task checkboxes based on actual code state plus verified searches.
8. Run the required full gates:
   - `make check`
   - `make test`
   - `make test-long`
   - `make lint`
9. Only after all gates pass, set `<passes>true</passes>`, switch task, commit, and push per the Ralph workflow.

### 10. Decisions locked by skeptical review

- Do not restore `config_file=pgtm.postgresql.conf`. Preserve the file, not the deleted backup-era startup override.
- Prefer extending the existing HA startup test to assert the default `config_file` plus the managed artifact on disk, instead of adding a separate new startup harness.
- Docs still appear likely to need no change unless execution discovers a contributor-facing statement that claims the helper surface still exists.

NOW EXECUTE
