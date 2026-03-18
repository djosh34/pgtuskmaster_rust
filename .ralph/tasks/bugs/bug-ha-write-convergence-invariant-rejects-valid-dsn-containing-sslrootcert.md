## Bug: HA write-convergence invariant rejects a valid DSN containing sslrootcert <status>not_started</status> <passes>false</passes>

<description>
The HA ultra-long suite now reaches scenario startup far enough to initialize the write-convergence invariant, but that invariant fails with a false-negative DSN validation error:

`write-convergence invariant failed: dsn did not contain 'sslrootcert'`

The captured DSN in the same error message does contain `sslrootcert=...`, so the invariant or its DSN parsing/checking is incorrect.

This was observed while running:
- `cargo nextest run --profile ultra-long -E 'binary(ha) & test(=ha_operator_switchovers::planned_switchover_keeps_cluster_healthy)'`

Explore and research the HA invariant and observer path first, then fix the validation so a valid DSN with `sslrootcert` is accepted.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
