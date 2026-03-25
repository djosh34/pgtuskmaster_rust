## Plan: Collapse Repeated HA Polling Loop Setup

### Why this reduction target

The next wrong-place overlap is the repeated HA polling loop setup in `tests/ha/support/steps/mod.rs`:

- `wait_for_authoritative_single_primary`, `wait_for_no_operator_primary`, `wait_for_targeted_switchover_rejection_precondition`, `wait_for_planned_switchover_precondition`, and `poll_for_cluster_observation` all open-code the same polling window setup and retry structure.
- Each function separately fetches `world.harness()?` to derive a deadline and poll interval, creates `last_error`, loops until the deadline, sleeps between attempts, and formats nearly identical timeout failures.
- This is step-module orchestration duplication, not distinct domain logic.

### Current overlap already verified

- All five functions live in `tests/ha/support/steps/mod.rs`, so the reduction can stay local to one module.
- The duplicated shape is concrete: deadline calculation, poll interval lookup, `last_error` tracking, retry loop, and final timeout error formatting.
- The real per-call behavior is only the attempt body and, for the observation-based waits, the terminal-container-failure check.

### Execution plan

1. Extract one shared polling driver in `tests/ha/support/steps/mod.rs`.
   - Own the repeated deadline, poll interval, sleep, and `last_error` bookkeeping in one helper.
   - Keep the helper local to the steps module; do not move it into `HaWorld` or `HarnessShared`.

2. Rebuild the repeated wait functions on top of that driver.
   - Convert `wait_for_authoritative_single_primary`, `wait_for_no_operator_primary`, `wait_for_targeted_switchover_rejection_precondition`, `wait_for_planned_switchover_precondition`, and `poll_for_cluster_observation` to supply only their attempt-specific logic.
   - Preserve the existing terminal-container-failure checks only where they already exist today.

3. Keep the helper narrow.
   - Do not introduce a large enum for every polling mode.
   - Do not move observer probing, primary selection, or switchover-status interpretation out of the existing call sites unless that directly reduces code.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+4811 -7205 diff: -2394` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Update the reduce-loop task state only after all gates pass.

### Guardrails

- Keep polling mechanics local to the steps module.
- Keep `HaWorld` responsible for scenario state and `HarnessShared` responsible for runtime mechanics.
- If this starts requiring a giant retry abstraction or pushes unrelated status interpretation into a generic helper, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
