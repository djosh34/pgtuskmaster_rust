## Bug: Primary-count invariant swallows observer failures <status>not_started</status> <passes>false</passes>

<description>
`tests/ha/support/invariants/primary_count.rs` currently converts `observer.state_via_member(member)` failures into `Ok((member, false))` inside `observe_member_primary`.
That means the background invariant can silently treat broken observation as "not primary" instead of surfacing a real failure, which hides harness/observer regressions and can report an incorrect zero-primary state.
Explore and research the codebase first, then fix the swallowed error handling so member-observation failures remain observable and the invariant reports them explicitly.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
