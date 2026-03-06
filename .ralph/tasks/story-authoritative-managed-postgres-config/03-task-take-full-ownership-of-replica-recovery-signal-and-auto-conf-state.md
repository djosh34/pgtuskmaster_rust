## Task: Take full ownership of replica, recovery signal, and `postgresql.auto.conf` state <status>done</status> <passes>true</passes> <priority>high</priority>

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
- [x] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [x] `pg_basebackup -R` is removed
- [x] Signal-file ownership is represented in typed Rust state and materialized deterministically by pgtuskmaster
- [x] Recovery-related managed config comes from the typed managed-config builder, not from Postgres-authored side effects
- [x] `postgresql.auto.conf` is explicitly removed or quarantined during managed startup preparation so it cannot silently override managed authority
- [x] `ALTER SYSTEM` is not used or depended on anywhere in the managed-config ownership path
- [x] No backup-era restore-bootstrap behavior is preserved under a new name
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

---

### Exhaustive checklist (must be treated as part of acceptance)

- [x] `src/postgres_managed_conf.rs`
  - [x] Replace the boolean standby-signal shortcut with explicit typed recovery-signal ownership that can represent `none`, `standby.signal`, and `recovery.signal`.
  - [x] Reserve recovery-related GUCs that would let `extra_gucs` bypass managed authority, including both follow settings and recovery-target/restore keys.
  - [x] Add serializer/unit coverage proving managed recovery config comes only from the typed builder.
- [x] `src/process/worker.rs`
  - [x] Remove `-R` from the `pg_basebackup` command builder.
  - [x] Update command-rendering tests so they explicitly prove `-R` is gone.
- [x] `src/process/jobs.rs`
  - [x] Add or revise typed process/job state if needed to express replica/recovery intent without relying on `pg_basebackup -R`.
- [x] `src/postgres_managed.rs`
  - [x] Add explicit typed signal-file materialization.
  - [x] Remove any backup-era framing from signal ownership.
  - [x] Implement the required `postgresql.auto.conf` policy explicitly: remove or quarantine it during managed startup preparation.
  - [x] Ensure cleanup is deterministic for managed starts and resumes.
- [x] `src/runtime/node.rs`
  - [x] Feed role/topology-derived recovery intent into the managed-config and signal-file materialization path.
  - [x] Keep replica-clone startup and resume-existing startup aligned with the same ownership model.
- [x] `src/ha/process_dispatch.rs`
  - [x] Ensure HA-driven follow/primary transitions consume the same typed recovery intent model if startup/restart behavior changes are required.
- [x] Shared managed-state parsing/helper surface
  - [x] Reuse one authoritative helper for reading previously managed replica state from disk so startup planning and HA dispatch cannot drift.
- [x] Real-binary tests
  - [x] Add or update tests proving `pg_basebackup` no longer writes recovery config side effects on pgtuskmaster’s behalf.
  - [x] Add or update tests proving the expected signal file exists, and unexpected stale signal files do not survive managed startup preparation.
  - [x] Add or update tests proving `postgresql.auto.conf` is removed or quarantined and cannot silently override the managed authoritative config.
- [x] Docs
  - [x] Update lifecycle docs for startup/recovery ownership.
  - [x] Update operator/contributor docs that still describe only `standby.signal` or omit the `postgresql.auto.conf` policy.

### Detailed execution plan

Dependency note before execution:
- The `<blocked_by>` entries for the backup-removal tasks point to the wrong story directory. The completed prerequisite files live under `.ralph/tasks/story-remove-backup-feature/`. Execution should not start by re-researching backup behavior; it should only verify those prerequisite removals are already present in code and then proceed.

1. Tighten the typed ownership model in `src/postgres_managed_conf.rs`.
- Introduce explicit typed signal ownership instead of the current `creates_standby_signal()` boolean shortcut.
- The typed model should represent all managed recovery signal states needed by current PostgreSQL behavior: no recovery signal, `standby.signal`, and `recovery.signal`.
- Keep replica follow intent and signal ownership adjacent in the type so callers cannot materialize config without also choosing the managed signal posture.
- Reserve any recovery-related GUC keys that would let `extra_gucs` bypass the managed builder, including the current replica keys plus restore/recovery-target keys that would undermine authoritative startup.
- Update serializer tests to assert the managed config still renders replica GUCs only from the typed builder and never through side effects.

2. Make `src/postgres_managed.rs` the single authority for startup-side file materialization.
- Extend `ManagedPostgresConfig` to expose every managed artifact path that startup logic and tests need to assert, including both signal-file paths, the active `postgresql.auto.conf` path, and the deterministic quarantine target.
- Materialize signal files deterministically from the typed ownership state:
  - create the one required signal file;
  - remove the other managed signal files if they are present;
  - do this on every managed startup path, not just fresh clone paths.
- Add explicit `postgresql.auto.conf` handling here rather than leaving it to PostgreSQL behavior:
  - implement one fixed policy, not a choice at execution time: quarantine any active `PGDATA/postgresql.auto.conf` to a deterministic managed filename that PostgreSQL will not read;
  - replace any older quarantined copy deterministically so repeated managed starts stay idempotent;
  - ensure the active `postgresql.auto.conf` file cannot remain in place when a managed start proceeds;
  - keep error reporting explicit if quarantine/removal fails.
- Keep the implementation free of ignored IO outcomes and avoid backup-era naming or comments.
- Expand unit tests in this module to cover:
  - primary removes both `standby.signal` and `recovery.signal`;
  - replica creates `standby.signal` and removes `recovery.signal`;
  - any stale opposite signal is cleaned up;
  - `postgresql.auto.conf` is removed or moved aside before start;
  - managed config remains authoritative after the cleanup.

