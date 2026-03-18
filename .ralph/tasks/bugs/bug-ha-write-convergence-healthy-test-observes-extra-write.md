## Bug: HA write convergence healthy test observes extra write <status>not_started</status> <passes>false</passes>

<description>
`cargo nextest run --test ha --profile default --no-tests fail` failed in `support::invariants::write_convergence::tests::one_primary_and_two_replicas_are_determined_healthy`.
The failure observed every member at count `4` on `public.write_convergence_invariant` row `1` even though the test expected all members to converge to `3` before the 250ms deadline.
Explore and research the write-convergence fixture and worker behavior first, then fix the extra-write mismatch or the incorrect expectation.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
