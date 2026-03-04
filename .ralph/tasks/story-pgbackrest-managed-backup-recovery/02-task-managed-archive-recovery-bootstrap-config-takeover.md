---
## Task: Own archive/recovery command flow and inject managed config before recovery starts <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Make pgtuskmaster authoritative for archive and restore command behavior, and ensure config takeover happens before PostgreSQL recovery so restores never boot with unsafe/incompatible backup-era config files.

**Scope:**
- Replace ad-hoc archive command wrapper behavior with pgBackRest-aware archive control integrated into managed startup.
- Integrate recovery/restore into startup/bootstrap flow so recovered nodes are brought up using pgtuskmaster-managed config from the first recovery boot.
- Enforce config takeover strategy for restored data dirs:
- remove/replace backup-shipped config artifacts that conflict with managed runtime
- write managed config artifacts (`pg_hba`, `pg_ident`, TLS files, and required recovery settings) before postgres starts recovery
- Preserve structured logging across restore and recovery phases so operators can debug failures from logs only.

**Context from research:**
- PostgreSQL 16 recovery is driven by signal files (`recovery.signal` / `standby.signal`) plus settings in `postgresql.conf`/`postgresql.auto.conf`: https://www.postgresql.org/docs/16/recovery-config.html
- Archive recovery needs valid `restore_command` behavior; bad/missing restore settings prevent recovery progress: https://www.postgresql.org/docs/16/runtime-config-wal.html
- Hot standby requires certain settings (`max_connections`, `max_prepared_transactions`, `max_locks_per_transaction`, `max_worker_processes`) to be at least primary values; mismatch can block standby startup: https://www.postgresql.org/docs/16/hot-standby.html
- `pg_basebackup` includes source config files, so relying on backup-era config is unsafe for cluster takeover restores: https://www.postgresql.org/docs/16/app-pgbasebackup.html
- pgBackRest restore supports recovery type control (`default`, `preserve`, `standby`) and `--recovery-option` mutation for generated recovery config: https://pgbackrest.org/command.html

**Expected outcome:**
- Recovery bootstrap path always applies pgtuskmaster-owned config before postgres starts.
- Restore failures caused by stale/conflicting backed-up config are prevented or surfaced with explicit structured error messages.
- Archive/recovery command control is explicit, testable, and no longer implicit side effects.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] Update managed config logic in `src/postgres_managed.rs` to support pre-recovery takeover:
- [ ] deterministic purge/replace policy for backup-restored `*.conf` artifacts
- [ ] explicit write order for managed files before restore/recovery startup
- [ ] strict error reporting for missing/invalid managed inputs
- [ ] Extend startup orchestration in `src/runtime/node.rs`:
- [ ] add restore-aware startup mode(s) and bootstrap path transitions
- [ ] ensure managed config materialization occurs before recovery start commands
- [ ] retain existing startup logging and timeout semantics
- [ ] Extend process job model in `src/process/jobs.rs`, `src/process/state.rs`, `src/process/worker.rs`:
- [ ] add restore/archive-related command specs and builders
- [ ] support pgBackRest restore command execution with typed options and safe validation
- [ ] keep subprocess line capture for restore/recovery command output
- [ ] Replace or evolve archive wrapper behavior in `src/logging/archive_wrapper.rs` and `src/logging/postgres_ingest.rs`:
- [ ] archive command path must be owned by pgtuskmaster and feed structured logs
- [ ] log ingestion must include archive/restore command events with stable attributes
- [ ] Add startup/recovery unit and integration tests covering:
- [ ] restore with backup-provided bad `max_connections` config must be corrected by managed takeover before recovery startup
- [ ] restore with missing/empty `postgresql.conf` must still start via managed injection
- [ ] restore with incompatible WAL/recovery settings must fail with actionable operator logs
- [ ] ensure normal non-restore bootstrap path stays correct
- [ ] Update docs for archive/recovery ownership model in `docs/src/operator/` pages (configuration + recovery runbooks)
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Plan

