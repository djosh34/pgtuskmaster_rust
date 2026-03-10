## Task: Add Whole Node Kill And Partial Recovery HA E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Add end-to-end HA scenarios that stop or kill whole nodes, not only PostgreSQL processes, and prove the cluster restores one primary before any full-cluster heal as long as DCS quorum still exists. The higher-order goal is to test the operator-visible failure modes the current suite misses: abrupt node death, clean node shutdown, partial recovery, and degraded-but-still-available quorum operation.

**Original user shift / motivation:** The user explicitly rejected softer shutdown semantics such as `pg_ctl stop -m immediate` being presented as "real node death". The task must move the harness toward an honest hard-failure boundary that behaves like machine death or `docker kill`: the node-owned process set disappears abruptly, without graceful cleanup, and the tests prove HA still converges correctly under that uglier real-world failure mode.

**Execution contract for this task:** The resulting HA E2E tests must remain parallel-safe. Do not "solve" binary locks, pid reuse confusion, or harness contention by serializing the suite. The task must preserve or improve compatibility with `cargo nextest`-style parallel execution by ensuring the runtime binary is built once and reused across test cases, with per-test namespaces and no suite-wide locking workaround.

**Scope:**
- Extend the HA E2E harness and multi-node scenarios in:
- `tests/ha/support/multi_node.rs`
- `tests/ha_multi_node_failover.rs`
- `src/test_harness/ha_e2e/handle.rs`
- `src/test_harness/ha_e2e/startup.rs`
- other directly related harness files under `src/test_harness/ha_e2e/`
- Add fixture support for explicit whole-node outage semantics with first-class helpers, not just ad hoc combinations of softer existing helpers.
- clean whole-node stop helper: stop the full node-owned process set in an orderly way, at minimum the HA runtime/API process and PostgreSQL for that node
- hard whole-node kill helper: use a hard OS kill path for the full node-owned process set, at minimum the HA runtime/API process and PostgreSQL for that node
- the hard-kill helper must use an explicit tracked-PID kill contract: hard-kill the `pgtuskmaster` runtime child PID that the harness started plus the current PostgreSQL postmaster PID read from `postmaster.pid`, and treat that explicit PID set as the node-owned kill contract
- do **not** introduce `systemd` as a required harness dependency for this task
- raw cgroups or container-backed execution are allowed only as explicit follow-up work if later needed, not as a prerequisite for delivering this task
- if a scenario/test layout colocates etcd with the node-owned process set, the whole-node helper must be able to include that colocated etcd process as part of the same node-down operation
- the helper must keep the node down until the scenario explicitly restarts it
- the helper must avoid ambiguous partial-liveness states where API is down but the runtime still updates DCS, or PostgreSQL is down but the runtime is still cleaning up
- the hard-kill helper is specifically required so future HA tests cannot claim “whole node failure” while only exercising mellow service-stop behavior
- Add focused scenarios for the user-requested real HA behaviors:
- take the primary node fully down and require a new leader before any heal,
- take a replica node fully down and prove the cluster stays healthy with one primary,
- take two whole nodes down so the remaining node enters fail-safe, then bring exactly one of the stopped nodes back and require election of exactly one primary without healing everything.
- This task does **not** need every scenario to execute both outage variants. It **does** need the combined coverage added by this task to include both clean-stop and hard-kill node-down cases somewhere in the new scenarios, because only the hard-kill variant proves the HA loop survives abrupt OS-level death.
- Keep scenarios deterministic and narrow. Do not combine unrelated fault stories in one test beyond what the scenario explicitly needs.

**Context from research:**
- Current failover tests call `stop_postgres_for_node(...)`, which only stops PostgreSQL and leaves the HA runtime alive.
- That is why current tests can pass even if a real hard node death deadlocks behind stale leader metadata.
- The harness already launches each runtime node as a real child process from the repo's `pgtuskmaster` binary in `src/test_harness/ha_e2e/util.rs` `spawn_runtime_node_process(...)` and wires it up during cluster startup in `src/test_harness/ha_e2e/startup.rs`.
- The harness and fixture already have first-class whole-node helpers in `src/test_harness/ha_e2e/handle.rs` and `tests/ha/support/multi_node.rs`, but the current so-called hard-kill path is still too soft for the user's standard because it uses `pg_ctl stop -m immediate` for PostgreSQL and graceful `shutdown_member(...)` for colocated etcd.
- The execution work is therefore no longer "invent whole-node helpers from scratch"; it is to tighten the failure boundary so "hard kill" truly means OS-level abrupt death of the node-owned process set and to keep the e2e assertions honest.
- The current suite is at risk of weak coverage if future tests compose separate graceful service stops and call that “node death”. This task must close that gap with real node-level helpers.
- The user explicitly wants real node-down semantics, including both clean-stop and hard-kill behavior, not only softer “service restart” semantics.
- The user explicitly does **not** want `systemd` to become a required dependency because the tests must remain runnable in containerized environments too.
- The user explicitly prefers manual tracked-PID hard killing over process-group/session assumptions because the harness owns a small fixed process set and `pg_ctl`/PostgreSQL startup grouping is not trusted as a stable contract.
- The user also explicitly wants HA e2e to keep running in parallel; if binary-build locking or runtime artifact contention appears, the fix must be nextest-friendly prebuild/reuse and namespace isolation rather than forcing serial execution.
- The repo already has a concrete nextest contract for this: `make test` and `make test-long` invoke `cargo nextest`, `.config/nextest.toml` keeps the `ha_*` binary split without a serial cap, and `tests/nextest_config_contract.rs` asserts that HA coverage must not regress into serialized execution.