3. Remove Postgres-authored recovery side effects from `src/process/worker.rs`.
- Remove `-R` from the `pg_basebackup` command builder so cloning no longer writes recovery config or signal files on pgtuskmaster’s behalf.
- Keep the rest of the clone command unchanged unless a test proves another flag depends on `-R`.
- Update command-construction tests so they explicitly assert the exact arg vector without `-R`.
- Audit neighboring tests for stale assumptions about basebackup-created follow config and update them to expect pgtuskmaster-managed follow config instead.

4. Verify whether `src/process/jobs.rs` needs a type change and keep it minimal if not.
- The current `BaseBackupSpec` likely does not need new fields because follow intent is already carried by `ManagedPostgresStartIntent` in the startup path.
- During execution, confirm no process job type still encodes “clone and let pg_basebackup decide recovery”.
- If no additional type is needed, leave this file unchanged and note that the acceptance item was satisfied by confirming intent already lives outside the basebackup job type.
- If a gap appears, add only the smallest typed field needed to keep recovery ownership out of subprocess side effects.

5. Add one shared helper for previously managed on-disk recovery state, then consume it from both startup and HA.
- Extract the “read managed replica follow intent from disk” logic into a single helper owned by the managed-postgres surface instead of keeping separate ad hoc parsing in HA/runtime code.
- That helper must read only pgtuskmaster-authored artifacts (`pgtm.postgresql.conf` plus managed signal files), never `postgresql.auto.conf`, and must return explicit errors for incomplete or conflicting managed state.
- Unit-test that helper directly so runtime and HA do not need to duplicate parser edge-case coverage.

6. Align startup planning in `src/runtime/node.rs` with authoritative managed state.
- Replace any logic that treats the mere presence of `standby.signal` as the durable authority for resume behavior.
- Resume planning should derive start intent from DCS/runtime topology first and use the shared managed-state helper only as a consistency check or, when DCS explicitly allows it, as the source for previously managed follow data.
- If existing local managed replica state is reused, parse only pgtuskmaster-authored managed files, never `postgresql.auto.conf`.
- Add explicit handling for stale or conflicting managed artifacts:
  - stale `recovery.signal` on a primary path must be cleaned or rejected deterministically;
  - stale `standby.signal` without a reconstructible follow target must remain a hard error rather than silently guessing.
- Update runtime startup tests to prove:
  - primary resume no longer depends on stale signal leftovers;
  - replica resume rebuilds follow intent from authoritative sources;
  - unmanaged `postgresql.auto.conf` contents do not influence chosen startup intent.

7. Keep HA-driven start behavior aligned in `src/ha/process_dispatch.rs`.
- Update `start_intent_from_dcs()` and `existing_replica_start_intent()` so HA dispatch uses the same typed recovery/signal ownership model as runtime startup.
- Replace the local managed-config parser in this file with the shared helper so startup-planning semantics and HA restart semantics cannot diverge.
- Preserve the existing behavior where a known leader yields replica follow intent, but ensure the resulting managed start path is responsible for signal creation and stale-file cleanup.
- If “existing replica state without DCS leader” remains supported, constrain it to previously managed config artifacts only and make the failure mode explicit when the managed files are incomplete or conflicting.
- Expand dispatch tests to assert:
  - leader-driven start yields replica config plus the correct managed signal outcome;
  - primary start removes both signal files;
  - preserved replica state comes from managed config and not from `pg_basebackup -R` output conventions.

8. Update real-binary coverage to prove Postgres tooling is no longer authoritative.
- Replace the `pg_basebackup -R` usage in `src/pginfo/worker.rs` real tests with a clone flow that does not ask `pg_basebackup` to write recovery config.
- Add or update a real-binary test in the most appropriate harness location to demonstrate the full managed behavior:
  - clone a data dir with plain `pg_basebackup`;
  - inject a stale `postgresql.auto.conf` override and stale signal files;
  - run the managed startup/materialization path;
  - assert the managed config file contains the expected replica/primary settings;
  - assert only the correct signal file remains;
  - assert the active `postgresql.auto.conf` no longer exists or has been quarantined out of the authoritative path.
- Keep these tests against real PostgreSQL binaries, not mocks, because the task is specifically about file-level PostgreSQL behavior.

9. Update docs for the new ownership contract.
- Update `docs/src/lifecycle/bootstrap.md` to state that replica clone no longer relies on `pg_basebackup -R`; pgtuskmaster writes follow config and signal files itself before managed start.
- Update `docs/src/lifecycle/recovery.md` to describe that recovery/rejoin uses managed config and managed signal-file cleanup, and that `postgresql.auto.conf` is treated as unmanaged/out-of-band state.
- Update `docs/src/operator/configuration.md` so the operator-facing ownership contract mentions the `postgresql.auto.conf` quarantine policy, not only the `standby.signal` story.
- Update the relevant contributor docs, at minimum `docs/src/contributors/ha-pipeline.md`, so internal architecture docs describe the same authoritative managed-artifact set used by the code.
- Remove any stale wording that implies PostgreSQL tooling may author recovery posture for the node.

10. Execute the full verification suite only after code and docs are complete.
- Run `make check`.
- Run `make test`.
- Run `make test-long`.
- Run `make lint`.
- Fix every failure rather than downgrading coverage or skipping suites.
- Only after all four pass: set `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit all tracked and untracked task artifacts with the required `task finished ...` message, push, and stop immediately.

NOW EXECUTE
