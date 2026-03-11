## Bug: Greenfield clone failure can report rejoined replica before it is queryable <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/02-task-add-low-hanging-ha-quorum-and-switchover-cucumber-features-on-greenfield-runner.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/03-task-deep-clean-legacy-black-box-test-infrastructure-after-greenfield-migration.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/04-task-add-advanced-docker-ha-harness-features-and-migrate-remaining-black-box-scenarios.md</blocked_by>

<description>
`ha_clone_failure_recovers_after_blocker_removed` currently reaches a trustworthy failure during the real `make test-long` ultra-long suite.

Observed on March 11, 2026 from:
- `make test-long`
- log artifact: `target/nextest/ultra-long/logs/pgtuskmaster_rust__ha_clone_failure_recovers_after_blocker_removed__clone_failure_recovers_after_blocker_removed.log`

The scenario successfully:
- bootstrapped `three_node_plain`
- chose a non-primary `blocked_node`
- inserted proof row `1:before-clone-failure`
- enabled the `pg_basebackup` blocker on `blocked_node`
- killed `blocked_node`, wiped its data directory, and restarted it onto the fresh-clone path
- inserted proof row `2:during-clone-failure` while the blocked node was broken
- verified the blocked node was not queryable and never appeared in primary history during the fault window
- disabled the blocker, restarted the node again, and observed blocker evidence for `pg_basebackup`
- observed the node rejoin as a replica

The scenario then failed on the final convergence check:
- step failure: `And the 3 online nodes contain exactly the recorded proof rows`
- last observed error: `psql: error: connection to server at "node-c" (172.25.0.6), port 5432 failed: Connection refused`

This is trustworthy product evidence because the harness completed the intended clone-blocker choreography, captured the expected blocker evidence, and only failed after the scenario had already accepted the restarted node as rejoined. Explore and research the replica recovery path first, then fix the product so a node reported as rejoined after clone-blocker removal is actually queryable and fully converged.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
