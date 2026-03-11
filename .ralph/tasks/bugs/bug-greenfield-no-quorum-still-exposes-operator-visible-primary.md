## Bug: Greenfield no quorum still exposes operator-visible primary <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/02-task-add-low-hanging-ha-quorum-and-switchover-cucumber-features-on-greenfield-runner.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/03-task-deep-clean-legacy-black-box-test-infrastructure-after-greenfield-migration.md</blocked_by>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/04-task-add-advanced-docker-ha-harness-features-and-migrate-remaining-black-box-scenarios.md</blocked_by>

<description>
Two advanced greenfield wrappers now reach the same trustworthy no-quorum product failure: after DCS quorum majority loss, `pgtm primary` still returns an operator-visible primary instead of failing closed.

Observed on March 10, 2026 from:
- `make test-long`
- `target/nextest/ultra-long/logs/pgtuskmaster_rust__ha_no_quorum_enters_failsafe__no_quorum_enters_failsafe.log`
- `target/nextest/ultra-long/logs/pgtuskmaster_rust__ha_no_quorum_fencing_blocks_post_cutoff_commits__no_quorum_fencing_blocks_post_cutoff_commits.log`

The harness successfully:
- bootstrapped `three_node_plain`
- waited for an authoritative stable primary
- stopped a DCS quorum majority
- in the fencing scenario, also started a bounded concurrent write workload and captured commit outcomes before the quorum cut

Both scenarios then failed on the operator-visible no-primary assertion:
- step failure: `Then there is no operator-visible primary across 3 online node`
- observed error: `expected pgtm primary via \`node-a\` to fail, but it returned targets: node-b`

This is trustworthy product evidence because the harness completed the intended quorum-loss choreography and the failure is in the product-visible authority outcome, not in harness setup. Explore and research the codebase first, then fix quorum-loss handling so a cluster without DCS quorum does not keep exposing an operator-visible primary.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
