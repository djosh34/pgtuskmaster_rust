## Plan: Collapse HA Switchover Step Glue Onto Observer Owners

### Why plan 84 is not the next execution slice

Plan 84 already proved that a pure boundary move is not enough here. Moving the helper logic out of `tests/ha/support/steps/mod.rs` without deleting enough same-area glue pushed `src/` + `tests/` above the current `34182` baseline once the observer-side coverage was added.

The next slice has to stay in the same HA support area, but it must delete more orchestration glue at the same time instead of only relocating semantics.

### Execution result on 2026-03-25

I executed this slice and then compressed it further by:

- deleting the thin switchover waiter wrappers in `tests/ha/support/steps/mod.rs`
- deleting the thin replica waiter wrappers in `tests/ha/support/steps/mod.rs`
- folding the new switchover owner assertions into the existing observer owner test instead of keeping a second test block

That still was not enough. `bash .ralph/git_current_lines.sh` landed at `34234`, which is `52` lines above the pre-slice `34182` baseline.

The missing deletion target is now verified in the current code: the replica waiters are still almost the same function twice, and the three switchover precondition waiters still repeat the same `state_via_member(...)` + `record_status_snapshot(...)` polling scaffold. The code changes from the failed run were reverted, but this broader same-area slice is now specific enough to execute.

### Execution result on 2026-03-25 (second rerun)

I reran the broader owner-side collapse with:

- observer-owned exact/minimum replica-count helpers
- observer-owned planned/targeted switchover interpretation helpers
- one shared step-side switchover polling helper
- one shared step-side replica polling path

This still missed the reduction gate after `cargo fmt`. `bash .ralph/git_current_lines.sh` landed at `34267`, which is `85` lines above the `34182` baseline.

The code changes from this rerun were reverted. The verified gap is that simply moving the switchover semantics into `observer/pgtm.rs` still adds too much owner/test surface relative to the duplicated step glue it deletes. The next plan must pair any owner move with a larger same-area deletion than the current replica-poller + switchover-helper collapse.

### Verified overlap in the current code

1. `tests/ha/support/observer/pgtm.rs` already owns the observation surface.
   - `ClusterStateObservation::compatible_primary(...)`
   - `ClusterStateObservation::replica_members(...)`
   - `PgtmObserver::observe_states(...)`
   - `PgtmObserver::state_via_member(...)`
   - the existing owner test `cluster_state_observation_owns_primary_and_replica_analysis(...)`

2. `tests/ha/support/steps/mod.rs` still owns pure state interpretation that does not belong to cucumber glue.
   - `wait_for_replicas(...)`
   - `wait_for_minimum_replicas(...)`
   - `wait_for_targeted_switchover_rejection_precondition(...)`
   - `wait_for_targeted_switchover_precondition(...)`
   - `wait_for_planned_switchover_precondition(...)`
   - `select_planned_switchover_target(...)`
   - `planned_switchover_target_rank(...)`
   - `member_slot_is_api_switchover_eligible(...)`

3. The repeated polling glue around those helpers is also local duplication.
   - all three switchover precondition waiters do the same `observer.state_via_member(...)` + `record_status_snapshot(...)` + `poll_until(...)` setup
   - replica waiters both rebuild the same authoritative-primary plus ready-replica checks and only differ in the count predicate
   - `wait_for_replicas(...)` and `wait_for_minimum_replicas(...)` are currently two copies of the same observation poller with only the count assertion changed

4. `tests/ha/support/world/mod.rs` already owns member-reference resolution and node actions.
   - `resolve_member_reference(...)`
   - `kill_nodes(...)`
   - `start_nodes(...)`
   - `fully_isolate_node_from_cluster(...)`
   - `isolate_node_from_observer_api_access(...)`
   - `heal_node_network_faults(...)`
   - That means any extra step helper must be tiny and obviously net-negative; do not invent a new world-side abstraction for this slice.

### Execution plan

1. Move the pure switchover interpretation onto the observer owner.
   - In `tests/ha/support/observer/pgtm.rs`, keep the ranking and eligibility logic beside the existing observation owner instead of beside step glue.
   - Rehome:
     - planned switchover target selection from a sampled `NodeState`
     - targeted switchover eligibility checks from a sampled `NodeState`
   - Keep these as methods or small free functions in `observer/pgtm.rs`; do not introduce a new trait, planner type, selector type, or DTO.

2. Make replica-count checks observer-owned and collapse the duplicate replica waiters.
   - Extend `ClusterStateObservation` with count-check helpers that reuse `compatible_primary(...)` and `replica_members(...)`:
     - exact ready replica count
     - minimum ready replica count
   - Replace `wait_for_replicas(...)` and `wait_for_minimum_replicas(...)` with one local replica polling path in `steps/mod.rs`; do not keep two near-identical wrappers once the count-specific error rendering lives in the owner.
   - Keep the failure rendering in the owner so `steps/mod.rs` stops rebuilding the same messages around the same observation data.

3. Delete the repeated switchover polling scaffolding from `tests/ha/support/steps/mod.rs`.
   - Replace the three switchover precondition waiters with one local polling helper that:
     - fetches `state_via_member(seed_member)`
     - records the snapshot
     - delegates only the pure interpretation to observer-owned helpers
   - The step file should retain only scenario phrasing, timeline labels, and the final request side effects.

4. Only collapse direct member-resolution wrappers if the result is clearly smaller after steps 1-3.
   - If a one-line `resolved_member(...)` helper removes enough repeated `world.resolve_member_reference(member_ref.as_str())?` boilerplate, keep it local to `steps/mod.rs`.
   - Do not add callback-heavy wrapper layers around world actions. If the helper does not reduce code immediately, skip it.

5. Reuse existing observer test fixtures instead of adding new parallel coverage.
   - Expand the existing observer owner test coverage in `tests/ha/support/observer/pgtm.rs` so the moved semantics are asserted from the owner side.
   - Reuse the current sample `NodeState` builder helpers already in that file.
   - Do not add a new step-level test module and do not add a second fixture stack just for switchover ranking.

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
- Do not widen visibility outside the HA support tree.
- Do not keep duplicate ranking or eligibility logic in `steps/mod.rs` after the move.
- If the shared switchover polling helper becomes larger than the three call sites it replaces, switch this plan back to `TO BE VERIFIED`.
- If owner-side test reuse is not enough and execution would need a large new fixture block, switch this plan back to `TO BE VERIFIED`.

### Expected yield

- Delete `select_planned_switchover_target(...)`, `planned_switchover_target_rank(...)`, and `member_slot_is_api_switchover_eligible(...)` from `tests/ha/support/steps/mod.rs`.
- Shrink the three switchover waiters down to one polling path plus observer-owned interpretation.
- Delete one of the two duplicate replica wait helpers by routing both exact and minimum-count cases through one polling path plus observer-owned count checks.
- Shrink switchover wait helpers down to one polling path plus observer-owned interpretation instead of three copies of the same snapshot loop.
- Keep coverage in `observer/pgtm.rs` by broadening existing owner tests rather than adding a new parallel test surface.

TO BE VERIFIED
