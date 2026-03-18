## Bug: Write convergence discards member worker errors <status>not_started</status> <passes>false</passes>

<description>
`tests/ha/support/invariants/write_convergence.rs` currently drops connection and task results with `let _ = ...` in the member worker loop.
This hides reconnect failures and completed task errors instead of surfacing them through the invariant health checks.
Explore and research the codebase first, then fix the swallowed error handling so failures are recorded and reported.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
