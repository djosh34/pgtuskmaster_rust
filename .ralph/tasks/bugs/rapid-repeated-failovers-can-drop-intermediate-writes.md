## Bug: Rapid Repeated Failovers Can Drop Intermediate Writes <status>not_started</status> <passes>false</passes>

<description>
The original `e2e_multi_node_repeated_leadership_changes_preserve_single_primary` scenario exposed a write-survival problem that is separate from the scenario's single-primary contract.

Observed failure chain:
- node A starts as primary
- node A fails and node B becomes primary
- a proof row written on node B after the first failover does not reach node C
- node B then fails and node C becomes primary
- the cluster later converges without the intermediate proof row, meaning the later winner did not contain the earlier acknowledged write

This needs source-level investigation and a dedicated fix. Explore the failover sequencing, candidate eligibility, and promotion safety rules first. In particular, verify whether rapid successive promotions can choose a replica that is still behind the current primary's committed writes, and whether the HA model needs an explicit freshness/catch-up requirement before a node is eligible to become the next primary.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
