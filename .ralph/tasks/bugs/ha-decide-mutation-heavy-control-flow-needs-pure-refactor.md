---
## Bug: HA decide mutation-heavy control flow needs pure refactor <status>blocked</status> <passes>false</passes>

<blocked_by>06-task-move-and-split-ha-e2e-tests-after-functional-rewrite</blocked_by>

<description>
This bug is intentionally deferred until the HA functional rewrite story is fully complete. It overlaps directly with the planned refactor work, and it does not make sense to force the bug queue to preempt the story that is supposed to absorb most or all of this concern.

Reassess this bug only after `story-ha-functional-rewrite` reaches its final task. At that point, answer a narrower question: how much mutation-heavy control flow is still present in the rewritten design, and what residual bug or cleanup work remains?

Current concern recorded here: `src/ha/decide.rs` implements deterministic HA decisions through shared mutable state (`next`, `candidates`, restore-status mutation, mutable phase variables) even though the logic should be expressible as pure functions returning complete outcomes. This was detected from PR #1 owner feedback and confirmed by reading the current code before the rewrite story was planned.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
