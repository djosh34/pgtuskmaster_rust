## Plan: Collapse HA Step Action Glue Onto `HaWorld`

### Why this is the next verified reduction target

Plan `95-collapse-duplicated-toml-literal-test-helpers.md` is already stale and back in `TO BE VERIFIED`: the shared TOML literal helper owner already exists, so continuing to aim at that overlap would waste the next execution pass.

The next live boundary problem is in the HA cucumber action steps:

- `tests/ha/support/steps/mod.rs` still owns a large pile of action-only step handlers that do little more than:
  - resolve a member alias into `ClusterMember`
  - fetch the harness/given again
  - call one existing world or harness owner
- `tests/ha/support/world/mod.rs` already owns the mutable scenario state and the concrete node/network side effects:
  - node start/stop state
  - network isolation/healing state
  - harness access
  - given-aware DCS routing

That is the wrong boundary for the current tree. The step file should mostly describe scenario phrasing and polling workflows. Stateful side-effect orchestration already belongs to `HaWorld`.

### Current overlap verified in code

Re-verified on 2026-03-25 against the live tree before re-arming execution:

- `HaWorld` already owns `resolve_member_reference(...)`, `kill_nodes(...)`, `start_nodes(...)`, `fully_isolate_node_from_cluster(...)`, `isolate_node_from_observer_api_access(...)`, and `heal_node_network_faults(...)`.
- `HarnessShared` already owns `stop_dcs_services(...)`, `start_dcs_services(...)`, `set_blocker(...)`, and `wipe_member_data_dir(...)`.
- `steps/mod.rs` still keeps private couriers `resolve_member(...)` and `set_dcs_services(...)` even though both only bounce into those existing owners.
- That means the plan is still a real code-reduction move rather than stale churn: the step file is still the wrong owner for this action glue.

1. Single-member action steps still repeat the same resolution glue.
   - `tests/ha/support/steps/mod.rs`
     - `i_kill_the_node_named(...)`
     - `i_restart_the_node_named(...)`
     - `i_fully_isolate_the_node_named_from_the_cluster(...)`
     - `i_cut_the_node_named_off_from_dcs(...)`
     - `i_isolate_the_node_named_from_observer_api_access(...)`
     - `i_heal_network_faults_on_the_node_named(...)`
     - `i_wipe_the_data_directory_on_the_node_named(...)`
   - Each resolves `member_ref` and then immediately delegates to one world/harness action.

2. Small fixed-list action steps repeat the same pairwise resolution glue.
   - `tests/ha/support/steps/mod.rs`
     - `i_kill_the_nodes_named(...)`
     - `i_start_only_the_fixed_nodes(...)`
   - Both resolve two member references and immediately pass the pair to the existing world node-action owners.

3. The DCS toggle matrix is still split across six wrappers even though the real ownership already exists elsewhere.
   - Full-set/quorum steps:
     - `i_stop_the_dcs_service(...)`
     - `i_stop_a_dcs_quorum_majority(...)`
     - `i_start_the_dcs_service(...)`
     - `i_restore_dcs_quorum(...)`
   - Local-per-member steps:
     - `i_stop_the_local_dcs_service_for_node_named(...)`
     - `i_start_the_local_dcs_service_for_node_named(...)`
   - The only moving parts are:
     - which service slice to target
     - whether to start or stop
   - `HaWorld`/`HarnessShared` already own the actual side effects.

4. Blocker toggles still duplicate the same parse-and-dispatch shape.
   - `tests/ha/support/steps/mod.rs`
     - `i_enable_the_blocker_on_the_node_named(...)`
     - `i_disable_the_blocker_on_the_node_named(...)`
   - Both resolve the member, parse the blocker kind, then call `world.harness()?.set_blocker(...)`.

5. Two step-local courier helpers exist only to support that wrapper stack.
   - `resolve_member(...)`
   - `set_dcs_services(...)`
   - They do not own domain logic; they only bounce work into `HaWorld`/`HarnessShared`.

### Execution plan

1. Move given-aware DCS orchestration onto `HaWorld`.
   - Add narrow world-owned actions for:
     - full DCS start/stop
     - quorum-majority DCS start/stop
     - local DCS start/stop for one resolved member
     - cut one resolved member off from DCS
   - Reuse the existing `given` ownership already stored on the harness instead of re-deriving DCS service selection inside the step file.

2. Move the remaining single-member side-effect wrappers onto `HaWorld` where it deletes step glue.
   - Add or reuse world-owned methods for:
     - kill/start one member
     - isolate/heal one member
     - wipe one member data directory
     - enable/disable one blocker
   - Keep multi-member bulk operations on the existing `kill_nodes(...)` / `start_nodes(...)` owners when that stays smaller.

3. Shrink `steps/mod.rs` to scenario phrasing plus polling workflows.
   - Rewrite the action-only step handlers so they become one-line calls into `HaWorld` plus, at most, one small local resolver helper if that remains net-negative.
   - Delete `set_dcs_services(...)`.
   - Delete `resolve_member(...)` if the remaining step handlers can call the world owner directly without growing.

4. Keep the switchover and polling workflows local for this slice.
   - Do not reopen the older plan-84/85 observer-owner churn.
   - Leave:
     - `poll_until(...)`
     - `poll_for_cluster_observation(...)`
     - switchover eligibility polling
     where they are unless a smaller deletion falls out naturally.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_current_lines.sh` and require total lines to fall below the current `34007`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require improvement beyond the current `+10293 -14361 diff: -4068`.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Do not add a new enum/struct/selector object just to encode start-vs-stop or DCS subset choice.
- Keep visibility inside `tests/ha/support`.
- Keep step failure wording materially equivalent unless the smaller owner-local version is clearly better.
- Do not move polling logic into `HaWorld` for this slice; the reduction target is action glue, not observation workflows.
- If moving actions onto `HaWorld` adds more boilerplate than it deletes, switch this plan back to `TO BE VERIFIED` and stop.

### Re-Verification On 2026-03-25

- Attempting to continue this slice by moving the remaining DCS toggles, blocker toggles, wipe, and cut-from-DCS couriers onto `HaWorld` failed the reduction gate.
- The trial edit raised tracked code from `34006` total lines to `34055` and weakened the diff reduction from `-4069` to `-4020`.
- That means the current execution shape is not a valid reduction. Any further work on this boundary needs a different deletion strategy before re-entering execution.

### Expected yield

- Delete the step-local DCS courier helper.
- Delete the step-local member-resolution courier helper, or reduce it to one genuinely small remaining helper.
- Remove repeated harness/given lookup boilerplate from the action-only cucumber steps.
- Flatten the HA support ownership boundary so `steps/mod.rs` is mostly phrasing and `world/mod.rs` owns the mutable side effects.

TO BE VERIFIED
