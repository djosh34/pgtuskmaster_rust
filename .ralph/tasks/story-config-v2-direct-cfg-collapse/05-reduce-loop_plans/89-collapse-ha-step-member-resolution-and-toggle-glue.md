## Plan: Collapse HA Step Member Resolution And Toggle Glue

### Why this is the next verified reduction target

Plan 86 already proved that deleting only the poller helper stack in `tests/ha/support/steps/mod.rs` is too small by itself. The remaining net-negative slice is still in the same owner area, but it has to remove the other repeated step glue at the same time instead of stopping after the pollers.

The current code still keeps three kinds of duplication in the HA cucumber layer:

- `tests/ha/support/steps/mod.rs`
  - repeats member-reference resolution before world-owned actions
  - repeats DCS start/stop toggle scaffolding for both the full service set and the quorum-majority subset
  - keeps boolean courier helpers for local DCS toggles and blocker toggles
- `tests/ha/support/world/mod.rs`
  - already owns member resolution and node-action state mutation
- `tests/ha/support/world/mod.rs`
  - already owns DCS service start/stop and blocker application

That is the right next reduction target because the owners already exist. The step file should be mostly scenario phrasing plus the few polling workflows that genuinely belong there.

### Verified overlap in the current code

1. Member-reference resolution is still repeated across many step handlers.
   - `tests/ha/support/steps/mod.rs`
     - `i_kill_the_node_named(...)`
     - `i_kill_the_nodes_named(...)`
     - `i_restart_the_node_named(...)`
     - `i_start_only_the_fixed_nodes(...)`
     - `i_request_a_targeted_switchover_to(...)`
     - `i_attempt_a_targeted_switchover_to_and_capture_the_operator_visible_error(...)`
     - `quoted_member_becomes_primary(...)`
     - `i_fully_isolate_the_node_named_from_the_cluster(...)`
     - `i_cut_the_node_named_off_from_dcs(...)`
     - `i_isolate_the_node_named_from_observer_api_access(...)`
     - `i_heal_network_faults_on_the_node_named(...)`
     - `i_wipe_the_data_directory_on_the_node_named(...)`
     - `set_local_dcs_service_for_node_named(...)`
     - `set_blocker_on_node_named(...)`
   - All of those repeat `world.resolve_member_reference(member_ref.as_str())?` and then immediately delegate to an existing world or harness owner.

2. The DCS toggle steps are still a four-way matrix with the same harness access pattern.
   - `tests/ha/support/steps/mod.rs`
     - `i_stop_the_dcs_service(...)`
     - `i_stop_a_dcs_quorum_majority(...)`
     - `i_start_the_dcs_service(...)`
     - `i_restore_dcs_quorum(...)`
   - The only moving parts are:
     - full DCS service set versus quorum-majority subset
     - start versus stop
   - The harness already owns `start_dcs_services(...)` and `stop_dcs_services(...)`.

3. The boolean courier helpers are still duplicated step glue.
   - `tests/ha/support/steps/mod.rs`
     - `set_local_dcs_service_for_node_named(...)` differs only by `started`
     - `set_blocker_on_node_named(...)` differs only by `enabled`
   - Each helper resolves the member reference, then immediately calls an existing owner:
     - `HarnessShared::start_dcs_services(...)` / `HarnessShared::stop_dcs_services(...)`
     - `HarnessShared::set_blocker(...)`

4. The poller cleanup from plan 86 is still available in the same file.
   - `tests/ha/support/steps/mod.rs`
     - `wait_for_replicas(...)` is still the reusable poller
     - the three switchover request steps still duplicate the same:
       - `observer.state_via_member(seed_member)?`
       - `record_status_snapshot(...)`
       - `poll_until(...)`
       - custom timeout message shape
   - Pairing that existing deletion with the member/toggle glue gives a broader same-area slice than plan 86 attempted.

### Execution plan

1. Finish the direct helper deletion from plan 86 inside `tests/ha/support/steps/mod.rs`.
   - Keep the observer and world public surfaces unchanged for this slice.
   - Collapse the switchover precondition stack directly in the step file:
     - inline the planned switchover target-selection helper stack into `i_request_a_planned_switchover(...)`
     - inline the targeted eligibility and targeted rejection polling helpers into their two callers
   - Delete the now-single-use helper functions after the inline.
   - Keep `poll_until(...)` and `poll_for_cluster_observation(...)` as the shared polling owners unless one of them naturally disappears during the rewrite.

2. Collapse member-reference boilerplate with one tiny local resolution path.
   - Add at most one small helper in `tests/ha/support/steps/mod.rs` for:
     - resolving one member reference
     - resolving a short fixed list of member references
   - Rebuild the action-only step handlers on top of that helper.
   - Do not add a generic callback layer or a new DTO; the helper should only delete the repeated `resolve_member_reference(...)` + collect boilerplate that is already present today.

3. Delete the boolean courier helper layer.
   - Inline or collapse `set_local_dcs_service_for_node_named(...)` so the two step handlers become direct calls through one tiny shared action path.
   - Inline or collapse `set_blocker_on_node_named(...)` the same way.
   - If a small local helper remains, it must be visibly smaller than the two deleted wrappers and must not introduce callback plumbing.

4. Collapse the DCS service toggle matrix onto one local action path.
   - Replace the four full-cluster/quorum-majority DCS step bodies with one shared helper that accepts:
     - which service set to use
     - whether to start or stop
   - Reuse `HarnessShared::start_dcs_services(...)` and `HarnessShared::stop_dcs_services(...)` as the actual owners.
   - Do not create a new enum just to encode the two service subsets if a direct helper with a slice argument stays smaller.

5. Keep the slice local and net-negative.
   - Stay inside:
     - `tests/ha/support/steps/mod.rs`
     - optionally `tests/ha/support/world/mod.rs` only if one tiny owner-side helper deletes more step glue than it adds
   - Do not move semantics into `observer/pgtm.rs` for this slice.
   - Do not add new tests unless a moved or rewritten helper loses existing coverage; prefer reusing the current HA support tests exactly as they are.

6. Validate the slice.
   - `cargo fmt`
   - `bash .ralph/git_current_lines.sh` and verify total drops below `34107`
   - `bash .ralph/git_diff_lines_since.sh` and verify the diff improves beyond `+10238 -14206 diff: -3968`
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`

### Guardrails

- No new enums, structs, selector objects, or trait layers for this slice.
- Keep the effective step behavior and failure messages equivalent unless the smaller shape clearly improves them.
- Do not widen visibility outside `tests/ha/support`.
- If the local resolution helper or DCS-toggle helper grows more abstract than the duplication it replaces, switch this plan back to `TO BE VERIFIED`.
- If the poller inline reveals that owner-side reuse in `observer/pgtm.rs` is required after all, switch this plan back to `TO BE VERIFIED` instead of reviving the failed plan-84/85 shape mid-execution.

### Expected yield

- Delete the remaining single-use switchover helper stack that plan 86 identified.
- Delete the boolean courier helpers for local DCS service toggles and blocker toggles.
- Delete the repeated member-resolution boilerplate from the action-only step handlers.
- Shrink the four DCS service toggle steps down to one shared action path plus step phrases.
- Keep the slice in one HA support area so it can go net-negative without adding new observer-owner code.

NOW EXECUTE
