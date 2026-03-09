## Bug: Runtime-restart replica can stall before replaying post-restart writes <status>not_started</status> <passes>false</passes>

<description>
During `make test-long` on 2026-03-09, `e2e_multi_node_primary_runtime_restart_recovers_without_split_brain` reached a stable post-restart state with `node-2` as primary and both `node-1` and `node-3` reporting replica roles, but `node-3` never replayed the post-restart proof row within the scenario window.

The exported failure was:
- `timed out waiting for expected rows on node-3; expected=["1:before-restart", "2:after-restart"]; last_observation=rows=["1:before-restart"]`

Evidence:
- nextest log: `target/nextest/ultra-long/logs/pgtuskmaster_rust__ha_multi_node_failover__e2e_multi_node_primary_runtime_restart_recovers_without_split_brain.log`
- scenario timeline: `.ralph/evidence/13-e2e-multi-node/ha-e2e-primary-runtime-restart-recovers-without-split-brain-1773073335978.timeline.log`

Explore and research the codebase first, then fix the underlying replica catch-up / follow-target gap without weakening the HA guarantees or hiding the replication defect behind broader test relaxations.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
