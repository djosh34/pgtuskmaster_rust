## Task: Add HA Restart And Leadership Churn E2E Coverage <status>completed</status> <passes>true</passes>

<priority>low</priority>

<description>
**Goal:** Add behavioural coverage for HA failures that are not simple one-shot failovers: process restarts, repeated leadership churn, and degraded-cluster transitions that happen across multiple events in sequence. The higher-order goal is to test the system the way real operators break it: not with one clean outage, but with restarts, retries, and more than one leadership event in the same test.

This task should specifically lean into the currently missing failure choreography that came out of research and the user feedback:
- node process restarts
- repeated failovers or quick successive leader changes
- additional HA failure cases beyond the existing etcd/API partition and single-stop failover scenarios

**Scope:**
- Extend the HA e2e suite with new scenarios focused on restart and churn behaviour.
- Prefer scenarios that remain parallel-safe and deterministic under the existing harness isolation model.
- Candidate coverage areas include:
- restarting the HA node process on the primary or replica and verifying stable recovery without role confusion
- inducing two leadership transitions in one scenario and proving the cluster does not accumulate stale authority or dual-primary evidence
- failing over when one replica is already degraded or unavailable
- issuing switchover intent to a node that cannot currently become a healthy target and verifying clear rejection or safe non-action
- Keep each scenario small enough to have one unambiguous failure story, instead of mixing too many fault types at once.

**Context from research:**
- The current suite already covers planned switchover, unassisted failover, no-quorum fail-safe, fencing, and a set of etcd/API partition cases.
- The current suite does not cover process restart behaviour, crash-loop style recovery, or multi-step leadership churn in a single scenario.
- The user explicitly asked for more HA failure tests and does not accept flaky tests as normal.
- The user wants full parallelism preserved, so these scenarios must stay isolated and must not rely on global serialization for correctness.

**Expected outcome:**
- The behavioural suite no longer treats HA as only a single clean transition problem.
- The system is validated against restart and churn patterns that commonly expose stale-leader, rejoin, or state-machine bugs.
- The new tests remain deterministic and parallel-safe.

</description>

<acceptance_criteria>
- [x] Add one or more e2e scenarios covering HA node/process restart behaviour with explicit assertions about recovered leadership and cluster convergence.
- [x] Add one or more e2e scenarios covering more than one leadership transition in the same scenario, with strict no-dual-primary assertions across the whole observation window.
- [x] At least one new scenario covers failover or switchover behaviour when the cluster is already degraded before the transition begins.
- [x] Each new scenario records a clear timeline artifact and uses unique names/tables/artifact paths so parallel execution remains safe.
- [x] The scenarios explicitly verify final SQL/data convergence, not only API phase changes.
- [x] The scenarios avoid ambiguous “something changed” assertions and instead check specific expected leadership and replication outcomes.
- [x] If a target node is intentionally ineligible or unhealthy, the scenario verifies the system fails safely rather than silently accepting an impossible transition.
- [x] The implementation reuses existing fixture helpers where possible, but extracts new helpers when that meaningfully reduces duplication and clarifies the scenario intent.
- [x] The implementation does not touch CLI code.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If the new scenarios belong in the long-running gate: `make test-long` — passes cleanly
</acceptance_criteria>

## Execution Plan

### Planning notes locked in before execution

- Current research shows the multi-node HA suite already has the right convergence primitives in [tests/ha/support/multi_node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/multi_node.rs): resilient stable-primary waits, SQL convergence checks, no-dual-primary observation windows, timeline artifacts, and external PostgreSQL/etcd fault injection.
- The missing seam for this task is not failover detection; it is node-process restart orchestration. The HA harness in [src/test_harness/ha_e2e/startup.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/ha_e2e/startup.rs) spawns each runtime node from an in-memory `runtime_cfg`, waits for API readiness, and then only retains a `JoinHandle`. After startup, the handle no longer retains the runtime config or log path needed to respawn one node deterministically.
- The current runtime-task bookkeeping is also restart-hostile. `ClusterFixture` stores `tasks: Vec<JoinHandle<...>>` and `ensure_runtime_tasks_healthy()` assumes task order matches node order, even though it uses `swap_remove`. A restart-capable helper should make task ownership explicit by node id instead of relying on positional alignment.
- Targeted switchover support already exists in production code through `switchover_to` validation in [src/api/controller.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/api/controller.rs) and CLI client support in [src/cli/client.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/client.rs). This task should reuse that surface in E2E coverage rather than adding new CLI behavior.
- The new scenarios belong in [tests/ha_multi_node_failover.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha_multi_node_failover.rs) and [tests/ha/support/multi_node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/multi_node.rs). Keeping them under the existing `tests/ha_*.rs` layout preserves the current `make test-long` routing and parallel nextest scheduling.
- There is no `update-docs` skill available in the current session. Execution should therefore update the relevant docs directly in-repo and then rely on `make lint` plus the normal docs build/lint wiring for verification.

### Phase 1: Make the HA harness capable of deterministic node-process restarts

