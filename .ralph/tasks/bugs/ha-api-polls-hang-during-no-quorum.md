## Bug: HA API polls hang during no-quorum fail-safe observation <status>completed</status> <passes>true</passes>

<description>
During `make test-long`, `e2e_no_quorum_enters_failsafe_strict_all_nodes` can hang indefinitely after etcd quorum loss.
Live debugging showed that all nodes had already become non-primary by SQL evidence, but every `GET /ha/state` call to the node APIs hung instead of returning a `FailSafe` phase snapshot.
The test harness needed a SQL-only fallback to keep the suite progressing, but the underlying bug is that HA API requests become unresponsive during the no-quorum window.

Explore and research the codebase first, then fix the runtime/API hang so HA state remains observable during no-quorum fail-safe transitions.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Detailed Implementation Plan (Verified after skeptical review, 2026-03-06)

### Evidence-backed code facts

- `src/bin/pgtuskmaster.rs` builds a Tokio `current_thread` runtime for the real node binary.
- `src/runtime/node.rs` runs pginfo, DCS, process, HA, debug API, and API workers together under one `tokio::try_join!`, so any blocking call inside a worker can starve unrelated work on that runtime.
- `src/api/worker.rs` serves `GET /ha/state` by reading the latest debug snapshot; the route itself is cheap and does not touch etcd directly.
- `src/dcs/worker.rs` calls synchronous `DcsStore` methods (`write_local_member`, `drain_watch_events`) inside `step_once`.
- `src/ha/apply.rs` calls synchronous DCS operations (`write_leader_lease`, `delete_leader`, `clear_switchover`) during effect dispatch.
- `src/dcs/etcd_store.rs` implements those synchronous methods by sending a command to a background etcd thread and then blocking the caller with `std::sync::mpsc::recv_timeout(COMMAND_TIMEOUT)` where `COMMAND_TIMEOUT` is 2 seconds.
- `src/ha/worker.rs` publishes the next HA state only after `apply_effect_plan(...)` returns, so a stalled DCS side effect can delay publication of the `FailSafe` phase itself.

### Skeptical review conclusions

The draft plan was directionally right about blocking DCS calls, but it was too broad in the proposed fix surface.

- `GET /ha/state` is not itself doing DCS I/O. `src/api/worker.rs` serves it from the latest debug snapshot, so widening the API/controller surface to async is not the narrowest fix for this bug.
- The starvation root is the real node binary using a Tokio `current_thread` runtime while both `dcs::worker` and `ha::apply` perform synchronous DCS calls that can block for up to 2 seconds on etcd command replies.
- There is a second, separate observability bug in `src/ha/worker.rs`: the worker publishes the next HA state only after side-effect dispatch returns, so even with a non-starved API thread the visible `FailSafe` transition can lag behind a blocking leader-lease release.
- Because of those findings, the earlier plan sections about a general async `DcsStore` adapter, API/controller async widening, and a larger `dcs::worker` refactor are not the narrowest first move for this task and are removed from the execution plan below.

### Working hypothesis to execute

The no-quorum hang is not primarily a `/ha/state` handler bug. The most likely failure chain is:

1. etcd quorum is lost
2. DCS or HA worker issues a synchronous DCS operation that blocks up to 2 seconds
3. because the node runtime is `current_thread`, that blocking wait starves the API and debug workers
4. on primary nodes, the HA worker may also delay publishing the new `FailSafe` state because it publishes only after the blocking lease-release/cleanup side effect returns

That would explain both observed symptoms together: SQL evidence already showed the node was no longer primary, while every `GET /ha/state` request hung instead of returning a `FailSafe` snapshot.

### Planned execution phases for the `NOW EXECUTE` pass

#### 1. Reproduce the starvation path with targeted tests before changing runtime code

- [ ] Add a focused regression for runtime responsiveness instead of relying only on the multi-minute e2e.
- [ ] Build a compact fixture that co-runs:
  - [ ] `api::worker`
  - [ ] `debug_api::worker`
  - [ ] `ha::worker`
  - [ ] a fake DCS store whose lease-release or member-write operation blocks longer than the HTTP polling timeout
