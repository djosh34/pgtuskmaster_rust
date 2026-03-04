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
