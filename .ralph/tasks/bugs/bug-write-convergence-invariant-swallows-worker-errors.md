## Bug: Write convergence invariant swallows worker errors <status>not_started</status> <passes>false</passes>

<description>
`tests/ha/support/invariants/write_convergence.rs` drops connection and worker errors instead of surfacing them.
This was detected while inspecting `tests/ha/support` for boundary problems: `run_member_worker` ignores `connect_member` failures with `let _ = err`, and `maintain_connected_member` also discards non-reconnect query failures and connection task results.
Explore and research the codebase first, then fix this so invariant failures are observable instead of silently retried or discarded.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
