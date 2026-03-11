## Bug: Greenfield broken rejoin can stay offline after blocker removal <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/02-task-add-low-hanging-ha-quorum-and-switchover-cucumber-features-on-greenfield-runner.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/03-task-deep-clean-legacy-black-box-test-infrastructure-after-greenfield-migration.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/04-task-add-advanced-docker-ha-harness-features-and-migrate-remaining-black-box-scenarios.md</blocked_by>

<description>
The advanced greenfield wrapper `ha_broken_replica_rejoin_does_not_block_healthy_quorum` now reaches a trustworthy product failure after the intended blocker choreography completes: once the broken rejoin blocker is removed and the affected node is restarted, the cluster still never returns to three online nodes.

Observed on March 11, 2026 from:
- parallel ultra-long suite: `make test-long`
- failing log: `target/nextest/ultra-long/logs/pgtuskmaster_rust__ha_broken_replica_rejoin_does_not_block_healthy_quorum__broken_replica_rejoin_does_not_block_healthy_quorum.log`

The harness successfully:
- bootstrapped `three_node_plain`
- waited for a stable primary and selected separate healthy and broken replicas
- created the proof table and inserted `1:before-broken-rejoin`
- killed the chosen `broken_replica`
- enabled the deterministic `rejoin` blocker
- restarted the blocked node while keeping it marked unavailable
- inserted `2:during-broken-rejoin` through the healthy primary
- verified the healthy primary stayed unique
- verified the blocked node never became primary during the broken rejoin window
- disabled the blocker and restarted the node again

The failure happened only on the final convergence check:
- step failure: `Then the 3 online nodes contain exactly the recorded proof rows`
- observed error: `timed out waiting for exact proof-row convergence on 3 nodes; last observed error: expected at least 3 online connection targets, observed 2`

This is a trustworthy product-side failure because the harness reached the intended broken-rejoin fault window, preserved healthy-primary service during that window, removed the blocker, and only then timed out waiting for the healed node to come back online and converge.

Explore and research the codebase first, then fix the broken rejoin recovery path so a healed replica reliably becomes online and converges after the blocker is removed, without destabilizing the healthy quorum during the fault window.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
