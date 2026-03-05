---
## Task: High Prio Remove Shell Archive Wrapper and Current Wiring <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
**Goal:** Completely remove the generated shell archive wrapper implementation and all runtime wiring that depends on it.

**Scope:**
- Remove the wrapper module and script generation path in [src/logging/archive_wrapper.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/logging/archive_wrapper.rs).
- Remove managed-postgres wiring that injects wrapper-based `archive_command` / `restore_command` in [src/postgres_managed.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed.rs).
- Remove or update ingest assumptions that rely on wrapper-produced archive/restore JSONL output in [src/logging/postgres_ingest.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/logging/postgres_ingest.rs).
- Remove obsolete docs describing generated shell wrapper behavior in operator docs.
- Keep backup execution paths functional where possible, but do not keep any legacy wrapper compatibility paths.

**Context from research:**
- Current implementation writes `pgtuskmaster-pgbackrest-wal-wrapper.sh`, then sets Postgres commands to call it.
- Wrapper builds JSON in shell and appends directly to archive log JSONL; this is the behavior being intentionally deleted.
- This removal task is intentionally destructive to clear the slate before reintroducing a Rust-native mechanism.

**Expected outcome:**
- No generated shell wrapper exists in code or docs.
- No runtime path writes or executes `pgtuskmaster-pgbackrest-wal-wrapper.sh`.
- No config/docs promise wrapper-produced archive JSON lines.
- Codebase compiles and test suite passes without this mechanism.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Remove [src/logging/archive_wrapper.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/logging/archive_wrapper.rs) and module exports/imports that reference it.
- [x] Update [src/logging/mod.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/logging/mod.rs) to remove `archive_wrapper` module exposure and any related dead code.
- [x] Update [src/postgres_managed.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/postgres_managed.rs) to remove wrapper creation and wrapper-based `archive_command`/`restore_command` insertion.
- [x] Update [src/logging/postgres_ingest.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/logging/postgres_ingest.rs) to remove dependencies on wrapper-produced archive file semantics or to gate behavior so no stale promises remain.
- [x] Remove/update config schema/defaults/docs references that were specific to the shell wrapper’s archive/restore event logging.
- [x] Update operator documentation files that currently describe generated wrapper behavior, including [docs/src/operator/observability.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/operator/observability.md) and [docs/src/operator/configuration.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/operator/configuration.md), so they do not mention shell wrapper ownership.
- [x] Remove/replace tests that validate shell wrapper generation/execution (currently in deleted module tests), preserving coverage for remaining behavior.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

---

## Plan

### Research summary (what exists today)

- Shell wrapper generator lives in `src/logging/archive_wrapper.rs` and generates `pgtuskmaster-pgbackrest-wal-wrapper.sh`.
- Wrapper is currently the only runtime mechanism that makes Postgres call pgBackRest for WAL archiving/restore:
  - `src/postgres_managed.rs` injects `archive_command` and `restore_command` to execute the wrapper when `backup.enabled || backup.bootstrap.enabled`.
- Wrapper also defines the current “structured archive/restore event” observability story:
  - Wrapper writes JSONL events to a dedicated archive/restore event log file.
  - `src/logging/postgres_ingest.rs` tails that file and emits `LogProducer::PostgresArchive` records.
  - `src/logging/postgres_ingest.rs` normalizes wrapper JSON into `pgtuskmaster.backup.*` / `backup.*` attributes.
  - Operator docs/runbooks currently promise these wrapper events when backup is enabled.

### Decisions (make the removal consistent and non-legacy)

- Fully delete the generated shell wrapper implementation and all call sites.
- Remove the wrapper-only config knob for archive/restore event logging from the config schema (greenfield; no backwards compatibility).
  - This ensures the codebase cannot accidentally preserve/assume wrapper-era contracts.
- Remove the wrapper-era ingest producer path (`LogProducer::PostgresArchive`) and any ingest logic/tests that depend on it.
- After this task, **pgtuskmaster will not own nor inject Postgres `archive_command` / `restore_command`**.
  - WAL archiving/restore wiring will be reintroduced later via a Rust-native mechanism (separate task).
- Docs must be updated to explicitly remove wrapper promises and to avoid directing operators to wrapper-era archive/restore event logging knobs.

### Step-by-step execution plan (patch order)

#### 0) Baseline inventory (before edits)

- Run targeted searches (avoid `.ralph/progress/*` noise) and keep the output in mind while deleting call sites:
  - `rg -n "archive_wrapper|ensure_pgbackrest_wal_wrapper|PostgresArchive|pgtuskmaster-pgbackrest-wal-wrapper\\.sh" -S src tests examples docs`
  - `rg -n "pgtuskmaster-pgbackrest-wal-wrapper\\.sh" -S docs/book` (generated docs are checked in)
