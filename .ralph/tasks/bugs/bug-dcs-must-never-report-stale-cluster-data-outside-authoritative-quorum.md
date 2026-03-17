## Bug: DCS must never report stale cluster data outside authoritative quorum <status>not_started</status> <passes>false</passes>

<description>
The intended product invariant is stricter than the behavior that was reintroduced during `.ralph/tasks/story-general-architecture-improvement-finding/07-task-collapse-duplicate-struct-trees-into-canonical-domain-adts-and-prove-the-struct-count-went-down.md`: DCS has only two meaningful states, and stale or reused cluster data is never allowed outside authoritative quorum.

The required model is:
- `Quorum(data in dcs)` when there is authoritative quorum majority
- `NoQuorum` when there is not

There is no third middle state, no degraded-but-still-visible member set, no observed-members exception, and no retention of old cluster pictures when DCS authority is lost. If DCS is not authoritative, the system must go directly into fail-safe and expose only the minimal local/disconnected state. It must not keep, reuse, or republish stale member visibility for any reason.

This bug was detected from the postmortem on the referenced task: tests were over-asserting unimportant behavior and encoded a different product decision than the intended safety invariant. That caused the redesign to preserve information that should have been dropped, which reintroduced a three-state model. Under the intended semantics, any test that expects visible members outside authoritative quorum is wrong and must be rewritten rather than driving product behavior.

Explore and research the codebase first, then fix the DCS/HA state model and the tests together so the product enforces the strict two-state invariant. Do not preserve or mask stale authority through disconnect-path retention, cached observed-members views, or any other form of reused cluster state.
</description>

<acceptance_criteria>
- [ ] DCS and HA state handling expose only two authority states in this area: authoritative quorum with cluster data, or non-authoritative fail-safe with no reused cluster data
- [ ] No code path keeps, republishes, or derives visible member state from stale DCS data once authoritative quorum is lost
- [ ] Tests that currently require member visibility or reused cluster pictures outside authoritative quorum are rewritten to assert the stricter fail-safe behavior instead
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this change impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