### 0) Non-negotiable design decisions (lock these in first)
- [ ] **Explicit restore intent:** restoring a node must never be an implicit side-effect of `backup.enabled`. Add a dedicated config knob that opts a node into running a restore during bootstrap when `PGDATA` is `Missing|Empty`.
- [ ] **Fail closed:** if restore intent is enabled, require pgBackRest to be fully configured (`process.binaries.pgbackrest`, stanza/repo) and require structured logging configuration for archive/recovery commands (see logging decisions below).
- [ ] **Pgtuskmaster owns recovery-critical Postgres settings from first boot:** for a restored/cloned `PGDATA`, Postgres must start using a pgtuskmaster-owned config file (`config_file=...`) and *must not* read backup-era `postgresql.auto.conf` / backup-era `archive_command` / backup-era `restore_command`.
- [ ] **Deterministic purge policy:** takeover must either (a) move conflicting backup-era artifacts into a timestamped quarantine directory under `PGDATA` for forensics, or (b) remove them. The choice must be deterministic and logged.
- [ ] **No external runtime deps in wrappers:** the archive/restore command wrapper must not depend on `python3` (or other “maybe installed” tools). Only POSIX sh + coreutils (`sed`, `date`, `mkdir`, etc.) is allowed.

### 1) Exhaustive checklist (files/modules and requirements)

#### Config surface
- [ ] `src/config/schema.rs`: add restore bootstrap config (provider-agnostic at the top level, pgBackRest-specific details nested under `backup.pgbackrest`).
  - [ ] Proposed shape (v2): `backup.bootstrap` (new) with:
    - [ ] `enabled: bool` (default false)
    - [ ] `provider: BackupProvider` (reuse existing enum; today only pgBackRest)
    - [ ] `takeover_policy: enum { quarantine, delete }` (default quarantine)
    - [ ] `recovery: enum { default }` for now (leave `standby` for task 03 when we can also set `primary_conninfo`)
- [ ] `src/config/defaults.rs`: defaults for new bootstrap/restore block.
- [ ] `src/config/parser.rs`: validation rules:
  - [ ] if `backup.bootstrap.enabled=true`: require `backup.enabled=true`
  - [ ] if `backup.enabled=true`: keep existing pgBackRest validation
  - [ ] if `backup.bootstrap.enabled=true`: require `logging.postgres.archive_command_log_file` (so restore/archive events are always observable during recovery)
  - [ ] validate all new enum fields non-empty / deny unknown fields.

#### Runtime startup orchestration
- [ ] `src/runtime/node.rs`: extend startup planning + execution:
  - [ ] Add `StartupMode::RestoreBootstrap` variant.
  - [ ] Selection logic: when `PGDATA` is `Missing|Empty` and **no init lock** exists, pick `RestoreBootstrap` if `backup.bootstrap.enabled=true`, else keep existing `InitializePrimary`.
  - [ ] Keep precedence: if a healthy leader exists, keep using `CloneReplica` (do not silently restore a replica; that is task 03).
  - [ ] Execution order for `RestoreBootstrap`:
    - [ ] `PgBackRestRestore` (startup subprocess capture enabled)
    - [ ] `postgres_managed::takeover_restored_data_dir(...)` (must run before any Postgres start)
    - [ ] `StartPostgres` using managed config (must include `config_file=...` + managed archive/restore command settings).
  - [ ] Add explicit structured startup log markers for each phase: `startup.phase = restore|takeover|start` so operators can reconstruct sequence from logs.

