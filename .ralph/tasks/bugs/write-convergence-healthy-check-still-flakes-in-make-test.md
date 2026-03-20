## Bug: Write convergence healthy check still flakes in make test after prior race fix <status>completed</status> <passes>true</passes>

<description>
`make test` failed while executing `.ralph/tasks/bugs/config-v2-operator-config-rejects-expected-transport-needed-by-ha-harness.md`, even though the current task only changed config_v2 operator parsing/tests.

The failing test was `tests/ha/support/invariants/write_convergence.rs::one_primary_and_two_replicas_are_determined_healthy`, and the recorded failure was:

`expected selected members to converge on accepted count \`3\` on \`public.write_convergence_invariant\` row \`1\` before 250ms; observed: \`node-a\`=4, \`node-b\`=4, \`node-c\`=4; last write attempt error: write-convergence invariant failed: increment fixture row timed out after 10ms`

Explore the current write-convergence invariant implementation and the prior completed race-fix task first, then determine whether this is a true regression, an uncovered second race, or a still-live flake in the healthy-check shutdown boundary.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly
</acceptance_criteria>

### Boundary diagnosis
- Re-verification on the current branch still invalidates the older shutdown-ordering theory. In [tests/ha/support/invariants/write_convergence.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/invariants/write_convergence.rs), `ensure_healthy()` reads `accepted_count` / `last_error` only after `ensure_running()` returns, and `ensure_running()` already closes, drains, and joins the worker. Any repair that only moves the snapshot later would still read the same stale watermark.
- The current healthy-step caller boundary is also already correct. `cluster becomes healthy` in [tests/ha/support/steps/mod.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/steps/mod.rs) reuses the exact `healthy_members` slice returned by `wait_for_authoritative_single_primary()`, so this task is not the earlier recovering-member overselection bug either.
- The concrete remaining mismatch is inside `perform_authoritative_write()`. The runner records `accepted_count` only when `increment_fixture_row()` returns `Ok(count)`, but the recorded flake says the last error was `increment fixture row timed out after 10ms` while all observed members had already converged to `4`. That combination is consistent with an ambiguous timeout: PostgreSQL committed the increment, but the client-side timeout fired before the `RETURNING` row was observed, leaving `accepted_count=3` and `last_error=timeout` even though the authoritative row is already `4`.
- That makes `WriteGate.accepted_count` the wrong canonical boundary for strong convergence after an ambiguous write failure. The strong health check compares real replicated table state, but the gate currently stores only the last count that was confirmed by the client round-trip. The execution fix should therefore reconcile the authoritative row count after quiescing instead of trusting the pre-timeout client result.
- Focused verification on the current branch passed for `cargo nextest run one_primary_and_two_replicas_are_determined_healthy --test ha`, so the execution turn should add a deterministic regression for the ambiguous-timeout path rather than relying on full-suite flake frequency.

### Execution plan
1. Flatten the boundary between worker writes and strong convergence by introducing one internal authoritative-count reconciliation path instead of separately reading `accepted_count` and `last_error` from `WriteGate`. Reuse the existing runner/gate types; do not add a new wrapper struct just to shuttle those two fields around.
2. After `ensure_running()` quiesces the worker, derive the expected strong-check watermark from the authoritative fixture row itself when the last write result was ambiguous. Keep the monotonic gate count for fast successful writes, but stop treating a timed-out client round-trip as proof that the authoritative row did not advance.
3. Add a narrow regression in [tests/ha/support/invariants/write_convergence.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/ha/support/invariants/write_convergence.rs) that synthesizes the timeout/commit split and proves `ensure_healthy()` no longer fails with `expected 3, observed 4`.
4. Run focused `cargo nextest` coverage for the write-convergence invariant while iterating, then finish with `make check`, `make lint`, `make test`, and `make test-long`.

### Constraints for execution
- Do not add a new wrapper struct just to carry `accepted_count` and `last_error`. Reuse `WriteGate`, a tuple, or one small existing helper boundary.
- Prefer code reduction over more shutdown helpers. If quiesce plus authoritative-count reconciliation can live behind one existing boundary, do that.
- Keep liveness-only cleanup separate from strong convergence semantics, but make the strong check read from the same authoritative source that decides the expected watermark after ambiguous timeouts.
- Do not run `cargo test`; use focused `cargo nextest` only for local coverage, then finish with the required `make` targets.

NOW EXECUTE
