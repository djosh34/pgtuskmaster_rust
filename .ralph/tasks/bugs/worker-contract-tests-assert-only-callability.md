---
## Bug: Worker contract tests only assert callability <status>done</status> <passes>true</passes>

<description>
[worker_contract_tests.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/worker_contract_tests.rs) primarily asserts that `step_once` functions are callable and return `Ok(())`, without validating resulting state changes or side effects. This means tests can pass even if core worker logic regresses or stops mutating state. Strengthen these tests with minimal behavioral assertions (state version bump, expected phase transitions, or expected publish effects), or split compile-time contract checks into non-test compile gates and add real behavioral tests.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

## Implementation Plan (Detailed)

### Research Summary (parallel exploration complete)
- `src/worker_contract_tests.rs` currently validates symbol existence and mostly callability.
- `step_once_contracts_are_callable` already wires all worker contexts, so it is the best place to add minimal behavior assertions without adding large fixture overhead.
- Workers already have deterministic publish/state side effects suitable for lightweight assertions:
  - `pginfo::worker::step_once` publishes new state on each call (`StatePublisher` version should increment from `0` to `1` in this test setup).
  - `dcs::worker::step_once` publishes state, sets `last_refresh_at`, and updates `last_published_pg_version` when local member write succeeds.
  - `process::worker::step_once` with empty inbox and no active runtime should keep `Idle` state unchanged (safe contract assertion: no unexpected transition to `Running`).
  - `ha::worker::step_once` should enter `FailSafe` for this contract fixture because the input DCS snapshot is `NotTrusted`; tick still increments to `1`, and worker remains `Running` if dispatch succeeds.
  - `api::worker::step_once` returns quickly on accept timeout and should not mutate `local_addr`.
  - `debug_api::worker::step_once` publishes a fresh snapshot (`SystemSnapshot` version bump from `0` to `1` + expected app/config projection).

### Planned Code Changes
- [x] Refactor `step_once_contracts_are_callable` into explicit per-worker assertion blocks inside the same test, preserving single-thread Tokio execution.
- [x] Add pginfo assertions after `step_once`:
  - [x] `pg_subscriber.latest().version == Version(1)`.
  - [x] `pg_subscriber.latest().value` has `common.worker == WorkerStatus::Running`.
  - [x] `common.sql == SqlStatus::Unreachable` for the intentionally invalid DSN contract stub.
- [x] Add dcs assertions after `step_once`:
  - [x] state publish version increments from `0` to `1`.
  - [x] `last_refresh_at.is_some()`.
  - [x] `dcs_ctx.last_published_pg_version == Some(pg_subscriber.latest().version)`.
  - [x] local cache contains self member id key.
- [x] Add process assertions after `step_once` with empty inbox:
  - [x] published state remains `ProcessState::Idle`.
  - [x] `running_job_id()` remains `None`.
  - [x] `last_outcome` stays `None`.
- [x] Add ha assertions after `step_once`:
  - [x] `ha_ctx.state.phase == HaPhase::FailSafe`.
  - [x] `ha_ctx.state.tick == 1`.
  - [x] `ha_ctx.state.worker == WorkerStatus::Running`.
  - [x] published HA subscriber version increments to `1` and mirrors ctx state.
- [x] Add api assertions after `step_once`:
  - [x] call `local_addr()` before and after `step_once`; assert unchanged.
  - [x] keep assertion bounded to timeout/no-connection behavior (no socket client required).
- [x] Add debug_api assertions after `step_once`:
  - [x] debug snapshot version increments to `1`.
  - [x] `snapshot.app == AppLifecycle::Starting`.
  - [x] `snapshot.config.version == Version(0)` (projection from unchanged config subscriber).
- [x] Keep `required_state_types_exist` and `worker_contract_symbols_exist` unchanged; they still provide compile-time contract value complementary to behavioral checks.

### Planned Verification Steps
- [x] `cargo test worker_contract_tests -- --nocapture` for focused feedback first.
- [x] Run required gates sequentially (no parallel cargo top-level commands):
  - [x] `make check`
  - [x] `make test`
  - [x] `make test`
  - [x] `make lint`
- [x] Update this task file checkboxes and set `<passes>true</passes>` only after all required gates pass.
- [x] Run `/bin/bash .ralph/task_switch.sh`.
- [x] Commit all changes including `.ralph` artifacts with message:
  - [x] `task finished worker-contract-tests-assert-only-callability: strengthen worker contract tests with behavioral assertions`
- [x] Append a concise learning to `AGENTS.md` if a new durable insight appears during implementation.

DONE
