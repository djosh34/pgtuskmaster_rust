---
## Bug: HA Matrix Scenario Flakes Under Real HA <status>not_started</status> <passes>false</passes>

<description>
The deleted `e2e_multi_node_real_ha_scenario_matrix` mega-scenario was non-deterministic under real binaries.
During repeated reproductions it oscillated between:
- planned switchover never settling away from the original primary, even after multiple successful `/switchover` submissions
- all surviving nodes getting stuck in `WaitingPostgresReachable` with `leader=none`
- API transport resets while PostgreSQL/process workers continuously retried startup

The coverage was removed from the gate because dedicated ultra-long tests already cover planned switchover, unassisted failover, no-quorum fail-safe, and fencing with stronger focused assertions.

Explore and research the HA runtime and process-worker interaction first, then decide whether the underlying issue is:
- switchover action semantics under repeated intent writes
- process worker retry / startup-loop behavior after demotion and promotion churn
- DCS / leader-lease handling during combined switchover + no-quorum sequencing

Reintroduce a combined matrix scenario only after the runtime behavior is understood and the scenario is deterministic.
</description>

<acceptance_criteria>
- [ ] Root cause is identified from code + evidence, not just hidden by retries
- [ ] If runtime code changes are made: `make check` — passes cleanly
- [ ] If runtime code changes are made: `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] If ultra-long coverage is reintroduced or changed: `make test-long` — passes cleanly (ultra-long-only)
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
