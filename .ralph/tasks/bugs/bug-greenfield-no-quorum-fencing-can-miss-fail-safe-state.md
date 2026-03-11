## Bug: Greenfield no quorum fencing can miss fail-safe state <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
The advanced greenfield wrapper `ha_no_quorum_fencing_blocks_post_cutoff_commits` now reaches a deeper trustworthy no-quorum product failure after the operator-visible-primary symptom is avoided: at least one running node still never reports fail-safe state after DCS quorum loss.

Observed on March 11, 2026 from:
- `make test-long`
- `target/nextest/ultra-long/logs/pgtuskmaster_rust__ha_no_quorum_fencing_blocks_post_cutoff_commits__no_quorum_fencing_blocks_post_cutoff_commits.log`

The harness successfully:
- bootstrapped `three_node_plain`
- waited for an authoritative stable primary
- created the bounded workload table and started concurrent writes
- stopped a DCS quorum majority
- verified there was no operator-visible primary across the 3 online nodes

The scenario then failed on the explicit fail-safe-state assertion:
- step failure: `And every running node reports fail_safe in debug output`
- observed error: `member \`node-a\` debug output did not contain fail_safe`

This is trustworthy product evidence because the harness completed the intended quorum-loss and workload choreography, already observed the fail-closed primary view, and only then failed on the product's internal fail-safe-state reporting contract. Explore and research the codebase first, then fix quorum-loss fencing so every surviving node enters and reports fail-safe state consistently once quorum is lost.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
