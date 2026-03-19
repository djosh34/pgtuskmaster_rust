## Bug: Write convergence healthy invariant can stall with zero shared writes <status>not_started</status> <passes>false</passes>

<description>
`make test` failed in `tests/ha/support/invariants/write_convergence.rs` for `one_primary_and_two_replicas_are_determined_healthy`.
The test timed out waiting for the shared counter to reach `3`, but observed `0`, which means the writer loop never recorded any successful shared writes during the healthy primary-plus-replicas scenario.

Explore and research the codebase first, then fix the invariant or the underlying writer/probe coordination so the healthy-path test stops stalling.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
