## Plan: Collapse HA Targeted Switchover Precondition Glue

### Why this is the next verified reduction target

Plan `96-collapse-ha-step-action-glue-onto-world-owner.md` is no longer a valid execution target. Re-verification on 2026-03-25 showed that moving more action couriers onto `HaWorld` increased tracked code instead of reducing it.

The next live deletion target is still in `tests/ha/support/steps/mod.rs`, but it is narrower and stays local to the step owner:

- the three switchover-request handlers still repeat the same precondition shell
- that overlap already lives in one file that owns the polling helpers
- collapsing that overlap does not require adding new `HaWorld` methods

This is a better boundary move than plan 96 because it deletes duplicated step-local orchestration instead of shifting it into a larger owner.

### Current overlap verified in code

Re-verified on 2026-03-25 against the live tree:

1. The two targeted switchover handlers duplicate the same precondition polling scaffold.
   - `tests/ha/support/steps/mod.rs`
     - `i_request_a_targeted_switchover_to(...)`
     - `i_attempt_a_targeted_switchover_to_and_capture_the_operator_visible_error(...)`
   - Both:
     - resolve `member_ref` into `ClusterMember`
     - load `current_primary`
     - load `failover_deadline`
     - call `poll_until(...)`
     - fetch `status` via `observer.state_via_member(seed_member)`
     - record a status snapshot
     - inspect `status.dcs.member(&member_id.member_id())`
     - diverge only on whether eligibility is required or forbidden

2. Those same two handlers duplicate the same request-dispatch shape after polling.
   - Both call `observer.switchover_request_via_member(seed_member, Some(member_id))`.
   - The only real difference is the expected outcome:
     - success path records a success note
     - rejection path records the rendered error and fails if the request unexpectedly succeeds

3. The planned switchover handler still shares the same final request-and-note tail.
   - `i_request_a_planned_switchover(...)` computes `target_member` differently, but once it has a target it performs the same observer request plus note recording shape as the targeted-success path.
   - That means one local request helper can have multiple real callers without growing `HaWorld`.

4. The needed supporting primitives already live in the step file.
   - `poll_until(...)`
   - `member_slot_is_api_switchover_eligible(...)`
   - existing note/snapshot wording
   - No new enum, DTO, or world owner is needed for this slice.

### Execution plan

1. Collapse the two targeted switchover handlers onto one shared local workflow in `tests/ha/support/steps/mod.rs`.
   - Introduce one helper with two real callers that owns:
     - resolving the target member
     - reading `current_primary`
     - loading the failover deadline
     - polling for either eligibility or ineligibility
     - issuing the targeted request
   - Pass the expectation as a plain boolean or similarly minimal local parameter.
   - Do not add a new enum for success-vs-rejection mode.

2. Keep the planned switchover target-selection logic local and single-purpose.
   - Do not extract the ranking logic into a single-caller helper.
   - Only share the final request-and-note dispatch with the targeted-success path if the helper has at least two real callers and reduces total lines.

3. Delete repeated targeted precondition branches by collapsing message construction in place.
   - Keep the current error wording materially equivalent.
   - Build the eligible-vs-ineligible message variants inside the shared workflow instead of duplicating whole `poll_until(...)` closures.

4. Keep ownership inside `tests/ha/support/steps/mod.rs`.
   - Do not add new `HaWorld` methods for this slice.
   - Do not move polling helpers into `HaWorld`.
   - Do not add new structs or mode types just to parameterize this overlap.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_current_lines.sh` and require total lines to fall below the current `34006`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require improvement beyond the current `+10308 -14377 diff: -4069`.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- If the shared targeted workflow needs more parameters or boilerplate than the duplicated handlers it replaces, switch this plan back to `TO BE VERIFIED`.
- Do not add a helper that only has one real caller.
- Do not weaken the rejection assertion that a targeted switchover must fail when the target is still ineligible.
- Do not change the observer polling semantics, only the duplicated orchestration shape.

### Expected yield

- Delete one duplicated targeted-switchover polling shell.
- Delete one duplicated targeted-switchover request-dispatch shell.
- Reduce `tests/ha/support/steps/mod.rs` without increasing `tests/ha/support/world/mod.rs`.
- Improve the line-count gates with a strictly local reduction instead of another owner expansion.

NOW EXECUTE