#### Managed Postgres takeover (pre-recovery)
- [ ] `src/postgres_managed.rs`: split responsibilities into two explicit phases:
  - [ ] `takeover_restored_data_dir(cfg, policy)`:
    - [ ] Deterministically quarantine/delete conflicting artifacts from restored backups:
      - [ ] `postgresql.conf`
      - [ ] `postgresql.auto.conf`
      - [ ] `pg_hba.conf`, `pg_ident.conf` (backup-era security risk)
      - [ ] any existing `pgtm.*` managed artifacts (stale)
      - [ ] signal files (`recovery.signal`, `standby.signal`) **unless** we explicitly decide to preserve them (for now: remove to avoid “accidental standby”; task 03 can re-introduce with full wiring).
    - [ ] Ensure `PGDATA` contains a pgtuskmaster-owned config file, e.g. `PGDATA/pgtm.postgresql.conf`.
      - [ ] File must be valid even if the backup shipped no `postgresql.conf` or shipped an empty one.
      - [ ] The managed config must include *at minimum*:
        - [ ] `archive_command` ownership (wrapper path)
        - [ ] `restore_command` ownership (wrapper path)
        - [ ] logging config overrides that are required for our ingest pipeline (if any)
    - [ ] Ensure managed artifacts exist in a strict write order (to make logs/actionable failures deterministic):
      1) create quarantine dir (if enabled)
      2) quarantine/delete conflicting artifacts
      3) write command wrappers (archive/restore) + ensure parent directories exist
      4) write managed config file (`pgtm.postgresql.conf`)
      5) write managed `pg_hba`, `pg_ident`, TLS artifacts (or leave to start-time step below, but be consistent)
  - [ ] `materialize_managed_postgres_config(cfg)` (existing) becomes the “start-time settings” function:
    - [ ] Must return `extra_settings` that includes `config_file = PGDATA/pgtm.postgresql.conf` for restored/cloned data dirs (and eventually for all modes, if we decide to fully own config).
    - [ ] Must return `hba_file`, `ident_file`, TLS settings exactly as today.
    - [ ] Must be strict: missing TLS identity when required is a hard error.

#### Process job model (pgBackRest restore/archive)
- [ ] `src/process/jobs.rs`:
  - [ ] Extend `PgBackRestRestoreSpec` to include `pg1_path: PathBuf` (always set to `cfg.postgres.data_dir`) so restore target is deterministic.
  - [ ] Extend `PgBackRestArchivePushSpec` / `PgBackRestArchiveGetSpec` to include `pg1_path: PathBuf` as well (so archive command wrappers don’t rely on external pgBackRest config).
  - [ ] Optional (but recommended for this task): add an enum for restore type (`default|preserve|standby`) and store it in the spec instead of as an unvalidated option token.
- [ ] `src/backup/worker.rs`:
  - [ ] Update job builders to populate `pg1_path` fields.
- [ ] `src/backup/pgbackrest.rs`:
  - [ ] Render `--pg1-path <path>` for restore and archive operations from typed fields.
  - [ ] Tighten `validate_option_tokens` to forbid overriding any managed ownership flags:
    - [ ] `--stanza*`, `--repo*` (already)
    - [ ] `--pg1-path*` (new)
    - [ ] restore-only safety: forbid `--type*` and `--recovery-option*` if we move them into typed fields.
- [ ] `src/process/worker.rs`:
  - [ ] Update command builder to use the new typed fields and surface precise errors (no `unwrap/expect/panic`).
  - [ ] Ensure subprocess output capture stays enabled for restore/archive jobs (already supported).
  - [ ] Improve structured log attributes for these jobs (recommended): attach `pgbackrest.stanza`, `pgbackrest.repo`, `pgbackrest.op` to each subprocess line so operators don’t have to infer from raw CLI output.

#### Archive/restore wrapper + log ingestion
- [ ] `src/logging/archive_wrapper.rs`:
  - [ ] Replace `cp`-based wrapper with pgBackRest-aware wrapper(s):
    - [ ] Provide a single wrapper script (generated by pgtuskmaster) that supports:
      - [ ] `archive-push <src_path> <filename>` (Postgres `archive_command` signature via `%p %f`)
      - [ ] `archive-get <wal_segment> <dest_path>` (Postgres `restore_command` signature via `%f %p`)
    - [ ] Script must:
      - [ ] execute pgBackRest with stanza/repo/pg1-path/options derived from runtime config (rendered into the script, *not* environment-dependent)
      - [ ] capture exit status
      - [ ] append **one JSON line** per invocation to `logging.postgres.archive_command_log_file`
      - [ ] include stable attributes (schema below)
      - [ ] not depend on python/jq; implement minimal JSON escaping in shell.
  - [ ] Rename the module/file if necessary (`archive_wrapper` → `pgbackrest_command_wrapper`) but keep changes minimal unless the rename clarifies ownership.
