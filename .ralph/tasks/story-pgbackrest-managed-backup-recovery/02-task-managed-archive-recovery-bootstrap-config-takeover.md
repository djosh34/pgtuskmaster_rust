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
- [ ] Implement archive/restore command ownership and observability without runtime-generated shell scripts:
- [ ] archive/restore command behavior is owned by pgtuskmaster via a Rust-native mechanism (helper binary/subcommand)
- [ ] operators can diagnose restore/bootstrap failures from PgTool output capture + Postgres logs + ingest diagnostics
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
- [ ] **Recovery signal files are owned, not inherited:** restore bootstrap must not “trust” backup-shipped `recovery.signal` / `standby.signal`. For this task, restore bootstrap creates a pgtuskmaster-owned `recovery.signal` (primary-style archive recovery) and removes/quarantines any existing `standby.signal`. (Replica/standby wiring is deferred to task 05.)
- [ ] **Deterministic purge policy:** takeover must either (a) move conflicting backup-era artifacts into a timestamped quarantine directory under `PGDATA` for forensics, or (b) remove them. The choice must be deterministic and logged.
- [ ] **No external runtime deps in wrappers:** the archive/restore command wrapper must not depend on `python3` (or other “maybe installed” tools). It may be (a) POSIX sh + coreutils, or (b) a pgtuskmaster-owned compiled helper/subcommand invoked by a trivial sh wrapper.

### 1) Exhaustive checklist (files/modules and requirements)

#### Config surface
- [ ] `src/config/schema.rs`: add restore bootstrap config (provider-agnostic at the top level, pgBackRest-specific details nested under `backup.pgbackrest`).
  - [ ] Proposed shape (v2): `backup.bootstrap` (new) with:
    - [ ] `enabled: bool` (default false)
    - [ ] `provider: BackupProvider` (reuse existing enum; today only pgBackRest)
    - [ ] `takeover_policy: enum { quarantine, delete }` (default quarantine)
    - [ ] `recovery: enum { default }` for now (leave `standby` for task 05 when we can also set `primary_conninfo`)
- [ ] `src/config/defaults.rs`: defaults for new bootstrap/restore block.
- [ ] `src/config/parser.rs`: validation rules:
  - [ ] if `backup.bootstrap.enabled=true`: require `backup.enabled=true`
  - [ ] if `backup.enabled=true`: keep existing pgBackRest validation
  - [ ] enforce logging path ownership invariants to avoid tail/delete loops and accidental self-ingestion:
    - [ ] `logging.sinks.file.path` must not equal and must not be under any tailed Postgres input (`postgres.log_file`, `logging.postgres.pg_ctl_log_file`) and must not be inside `logging.postgres.log_dir`
  - [ ] validate all new enum fields non-empty / deny unknown fields.

#### Runtime startup orchestration
- [ ] `src/runtime/node.rs`: extend startup planning + execution:
  - [ ] Add `StartupMode::RestoreBootstrap` variant.
  - [ ] Selection logic: when `PGDATA` is `Missing|Empty` and **no init lock** exists, pick `RestoreBootstrap` if `backup.bootstrap.enabled=true`, else keep existing `InitializePrimary`.
  - [ ] Keep precedence: if a healthy leader exists, keep using `CloneReplica` (do not silently restore a replica; that is task 05).
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
      - [ ] signal files:
        - [ ] remove/quarantine any existing `standby.signal` (avoid accidental standby)
        - [ ] remove/quarantine any existing `recovery.signal` (never inherit backup intent)
        - [ ] write a fresh pgtuskmaster-owned `recovery.signal` for restore bootstrap so `restore_command` is actually exercised during recovery
    - [ ] Ensure `PGDATA` contains a pgtuskmaster-owned config file, e.g. `PGDATA/pgtm.postgresql.conf`.
      - [ ] File must be valid even if the backup shipped no `postgresql.conf` or shipped an empty one.
      - [ ] The managed config must include *at minimum*:
        - [ ] `archive_command` ownership (wrapper path)
        - [ ] `restore_command` ownership (wrapper path)
        - [ ] logging config overrides that are required for our ingest pipeline (if any)
      - [ ] Prefer `archive_command` = `<wrapper> archive-push %p` (do not pass `%f` unless we actually need it).
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
  - [ ] Tighten `validate_option_tokens` to forbid overriding any managed ownership flags using **exact option keys**, not prefix matches (to avoid accidentally forbidding legitimate pgBackRest flags like `--repo1-path`):
    - [ ] forbid `--stanza`, `--stanza=...`
    - [ ] forbid `--repo`, `--repo=...` (but allow `--repo1-*` / `--repo2-*` etc)
    - [ ] forbid `--pg1-path`, `--pg1-path=...`
    - [ ] restore-only safety: forbid `--type*` and `--recovery-option*` if we move them into typed fields.
- [ ] `src/process/worker.rs`:
  - [ ] Update command builder to use the new typed fields and surface precise errors (no `unwrap/expect/panic`).
  - [ ] Ensure subprocess output capture stays enabled for restore/archive jobs (already supported).
  - [ ] Improve structured log attributes for these jobs (recommended): attach `pgbackrest.stanza`, `pgbackrest.repo`, `pgbackrest.op` to each subprocess line so operators don’t have to infer from raw CLI output.

#### Archive/restore command ownership + observability
- [ ] Do not reintroduce runtime-generated shell scripts for Postgres `archive_command` / `restore_command` (the previous wrapper mechanism was intentionally removed).
- [ ] When this task reintroduces archive/restore command ownership, do it via a Rust-native mechanism (helper binary/subcommand) with deterministic, testable behavior.
- [ ] Until a first-class archive/restore observability mechanism exists, rely on:
  - PgTool subprocess output capture for pgBackRest restore/backup commands
  - local Postgres logs + ingest diagnostics (`origin=postgres_ingest`) for ingestion health

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
  - [ ] Add wrapper concurrency + path-escaping tests:
    - [ ] wrapper + log file paths containing spaces/quotes are handled correctly (or explicitly rejected by config validation)
    - [ ] concurrent `archive-get` invocations do not corrupt JSONL (no interleaved half-records)
  - [ ] Add config validation tests for the new logging path-ownership invariants (reject overlapping sink/source paths).

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

NOW EXECUTE
