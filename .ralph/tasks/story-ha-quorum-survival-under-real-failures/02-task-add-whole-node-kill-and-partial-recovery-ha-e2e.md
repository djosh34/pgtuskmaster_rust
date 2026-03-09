## Task: Add Whole Node Kill And Partial Recovery HA E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Add end-to-end HA scenarios that stop or kill whole nodes, not only PostgreSQL processes, and prove the cluster restores one primary before any full-cluster heal as long as DCS quorum still exists. The higher-order goal is to test the operator-visible failure modes the current suite misses: abrupt node death, clean node shutdown, partial recovery, and degraded-but-still-available quorum operation.

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
- The current harness can restart a runtime process, but it does not yet express “this whole node is down and stays down” as a first-class test operation.
- The current suite is at risk of weak coverage if future tests compose separate graceful service stops and call that “node death”. This task must close that gap with real node-level helpers.
- The user explicitly wants real node-down semantics, including both clean-stop and hard-kill behavior, not only softer “service restart” semantics.

**Expected outcome:**
- The HA E2E suite proves quorum survival under whole-node outage semantics, including both clean-stop and hard-kill cases, not only PostgreSQL death.
- There is explicit coverage for partial recovery after a two-node outage: one node returns, quorum is restored, and exactly one primary exists again before any full heal.
- The suite now matches the failures a real operator will try first.

</description>

<acceptance_criteria>
- [ ] Add first-class harness helpers for clean whole-node stop, hard whole-node kill, and whole-node restart that ensure both the HA runtime and PostgreSQL for that node are down until explicitly restarted.
- [ ] The hard whole-node kill path must be a dedicated helper representing node death, not merely a scenario-local sequence of separate graceful stop calls.
- [ ] The helper must target the node-owned process set as one failure unit:
- [ ] at minimum HA runtime/API plus PostgreSQL for that node
- [ ] plus colocated etcd too when a scenario/test layout actually runs etcd as part of that node failure unit
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
- [ ] hard-kill coverage must use actual OS kill behavior for both the runtime and PostgreSQL, not only graceful stop helpers.
- [ ] hard-kill coverage must exercise the first-class whole-node kill helper rather than manually stitching together softer stop operations inside the scenario.
- [ ] Every new scenario uses whole-node outage semantics, not only `pg_ctl stop` on the database.
- [ ] Every new scenario asserts no dual-primary window and verifies final SQL/data convergence on the nodes that are supposed to be online.
- [ ] Every new scenario includes explicit pre-heal and post-heal checkpoints:
- [ ] pre-heal quorum-survival result while some nodes are still down,
- [ ] post-heal convergence after the remaining nodes are brought back when that is part of the scenario.
- [ ] Timeline artifacts clearly distinguish outage type, node death or stop time, quorum loss, quorum restoration, and new-primary election.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
