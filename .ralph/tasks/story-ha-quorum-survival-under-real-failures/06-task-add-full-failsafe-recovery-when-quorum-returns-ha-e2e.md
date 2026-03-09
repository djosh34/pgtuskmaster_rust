## Task: Add Full FailSafe Recovery When Quorum Returns HA E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Add realistic recovery coverage for a full-cluster fail-safe event where no node can coordinate, followed by restoration of quorum through reconnecting two or all nodes. The higher-order goal is to prove that fail-safe is a temporary safety mode and that the cluster deterministically returns to exactly one primary when enough connectivity is restored.

**Scope:**
- Extend HA E2E coverage in:
- `tests/ha/support/multi_node.rs`
- `tests/ha/support/partition.rs`
- `tests/ha_multi_node_failover.rs`
- `tests/ha_partition_isolation.rs`
- relevant harness helpers for coordinated stop/reconnect operations
- Add scenarios that intentionally drive all nodes into fail-safe through full connectivity loss or full-cluster outage, then restore:
- quorum via exactly two nodes,
- optionally all three nodes afterward.
- Require that once quorum returns, one and only one primary is elected before full heal-all is required.

**Context from research:**
- Current tests cover entry into fail-safe during etcd quorum loss and recovery after etcd restoration, but they do not directly exercise a “nobody can coordinate, then quorum returns partially” story as an operator would experience it.
- The user wants explicit proof that after fail-safe, reconnecting two nodes is enough to get one primary again.

**Expected outcome:**
- The suite proves fail-safe is recoverable through partial quorum restoration.
- Operators can trust that reconnecting enough nodes restores service without waiting for every node to return.
- The recovery path is validated both at the HA phase level and at the SQL write/read level.

</description>

<acceptance_criteria>
- [ ] Add at least one scenario that drives the cluster into a full fail-safe state with no coordinating quorum and then restores exactly two nodes, requiring election of exactly one primary before the final node is healed.
- [ ] Add or extend a scenario that restores all three nodes after fail-safe and verifies final SQL/data convergence.
- [ ] The scenarios assert no dual-primary window during recovery and require a stable primary before declaring success.
- [ ] The scenarios include explicit timeline evidence for fail-safe entry, quorum restoration, and primary election.
- [ ] The scenarios verify post-recovery SQL writes on the elected primary and convergence to all expected online nodes.
- [ ] The two-node restoration scenario must include a strict pre-heal assertion: once exactly two nodes are back, there is one stable primary and successful proof writes before the third node returns.
- [ ] The all-node restoration scenario must include a post-heal assertion that the final cluster converges to exactly one primary with identical proof rows on every node.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
