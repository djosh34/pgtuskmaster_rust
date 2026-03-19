## Bug: Write convergence overcounts isolated-primary writes during failover and cleanup <status>not_started</status> <passes>false</passes>

<description>
`make test-long` exposed a second write-convergence failure mode while executing the health-check race refactor.

In `ha_primary_faults_fail_over_then_recover::primary_isolated_from_majority` and `ha_rejoin_and_restart_recovery::rewind_failure_old_primary_rejoins`, the background invariant expected one more committed write than the surviving majority ever converged (`expected 19/21`, observed `18/20` on the surviving nodes and the isolated primary unavailable). In `ha_quorum_loss_and_dcs_loss::lone_survivor_with_only_local_dcs`, the invariant also kept demanding convergence while two members were intentionally gone and not exposing Postgres ports.

Explore the invariant runner and the harness cleanup/background-check path first, then fix the boundary so a locally committed write on a doomed primary is not treated as a cluster-wide durability baseline and unreachable members do not cause false invariant failures in intentionally unhealthy scenarios.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
