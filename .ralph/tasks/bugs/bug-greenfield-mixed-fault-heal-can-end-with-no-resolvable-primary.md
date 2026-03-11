## Bug: Greenfield mixed-fault heal can end with no resolvable primary <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
The advanced greenfield wrapper `ha_mixed_network_faults_heal_converges` exposes a distinct trustworthy post-heal recovery failure: after the intended mixed DCS plus API isolation is healed, the cluster can remain in a state where every observer seed rejects primary resolution because sampled members disagree on the leader.

Observed on March 11, 2026 from:
- `make test-long`
- exported log: `target/nextest/ultra-long/logs/pgtuskmaster_rust__ha_mixed_network_faults_heal_converges__mixed_network_faults_heal_converges.log`

The harness successfully:
- bootstrapped `three_node_plain`
- waited for a stable primary as `initial_primary`
- chose a different non-primary node as `api_isolated_node`
- created the proof table and inserted `1:before-mixed-faults`
- cut `initial_primary` off from DCS
- isolated `api_isolated_node` from observer API access
- verified the DCS-cut primary entered fail-safe or otherwise lost authority safely
- verified there was no dual-primary evidence during the mixed-fault window
- healed all network faults

The failure happened on the first post-heal recovery assertion:
- step failure: `Then exactly one primary exists across 3 running nodes as "final_primary"`
- observed error from every observer seed: `resolution error: cannot resolve primary from sampled cluster state: sampled nodes disagree on leader: <none>, node-a, node-b`

This is distinct from the earlier mixed-fault bugs because the fault choreography and the initial safety checks both succeeded; the broken behavior is specifically that the healed cluster still cannot converge to one resolvable leader afterward.

Explore and research the codebase first, then fix the post-heal recovery path so a healed mixed-fault cluster converges to exactly one resolvable primary instead of leaving sampled members disagreeing on the leader.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
