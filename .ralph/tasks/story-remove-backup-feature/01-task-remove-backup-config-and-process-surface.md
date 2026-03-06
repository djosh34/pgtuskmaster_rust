## Task: Remove backup config and pgBackRest process vocabulary while keeping basebackup replica cloning <status>completed</status> <passes>true</passes> <priority>high</priority>

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
- [x] Remove `BackupConfig`, `BackupProvider`, `BackupBootstrapConfig`, `BackupTakeoverPolicy`, `BackupRecoveryMode`, `PgBackRestConfig`, `BackupOptions`, and related `*V2Input` structures from `src/config/schema.rs`.
- [x] Remove `backup_timeout_ms` from `ProcessConfig` and `ProcessConfigV2Input` in `src/config/schema.rs`.
- [x] Remove `pgbackrest` from `BinaryPaths` and `BinaryPathsV2Input` in `src/config/schema.rs`.
- [x] Remove backup-related defaults and normalization helpers from `src/config/defaults.rs`.
- [x] Remove backup-related validation logic from `src/config/parser.rs`, including option-token validation and conditional pgBackRest requirements.
- [x] Remove backup-related re-exports from `src/config/mod.rs`.
- [x] Delete `src/backup/mod.rs`, `src/backup/provider.rs`, `src/backup/pgbackrest.rs`, and `src/backup/worker.rs`.
- [x] Remove all pgBackRest specs from `src/process/jobs.rs`, including version/info/check/backup/restore/archive push/archive get.
- [x] Remove all pgBackRest active job kinds and job kind strings from `src/process/state.rs`.
- [x] Remove all pgBackRest command-building, timeout mapping, spool-path handling, and log identity cases from `src/process/worker.rs`.
- [x] Keep `BaseBackupSpec`, `ProcessJobKind::BaseBackup`, `ActiveJobKind::BaseBackup`, and the `pg_basebackup` command path intact and green.
- [x] Update every known `RuntimeConfig` / `BinaryPaths` literal fallout site to compile without backup fields:
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
- [x] Search for any additional `RuntimeConfig` / `BinaryPaths` fallout under `src/`, `tests/`, and `examples/` and remove remaining backup literals discovered during implementation.
- [x] Update `src/lib.rs` to stop declaring the deleted backup module.
- [x] Confirm by search that `src/` no longer contains `BackupConfig`, `backup_timeout_ms`, `process.binaries.pgbackrest`, or `PgBackRest*` types.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Detailed Execution Plan (Draft 1, 2026-03-06)

### 1. Scope lock and overlap map

- This task owns deletion of the root backup config vocabulary and the pgBackRest process/job vocabulary.
- During research, additional compile blockers appeared outside the original fallout list:
  - `src/runtime/node.rs`
  - `src/postgres_managed.rs`
  - `src/api/fallback.rs`
  - `src/api/events.rs`
  - `src/api/worker.rs`
  - `src/bin/pgtuskmaster.rs`
  - `src/test_harness/ha_e2e/{config,startup}.rs`
  - `src/backup/archive_command.rs`
  - `src/wal.rs`
  - `src/wal_passthrough.rs`
  - `tests/wal_passthrough.rs`
- The execution pass must treat "compile fallout after deleting root types" as in-scope even when it overlaps later cleanup tasks. Build-green takes precedence over the earlier estimate.
- Skeptical review change: Draft 1 deferred most archive/restore helper and restore-bootstrap cleanup to later tasks. That was too optimistic. `RestoreBootstrap`, `archive_command` / `restore_command` ownership, the `wal` CLI helper, and WAL passthrough event ingestion are not incidental fallout; they are the runtime/API/CLI embodiment of the backup feature being deleted here and must be removed or refactored in this task.
- Functional cleanup still intentionally deferred after this task:
  - broad harness/provenance/install-script cleanup that is not required for a green build belongs to Task 04
- One structural caveat must be handled deliberately during execution: once `src/lib.rs` stops declaring the `backup` module and `src/backup/{mod,provider,pgbackrest,worker}.rs` are deleted, `src/backup/archive_command.rs` cannot remain wired the same way. The verified plan is to delete that file in this task together with the `wal` helper surfaces that depend on it.

### 2. Remove root config schema vocabulary

