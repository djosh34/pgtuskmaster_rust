## Bug: Write convergence invariant swallows connect errors <status>not_started</status> <passes>false</passes>

<description>
`tests/ha/support/invariants/write_convergence.rs` currently drops `connect_member` failures in `run_member_worker` with `let _ = err;` and then just sleeps and retries.

That hides real harness failures, makes debugging reconnect issues much harder, and violates the repo rule against swallowing errors.

Explore the surrounding harness and invariant code first, then fix the boundary so the failure stays visible and typed instead of being discarded.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