**Expected outcome:**
- The HA E2E suite proves quorum survival under whole-node outage semantics, including both clean-stop and hard-kill cases, not only PostgreSQL death.
- There is explicit coverage for partial recovery after a two-node outage: one node returns, quorum is restored, and exactly one primary exists again before any full heal.
- The suite now matches the failures a real operator will try first.
- The hard-kill helper name now matches reality: it kills a node-owned OS process boundary rather than routing the failure through graceful service-control commands.

</description>

<acceptance_criteria>
- [ ] Add first-class harness helpers for clean whole-node stop, hard whole-node kill, and whole-node restart that ensure both the HA runtime and PostgreSQL for that node are down until explicitly restarted.
- [ ] The hard whole-node kill path must be a dedicated helper representing node death, not merely a scenario-local sequence of separate graceful stop calls.
- [ ] The helper must target the node-owned process set as one failure unit:
- [ ] at minimum HA runtime/API plus PostgreSQL for that node
- [ ] plus colocated etcd too when a scenario/test layout actually runs etcd as part of that node failure unit
- [ ] The primary hard-kill implementation uses an explicit tracked-PID kill contract that works in normal native runs and inside containers: hard-kill the tracked runtime PID and the current PostgreSQL postmaster PID from `postmaster.pid`, plus any explicitly tracked colocated etcd PID when topology says it belongs to the same node failure unit.
- [ ] The task must not assume `pg_ctl start` or any process-group/session behavior provides the kill boundary automatically; the harness must identify the current postmaster PID explicitly and kill it directly.
- [ ] Both whole-node outage helpers must prove both services are really down before the scenario continues:
- [ ] node API is unreachable,
- [ ] node PostgreSQL is unreachable,
- [ ] no further DCS heartbeat updates from that node are observed after the stop window.
- [ ] Register at least one new scenario in [tests/ha_multi_node_failover.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha_multi_node_failover.rs) covering complete primary-node outage and requiring a new stable primary before any heal of the dead node.
- [ ] That primary-death scenario must have a bounded pre-heal assertion: within the configured timeout, one and only one surviving node becomes primary while the dead node stays fully offline.
- [ ] Register at least one new scenario covering complete replica-node outage and proving the surviving cluster remains operational with exactly one primary.
- [ ] That replica-death scenario must prove the original primary remains writable and unique before any heal of the dead replica.
- [ ] Register at least one new scenario covering two complete node deaths followed by restart of exactly one dead node, with assertions that:
- [ ] the lone remaining node enters fail-safe while quorum is absent,
- [ ] restarting one stopped node restores quorum,
- [ ] exactly one primary is elected before the third node is healed,
- [ ] the cluster remains in a degraded but operational state rather than requiring full heal-all.
- [ ] Across the scenarios added by this task, there is explicit coverage for both outage variants:
- [ ] at least one scenario uses clean whole-node stop semantics,
- [ ] at least one scenario uses hard whole-node kill semantics,
- [ ] hard-kill coverage must use actual OS kill behavior for the node-owned process boundary and must not rely on `pg_ctl stop -m immediate` or other graceful-service control paths being labeled as node death.
- [ ] hard-kill coverage must exercise the first-class whole-node kill helper rather than manually stitching together softer stop operations inside the scenario.
- [ ] Every new scenario uses whole-node outage semantics, not only `pg_ctl stop` on the database.
- [ ] Every new scenario asserts no dual-primary window and verifies final SQL/data convergence on the nodes that are supposed to be online.
- [ ] Every new scenario includes explicit pre-heal and post-heal checkpoints:
- [ ] pre-heal quorum-survival result while some nodes are still down,
- [ ] post-heal convergence after the remaining nodes are brought back when that is part of the scenario.
- [ ] Timeline artifacts clearly distinguish outage type, node death or stop time, quorum loss, quorum restoration, and new-primary election.
- [ ] The HA E2E harness and scenarios remain safe under parallel execution; this task must not introduce suite-wide serialization, global mutable runtime-binary paths, or lock-based workarounds that force HA tests to run one-at-a-time.
- [ ] Verification for this task includes a nextest-friendly path that builds the `pgtuskmaster` runtime binary once and reuses it across parallel HA E2E tests instead of triggering repeated compile/build locking during the test run.
- [ ] If this task changes the HA runner split or binary reuse contract, it updates `.config/nextest.toml`, `tests/nextest_config_contract.rs`, and `Makefile` together so the no-serial parallel policy remains explicit and enforced.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

