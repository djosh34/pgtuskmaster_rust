## Bug: Write convergence healthy check still flakes in make test after prior race fix <status>not_started</status> <passes>false</passes>

<description>
`make test` failed while executing `.ralph/tasks/bugs/config-v2-operator-config-rejects-expected-transport-needed-by-ha-harness.md`, even though the current task only changed config_v2 operator parsing/tests.

The failing test was `tests/ha/support/invariants/write_convergence.rs::one_primary_and_two_replicas_are_determined_healthy`, and the recorded failure was:

`expected selected members to converge on accepted count \`3\` on \`public.write_convergence_invariant\` row \`1\` before 250ms; observed: \`node-a\`=4, \`node-b\`=4, \`node-c\`=4; last write attempt error: write-convergence invariant failed: increment fixture row timed out after 10ms`

Explore the current write-convergence invariant implementation and the prior completed race-fix task first, then determine whether this is a true regression, an uncovered second race, or a still-live flake in the healthy-check shutdown boundary.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
