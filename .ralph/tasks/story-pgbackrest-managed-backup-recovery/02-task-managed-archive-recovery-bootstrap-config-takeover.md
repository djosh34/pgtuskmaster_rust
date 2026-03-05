---
## Task: Own archive/recovery command flow and inject managed config before recovery starts <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Make pgtuskmaster authoritative for archive and restore command behavior, and ensure config takeover happens before PostgreSQL recovery so restores never boot with unsafe/incompatible backup-era config files.

**PO Directive (2026-03-05):** Use pgBackRest config-method ownership only. Do not rely on repo-local wrapper/hack paths; use minimal CLI flags, with behavior/config sourced from managed config surfaces first.

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
- [x] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [x] Update managed config logic in `src/postgres_managed.rs` to support pre-recovery takeover:
- [x] deterministic purge/replace policy for backup-restored `*.conf` artifacts
- [x] explicit write order for managed files before restore/recovery startup
- [x] strict error reporting for missing/invalid managed inputs
- [x] Extend startup orchestration in `src/runtime/node.rs`:
- [x] add restore-aware startup mode(s) and bootstrap path transitions
- [x] ensure managed config materialization occurs before recovery start commands
- [x] retain existing startup logging and timeout semantics
- [x] Extend process job model in `src/process/jobs.rs`, `src/process/state.rs`, `src/process/worker.rs`:
- [x] add restore/archive-related command specs and builders
- [x] support pgBackRest restore command execution with typed options and safe validation
- [x] keep subprocess line capture for restore/recovery command output
- [x] Implement archive/restore command ownership and observability without runtime-generated shell scripts:
- [x] archive/restore command behavior is owned by pgtuskmaster via a Rust-native mechanism (helper binary/subcommand)
- [x] operators can diagnose restore/bootstrap failures from PgTool output capture + Postgres logs + ingest diagnostics
- [x] Add startup/recovery unit and integration tests covering:
- [x] restore with backup-provided bad `max_connections` config must be corrected by managed takeover before recovery startup
- [x] restore with missing/empty `postgresql.conf` must still start via managed injection
- [x] restore with incompatible WAL/recovery settings must fail with actionable operator logs
- [x] ensure normal non-restore bootstrap path stays correct
- [x] Update docs for archive/recovery ownership model in `docs/src/operator/` pages (configuration + recovery runbooks)
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Plan

### 0) Non-negotiable design decisions (lock these in first)
- [x] **Explicit restore intent:** restoring a node must never be an implicit side-effect of `backup.enabled`. Add a dedicated config knob that opts a node into running a restore during bootstrap when `PGDATA` is `Missing|Empty`.
- [x] **Fail closed:** if restore intent is enabled, require pgBackRest to be fully configured (`process.binaries.pgbackrest`, stanza/repo) and require structured logging configuration for archive/recovery commands (see logging decisions below).
- [x] **Pgtuskmaster owns recovery-critical Postgres settings from first boot:** for a restored/cloned `PGDATA`, Postgres must start using a pgtuskmaster-owned config file (`config_file=...`) and *must not* read backup-era `postgresql.auto.conf` / backup-era `archive_command` / backup-era `restore_command`.
- [x] **Recovery signal files are owned, not inherited:** restore bootstrap must not “trust” backup-shipped `recovery.signal` / `standby.signal`. For this task, restore bootstrap creates a pgtuskmaster-owned `recovery.signal` (primary-style archive recovery) and removes/quarantines any existing `standby.signal`. (Replica/standby wiring is deferred to task 05.)
- [x] **Deterministic purge policy:** takeover must either (a) move conflicting backup-era artifacts into a timestamped quarantine directory under `PGDATA` for forensics, or (b) remove them. The choice must be deterministic and logged.
- [x] **No external runtime deps in wrappers:** the archive/restore command wrapper must not depend on `python3` (or other “maybe installed” tools). It may be (a) POSIX sh + coreutils, or (b) a pgtuskmaster-owned compiled helper/subcommand invoked by a trivial sh wrapper.

### 1) Exhaustive checklist (files/modules and requirements)

#### Config surface
- [x] `src/config/schema.rs`: add restore bootstrap config (provider-agnostic at the top level, pgBackRest-specific details nested under `backup.pgbackrest`).
  - [x] Actual shape (v2): `backup.bootstrap` with:
    - [x] `enabled: bool` (default false)
    - [x] `takeover_policy: enum { quarantine, delete }` (default quarantine)
    - [x] `recovery_mode: enum { default }` for now (leave `standby` for task 05 when we can also set `primary_conninfo`)
  - [x] Backup provider remains `backup.provider` (today only `pgbackrest`).
