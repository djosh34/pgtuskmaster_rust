## Bug: HA rewind failure rejoin scenario flakes between no-primary recovery and write convergence cleanup <status>not_started</status> <passes>false</passes>

<description>
`make test-long` currently fails in `pgtuskmaster_rust::ha::ha_rejoin_and_restart_recovery::rewind_failure_old_primary_rejoins`.
Observed failure modes:
`Then cluster becomes healthy` can time out with both visible members reporting `no_primary(leaseopen)`,
and the scenario can also pass its steps but still fail cleanup because the background write-convergence invariant never reconnects one member within the 15s deadline.
Explore and research the codebase first, then fix the HA/runtime behavior or the harness assumptions so this ultra-long scenario passes cleanly and deterministically.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
