## Bug: Greenfield replica flap can finish with restarted replica still unqueryable <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/02-task-deep-clean-legacy-black-box-test-infrastructure-after-greenfield-migration.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/03-task-add-low-hanging-ha-quorum-and-switchover-cucumber-features-on-greenfield-runner.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/04-task-add-advanced-docker-ha-harness-features-and-migrate-remaining-black-box-scenarios.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/05-task-produce-ha-refactor-option-artifacts-email-review-and-stop-ralph.md</blocked_by>

<description>
`ha_replica_flap_keeps_primary_stable` currently reaches a trustworthy failure during the real `make test-long` ultra-long suite.

Observed on March 11, 2026 from:
- `make test-long`
- log artifact: `target/nextest/ultra-long/logs/pgtuskmaster_rust__ha_replica_flap_keeps_primary_stable__replica_flap_keeps_primary_stable.log`

The scenario successfully:
- bootstrapped `three_node_plain`
- chose a non-primary `flapping_replica`
- completed three kill and restart cycles against that replica
- preserved the same `initial_primary` as the only primary during each flap cycle
- inserted proof rows `1:before-flap`, `2:during-flap-cycle-1`, `3:during-flap-cycle-2`, and `4:during-flap-cycle-3`
- reported the restarted replica as rejoined after each restart

The final convergence step then failed with:
- `timed out waiting for exact proof-row convergence on 3 nodes`
- last observed error: `psql: error: connection to server at "node-c" (172.27.0.6), port 5432 failed: Connection refused`

This is a trustworthy bug because the wrapper completes the intended flap behavior and only fails at the final all-node proof convergence check, showing that at least one restarted replica can still be unqueryable even after the scenario observed it as rejoined.

Explore and research the codebase first, then fix the replica flap recovery path so a repeatedly restarted replica is truly queryable and converged before the scenario reports success.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