- [x] `src/config/defaults.rs`: defaults for new bootstrap/restore block.
- [x] `src/config/parser.rs`: validation rules:
  - [x] if `backup.bootstrap.enabled=true`: require `backup.enabled=true`
  - [x] if `backup.enabled=true`: keep existing pgBackRest validation
  - [x] enforce logging path ownership invariants to avoid tail/delete loops and accidental self-ingestion:
    - [x] `logging.sinks.file.path` must not equal and must not be under any tailed Postgres input (`postgres.log_file`, `logging.postgres.pg_ctl_log_file`) and must not be inside `logging.postgres.log_dir`
  - [x] validate all new enum fields non-empty / deny unknown fields.

#### Runtime startup orchestration
- [x] `src/runtime/node.rs`: extend startup planning + execution:
  - [x] Add `StartupMode::RestoreBootstrap` variant.
  - [x] Selection logic: when `PGDATA` is `Missing|Empty` and **no init lock** exists, pick `RestoreBootstrap` if `backup.bootstrap.enabled=true`, else keep existing `InitializePrimary`.
  - [x] Keep precedence: if a healthy leader exists, keep using `CloneReplica` (do not silently restore a replica; that is task 05).
  - [x] Execution order for `RestoreBootstrap`:
    - [x] `PgBackRestRestore` (startup subprocess capture enabled)
    - [x] `postgres_managed::takeover_restored_data_dir(...)` (must run before any Postgres start)
    - [x] `StartPostgres` using managed config (must include `config_file=...` + managed archive/restore command settings).
  - [x] Add explicit structured startup log markers for each phase: `startup.phase = restore|takeover|start` so operators can reconstruct sequence from logs.

#### Managed Postgres takeover (pre-recovery)
- [x] `src/postgres_managed.rs`: split responsibilities into two explicit phases:
  - [x] `takeover_restored_data_dir(cfg, policy)`:
    - [x] Deterministically quarantine/delete conflicting artifacts from restored backups:
      - [x] `postgresql.conf`
      - [x] `postgresql.auto.conf`
      - [x] `pg_hba.conf`, `pg_ident.conf` (backup-era security risk)
      - [x] any existing `pgtm.*` managed artifacts (stale)
      - [x] signal files:
        - [x] remove/quarantine any existing `standby.signal` (avoid accidental standby)
        - [x] remove/quarantine any existing `recovery.signal` (never inherit backup intent)
        - [x] write a fresh pgtuskmaster-owned `recovery.signal` for restore bootstrap so `restore_command` is actually exercised during recovery
    - [x] Ensure `PGDATA` contains a pgtuskmaster-owned config file, e.g. `PGDATA/pgtm.postgresql.conf`.
      - [x] File must be valid even if the backup shipped no `postgresql.conf` or shipped an empty one.
      - [x] The managed config must include *at minimum*:
        - [x] `archive_command` ownership (wrapper path)
        - [x] `restore_command` ownership (wrapper path)
        - [x] logging config overrides that are required for our ingest pipeline (if any)
      - [x] Prefer `archive_command` = `<wrapper> archive-push %p` (do not pass `%f` unless we actually need it).
    - [x] Ensure managed artifacts exist in a strict write order (to make logs/actionable failures deterministic):
      1) create quarantine dir (if enabled)
      2) quarantine/delete conflicting artifacts
      3) write command wrappers (archive/restore) + ensure parent directories exist
      4) write managed config file (`pgtm.postgresql.conf`)
      5) write managed `pg_hba`, `pg_ident`, TLS artifacts (or leave to start-time step below, but be consistent)
  - [x] `materialize_managed_postgres_config(cfg)` (existing) becomes the “start-time settings” function:
    - [x] Must return `extra_settings` that includes `config_file = PGDATA/pgtm.postgresql.conf` for restored/cloned data dirs (and eventually for all modes, if we decide to fully own config).
    - [x] Must return `hba_file`, `ident_file`, TLS settings exactly as today.
    - [x] Must be strict: missing TLS identity when required is a hard error.

#### Process job model (pgBackRest restore/archive)
- [x] `src/process/jobs.rs`:
  - [x] Extend `PgBackRestRestoreSpec` to include `pg1_path: PathBuf` (always set to `cfg.postgres.data_dir`) so restore target is deterministic.
  - [x] Extend `PgBackRestArchivePushSpec` / `PgBackRestArchiveGetSpec` to include `pg1_path: PathBuf` as well (so archive command wrappers don’t rely on external pgBackRest config).
  - [x] Optional (but recommended for this task): add an enum for restore type (`default|preserve|standby`) and store it in the spec instead of as an unvalidated option token.
- [x] `src/backup/worker.rs`:
  - [x] Update job builders to populate `pg1_path` fields.
