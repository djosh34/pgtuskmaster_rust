## Plan: Collapse HA Step Observer State Machines Onto Observer Owner

### Why this is the next verified reduction target

The HA cucumber support still keeps cluster-state interpretation inside the step layer even though the observer layer already owns the raw state acquisition and routing surface.

- `tests/ha/support/observer/pgtm.rs`
  - `ClusterStateObservation::compatible_primary(...)`
  - `ClusterStateObservation::replica_members(...)`
  - `PgtmObserver::observe_states(...)`
  - `PgtmObserver::state_via_member(...)`
  - `PgtmObserver::postgres_routing_target(...)`
  - `PgtmObserver::switchover_request_via_member(...)`
- `tests/ha/support/steps/mod.rs`
  - `wait_for_replicas(...)`
  - `wait_for_minimum_replicas(...)`
  - `wait_for_authoritative_single_primary(...)`
  - `wait_for_no_operator_primary(...)`
  - `wait_for_targeted_switchover_rejection_precondition(...)`
  - `wait_for_targeted_switchover_precondition(...)`
  - `wait_for_planned_switchover_precondition(...)`
  - `select_planned_switchover_target(...)`
  - `planned_switchover_target_rank(...)`
  - `member_slot_is_api_switchover_eligible(...)`

That is a boundary problem: the step layer should orchestrate scenarios and timeline labels, not own DCS member eligibility rules, replica selection rules, or switchover-target ranking semantics.

### Current overlap verified in code

1. The observer module already owns cluster-observation semantics.
   - `tests/ha/support/observer/pgtm.rs`
     - `compatible_primary(...)` decides whether a sampled cluster view is authoritative.
     - `replica_members(...)` decides which members count as ready replicas.
     - `state_via_member(...)` and `observe_states(...)` are the only status acquisition entry points.

2. The step layer rebuilds more observer semantics on top of those owners.
   - `tests/ha/support/steps/mod.rs`
     - `wait_for_replicas(...)` and `wait_for_minimum_replicas(...)` recompose `compatible_primary(...)` plus `replica_members(...)` into more cluster-selection rules.
     - `wait_for_authoritative_single_primary(...)` repeats the authoritative-primary path, then performs writable-primary probing and healthy-member collection locally.
     - `wait_for_no_operator_primary(...)` re-derives the "no visible writable primary" rule from observer results.

3. Switchover eligibility lives in the wrong module.
   - `tests/ha/support/steps/mod.rs`
     - `member_slot_is_api_switchover_eligible(...)`, `planned_switchover_target_rank(...)`, and `select_planned_switchover_target(...)` are pure `NodeState` / `DcsMemberState` interpretation helpers, but they live beside cucumber step glue instead of beside the observer owner.
     - `wait_for_targeted_switchover_*` and `wait_for_planned_switchover_precondition(...)` use those helpers only after calling `observer.state_via_member(...)`.

4. The step layer also shows repeated glue around this misplaced logic.
   - `tests/ha/support/steps/mod.rs`
     - `world.resolve_member_reference(member_ref.as_str())?` appears fourteen times.
     - Several step handlers differ only by which observer/world action they invoke after the same resolution boilerplate.
   - This repetition is not the primary slice, but it becomes easier to flatten once the state-machine logic moves out of the step file.

### Revised execution plan

1. Move pure observer semantics to `tests/ha/support/observer/pgtm.rs`.
   - Extend `ClusterStateObservation` with the query methods that steps currently rebuild from full-cluster observations:
     - exact replica count checks
     - minimum replica count checks
   - Move the single-node switchover interpretation helpers beside that owner in the same observer module:
     - planned switchover target selection from a sampled `NodeState`
     - targeted switchover eligibility and ineligibility checks from a sampled `NodeState`
   - Keep these as methods or small pure functions on existing owners in the observer module instead of introducing new wrapper structs, enums, or query objects.

2. Keep `steps/mod.rs` as orchestration only.
   - Rewrite the wait helpers so they call observer-owned query methods instead of carrying local DCS ranking logic.
   - Delete:
     - `select_planned_switchover_target(...)`
     - `planned_switchover_target_rank(...)`
     - `member_slot_is_api_switchover_eligible(...)`
   - Inline or collapse any wait helper branches that become one-liners after the semantic move.

3. Reuse observer tests instead of adding parallel step tests.
   - Extend `tests/ha/support/observer/pgtm.rs` unit coverage to own:
     - planned target ranking
     - targeted switchover eligibility/ineligibility
     - replica-count queries
   - Remove any step-level tests that only cover these pure interpretation rules if they become redundant.

4. Only then flatten repeated step glue if it is still net-negative.
   - If the semantic move leaves obvious repeated member-resolution/action wrappers in `steps/mod.rs`, collapse them only when the helper is smaller than the removed duplication.
   - Do not add generic callback-heavy abstractions that make the step file harder to read.
   - Do not widen `pgtuskmaster_rust` visibility just to reuse crate-internal WAL helpers; if WAL ranking stays test-local, keep it local to `observer/pgtm.rs`.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_current_lines.sh` and confirm total lines drop below the current `34182`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+10077 -13970 diff: -3893`.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Do not introduce a new "query object", "selector", "planner", or parallel observer DTO just to move logic around.
- Keep timeline recording and cucumber step phrases in `tests/ha/support/steps/mod.rs`; only move pure cluster-state interpretation.
- Do not widen visibility or add public APIs outside the HA support tree for this slice.
- If the observer methods need `HarnessShared` or step-layer context to stay readable, switch this plan back to `TO BE VERIFIED` instead of forcing a cross-layer dependency.

### Expected yield

- Delete the pure DCS/switchover ranking helpers from the step file.
- Shrink multiple wait helpers down to orchestration around existing observer owners.
- Concentrate cluster-state interpretation tests into `observer/pgtm.rs`, which should remove duplicated support logic and keep the slice net-negative.

### Execution result on 2026-03-25

Attempted execution verified that the semantic boundary move is valid, but the slice failed the reduction loop:

- focused `cargo nextest` coverage for the moved HA observer semantics passed
- the attempted observer-test additions plus helper moves pushed `src/` + `tests/` back above the pre-slice total instead of below `34182`
- reverting the code changes restored the baseline total, so this needs a broader reduction plan before another execution pass

Next replanning pass should either:

- pair this observer-owner move with a larger same-area deletion that clearly exceeds the added observer coverage, or
- choose a different reduction target with a better guaranteed line delta

TO BE VERIFIED