- Run `make check` once to confirm baseline is green before starting (if it isn’t, stop and create a bug task).

#### 1) Remove wrapper module + exports

- Delete `src/logging/archive_wrapper.rs`.
- Update `src/logging/mod.rs`:
  - remove `pub(crate) mod archive_wrapper;`
  - remove `LogProducer::PostgresArchive` variant (it is wrapper-only and will be removed in Step 3).

#### 2) Remove managed-postgres wrapper wiring

- Update `src/postgres_managed.rs`:
  - delete the `ensure_pgbackrest_wal_wrapper(...)` call.
  - delete injection of `archive_command`/`restore_command` strings that execute the wrapper.
  - ensure backup-related paths still validate and node startup still works (even if WAL archiving is no longer owned here).

#### 3) Remove wrapper-era ingest source and backup JSON normalization

- Update `src/logging/postgres_ingest.rs`:
  - remove the archive/restore event log tail/read loop and any wrapper-only origin strings.
  - remove backup-wrapper JSON normalization of `pgtuskmaster.backup.*` if it is now unused.
  - simplify cleanup protection logic that previously protected the archive log file.
- Remove or repurpose any enum variants / types only used for wrapper ingestion.

#### 4) Remove wrapper-era archive/restore event logging knob from config schema + validation

- Update `src/config/schema.rs`, `src/config/defaults.rs`, and `src/config/parser.rs` to remove wrapper-era archive/restore event logging config and validation.
  - delete/replace tests that assert the wrapper-era invariants (including the dot-segment log-dir coupling test).
- Update all config literals / fixtures / JSON payloads that still mention wrapper-era archive/restore event logging.
  - `tests/bdd_api_http.rs` (shared `sample_runtime_config`)
  - `examples/debug_ui_smoke_server.rs`
  - `src/api/fallback.rs`
  - `src/api/worker.rs`
  - `src/dcs/state.rs`
  - `src/dcs/worker.rs`
  - `src/dcs/store.rs`
  - `src/dcs/etcd_store.rs`
  - `src/ha/decide.rs`
  - `src/ha/worker.rs`
  - `src/runtime/node.rs`
  - `src/debug_api/worker.rs`
  - `src/worker_contract_tests.rs`

#### 5) Update or remove wrapper-focused tests

- Remove wrapper module tests (they will disappear with `archive_wrapper.rs` deletion).
- Update `src/logging/postgres_ingest.rs` tests:
  - remove wrapper generation + wrapper command injection into `postgresql.conf`.
  - remove assertions requiring wrapper-era archive producer records.
  - keep/strengthen assertions for the remaining ingest guarantees (pg_ctl log ingestion, pg_tool output capture, postgres.json/plain log parsing).

#### 6) Remove/update operator docs and runbooks that mention wrapper behavior

- Update docs sources to remove wrapper promises and the wrapper-era archive/restore event logging requirements:
  - `docs/src/operator/observability.md`
  - `docs/src/operator/configuration.md`
  - `docs/src/operator/troubleshooting.md`
  - `docs/src/operator/recovery-bootstrap-runbook.md`
- Ensure docs explicitly say:
  - no generated shell wrapper exists
  - no wrapper-produced JSONL archive/restore events exist
  - any recovery/bootstrap troubleshooting guidance no longer depends on wrapper event logs
- Regenerate checked-in rendered docs after source edits:
  - `docs/book/*` (at minimum `docs/book/print.html` and search index files)
  - Use `make docs-build` (and `make docs-hygiene` if present) to keep docs/book consistent.
- Update stale task docs that still promise wrapper-era behavior:
  - `.ralph/tasks/story-pgbackrest-managed-backup-recovery/02-task-managed-archive-recovery-bootstrap-config-takeover.md`
  - `.ralph/tasks/story-rust-system-harness/38-task-unified-structured-logging-and-postgres-binary-ingestion.md` (if it encodes wrapper-era contracts)

#### 7) Cleanup sweep (after edits)

- Run searches and ensure **zero** results in production/tests/docs surfaces (ignore `.ralph/progress/*` history):
  - `rg -n "archive_wrapper|ensure_pgbackrest_wal_wrapper|PostgresArchive|pgtuskmaster-pgbackrest-wal-wrapper\\.sh" -S src tests examples docs`
- Ensure no docs mention “generated wrapper” or “archive wrapper JSONL”.

#### 8) Verification (must be 100% green)

- Run, in order:
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`
  - `make docs-build` (docs are checked in; keep `docs/book` consistent)
  - `make docs-hygiene` (if present in this repo)
- If any failures require external binaries, install them (no skipping tests).

NOW EXECUTE
