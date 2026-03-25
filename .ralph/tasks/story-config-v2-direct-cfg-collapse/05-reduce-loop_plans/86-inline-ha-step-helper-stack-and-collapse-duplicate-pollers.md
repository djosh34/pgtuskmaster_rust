## Plan: Inline HA Step Helper Stack And Collapse Duplicate Pollers

### Why plans 84 and 85 should not be rerun

The last two slices already verified that moving switchover semantics into `tests/ha/support/observer/pgtm.rs` is the wrong immediate reduction target. The step file does contain misplaced logic, but rehoming that logic added more owner-side code and tests than it deleted from `tests/ha/support/steps/mod.rs`.

The next slice should stay inside the same HA support area, but it should delete helper layers directly instead of introducing more observer-owner surface.

### Verified overlap in the current code

1. The replica waiters are duplicate pollers with only the count predicate changed.
   - `wait_for_replicas(...)` and `wait_for_minimum_replicas(...)` both:
     - capture `world.online_member_ids()`
     - call `poll_for_cluster_observation(...)`
     - resolve the compatible primary
     - collect ready replicas
     - differ only in `== expected` versus `>= minimum`

2. The switchover precondition waiters are single-caller helpers that fragment one workflow.
   - `wait_for_planned_switchover_precondition(...)` is called only by `i_request_a_planned_switchover(...)`
   - `wait_for_targeted_switchover_precondition(...)` is called only by `i_request_a_targeted_switchover_to(...)`
   - `wait_for_targeted_switchover_rejection_precondition(...)` is called only by `i_attempt_a_targeted_switchover_to_and_capture_the_operator_visible_error(...)`
   - each helper repeats the same `observer.state_via_member(...)` plus `record_status_snapshot(...)` plus `poll_until(...)` scaffold

3. The planned switchover selector stack is also single-use glue.
   - `select_planned_switchover_target(...)` is called only by `wait_for_planned_switchover_precondition(...)`
   - `planned_switchover_target_rank(...)` is called only by `select_planned_switchover_target(...)`
   - `member_slot_is_api_switchover_eligible(...)` is only used inside that switchover helper cluster

4. `tests/ha/support/steps/mod.rs` also still has a few paired wrappers with obvious overlap after the main helper removal.
   - blocker enable/disable steps differ only by a boolean
   - local DCS start/stop steps differ only by the service action
   - DCS full-cluster start/stop and quorum-majority start/stop pairs have the same harness access pattern
   - only collapse these pairs if the result is plainly smaller without introducing callback-heavy glue

### Execution plan

1. Keep the observer and world public surfaces unchanged for this slice.
   - Do not move switchover ranking, eligibility, or replica-count logic into `observer/pgtm.rs`.
   - Do not add new observer-owner tests for this slice.
   - The verified problem now is helper layering in the step file, not missing owner APIs.

2. Replace the two replica waiters with one shared polling path in `tests/ha/support/steps/mod.rs`.
   - Keep using the existing `poll_for_cluster_observation(...)`.
   - Collapse `wait_for_replicas(...)` and `wait_for_minimum_replicas(...)` into one helper that:
     - resolves the compatible primary once per poll
     - collects ready replicas once per poll
     - lets the caller supply the count assertion and message rendering
   - Do not introduce a new enum or DTO just to encode exact versus minimum counts.

3. Inline the switchover precondition waiters into their only step callers.
   - Move the polling closure from `wait_for_planned_switchover_precondition(...)` into `i_request_a_planned_switchover(...)`
   - Move the polling closure from `wait_for_targeted_switchover_precondition(...)` into `i_request_a_targeted_switchover_to(...)`
   - Move the polling closure from `wait_for_targeted_switchover_rejection_precondition(...)` into `i_attempt_a_targeted_switchover_to_and_capture_the_operator_visible_error(...)`
   - After the inline, delete all three helper functions from `tests/ha/support/steps/mod.rs`

4. Inline the planned switchover selector stack into the planned request step unless a remaining helper is reused at least twice.
   - Delete `select_planned_switchover_target(...)`
   - Delete `planned_switchover_target_rank(...)`
   - Inline the ranking and selection directly into the planned switchover poll closure
   - Keep `member_slot_is_api_switchover_eligible(...)` only if it still serves more than one call site after the inline; otherwise inline it too

5. Only after steps 2 through 4, collapse the obvious paired wrappers if they remain clearly net-negative.
   - Candidate pairs:
     - blocker enable/disable
     - local DCS start/stop
     - cluster DCS start/stop and quorum-majority start/stop
   - Prefer direct local helper extraction only when at least two call sites share the exact same shape.
   - If the helper needs callback plumbing or hides the actual step behavior, skip it.

6. Validate the slice.
   - `cargo fmt`
   - `bash .ralph/git_current_lines.sh` and verify total drops below `34182`
   - `bash .ralph/git_diff_lines_since.sh` and verify the diff improves from the current baseline
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`

### Guardrails

- No new enums, structs, selector objects, or trait layers for this slice.
- Do not widen visibility outside `tests/ha/support`.
- Do not move logic into `observer/pgtm.rs` unless execution proves the step-file inline is impossible; if that happens, switch this plan back to `TO BE VERIFIED`.
- If the shared replica polling helper or any paired-wrapper helper becomes more abstract than the duplicated code it replaces, inline instead of adding another wrapper.
- If line count is not lower after formatting, revert the code changes, switch this plan back to `TO BE VERIFIED`, and stop.

### Expected yield

- Delete the three switchover precondition helper functions.
- Delete the planned switchover selector helper stack.
- Delete one of the two duplicate replica waiters and keep only one shared replica polling path.
- Potentially delete a small second layer of paired wrapper glue after the main inline work if the result is still plainly smaller.
- Achieve the reduction without adding more observer-owner code or owner-side test fixtures.

NOW EXECUTE
