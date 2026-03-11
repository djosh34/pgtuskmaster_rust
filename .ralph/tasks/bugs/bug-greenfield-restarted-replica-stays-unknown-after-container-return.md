## Bug: Greenfield restarted replica stays unknown after container return <status>not_started</status> <passes>false</passes>
<blocked_by>.ralph/tasks/story-greenfield-cucumber-ha-harness/*</blocked_by>


<description>
`ha_replica_outage_keeps_primary_stable` on the greenfield Docker HA harness reaches a trustworthy product-side failure after the intended outage action, not a harness startup/orchestration failure.

Observed on March 10, 2026 from:
- feature wrapper: `cargo nextest run --workspace --profile ultra-long --no-fail-fast --no-tests fail --target-dir /tmp/pgtuskmaster_rust-target --config 'build.incremental=false' --test ha_replica_outage_keeps_primary_stable`
- run directory: `cucumber_tests/ha/runs/replica_outage_keeps_primary_stable/replica-outage-keeps-primary-stable-1773164470528-2697089/`

The run successfully:
- bootstrapped `three_node_plain`
- reached one stable primary
- selected a replica to stop
- created the proof table
- wrote `1:before-replica-outage`
- verified all three nodes contained the row
- killed the chosen replica
- verified the original primary stayed primary
- wrote `2:during-replica-outage`
- restarted the chosen replica container

The failure happened only on the intended HA assertion:
- step failure: `Then the node named "stopped_replica" rejoins as a replica`
- observed error after the full recovery deadline: `member 'node-a' role is 'unknown' instead of 'replica'`

Live container logs and preserved artifacts point at a product rejoin problem:
- `node-a` repeatedly tries `pg_ctl start`
- PostgreSQL reports `lock file "/var/lib/pgtuskmaster/socket/.s.PGSQL.5432.lock" already exists`
- the node never converges back to a sampled replica role before the deadline

Preserved evidence:
- `artifacts/timeline.json`
- `artifacts/compose-logs.txt`
- `artifacts/compose-ps.json`
- `artifacts/inspect-node-a.json`
- `artifacts/observer-debug-verbose.json`

Explore and research the codebase first, then fix the restarted-replica rejoin path so a killed replica can return as a sampled replica on the greenfield Docker harness without getting stuck in `role=unknown` and repeated PostgreSQL start attempts.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
