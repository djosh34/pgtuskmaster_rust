## Task: Take full ownership of replica, recovery signal, and `postgresql.auto.conf` state <status>not_started</status> <passes>false</passes> <priority>high</priority>

<blocked_by>01-task-remove-backup-config-and-process-surface,02-task-remove-runtime-restore-bootstrap-and-archive-helper-wiring,04-task-remove-backup-harness-installers-and-gate-selection,05-task-remove-backup-docs-and-obsolete-task-artifacts</blocked_by>
<blocked_by>01-task-introduce-a-typed-managed-postgres-conf-model-and-serializer,02-task-make-pgtm-postgresql-conf-the-only-startup-config-entrypoint</blocked_by>

<description>
**Goal:** Remove the remaining places where Postgres tooling or leftover PGDATA files decide recovery behavior, and make pgtuskmaster fully own replica/recovery config through typed state, managed `.conf` contents, and managed signal files.
The higher-order goal is to make recovery posture derive only from pgtuskmaster config plus DCS/runtime state, never from `pg_basebackup -R`, inherited `postgresql.auto.conf`, or stale signal files.

**Scope:**
- Remove `pg_basebackup -R`.
- Introduce typed ownership of signal-file state (`none`, `standby.signal`, `recovery.signal`) and materialize it deterministically.
- Make recovery-related GUCs in `pgtm.postgresql.conf` come from the typed managed-config builder instead of Postgres side effects.
- Take full ownership of `postgresql.auto.conf` by making it non-authoritative on every managed start path. The implementation must not leave this as an open design choice.
- The required policy for this story is: pgtuskmaster must never rely on `ALTER SYSTEM`, must treat `postgresql.auto.conf` as unmanaged/out-of-band state, and must remove or quarantine it during managed startup preparation so it cannot silently override `pgtm.postgresql.conf`.
- Keep the design centered on current non-backup HA needs only; do not reintroduce pgBackRest-era restore bootstrap behavior.

**Context from research:**
- `src/process/worker.rs` still runs `pg_basebackup` with `-R`, which allows Postgres tooling to author recovery config side effects on pgtuskmaster’s behalf.
- `src/postgres_managed.rs` already has takeover logic around `recovery.signal`, `standby.signal`, and `postgresql.auto.conf`, but that logic is currently framed around backup-era restore ownership rather than steady-state full config authority.
- If `postgresql.auto.conf` is left active, pgtuskmaster cannot truthfully claim that its managed config file is the only determinant of effective Postgres config.
- Because this is a greenfield zero-user project, this task should optimize for clean ownership rather than preserving `ALTER SYSTEM` compatibility.

**Expected outcome:**
- Replica and recovery posture are materialized only by pgtuskmaster.
- `pg_basebackup` no longer writes follow/recovery config on behalf of the runtime.
- `postgresql.auto.conf` no longer acts as a shadow authority over managed starts.

</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] `pg_basebackup -R` is removed
- [ ] Signal-file ownership is represented in typed Rust state and materialized deterministically by pgtuskmaster
- [ ] Recovery-related managed config comes from the typed managed-config builder, not from Postgres-authored side effects
- [ ] `postgresql.auto.conf` is explicitly removed or quarantined during managed startup preparation so it cannot silently override managed authority
- [ ] `ALTER SYSTEM` is not used or depended on anywhere in the managed-config ownership path
- [ ] No backup-era restore-bootstrap behavior is preserved under a new name
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

---

### Exhaustive checklist (must be treated as part of acceptance)

- [ ] `src/process/worker.rs`
  - [ ] Remove `-R` from the `pg_basebackup` command builder.
  - [ ] Update command-rendering tests so they explicitly prove `-R` is gone.
- [ ] `src/process/jobs.rs`
  - [ ] Add or revise typed process/job state if needed to express replica/recovery intent without relying on `pg_basebackup -R`.
- [ ] `src/postgres_managed.rs`
  - [ ] Add explicit typed signal-file materialization.
  - [ ] Remove any backup-era framing from signal ownership.
  - [ ] Implement the required `postgresql.auto.conf` policy explicitly: remove or quarantine it during managed startup preparation.
  - [ ] Ensure cleanup is deterministic for managed starts and resumes.
- [ ] `src/runtime/node.rs`
  - [ ] Feed role/topology-derived recovery intent into the managed-config and signal-file materialization path.
  - [ ] Keep replica-clone startup and resume-existing startup aligned with the same ownership model.
- [ ] `src/ha/process_dispatch.rs`
  - [ ] Ensure HA-driven follow/primary transitions consume the same typed recovery intent model if startup/restart behavior changes are required.
- [ ] Real-binary tests
  - [ ] Add or update tests proving `pg_basebackup` no longer writes recovery config side effects on pgtuskmaster’s behalf.
  - [ ] Add or update tests proving the expected signal file exists, and unexpected stale signal files do not survive managed startup preparation.
  - [ ] Add or update tests proving `postgresql.auto.conf` is removed or quarantined and cannot silently override the managed authoritative config.