- [x] `src/backup/pgbackrest.rs`:
  - [x] Render `--pg1-path <path>` for restore and archive operations from typed fields.
  - [x] Tighten `validate_option_tokens` to forbid overriding any managed ownership flags using **exact option keys**, not prefix matches (to avoid accidentally forbidding legitimate pgBackRest flags like `--repo1-path`):
    - [x] forbid `--stanza`, `--stanza=...`
    - [x] forbid `--repo`, `--repo=...` (but allow `--repo1-*` / `--repo2-*` etc)
    - [x] forbid `--pg1-path`, `--pg1-path=...`
    - [x] restore-only safety: forbid `--type*` and `--recovery-option*` if we move them into typed fields.
- [x] `src/process/worker.rs`:
  - [x] Update command builder to use the new typed fields and surface precise errors (no `unwrap/expect/panic`).
  - [x] Ensure subprocess output capture stays enabled for restore/archive jobs (already supported).
  - [x] Improve structured log attributes for these jobs (recommended): attach `pgbackrest.stanza`, `pgbackrest.repo`, `pgbackrest.op` to each subprocess line so operators don’t have to infer from raw CLI output.

#### Archive/restore command ownership + observability
- [x] Do not reintroduce runtime-generated shell scripts for Postgres `archive_command` / `restore_command` (the previous wrapper mechanism was intentionally removed).
- [x] When this task reintroduces archive/restore command ownership, do it via a Rust-native mechanism (helper binary/subcommand) with deterministic, testable behavior.
- [x] Until a first-class archive/restore observability mechanism exists, rely on:
  - PgTool subprocess output capture for pgBackRest restore/backup commands
  - local Postgres logs + ingest diagnostics (`origin=postgres_ingest`) for ingestion health

#### Tests
- [x] `src/runtime/node.rs`:
  - [x] Unit tests for startup planning:
    - [x] restore bootstrap selected only when `backup.bootstrap.enabled=true` and `PGDATA` is `Missing|Empty` and no init lock is present
    - [x] restore bootstrap not selected when a healthy leader exists (clone still preferred)
  - [x] Unit test for execution ordering: `restore -> takeover -> start` (via a test command runner / deterministic instrumentation).
- [x] `src/postgres_managed.rs`:
  - [x] Unit tests for takeover:
    - [x] backup-era `postgresql.conf` containing `max_connections=1` is quarantined/deleted and does not influence the generated managed config
    - [x] missing/empty `postgresql.conf` still results in a valid managed config file
    - [x] existing `postgresql.auto.conf` is removed/quarantined so it cannot resurrect stale settings
    - [x] signal files are removed/quarantined by default
- [x] `src/backup/pgbackrest.rs` + `src/process/worker.rs`:
  - [x] Unit tests for:
    - [x] deterministic rendering of `restore` / `archive-*` including `--pg1-path`
    - [x] rejection of forbidden override tokens (`--pg1-path`, `--type`, `--recovery-option`) when those are owned by typed spec fields.
- [x] Real-binary integration tests (must run under `make test`, not optional):
  - [x] Add a focused scenario that simulates a “restored” data dir (without needing to run a full pgBackRest backup/restore yet):
    - [x] create a PGDATA with a conflicting `postgresql.conf` and `postgresql.auto.conf`
    - [x] run the startup path that performs takeover then starts Postgres
    - [x] assert Postgres starts successfully and `SHOW max_connections` reflects defaults (or expected managed baseline), proving backup-era config did not apply.
  - [x] Add a failure-path test where recovery cannot proceed (e.g., configure `restore_command` to an invalid path) and assert startup fails with actionable `PgTool` + ingest logs.
  - [x] Add wrapper concurrency + path-escaping tests:
    - [x] wrapper + log file paths containing spaces/quotes are handled correctly (or explicitly rejected by config validation)
    - [x] concurrent `archive-get` invocations do not corrupt JSONL (no interleaved half-records)
  - [x] Add config validation tests for the new logging path-ownership invariants (reject overlapping sink/source paths).

#### Docs
- [x] `docs/src/operator/configuration.md`: document `backup.bootstrap` and the takeover policy, including “what files are removed/quarantined”.
- [x] `docs/src/operator/troubleshooting.md`: add a “restore/recovery bootstrap” section with common failure fingerprints and where to look in logs.
- [x] `docs/src/operator/observability.md`: document the JSONL schema for archive/restore command events + example queries.
- [x] `docs/src/operator/index.md`: link a new runbook page.
- [x] Add `docs/src/operator/recovery-bootstrap-runbook.md`: step-by-step operator flow for restore bootstrap, including required config and the expected log sequence.

### 2) Required gates (100% green)
- [x] `make check`
- [x] `make test`
- [x] `make test-long`
- [x] `make lint`

NOW EXECUTE
