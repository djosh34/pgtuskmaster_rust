## Bug: Greenfield lone survivor remains primary after quorum loss <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/02-task-add-low-hanging-ha-quorum-and-switchover-cucumber-features-on-greenfield-runner.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/03-task-deep-clean-legacy-black-box-test-infrastructure-after-greenfield-migration.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/04-task-add-advanced-docker-ha-harness-features-and-migrate-remaining-black-box-scenarios.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/05-task-produce-ha-refactor-option-artifacts-email-review-and-stop-ralph.md</blocked_by>

<description>
The greenfield Docker HA harness now exposes the same quorum-loss behavior bug in multiple wrappers: after two nodes are stopped, the lone remaining node is still operator-visible as a primary instead of withdrawing primary visibility until quorum is restored.

Observed on March 10-11, 2026 from:
- feature wrapper: `cargo nextest run --workspace --profile ultra-long --no-fail-fast --no-tests fail --target-dir /tmp/pgtuskmaster_rust-target --config 'build.incremental=false' --failure-output immediate-final --final-status-level slow --status-level slow --test ha_two_node_outage_one_return_restores_quorum`
- failing parallel ultra-long wrapper: `ha_two_node_loss_one_good_return_one_broken_return_recovers_service`

The harness successfully:
- bootstrapped `three_node_plain`
- waited for an authoritative stable primary
- selected both non-primary nodes as the outage targets
- created the proof table
- inserted a proof row before the outage window
- killed the two chosen replica containers

The failure happened on the first post-outage quorum assertion in both scenarios:
- `ha_two_node_outage_one_return_restores_quorum`: `Then there is no operator-visible primary across 1 online node`
- `ha_two_node_loss_one_good_return_one_broken_return_recovers_service`: `Then the lone online node is not treated as a writable primary`
- observed error in both cases: the observer still resolved the lone survivor as the only primary target

The preserved scenario timeline shows the intended topology at the point of failure:
- `node-a` and `node-c` are `sampled=false role=unknown`
- `node-b` remains `sampled=true role=primary`
- warnings report both stopped nodes as unreachable and only `1/3` sampled members

This is a trustworthy product-side failure because the harness reached the intended degraded state and the observer reported the lone survivor as the only sampled primary instead of withdrawing primary visibility after quorum loss.

Explore and research the codebase first, then fix quorum-loss handling so a one-node survivor is not exposed as a writable/operator-visible primary after two-node outage.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
