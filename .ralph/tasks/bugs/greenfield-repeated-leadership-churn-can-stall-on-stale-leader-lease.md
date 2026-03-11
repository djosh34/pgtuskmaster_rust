## Bug: Greenfield Repeated Leadership Churn Can Stall On Stale Leader Lease <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
The advanced greenfield wrapper `ha_repeated_leadership_changes_preserve_single_primary` can reach a trustworthy repeated-failover product failure where the third leader is never established because a stale leader lease blocks the remaining healthy node.

Observed on March 10, 2026 from:
- focused wrapper run: `cargo nextest run --profile ultra-long --no-fail-fast --target-dir /tmp/pgtuskmaster_rust-target --config build.incremental=false --test ha_repeated_leadership_changes_preserve_single_primary`
- live cluster evidence from run `cucumber_tests/ha/runs/repeated_leadership_changes_preserve_single_primary/repeated-leadership-changes-preserve-single-primary-1773178102283-191365`

Current trustworthy scenario shape:
- bootstrap `three_node_plain`
- record `primary_a`
- kill `primary_a`
- observe a different stable `primary_b`
- restart `primary_a` and wait for it to rejoin as a replica
- cut the restarted old primary off from DCS so it remains ineligible for the next promotion
- kill `primary_b`
- expect the remaining healthy node to become `primary_c`

Instead of establishing that third primary, the cluster degrades into a stale-leader state:
- `pgtm status` via the live observer shows `sampled 2/3`, `leader_mismatch`, and degraded trust
- `node-b` reports `role=replica`, `trust=not_trusted`, `phase=fail_safe`, `leader=node-a`
- `node-c` reports `role=replica`, `trust=fail_safe`, `phase=fail_safe`, `leader=node-c`
- `node-a` is down and no longer sampleable
- live node logs repeatedly show `ha action failed ... path already exists: /ha-cucumber-cluster/leader` while the healthy node tries to acquire leadership

This indicates the second failover can stall on stale leader state instead of converging to the only remaining eligible primary. Explore and research the DCS leader-loss / stale-lease recovery path first, then fix the product so repeated failovers either clear or safely supersede dead leadership records quickly enough for the next healthy node to promote.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
