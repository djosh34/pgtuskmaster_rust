## Bug: Healthy write convergence over-selects recovering members <status>not_started</status> <passes>false</passes>

<description>
`make test-long` exposed two remaining failures after moving strong write-convergence checks out of cleanup and into `cluster becomes healthy`:

- `ha_rejoin_and_restart_recovery::blocked_basebackup_recovery_recovers_after_unblock`
- `ha_primary_faults_fail_over_then_recover::primary_isolated_from_majority`

In both scenarios, the health step finds a healthy authoritative primary, but the follow-up accepted-write convergence check still selects a member whose Postgres probe is not yet reconnectable (`node-a` / `node-b` connection errors) even though the surviving healthy members already agree on the fixture row count.

Explore the health-step membership boundary first. The strong accepted-write convergence set should be derived from the same successful healthy observation/probe boundary, not from `HaWorld::online_member_ids()` intent state alone.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