<implementation_plan>

## Execution Plan

1. Tighten the existing whole-node outage bookkeeping in the HA harness instead of inventing a second parallel model in the fixture.
   The harness already carries `WholeNodeOutageKind` and per-node outage state in [src/test_harness/ha_e2e/handle.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/ha_e2e/handle.rs). Keep the harness as the source of truth for "this node is intentionally down until restart", and remove any remaining task text or helper behavior that still assumes the old in-process runtime-task model.

2. Preserve and rely on the existing process-backed runtime model rather than planning a refactor that is already done.
   The runtime nodes are already launched as real child processes through `spawn_runtime_node_process(...)` in [src/test_harness/ha_e2e/util.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/ha_e2e/util.rs) and wired into startup in [src/test_harness/ha_e2e/startup.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/ha_e2e/startup.rs). Do not spend this task redoing that architectural step; instead audit the current child-process metadata and restart flow so it cleanly supports the stronger hard-kill semantics below.

3. Define and implement a per-node explicit tracked-PID hard-kill contract without depending on `systemd` or process-group/session semantics.
   Update the harness startup and lifecycle plumbing in [src/test_harness/ha_e2e/util.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/ha_e2e/util.rs), [src/test_harness/ha_e2e/startup.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/ha_e2e/startup.rs), and [src/test_harness/ha_e2e/handle.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/ha_e2e/handle.rs) so hard kill has a concrete contract that works under native and containerized test runs. The contract should be explicit and simple: hard-kill the tracked `pgtuskmaster` child PID and the current PostgreSQL postmaster PID from `postmaster.pid`, plus any explicitly tracked colocated etcd PID when topology says it is part of the same node failure unit. Do not leave any part of the node-death contract implicit in process-group/session behavior.

4. Replace the current too-soft hard-kill implementation with one that does not route node death through graceful service-control commands.
   Rework `kill_whole_node(...)` in [src/test_harness/ha_e2e/handle.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/ha_e2e/handle.rs) and the supporting helpers in [src/test_harness/ha_e2e/util.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/ha_e2e/util.rs) so the hard-kill variant sends actual OS kill signals to the node-owned process boundary and no longer relies on `pg_ctl stop -m immediate` being "close enough". If the task still needs a separate "immediate PostgreSQL stop" helper for non-node-failure scenarios, keep it as a distinct softer primitive and do not let the e2e stories in this task call it a hard node death.

5. Keep dedicated whole-node helpers on `TestClusterHandle` for clean stop, hard kill, and restart, but make the contracts stricter.
   `stop_whole_node(...)`, `kill_whole_node(...)`, and `restart_whole_node(...)` already exist in [src/test_harness/ha_e2e/handle.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/ha_e2e/handle.rs). Tighten them so the hard-kill helper represents abrupt node loss, the clean-stop helper represents orderly shutdown, repeated stop/kill calls against an already-offline node still fail fast, and restart only succeeds when the node is explicitly marked offline.

6. Make the HA E2E runtime-binary workflow explicitly nextest-friendly and parallel-safe.
   Update the harness/test-tooling contract so parallel HA E2E execution can reuse a single freshly built `pgtuskmaster` binary rather than fighting over build artifacts or re-triggering compile locks inside tests. Ground this in the repo's existing runner surface: `make test`, `make test-long`, `.config/nextest.toml`, and `tests/nextest_config_contract.rs`. If the HA runner split, binary reuse path, or nextest profile behavior must change, update those files together. Do not solve the problem by adding global serialization, disabling parallelism, setting `test-threads = 1`, or introducing a "run HA e2e alone" rule.

7. Keep optional colocated-etcd participation scoped and topology-driven so it does not block the main node-failure coverage.
   The current `TestConfig` in [src/test_harness/ha_e2e/config.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/ha_e2e/config.rs) models etcd members separately from database nodes, and the present three-node HA fixture does not colocate them. Add only the minimal explicit mapping needed for the helper API to include etcd when a future topology truly declares a colocated failure unit, and validate that mapping fast in config/startup. Do not entangle the first execution pass with a topology rewrite of the default multi-node fixture if the new scenarios can satisfy the user-visible outage goals against the existing separate-etcd layout.

