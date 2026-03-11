## Bug: Greenfield mixed-fault heal can leave primary unqueryable <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
`ha_mixed_network_faults_heal_converges` still exposes a trustworthy HA/product failure after the harness cleanup fixes removed the stale-Docker resource noise from `make test-long`.

Observed on March 11, 2026 from:
- cleaned suite rerun: `make test-long`
- exported log: `target/nextest/ultra-long/logs/pgtuskmaster_rust__ha_mixed_network_faults_heal_converges__mixed_network_faults_heal_converges.log`

The harness successfully:
- bootstrapped `three_node_plain`
- waited for a stable primary as `initial_primary`
- isolated a different non-primary node from observer API access
- cut `initial_primary` off from DCS
- verified the DCS-cut primary entered fail-safe or otherwise lost primary authority safely
- verified there was no dual-primary evidence during the transition window
- healed all network faults

The failure happened on the post-heal recovery assertion:
- step failure: `Then exactly one primary exists across 3 running nodes as "final_primary"`
- observed error: `psql: error: connection to server at "node-b" (...), port 5432 failed: Connection refused`

This is a trustworthy failure because the cleaned rerun no longer failed from leaked Docker networks or disk exhaustion, and the scenario reached the intended mixed-fault choreography plus heal before failing on final Postgres reachability.

Explore and research the codebase first, then fix the post-heal recovery path so a healed mixed-fault cluster converges to one queryable primary instead of advertising a target whose Postgres endpoint still refuses connections.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
