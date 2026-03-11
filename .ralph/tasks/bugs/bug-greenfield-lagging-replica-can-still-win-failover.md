## Bug: Greenfield lagging replica can still win failover <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
The advanced greenfield wrapper `ha_lagging_replica_is_not_promoted` now reaches a trustworthy product failure: the degraded replica still appears in the primary history during failover.

Observed on March 10, 2026 from:
- `make test-long`
- failing log: `target/nextest/ultra-long/logs/pgtuskmaster_rust__ha_lagging_replica_is_not_promoted__lagging_replica_is_not_promoted.log`

The harness successfully:
- bootstrapped `three_node_plain`
- waited for an authoritative stable primary
- selected one replica as `degraded_replica`
- selected the other replica as `healthy_replica`
- created the proof table
- inserted `1:before-lagging-failover`
- isolated the old primary and the degraded replica on the postgres path
- started tracking primary history
- killed the old primary
- observed exactly one primary across the 2-node survivor set as `healthy_replica`

The scenario then failed on the no-promotion safety assertion:
- step failure: `And the primary history never included "degraded_replica"`
- observed error: `primary history unexpectedly included \`node-c\`: node-c`

This is trustworthy product evidence because the harness completed the intended lagging/degraded choreography and only failed after the cluster had already exposed a concrete primary-history record showing the degraded replica as authoritative. Explore and research the codebase first, then fix failover eligibility so an observably worse replica does not become primary during this scenario.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