8. Add post-stop verification helpers that prove the node is actually down before the scenario continues.
   Introduce fixture- or harness-level assertions that check all three required conditions after a whole-node outage: the API is unreachable, SQL/PostgreSQL is unreachable, and no fresh DCS/member heartbeat continues from that node after the stop window. The DCS-side verification should not rely only on API failure; it should inspect current HA/DCS observations and reject a case where the node’s API is gone but the runtime still appears fresh in DCS. Put these reusable checks in [tests/ha/support/multi_node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/multi_node.rs) near the other polling/assertion helpers so every new scenario can use the same proof step.

9. Extend `ClusterFixture` with explicit whole-node operations and timeline events, then stop using raw `stop_postgres_for_node(...)` in the new scenarios.
   Add fixture methods in [tests/ha/support/multi_node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/multi_node.rs) that wrap the new `TestClusterHandle` APIs, record whether the outage was a clean stop or a hard kill, and emit timeline messages that distinguish node-down time, quorum loss, quorum restoration, and new-primary election. Keep `stop_postgres_for_node(...)` for the existing older scenarios unless this task’s execution naturally replaces one, but every scenario added by this task must exclusively use the new whole-node helpers so the coverage cannot regress into process-only failure semantics.

10. Add a focused primary-whole-node-outage scenario that proves pre-heal failover with the dead primary still offline.
   Register a new test entry in [tests/ha_multi_node_failover.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha_multi_node_failover.rs) and implement the scenario in [tests/ha/support/multi_node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/multi_node.rs). The flow should be: bootstrap a stable primary, create a proof table and initial row, bring the primary fully down with one outage variant, assert the dead node remains offline, wait for exactly one surviving node to become the new stable primary before any heal, assert no dual-primary window during the transition, write post-failover proof rows on the new primary, then restart the dead node and verify replication/data convergence across the nodes that should be online. This scenario is the best place to use the hard-kill variant so the suite proves abrupt OS-level node death, not only orderly shutdown.

11. Add a focused replica-whole-node-outage scenario that proves the cluster stays healthy and writable without healing the dead replica first.
   Register a second test in [tests/ha_multi_node_failover.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha_multi_node_failover.rs) and implement it in [tests/ha/support/multi_node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/multi_node.rs). The flow should keep the original primary online, fully stop one replica with the clean-stop variant, prove the primary remains the only writable primary during the degraded state, perform a write before healing, and then restart the stopped replica and assert final row convergence on all intended online nodes. This gives the task explicit clean-stop coverage separate from the hard-kill primary-death coverage.

12. Add the partial-recovery quorum-restoration scenario as a separate narrow test rather than overloading the earlier failover stories.
   Implement a third scenario in [tests/ha/support/multi_node.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/multi_node.rs) and register it in [tests/ha_multi_node_failover.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha_multi_node_failover.rs). Start from a healthy three-node cluster, take two whole nodes down, prove the lone survivor enters fail-safe while quorum is absent, then restart exactly one of the stopped nodes and require quorum restoration plus election of exactly one primary before the third node is healed. Assert the cluster remains degraded-but-operational with two nodes online, accept writes in that state, and only then heal the final node and verify convergence. Keep this scenario deterministic by choosing explicit node IDs and fixed assertion order instead of mixing in unrelated workload stress.

13. Add or update harness-level unit tests so the new helper API itself is hard to misuse.
   Extend source-level tests under [src/test_harness/ha_e2e/](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/ha_e2e/) to cover the new node-outage bookkeeping: repeated stop/kill against an already-down node should error, restart of a node that is not down should error, and any colocation mapping validation should fail fast on unknown node IDs or unknown etcd member names. If a utility function is added for kill behavior or offline-state tracking, add targeted unit tests there rather than leaving correctness implied only by the long E2E paths.

14. Update docs after the code lands, using the `k2-docs-loop` skill rather than hand-waving the operator story.
   Once the helpers and scenarios are in place, use the repo’s `k2-docs-loop` skill to refresh the HA docs that describe failure handling so they explicitly distinguish PostgreSQL-process failure, runtime-only restart, clean whole-node stop, and hard whole-node kill. Remove any stale docs that imply the current suite already covered “node death” when it only covered `pg_ctl stop`, and make sure the partial-recovery quorum-restoration story is documented in operator-facing terms.

15. Execute full verification only after the code and docs are complete, then update task state and Ralph bookkeeping in one pass.
   Run `make check`, `make test`, `make test-long`, and `make lint` from the repo root and do not skip any suite. Include the nextest-friendly parallel HA E2E path required by this task and treat loss of parallel execution as a regression, not an acceptable compromise. If failures reveal harness flakiness or missing environment dependencies, fix them rather than weakening coverage or serializing the suite. Once all required commands pass, tick every completed acceptance checkbox in this task file, set `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit all changes including `.ralph` state with the required `task finished ...` message, push with `git push`, and then quit immediately.

</implementation_plan>

NOW EXECUTE
