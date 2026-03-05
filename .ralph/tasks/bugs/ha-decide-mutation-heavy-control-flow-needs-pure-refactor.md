---
## Bug: HA decide mutation-heavy control flow needs pure refactor <status>not_started</status> <passes>false</passes>

<description>
`src/ha/decide.rs` currently implements deterministic HA decisions through shared mutable state (`next`, `candidates`, restore-status mutation, mutable phase variables) even though the logic should be expressible as pure functions returning complete outcomes. This was detected from PR #1 owner feedback and confirmed by reading the current code.

Explore and research the codebase first, then fix. Focus on the mutation-heavy HA decision flow, the restore guard helpers that mutate shared structures through `&mut`, and the resulting readability/correctness risks in related HA files.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
