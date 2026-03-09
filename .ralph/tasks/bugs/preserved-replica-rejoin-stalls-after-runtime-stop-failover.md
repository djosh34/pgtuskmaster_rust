## Bug: Preserved Replica Rejoin Stalls After Runtime Stop Failover <status>not_started</status> <passes>false</passes>

<description>
The degraded replica failover scenario exposed a separate recovery bug after the harness stop path was corrected to explicitly stop postgres.

When a replica runtime is stopped, postgres is stopped, the primary fails over to the healthy sibling, and the degraded replica later restarts with its existing data directory preserved, the restarted replica becomes queryable again but can stall without replicating newly inserted rows from the promoted primary. The failing evidence is the long HA scenario `e2e_multi_node_degraded_replica_failover_promotes_only_healthy_target`, which times out waiting for the post-failover proof row on the restarted degraded node unless the test wipes the replica data directory and forces a fresh clone before restart.

Explore and research the restart/rejoin path first, then fix the product behavior rather than papering over it in the test harness. In particular, inspect startup-mode selection, managed recovery config regeneration for preserved replica data dirs, and HA/process interactions after a runtime-stop plus later restart against a new primary.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
