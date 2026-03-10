## Task: Add Broken Returning Node And Single Good Recovery HA E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Cover the case where two nodes are down, quorum is lost, and then only one node returns successfully while another remains broken and cannot start PostgreSQL. The higher-order goal is to prove the cluster can recover to one primary as soon as quorum is restored by one good returning node, without requiring every failed node to recover cleanly.

**Execution contract for this task:** The HA E2E coverage added here must remain safe under parallel execution. Do not serialize the suite to avoid runtime-binary or pid-file contention. The task must preserve or improve nextest-friendly execution where the runtime binary is built once and reused across isolated test namespaces.

**Whole-node outage semantics for this task:**
- Reuse the explicit outage modes introduced in task 02.
- This task does not need to cover both clean-stop and hard-kill variants by itself, but it must use true whole-node outage semantics for whichever variant it chooses, not only database stop semantics.
- If this task uses the hard-kill path from task 02, that path must mean honest OS-level abrupt death, not `pg_ctl stop -m immediate`.
- If this task uses the hard-kill path from task 02, it must use the explicit tracked runtime PID plus current `postmaster.pid` PID kill contract from task 02.

**Scope:**
- Extend multi-node HA E2E coverage in:
- `tests/ha/support/multi_node.rs`
- `tests/ha_multi_node_failover.rs`
- `src/test_harness/ha_e2e/`
- reuse or extend existing failure-wrapper support for broken recovery binaries / broken startup paths
- Add a scenario with this exact shape:
- start 3 nodes,
- stop two whole nodes,
- observe fail-safe on the remaining lone node,
- restart one stopped node successfully,
- keep the other stopped or deliberately broken so PostgreSQL cannot come online,
- require the surviving 2-node quorum to elect exactly one primary before the third node is healed.
- Keep the broken node explicitly broken in a way the harness can prove, such as failed bootstrap/startup/recovery, rather than just “slow”.

**Context from research:**
- The user explicitly called out “two nodes die, one returns with disk issue, the other good one should still become leader”.
- The current suite has no real partial-recovery coverage of this shape.
- Existing wrapper-based recovery-failure tests already provide some patterns for deterministic breakage of node startup/recovery paths.
- The user wants HA E2E to keep running in parallel; this task must not weaken coverage by imposing serial execution.

**Expected outcome:**
- The suite proves quorum restoration by one healthy returning node is enough to recover service, even if another failed node stays broken.
- The cluster does not require “heal all” to regain one primary.
- The broken node remains out of the leader set until it is actually healthy.

</description>

<acceptance_criteria>
- [ ] Add one new scenario where two whole nodes are taken down, using one of the explicit whole-node outage variants from task 02, the lone survivor enters fail-safe, and exactly one returning healthy node restores quorum and primary election before the third node is healed.
- [ ] In that scenario, keep the third node stopped or broken so its PostgreSQL cannot come online, and prove that this does not block recovery of one primary on the healthy quorum pair.
- [ ] Reuse or extend deterministic node-breakage helpers so the broken node failure mode is explicit and reproducible.
- [ ] The scenario asserts exactly one primary after quorum restoration, no dual-primary window, and successful post-recovery writes on the elected primary.
- [ ] The scenario verifies the broken node never becomes an eligible primary while still broken.
- [ ] The scenario includes a bounded pre-heal assertion after only one node returns: service is restored with one stable primary and a successful proof write before the broken third node is healed.
- [ ] The scenario includes a post-heal assertion, if the broken node is eventually fixed in the same test, that it rejoins as replica only and does not disturb the elected primary.
- [ ] The scenario and harness changes remain compatible with parallel `nextest`-style execution and do not require suite-wide serialization or repeated in-test runtime-binary builds.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
