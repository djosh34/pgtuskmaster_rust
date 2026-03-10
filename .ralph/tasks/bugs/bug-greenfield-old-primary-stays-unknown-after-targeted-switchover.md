## Bug: Greenfield old primary stays unknown after targeted switchover <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/02-task-deep-clean-legacy-black-box-test-infrastructure-after-greenfield-migration.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/03-task-add-low-hanging-ha-quorum-and-switchover-cucumber-features-on-greenfield-runner.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/04-task-add-advanced-docker-ha-harness-features-and-migrate-remaining-black-box-scenarios.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/05-task-produce-ha-refactor-option-artifacts-email-review-and-stop-ralph.md</blocked_by>

<description>
`ha_targeted_switchover_promotes_requested_replica` now executes the intended targeted switchover action on the greenfield Docker HA harness, but the former primary does not converge back to a replica role after the requested target takes leadership.

Observed on March 10, 2026 from:
- `make test-long`
- exported log: `target/nextest/ultra-long/logs/pgtuskmaster_rust__ha_targeted_switchover_promotes_requested_replica__targeted_switchover_promotes_requested_replica.log`

The harness successfully:
- bootstrapped `three_node_plain`
- waited for an authoritative stable primary as `old_primary`
- chose `target_replica` and `other_replica`
- created and verified the proof table state before switchover
- submitted `pgtm switchover request --switchover-to <target_replica>`
- observed `target_replica` become the only primary
- verified the primary history never included `other_replica`

The failure happened only on the post-action HA assertion:
- step failure: `And the node named "old_primary" remains online as a replica`
- observed error: `member 'node-b' role is 'unknown' instead of 'replica'`

This is a trustworthy product-side failure because the targeted switchover request was honored and leadership moved to the requested replica. The remaining broken behavior is the demotion or rejoin of the former primary after the handoff.

Explore and research the codebase first, then fix targeted switchover demotion or rejoin behavior so the old primary reliably converges to a sampled replica role after a successful targeted switchover.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