- In `src/config/schema.rs`, delete:
  - `RuntimeConfig.backup`
  - `ProcessConfig.backup_timeout_ms`
  - `BinaryPaths.pgbackrest`
  - `BackupConfig`, `BackupProvider`, `BackupBootstrapConfig`, `BackupTakeoverPolicy`, `BackupRecoveryMode`, `BackupOptions`, `PgBackRestConfig`
  - matching `*V2Input` structs/enums and any serde/default annotations tied to those fields
- Rebuild the surviving `RuntimeConfig`, `ProcessConfig`, and `BinaryPaths` shapes so they remain internally consistent without placeholder backup abstractions.
- Keep `pg_basebackup` and every field needed by `BaseBackupSpec` intact.

### 3. Remove backup defaults, normalization, and validation

- In `src/config/defaults.rs`, delete backup-specific default constants, config constructors, and normalization helpers.
- In `src/config/parser.rs`, remove:
  - validation for `process.backup_timeout_ms`
  - validation for `process.binaries.pgbackrest`
  - all `backup.*` validation branches
  - pgBackRest option-token validation helpers and required-field checks
- In `src/config/mod.rs`, stop re-exporting deleted backup config types.
- After these edits, `load_runtime_config` and `validate_runtime_config` should only know about the surviving non-backup schema.

### 4. Delete pgBackRest process vocabulary

- In `src/process/jobs.rs`, remove all `PgBackRest*Spec` structs and the corresponding `ProcessJobKind` variants.
- In `src/process/state.rs`, remove matching `ActiveJobKind` variants, string labels, and `is_pgbackrest_job()`.
- In `src/process/worker.rs`, remove all pgBackRest-only behavior:
  - spool-dir preflight
  - timeout mapping to `backup_timeout_ms`
  - active-kind mapping
  - log identity strings
  - command builders using `process.binaries.pgbackrest`
  - unit/real-binary tests for pgBackRest job execution
- Preserve `BaseBackupSpec`, `ProcessJobKind::BaseBackup`, `ActiveJobKind::BaseBackup`, and the `pg_basebackup` command path exactly.

### 5. Delete crate backup modules at the root

- Delete:
  - `src/backup/mod.rs`
  - `src/backup/archive_command.rs`
  - `src/backup/provider.rs`
  - `src/backup/pgbackrest.rs`
  - `src/backup/worker.rs`
- Update `src/lib.rs` to stop declaring `pub(crate) mod backup;`.

### 6. Remove restore-bootstrap and WAL-helper behavior, not just root types

- In `src/runtime/node.rs`, delete the backup-driven startup path rather than leaving a partial shell:
  - remove `StartupMode::RestoreBootstrap`
  - remove startup planning branches keyed off `cfg.backup.*`
  - preserve only the non-backup modes: `InitializePrimary`, `CloneReplica`, `ResumeExisting`
- In `src/postgres_managed.rs`, remove pgtuskmaster-owned archive/restore helper config materialization and injected `archive_command` / `restore_command` settings that depend on deleted backup config.
- Delete `src/wal.rs` and `src/wal_passthrough.rs` if they become dead after removing archive helper wiring; do not preserve a pgBackRest passthrough shim once the root feature is gone.
- Remove the `wal` helper subcommand and help text from `src/bin/pgtuskmaster.rs` if it no longer has a supported backend after the backup feature deletion.
- Remove WAL passthrough API/event ingestion surfaces that only existed for the pgBackRest helper path, including `src/api/events.rs` and matching API worker routes/tests if they become dead.

### 7. Repair compile fallout in runtime, API, DCS, HA, logging, examples, and tests

- Update every `RuntimeConfig`, `ProcessConfig`, and `BinaryPaths` literal to match the new schema:
  - remove `backup: ...`
  - remove `backup_timeout_ms: ...`
  - remove `pgbackrest: ...`
- Update imports so deleted backup types are no longer referenced.
- Required known fallout set from the task description:
  - `examples/debug_ui_smoke_server.rs`
  - `src/dcs/{etcd_store,state,store,worker}.rs`
  - `src/debug_api/worker.rs`
  - `src/ha/{decide,events,process_dispatch,worker}.rs`
  - `src/logging/{mod,postgres_ingest}.rs`
  - `src/worker_contract_tests.rs`
  - `tests/bdd_api_http.rs`