- [ ] Run that fixture under an explicit Tokio multi-thread runtime and assert that `GET /ha/state` still returns within the configured timeout while the blocking DCS call is pending.
- [ ] Add a second regression that targets `ha::worker::step_once` specifically: when fail-safe is selected and leader release blocks, subscribers must still observe `ha_phase=FailSafe` before the DCS side effect finishes.
- [ ] Keep these regressions narrow and deterministic; the long e2e remains confirmation, not the only proof.

Files expected to change in this phase:

- `src/api/worker.rs`
- `src/ha/worker.rs`
- `src/ha/apply.rs`
- `src/worker_contract_tests.rs` or nearby worker/unit test modules if that is the cleanest place for a compact cross-worker regression

#### 2. Fix the root runtime starvation in the real node binary

- [ ] Change `src/bin/pgtuskmaster.rs` from `tokio::runtime::Builder::new_current_thread()` to `tokio::runtime::Builder::new_multi_thread()`.
- [ ] Set an explicit worker-thread count greater than one instead of relying on the host default. Use enough capacity that simultaneous blocking in DCS and HA does not starve API/debug work; `worker_threads(4)` is the intended starting point.
- [ ] Keep the CLI behavior unchanged otherwise: same config parsing, same error handling, same `block_on` entrypoint.
- [ ] Do not churn unrelated current-thread unit tests or harness runtimes unless a failing test proves they must change.

Primary files:

- `src/bin/pgtuskmaster.rs`

#### 3. Publish HA state before potentially blocking DCS side effects

- [ ] Preserve the existing decision logic in `src/ha/decide.rs`; this task is about execution ordering and observability, not policy.
- [ ] In `src/ha/worker.rs`, compute the next HA state, publish the selected phase/decision immediately, and only then run `apply_effect_plan(...)`.
- [ ] Keep the phase/role transition logs correct and emitted once per actual transition.
- [ ] If effect dispatch later fails, publish a follow-up state update with the same tick/phase/decision and `WorkerStatus::Faulted(...)` so the already-selected `FailSafe` phase remains visible together with the dispatch error.
- [ ] Ensure a blocked leader-lease release or switchover cleanup cannot suppress or significantly delay visible fail-safe publication.

Primary files:

- `src/ha/worker.rs`
- `src/ha/apply.rs`
- `src/ha/events.rs` only if event ordering or duplicate-transition logging needs adjustment

#### 4. Keep the DCS/API surface narrow for this bug

- [ ] Do not introduce a general async wrapper around `DcsStore` in this task unless the focused regressions prove the runtime-thread fix is insufficient.
- [ ] Do not widen `api::controller` or `api::worker` to async solely for this issue; the failing read path is `GET /ha/state`, which reads the debug snapshot rather than touching DCS directly.
- [ ] Leave `dcs::worker::step_once` structurally unchanged unless the post-fix regressions still show an observability gap that cannot be explained by runtime starvation or HA publish ordering.
- [ ] If a local offload is still required after steps 2-3, prefer the smallest call-site-scoped change over a new cross-cutting DCS abstraction.

Primary files:

- `src/api/worker.rs`
- `src/api/controller.rs`
- `src/dcs/worker.rs`
- `src/dcs/store.rs`

#### 5. Tighten the long no-quorum scenario so it really catches observability regressions

- [ ] Keep the SQL fallback utilities for diagnostics, but stop allowing them to satisfy this scenario's success condition.
- [ ] For `e2e_no_quorum_enters_failsafe_strict_all_nodes`, require successful API observation during convergence:
  - [ ] every node must answer `/ha/state`
  - [ ] at least one node must be observed in `FailSafe` via API
  - [ ] all nodes must be observed non-primary, with SQL remaining supporting evidence only
- [ ] Prefer a dedicated strict helper or flag for this scenario rather than weakening the general harness for other fault tests.
- [ ] Update failure messages so future regressions distinguish:
  - [ ] API timeout / runtime starvation
  - [ ] stale HA publication
  - [ ] legitimate SQL or PostgreSQL reachability issues

