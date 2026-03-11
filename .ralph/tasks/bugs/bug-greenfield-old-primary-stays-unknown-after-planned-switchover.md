## Bug: Greenfield old primary stays unknown after planned switchover <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>


<description>
`ha_planned_switchover_changes_primary_cleanly` now executes the intended planned switchover action on the greenfield Docker HA harness, but the former primary does not converge back to a replica role.

Observed on March 10, 2026 from:
- feature wrapper: `cargo nextest run --workspace --profile ultra-long --no-fail-fast --no-tests fail --target-dir /tmp/pgtuskmaster_rust-target --config 'build.incremental=false' --test ha_planned_switchover_changes_primary_cleanly`

The harness successfully:
- bootstrapped `three_node_plain`
- waited for an authoritative stable primary
- created the proof table
- inserted and verified `1:before-planned-switchover`
- recorded `pgtm primary` and `pgtm replicas`
- submitted `pgtm switchover request`
- observed a different stable primary as `new_primary`

The failure happened only on the post-action HA assertion:
- step failure: `And the node named "old_primary" remains online as a replica`
- observed error: `member 'node-b' role is 'unknown' instead of 'replica'`

This is a trustworthy product-side failure because the switchover action was accepted and leadership moved. The old primary remained online, but it did not settle back into `replica` role after handoff.

Explore and research the codebase first, then fix planned switchover demotion/rejoin behavior so the old primary reliably converges to a sampled replica role after a successful switchover.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