- [x] Extend the HA harness handle layer so a single node can be restarted without rebuilding the whole cluster.
- [x] Add the restart metadata that is currently discarded after startup:
  - [x] persist the per-node runtime configuration needed to call `crate::runtime::run_node_from_config(...)` again,
  - [x] persist the per-node PostgreSQL log path used by `wait_for_node_api_ready_or_task_exit(...)`,
  - [x] keep existing node identity, data-dir, API, raw PostgreSQL, and SQL/proxy port metadata intact.
- [x] Replace positional runtime-task tracking with node-keyed tracking that survives restarts cleanly.
  - [x] Refactor the `TestClusterHandle` / `ClusterFixture` task storage so health checks, shutdown, and per-node replacement are keyed by node id rather than implicit vector order.
  - [x] Preserve current shutdown behavior, but make it iterate explicit node-task records so a restarted node cannot be misattributed or leaked.
- [x] Add a focused harness helper in the `ha_e2e` support layer that restarts one HA runtime process:
  - [x] abort or await the existing runtime task for the selected node,
  - [x] confirm API unavailability or task exit so the restart event is real rather than only queued,
  - [x] spawn a fresh local runtime task from the saved config,
  - [x] wait for that node API to become ready again through the existing readiness helper,
  - [x] replace the stored task handle for that node.
- [x] Keep this helper as a node-process restart only. It should not directly stop PostgreSQL, rewrite DCS state, or mutate any cluster-global fixture state beyond the selected node task replacement.
- [x] Add focused harness coverage for the restart bookkeeping layer, but do not require a second live multi-process restart smoke test outside the scenario suite.
  - [x] Prefer a small `ha_e2e` unit/integration test around node-keyed task replacement and restart metadata preservation near [src/test_harness/ha_e2e/mod.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/ha_e2e/mod.rs).
  - [x] Let the new multi-node restart scenario be the live end-to-end proof that a real runtime process can be stopped and respawned cleanly. Adding a second live restart smoke test would duplicate long-test cost without adding much signal.

### Phase 2: Add reusable multi-node fixture helpers for restart, targeted switchover, and repeated transition assertions

- [x] Extend [tests/ha/support/multi_node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/multi_node.rs) with small helpers instead of open-coding restart and churn logic inside each scenario.
- [x] Add a fixture helper to restart the HA runtime process for a named node and emit explicit timeline entries before stop, after API loss, after restart, and after API recovery.
- [x] Add a targeted switchover rejection helper that uses the existing API client surface directly with `switchover_to`.
  - [x] Use `CliApiClient::post_switchover(Some(target))` or an equivalent direct client call inside the fixture, instead of routing this path through the current CLI wrapper that only proves accepted requests.
  - [x] Support failure assertions when an unhealthy or ineligible target must be rejected, preserving the concrete error text from the API response so the scenario can prove safe rejection rather than a silent no-op.
  - [x] Capture the healthy-target misbehavior uncovered during execution as a follow-up bug instead of hiding it in the churn scenario.
- [x] Add a helper for repeated leadership transitions that records the ordered primary sequence, not just the final primary.
  - [x] Reuse the existing resilient stable-primary waits.
  - [x] Add a stronger assertion than the current single-transition `assert_phase_history_contains_failover(...)`, because this task needs to prove two distinct leadership changes in one scenario.
- [x] Add any small SQL-proof helpers needed to keep the new scenarios terse and unambiguous:
  - [x] unique table-name generation per scenario,
  - [x] seed rows before each fault,
  - [x] final row-set convergence checks across all nodes,
  - [x] optional helper to assert a node remains the only SQL-visible primary during a bounded observation window when the scenario depends on that.

### Phase 3: Add one focused node-process restart scenario

- [x] Add a new multi-node E2E scenario dedicated to HA node-process restart behavior and register it in [tests/ha_multi_node_failover.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha_multi_node_failover.rs).
- [x] Keep the story narrow: restart the elected primary node's HA runtime process, not PostgreSQL itself, so the test isolates controller/API restart and lease-handling behavior rather than conflating it with a full database crash.
- [x] Planned scenario flow:
  - [x] start a 3-node fixture and wait for a stable bootstrap primary,
  - [x] create a task-specific proof table and replicate a baseline row to all nodes,
  - [x] begin a bounded no-dual-primary observation window that spans the restart event,
  - [x] restart the HA runtime process on the current primary node,
  - [x] wait for the restarted node API to return and for the cluster to settle back to exactly one stable primary,
  - [x] allow either safe outcome: the same member remains primary after a short restart, or one different member becomes the new stable primary after a clean leadership handoff,
  - [x] reject unsafe outcomes: dual-primary evidence, no recoverable stable primary, or divergent SQL row sets after the restart,
  - [x] write a post-restart proof row on the final primary and verify it converges to every node.
- [x] Record the timeline artifact so it is obvious whether leadership stayed put or changed during the restart and when convergence was restored.

### Phase 4: Add one repeated-leadership-churn scenario with strict whole-window no-dual-primary evidence

