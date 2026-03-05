---
## Task: Remove runtime restore bootstrap and the archive_command helper/proxy wiring <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Delete the runtime-owned restore bootstrap path and the hacky archive/restore helper stack, including the local event-ingest API used only for archive_command/restore_command passthrough logging.

**Scope:**
- Remove startup-time restore bootstrap selection and execution from the runtime.
- Remove managed Postgres ownership of `archive_mode`, `archive_command`, `restore_command`, helper JSON files, recovery takeover files, and self-executable helper lookup.
- Remove the `pgtuskmaster wal ...` subcommand, `wal_passthrough`, `wal`, and the `/events/wal` ingest endpoint.

**Context from research:**
- The archive/restore helper stack was added in commits `6be6c5d` and `fafbc5e` on 2026-03-05.
- Current runtime restore bootstrap selection is in `src/runtime/node.rs`, especially the `cfg.backup.enabled` validation gate, `RestoreBootstrap` startup mode, and restore startup actions.
- Current managed wiring is in `src/postgres_managed.rs`, where backup mode injects `archive_mode`, `archive_command`, `restore_command`, writes `pgtm.pgbackrest.archive.json`, and owns restore-specific takeover cleanup.
- Current helper path is spread across `src/bin/pgtuskmaster.rs`, `src/wal.rs`, `src/wal_passthrough.rs`, `src/self_exe.rs`, `src/backup/archive_command.rs`, `src/api/events.rs`, and the `/events/wal` route in `src/api/worker.rs`.
- This task must not remove normal managed HBA/ident/TLS materialization or normal Postgres start behavior.

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