Primary files:

- `tests/ha/support/multi_node.rs`
- `tests/ha_multi_node_failsafe.rs`
- `src/test_harness/ha_e2e/util.rs` only if a sharper assertion helper is needed

#### 6. Update docs to match the fixed runtime behavior

- [ ] Update operator/contributor docs that describe `/ha/state` observability during degraded coordination.
- [ ] Make the wording precise: the API is expected to remain responsive enough to expose fail-safe state during no-quorum transitions, even if DCS writes are failing or timing out.
- [ ] Explain the two relevant guarantees after the fix:
  - [ ] the node runtime no longer shares all workers on one blocked Tokio thread
  - [ ] HA publishes the selected fail-safe phase before slow DCS cleanup completes
- [ ] Remove any stale wording that implicitly accepts API blackouts during fail-safe as normal.
- [ ] If docs under `docs/src/` change, rebuild tracked generated docs artifacts as required by the repository workflow.

Likely docs:

- `docs/src/interfaces/node-api.md`
- `docs/src/operator/observability.md`
- `docs/src/lifecycle/failsafe-fencing.md`

#### 7. Verification and closeout order for the `NOW EXECUTE` pass

- [ ] Run the focused new regression tests first.
- [ ] Run the no-quorum long test directly while iterating.
- [ ] Once code and docs settle, run all required gates with no skips:
  - [ ] `make check`
  - [ ] `make test`
  - [ ] `make test-long`
  - [ ] `make lint`
- [ ] Update this task file with execution notes, exact tests/evidence used, completed acceptance checkboxes, and `<passes>true</passes>` only after all required gates pass.
- [ ] Run `/bin/bash .ralph/task_switch.sh`
- [ ] Commit all tracked changes, including `.ralph` state, with the required `task finished [task name]: ...` message
- [ ] `git push`

### Review delta applied in this pass

- [x] Removed the general async `DcsStore` adapter from the plan.
- [x] Removed the planned async widening of `api::controller` / `api::worker` from the first implementation pass.
- [x] Replaced the main fix strategy with an explicit multi-thread node runtime plus HA publish-before-dispatch ordering.
- [x] Tightened the long-test plan so API observability is required for success instead of SQL fallback satisfying the scenario.

NOW EXECUTE

## Execution Notes

- Switched the real node binary in `src/bin/pgtuskmaster.rs` to a Tokio multi-thread runtime with `worker_threads(4)` so synchronous DCS waits in HA or DCS workers cannot starve API/debug work on a single runtime thread.
- Changed `src/ha/worker.rs` so the selected HA state is published before running potentially blocking DCS side effects, and a follow-up `Faulted` status is published if effect dispatch fails after the state transition becomes visible.
- Added focused regressions for both sides of the bug:
  - `src/ha/worker.rs`: `step_once_publishes_failsafe_before_blocking_release_leader`
  - `src/worker_contract_tests.rs`: `ha_state_api_stays_responsive_while_ha_release_leader_blocks`
- Tightened the strict no-quorum multi-node helper in `tests/ha/support/multi_node.rs` so success requires live `/ha/state` observability rather than SQL-only fallback.
- Hardened DCS trust evaluation in `src/dcs/state.rs` and `src/dcs/worker.rs` to downgrade stale self/leader/member observations to `FailSafe`, which stabilized the long no-quorum integrity scenario after extended quorum loss.
- Updated docs in `docs/src/interfaces/node-api.md`, `docs/src/operator/observability.md`, and `docs/src/lifecycle/failsafe-fencing.md` to describe the expected API responsiveness and publish-before-cleanup behavior during fail-safe transitions.

### Verification Evidence

- Focused regression: `cargo test step_once_publishes_failsafe_before_blocking_release_leader`
- Focused regression: `cargo test ha_state_api_stays_responsive_while_ha_release_leader_blocks`
- Scenario check during iteration: `cargo test --test ha_multi_node_failsafe e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity -- --nocapture`
- Required gates passed on 2026-03-06:
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`
