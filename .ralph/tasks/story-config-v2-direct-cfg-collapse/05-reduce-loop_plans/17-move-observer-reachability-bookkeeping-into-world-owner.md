## Plan: Move Observer Reachability Bookkeeping Into World Owner

### Why this reduction target

The next wrong-place duplication is the observer reachability bookkeeping in `tests/ha/support/steps/mod.rs`:

- `i_fully_isolate_the_node_named_from_the_cluster`, `i_isolate_the_node_named_from_observer_api_access`, `i_heal_network_faults_on_the_node_named`, and `i_heal_all_network_faults` all pair harness network-fault actions with direct `HaWorld` mutation.
- Each step mutates `world.mark_observer_unreachable(...)`, `world.clear_observer_unreachable(...)`, or `world.clear_observer_unreachable_members()` immediately around the runtime fault action.
- `HaWorld` already owns `observer_unreachable_members`, so step code should not keep open-coding the bookkeeping for that scenario state.

### Current overlap already verified

- `tests/ha/support/world/mod.rs` owns `observer_unreachable_members`, `mark_observer_unreachable`, `clear_observer_unreachable`, and `clear_observer_unreachable_members`.
- The only call sites for those helpers are the four network-fault steps in `tests/ha/support/steps/mod.rs`.
- The harness already owns the runtime fault behavior through `isolate_member_from_all_peers_on_path`, `cut_member_off_from_dcs`, `isolate_member_from_observer_on_api`, `isolate_member_from_observer_on_postgres`, `heal_member_network_faults`, and `clear_all_network_faults`.

### Execution plan

1. Move observer reachability bookkeeping into `HaWorld`.
   - Add world-owned helpers that perform the relevant network-fault action and update `observer_unreachable_members` in the same owner-local place.
   - Delete `mark_observer_unreachable`, `clear_observer_unreachable`, and `clear_observer_unreachable_members` if the new helpers make them unused.

2. Let steps only choose members.
   - Update the four network-fault steps to resolve the correct member set, then delegate to the new world-owned helpers.
   - Keep the fixed traffic-path list local to the world helper instead of recreating it in the step layer.

3. Keep the boundary flat.
   - Do not move cucumber phrase parsing or blocker-name parsing into `HaWorld`.
   - Do not introduce a generic network-action enum just to replace a small set of concrete world methods.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+4781 -7174 diff: -2393` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Update the reduce-loop task state only after all gates pass.

### Guardrails

- Keep `HaWorld` responsible for scenario runtime state, not cucumber step parsing.
- Keep `HarnessShared` responsible for fault injection mechanics, not observer reachability bookkeeping.
- If this starts requiring a new lifecycle/network enum or pushes parsing logic into world methods, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