- Additional fallout discovered during research that should be treated as in-scope if compilation fails after root deletion:
  - `src/runtime/node.rs`
  - `src/postgres_managed.rs`
  - `src/api/events.rs`
  - `src/api/fallback.rs`
  - `src/api/worker.rs`
  - `src/bin/pgtuskmaster.rs`
  - `src/test_harness/ha_e2e/{config,startup}.rs`
  - `src/wal.rs`
  - `src/wal_passthrough.rs`
  - `tests/wal_passthrough.rs`
- The execution pass should not leave temporary compatibility shims for deleted backup fields. Remove the callers instead.

### 8. Handle runtime/harness overlap intentionally instead of rediscovering it

- `src/runtime/node.rs` currently uses both `cfg.backup.*` and pgBackRest restore jobs. Once this task removes backup config and process kinds, those branches must be deleted, not merely bypassed.
- For the `NOW EXECUTE` pass, treat removal of `RestoreBootstrap` as the intended product change:
  - keep `InitializePrimary`, `CloneReplica`, `ResumeExisting`
  - keep replica cloning through `BaseBackup`
  - if the cluster is uninitialized and the data directory is `Missing|Empty`, the surviving non-backup path should be primary initialization unless replica-clone conditions apply
- `src/test_harness/ha_e2e/{config,startup}.rs` currently synthesizes runtime backup config objects. If those files block compilation after root deletion, remove that root-vocabulary usage now rather than reintroducing compatibility types.
- If this execution pass ends up consuming part of later-task scope, that is acceptable; do not keep dead backup scaffolding just to preserve story boundaries.

### 9. Search-driven completion criteria for the execution pass

- After the compile fallout settles, run repository searches and clear all `src/` hits for:
  - `BackupConfig`
  - `backup_timeout_ms`
  - `process.binaries.pgbackrest`
  - `PgBackRest`
- Also search for deleted runtime-helper vocabulary that Draft 1 failed to promote to in-scope removal:
  - `RestoreBootstrap`
  - `archive_command`
  - `restore_command`
  - `WalPassthrough`
- Also search `tests/`, `examples/`, `docs/`, and `Makefile` for stale backup/pgBackRest references created by this root removal and delete/update them if they are now incorrect.
- Keep explicit note of any remaining `pgbackrest` references that are intentionally deferred to later tasks, and verify they are outside the deleted root config/process/runtime-helper surface.

### 10. Documentation updates required by this task

- Search docs for the removed config/process surface, especially any references to:
  - `backup.*`
  - `process.backup_timeout_ms`
  - `process.binaries.pgbackrest`
  - pgBackRest process jobs or restore/bootstrap behavior described as currently supported
- Update or delete stale docs in the same execution pass so the repository no longer documents removed config keys or removed process/runtime-helper vocabulary.
- Prefer editing `docs/src/...` sources only. Generated `docs/book/...` output is untracked here and should not be hand-edited.

### 11. Planned execution order for the later `NOW EXECUTE` pass

1. Remove config schema/types/exports in `src/config/{schema,defaults,parser,mod}.rs`.
2. Remove pgBackRest job/state/worker vocabulary in `src/process/{jobs,state,worker}.rs`.
3. Delete `src/backup/{mod,archive_command,provider,pgbackrest,worker}.rs` and update `src/lib.rs`.
4. Remove restore-bootstrap and WAL-helper runtime/API/CLI behavior in `src/runtime/node.rs`, `src/postgres_managed.rs`, `src/bin/pgtuskmaster.rs`, `src/wal.rs`, `src/wal_passthrough.rs`, `src/api/events.rs`, and their direct callers/tests.
5. Repair remaining compile fallout across runtime/API/DCS/HA/logging/examples/tests, prioritizing root-type deletions over temporary shims.
6. Search `src/`, `tests/`, `examples/`, `docs/`, and `Makefile` for stale root-vocabulary references and remove remaining direct hits.
7. Run targeted tests around config parsing, startup planning, and surviving process worker behavior first if needed to debug quickly.
8. Run mandatory gates in full:
   - `make check`
   - `make test`
   - `make test-long`
   - `make lint`
9. Only after all gates pass, update task checkboxes/status metadata per the task workflow.

NOW EXECUTE