- [x] Add a second scenario dedicated to two leadership transitions in one test and register it in [tests/ha_multi_node_failover.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha_multi_node_failover.rs).
- [x] Use deterministic failover choreography to keep the scenario stable while still exercising leadership churn.
- [x] Planned scenario flow:
  - [x] start a 3-node fixture and wait for a stable bootstrap primary,
  - [x] seed a unique proof table and replicate an initial row everywhere,
  - [x] stop PostgreSQL on the bootstrap leader and wait for a new stable primary that differs from the original leader,
  - [x] assert former-primary demotion or unreachability the same way the existing failover scenarios do,
  - [x] write and replicate a second proof row on the first successor,
  - [x] stop PostgreSQL on that new leader and wait for a third stable state that differs from the immediately prior leader,
  - [x] allow either return-to-original-leader or promotion of the remaining replica, but require two distinct ordered leadership changes in the recorded sequence,
  - [x] sample HA state across the whole churn window and assert no dual-primary evidence in the aggregated stats,
  - [x] write a final proof row on the last stable primary and verify all rows converge across every node.
- [x] Strengthen the final assertions so they prove more than "leadership changed at least once":
  - [x] ordered primary sequence contains three stable snapshots (`primary_a -> primary_b -> primary_c_or_a`),
  - [x] phase history shows demotion of each relinquishing leader,
  - [x] SQL-visible convergence is enforced through replicated proof rows, not only API phase changes,
  - [x] the row set is identical on all nodes after the second transition.

### Phase 5: Add degraded-cluster transition coverage as two separate scenarios, not one overloaded scenario

- [x] Cover degraded-cluster behavior with separate scenarios so each failure story stays unambiguous.

- [x] Scenario A: failover while one replica is already unavailable.
  - [x] start a 3-node fixture and wait for a stable primary,
  - [x] identify one replica as the intentionally degraded node and stop PostgreSQL on that replica,
  - [x] verify the remaining healthy pair still converges to one primary and one viable replica view before the main transition begins,
  - [x] stop PostgreSQL on the current primary to force failover under degraded conditions,
  - [x] require promotion of the one remaining healthy replica,
  - [x] assert no dual-primary evidence during the degraded failover window,
  - [x] write a post-failover proof row on the promoted node and verify convergence on every node that becomes reachable again,
  - [x] explicitly confirm the degraded replica does not silently remain a valid promotion candidate while it is down.

- [x] Scenario B: targeted switchover request to an unhealthy or ineligible node is rejected safely.
  - [x] start a fresh 3-node fixture so the rejection story is isolated from the degraded failover story above,
  - [x] wait for a stable primary and choose one replica to degrade by stopping PostgreSQL on it,
  - [x] issue a targeted switchover request to that degraded replica through the direct fixture API client helper so the scenario can inspect the rejection body deterministically,
  - [x] assert the request is rejected with the existing "not an eligible switchover target" style error rather than accepted and left hanging,
  - [x] verify the primary does not change as a side effect of the rejected request,
  - [x] assert no dual-primary evidence in the rejection observation window,
  - [x] write and replicate a proof row through the still-healthy leader path to show the cluster remained operational after the safe rejection.
- [x] Keep the degraded-failover scenario and the ineligible-target scenario in separate tests even if they share setup helpers. Combining them would make it harder to diagnose whether a failure came from target validation or from degraded failover recovery.

### Phase 6: Keep the scenarios parallel-safe, artifact-rich, and scoped to HA tests only

- [x] Reuse the existing isolated namespace, dynamic ports, unique-table naming, and timeline artifact patterns so the new scenarios remain safe under parallel nextest execution.
- [x] Give each scenario a unique scenario name, table name, and artifact prefix derived from the per-test token generator.
- [x] Continue using only allowed post-start controls:
  - [x] `GET /ha/state` observation,
  - [x] admin switchover requests through the API/CLI path,
  - [x] SQL reads and writes for proof data,
  - [x] external PostgreSQL or process fault injection.
- [x] Do not touch CLI production code. If a helper is needed for targeted switchover requests, keep it inside the E2E fixture and reuse existing client APIs.

### Phase 7: Docs and verification

- [x] Update the docs that describe what the long HA gate actually covers.
  - [x] Minimum expected docs touch: [docs/src/how-to/run-tests.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/how-to/run-tests.md), so it explicitly says the long HA bucket now covers node-process restarts, repeated leadership churn, degraded failover, and targeted switchover rejection.
  - [x] Execution did not reveal a second stale HA operations page that required immediate correction.
- [x] Run the full required verification sequence after implementation:
  - [x] `make check`
  - [x] `make test`
  - [x] `make test-long`
  - [x] `make lint`
- [x] Only after all gates pass:
  - [x] mark completed acceptance criteria and execution-plan checkboxes,
  - [x] set `<passes>true</passes>`,
  - [x] run `/bin/bash .ralph/task_switch.sh`,
  - [x] commit all tracked changes including `.ralph` files with the required `task finished [task name]: ...` message and explicit test evidence,
  - [x] push with `git push`.

NOW EXECUTE
