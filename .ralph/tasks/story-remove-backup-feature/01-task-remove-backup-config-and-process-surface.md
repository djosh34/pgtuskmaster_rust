## Task: Remove backup config and pgBackRest process vocabulary while keeping basebackup replica cloning <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
**Goal:** Delete the backup feature's config and process-language surface completely, while preserving `pg_basebackup`-based replica creation as a non-backup bootstrap path.
This story is an immediate blocker: the backup feature must be removed before continuing broader rewrite work, because the leftover pgBackRest/archive/restore surface keeps reintroducing complexity and false dependencies across the runtime.

**Scope:**
- Remove all runtime config schema/default/parser/default exports for `backup.*`, `process.backup_timeout_ms`, and `process.binaries.pgbackrest`.
- Delete the pgBackRest provider/rendering/job-builder modules and all process job/spec/state variants tied to pgBackRest operations.
- Update every compile fallout site that currently carries backup-shaped `RuntimeConfig` / `BinaryPaths` literals after those fields are deleted.
- Keep `BaseBackupSpec`, `ProcessJobKind::BaseBackup`, `process.binaries.pg_basebackup`, and the replica clone path intact.
- Do not partially preserve a “generic backup abstraction”. This task removes the feature vocabulary at the root.

**Context from research:**
- The backup feature was introduced primarily in commit `023be6f` on 2026-03-04 and then spread outward; this task should remove that vocabulary at the root so later tasks can delete runtime and HA wiring cleanly.
- Current config surface lives in `src/config/schema.rs`, `src/config/defaults.rs`, `src/config/parser.rs`, and `src/config/mod.rs`.
- Current process surface lives in `src/process/jobs.rs`, `src/process/state.rs`, `src/process/worker.rs`, and `src/backup/{mod,provider,pgbackrest,worker}.rs`.
- Current compile-fallout files already known to carry backup-shaped literals include:
  - `examples/debug_ui_smoke_server.rs`
  - `src/dcs/{etcd_store,state,store,worker}.rs`
  - `src/debug_api/worker.rs`
  - `src/ha/{decide,events,process_dispatch,worker}.rs`
  - `src/logging/{mod,postgres_ingest}.rs`
  - `src/worker_contract_tests.rs`
  - `tests/bdd_api_http.rs`
- `BaseBackup` predates pgBackRest and is required to keep replica creation working; do not delete or weaken it.

**Definite removal boundary for this task:**
- `BackupConfig`, `BackupProvider`, `BackupBootstrapConfig`, `BackupTakeoverPolicy`, `BackupRecoveryMode`, `PgBackRestConfig`, `BackupOptions`
- `process.backup_timeout_ms`
- `process.binaries.pgbackrest`
- all `PgBackRest*` process specs, state variants, timeout handling, spool-dir handling, and command rendering
- `src/backup/mod.rs`
- `src/backup/provider.rs`
- `src/backup/pgbackrest.rs`
- `src/backup/worker.rs`

**Preserve boundary for this task:**
- `pg_basebackup` replica clone support
- `BaseBackupSpec`
- `ProcessJobKind::BaseBackup`
- `ActiveJobKind::BaseBackup`
- non-backup process worker behavior
- runtime/API/HA logic not directly required to delete the config/process vocabulary

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
- [ ] Update every known `RuntimeConfig` / `BinaryPaths` literal fallout site to compile without backup fields:
  - `examples/debug_ui_smoke_server.rs`
  - `src/dcs/etcd_store.rs`
  - `src/dcs/state.rs`
  - `src/dcs/store.rs`
  - `src/dcs/worker.rs`
  - `src/debug_api/worker.rs`
  - `src/ha/decide.rs`
  - `src/ha/events.rs`
  - `src/ha/process_dispatch.rs`
  - `src/ha/worker.rs`
  - `src/logging/mod.rs`
  - `src/logging/postgres_ingest.rs`
  - `src/worker_contract_tests.rs`
  - `tests/bdd_api_http.rs`
- [ ] Search for any additional `RuntimeConfig` / `BinaryPaths` fallout under `src/`, `tests/`, and `examples/` and remove remaining backup literals discovered during implementation.
- [ ] Update `src/lib.rs` to stop declaring the deleted backup module.
- [ ] Confirm by search that `src/` no longer contains `BackupConfig`, `backup_timeout_ms`, `process.binaries.pgbackrest`, or `PgBackRest*` types.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