- [ ] `src/logging/postgres_ingest.rs`:
  - [ ] Stop generating wrappers in ingest worker; wrapper creation belongs to startup/managed config and must fail fast if it cannot be created.
  - [ ] Ingest JSONL records from `archive_command_log_file` and normalize into stable attributes:
    - [ ] `backup.event_kind = archive_push|archive_get`
    - [ ] `backup.provider = pgbackrest`
    - [ ] `backup.stanza`, `backup.repo`
    - [ ] `backup.pg1_path`
    - [ ] `backup.wal_path` (push) / `backup.wal_segment` + `backup.destination_path` (get)
    - [ ] `backup.status_code`, `backup.success`
    - [ ] include raw pgBackRest stderr/stdout (truncated) as `backup.output` for operator debugging.

#### Tests
- [ ] `src/runtime/node.rs`:
  - [ ] Unit tests for startup planning:
    - [ ] restore bootstrap selected only when `backup.bootstrap.enabled=true` and `PGDATA` is `Missing|Empty` and no init lock is present
    - [ ] restore bootstrap not selected when a healthy leader exists (clone still preferred)
  - [ ] Unit test for execution ordering: `restore -> takeover -> start` (via a test command runner / deterministic instrumentation).
- [ ] `src/postgres_managed.rs`:
  - [ ] Unit tests for takeover:
    - [ ] backup-era `postgresql.conf` containing `max_connections=1` is quarantined/deleted and does not influence the generated managed config
    - [ ] missing/empty `postgresql.conf` still results in a valid managed config file
    - [ ] existing `postgresql.auto.conf` is removed/quarantined so it cannot resurrect stale settings
    - [ ] signal files are removed/quarantined by default
- [ ] `src/backup/pgbackrest.rs` + `src/process/worker.rs`:
  - [ ] Unit tests for:
    - [ ] deterministic rendering of `restore` / `archive-*` including `--pg1-path`
    - [ ] rejection of forbidden override tokens (`--pg1-path`, `--type`, `--recovery-option`) when those are owned by typed spec fields.
- [ ] Real-binary integration tests (must run under `make test`, not optional):
  - [ ] Add a focused scenario that simulates a “restored” data dir (without needing to run a full pgBackRest backup/restore yet):
    - [ ] create a PGDATA with a conflicting `postgresql.conf` and `postgresql.auto.conf`
    - [ ] run the startup path that performs takeover then starts Postgres
    - [ ] assert Postgres starts successfully and `SHOW max_connections` reflects defaults (or expected managed baseline), proving backup-era config did not apply.
  - [ ] Add a failure-path test where recovery cannot proceed (e.g., configure `restore_command` to an invalid path) and assert startup fails with actionable `PgTool` + ingest logs.

#### Docs
- [ ] `docs/src/operator/configuration.md`: document `backup.bootstrap` and the takeover policy, including “what files are removed/quarantined”.
- [ ] `docs/src/operator/troubleshooting.md`: add a “restore/recovery bootstrap” section with common failure fingerprints and where to look in logs.
- [ ] `docs/src/operator/observability.md`: document the JSONL schema for archive/restore command events + example queries.
- [ ] `docs/src/operator/index.md`: link a new runbook page.
- [ ] Add `docs/src/operator/recovery-bootstrap-runbook.md`: step-by-step operator flow for restore bootstrap, including required config and the expected log sequence.

### 2) Required gates (100% green)
- [ ] `make check`
- [ ] `make test`
- [ ] `make test-long`
- [ ] `make lint`

TO BE VERIFIED
