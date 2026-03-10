## Bug: Greenfield two-node quorum restore node exits before primary recovers <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/02-task-deep-clean-legacy-black-box-test-infrastructure-after-greenfield-migration.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/03-task-add-low-hanging-ha-quorum-and-switchover-cucumber-features-on-greenfield-runner.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/04-task-add-advanced-docker-ha-harness-features-and-migrate-remaining-black-box-scenarios.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/05-task-produce-ha-refactor-option-artifacts-email-review-and-stop-ralph.md</blocked_by>

<description>
`ha_two_node_outage_one_return_restores_quorum` reaches the intended quorum-loss state on the greenfield Docker HA harness, but the first restarted replica does not remain online long enough for the two-node subset to re-elect a primary.

Observed on March 10, 2026 from:
- `make test-long`
- exported log: `target/nextest/ultra-long/logs/pgtuskmaster_rust__ha_two_node_outage_one_return_restores_quorum__two_node_outage_one_return_restores_quorum.log`

The harness successfully:
- bootstrapped `three_node_plain`
- waited for an authoritative stable primary as `initial_primary`
- selected the two non-primary nodes as `stopped_node_a` and `stopped_node_b`
- created and verified the proof table state before the outage
- killed both non-primary nodes
- verified there was no operator-visible primary across the lone surviving node
- verified the lone survivor was not treated as a writable primary
- restarted only `stopped_node_a`

The failure happened on the first quorum-restore assertion:
- step failure: `Then exactly one primary exists across 2 running nodes as "restored_primary"`
- observed error: `expected 2 sampled members, observed 1 sampled out of 3 discovered`
- observer warnings included `degraded_trust=node node-b reports trust fail_safe`
- the scenario also recorded `terminal container failure detected: node-a=exited`

This is a trustworthy product-side failure because the harness completed bootstrap, executed the intended outage and recovery sequence, and only failed when the restarted node exited or stayed unsampleable instead of restoring a healthy two-node quorum.

Explore and research the codebase first, then fix the quorum-restore path so the first restarted replica remains online and the restored two-node subset can recover exactly one sampled primary.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
