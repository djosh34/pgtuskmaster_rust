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
