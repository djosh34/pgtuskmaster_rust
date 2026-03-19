## Bug: Write convergence health check can observe one extra committed write after stopping workers <status>not_started</status> <passes>false</passes>

<description>
`make test` exposed a failure in `tests/ha/support/invariants/write_convergence.rs::one_primary_and_two_replicas_are_determined_healthy` where `ensure_healthy()` expected all members to converge to count `3` but observed `4` on every member instead.

This was detected while running the full suite during the HA boundary collapse task. Explore the invariant runner and the surrounding HA/test timing first, then fix the race or behavioral leak so the health check samples a stable committed count.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
