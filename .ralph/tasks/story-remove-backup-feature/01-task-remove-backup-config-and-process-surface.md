---
## Task: Remove backup config and pgBackRest process vocabulary while keeping basebackup replica cloning <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Delete the backup feature's config and process-language surface completely, while preserving `pg_basebackup`-based replica creation as a non-backup bootstrap path.

**Scope:**
- Remove all runtime config schema/default/parser/default exports for `backup.*`, `process.backup_timeout_ms`, and `process.binaries.pgbackrest`.
- Delete the pgBackRest provider/rendering/job-builder modules and all process job/spec/state variants tied to pgBackRest operations.
- Keep `BaseBackupSpec`, `ProcessJobKind::BaseBackup`, `process.binaries.pg_basebackup`, and the replica clone path intact.

**Context from research:**
- The backup feature was introduced primarily in commit `023be6f` on 2026-03-04 and then spread outward; this task should remove that vocabulary at the root so later tasks can delete runtime and HA wiring cleanly.
- Current config surface lives in `src/config/schema.rs`, `src/config/defaults.rs`, `src/config/parser.rs`, and `src/config/mod.rs`.
- Current process surface lives in `src/process/jobs.rs`, `src/process/state.rs`, `src/process/worker.rs`, and `src/backup/{mod,provider,pgbackrest,worker}.rs`.
- `BaseBackup` predates pgBackRest and is required to keep replica creation working; do not delete or weaken it.

**Expected outcome:**
- `RuntimeConfig` no longer contains a backup block.
- The process worker no longer knows how to build or classify pgBackRest jobs.
- The crate no longer exports or compiles pgBackRest-specific provider modules.
- The only remaining "backup-like" bootstrap behavior is leader-to-replica cloning via `pg_basebackup`.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Remove `BackupConfig`, `BackupProvider`, `BackupBootstrapConfig`, `BackupTakeoverPolicy`, `BackupRecoveryMode`, `PgBackRestConfig`, `BackupOptions`, and related `*V2Input` structures from `src/config/schema.rs`.
- [ ] Remove `backup_timeout_ms` from `ProcessConfig` and `ProcessConfigV2Input` in `src/config/schema.rs`.
- [ ] Remove `pgbackrest` from `BinaryPaths` and `BinaryPathsV2Input` in `src/config/schema.rs`.
- [ ] Remove backup-related defaults and normalization helpers from `src/config/defaults.rs`.
- [ ] Remove backup-related validation logic from `src/config/parser.rs`, including option-token validation and conditional pgBackRest requirements.
- [ ] Remove backup-related re-exports from `src/config/mod.rs`.
- [ ] Delete `src/backup/mod.rs`, `src/backup/provider.rs`, `src/backup/pgbackrest.rs`, and `src/backup/worker.rs`.
- [ ] Remove all pgBackRest specs from `src/process/jobs.rs`, including version/info/check/backup/restore/archive push/archive get.
- [ ] Remove all pgBackRest active job kinds and job kind strings from `src/process/state.rs`.
- [ ] Remove all pgBackRest command-building, timeout mapping, spool-path handling, and log identity cases from `src/process/worker.rs`.
- [ ] Keep `BaseBackupSpec`, `ProcessJobKind::BaseBackup`, `ActiveJobKind::BaseBackup`, and the `pg_basebackup` command path intact and green.
- [ ] Update every `RuntimeConfig` literal and `BinaryPaths` literal under `src/`, `tests/`, and `examples/` to compile without the removed fields.
- [ ] Update `src/lib.rs` to stop declaring the deleted backup module.
- [ ] Confirm by search that `src/` no longer contains `BackupConfig`, `backup_timeout_ms`, `process.binaries.pgbackrest`, or `PgBackRest*` types.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
