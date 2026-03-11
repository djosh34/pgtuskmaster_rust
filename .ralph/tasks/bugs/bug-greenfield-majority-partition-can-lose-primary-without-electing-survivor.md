## Bug: Greenfield majority partition can lose primary without electing survivor <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>

<description>
Two advanced greenfield partition scenarios expose the same trustworthy product failure: after isolating the old primary onto the 1-side minority, the healthy 2-node majority remains observable but never elects a surviving primary.

Observed on March 10, 2026 from `make test-long`:
- `ha_full_partition_majority_survives_old_primary_isolated`
- `ha_minority_old_primary_rejoins_safely_after_majority_failover`

The harness successfully:
- bootstrapped `three_node_plain`
- identified the current primary as the minority-isolated node
- applied the intended full partition only between the isolated old primary and the other two nodes across DCS, API, and postgres paths
- kept the 2-node majority observable through the greenfield observer

Both wrappers then failed on the majority-election assertion:
- step failure: `Then exactly one primary exists across 2 running nodes as "majority_primary"`
- observed error: `cluster has no sampled primary`
- preserved status evidence reported `sampled 2/3` with the isolated old primary unreachable, which shows the majority pair itself remained visible

This is trustworthy product evidence because the harness completed the intended partition choreography and the majority side stayed observable; the broken behavior is that the majority pair does not converge to a primary. Explore and research the leader-loss and majority-election path first, then fix the product so a healthy observable 2-node majority can elect exactly one survivor when the old primary is isolated onto the minority side.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
