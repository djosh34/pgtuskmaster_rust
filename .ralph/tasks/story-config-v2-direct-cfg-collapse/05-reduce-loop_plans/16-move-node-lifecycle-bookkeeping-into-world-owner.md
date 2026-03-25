## Plan: Move Node Lifecycle Bookkeeping Into World Owner

### Why this reduction target

The next wrong-place duplication is the node lifecycle glue in `tests/ha/support/steps/mod.rs`:

- `i_kill_the_node_named`, `i_kill_the_nodes_named`, `i_kill_all_database_nodes`, `i_restart_the_node_named`, and `i_start_only_the_fixed_nodes` all repeat the same orchestration pattern.
- Each step re-selects members, calls `world.harness()?.kill_node(...)` or `world.harness()?.start_node(...)`, then mutates `world.add_stopped_node(...)` or `world.remove_stopped_node(...)`.
- `HaWorld` already owns `stopped_members`, so step code should not keep open-coding the state transition bookkeeping around the harness call.

### Current overlap already verified

- `tests/ha/support/world/mod.rs` owns `stopped_members`, `add_stopped_node`, and `remove_stopped_node`.
- The only call sites for `add_stopped_node` and `remove_stopped_node` are the five lifecycle steps in `tests/ha/support/steps/mod.rs`.
- `tests/ha/support/world/mod.rs` already owns the actual node runtime behavior through `HarnessShared::kill_node` and `HarnessShared::start_node`.

### Execution plan

1. Move node lifecycle bookkeeping into `HaWorld`.
   - Add world-owned helpers that apply node stop/start behavior across a selected member iterator.
   - Keep the harness call plus `stopped_members` mutation in one owner-local place.
   - Delete `add_stopped_node` and `remove_stopped_node` if the new helpers make them unused.

2. Let steps only choose members.
   - Update the five lifecycle steps to resolve the correct member set, then delegate to the new world-owned helpers.
   - Reuse the existing `all_cluster_members()` helper instead of inventing a new node-action enum or scenario-specific wrapper layer.

3. Keep the boundary flat.
   - Do not move regex parsing or alias resolution into `HaWorld`.
   - Do not introduce a generic action enum just to replace two concrete lifecycle operations.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+4750 -7133 diff: -2383` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Update the reduce-loop task state only after all gates pass.

### Guardrails

- Keep `HaWorld` responsible for scenario runtime state, not cucumber phrase parsing.
- Keep `HarnessShared` responsible for container start/stop behavior, not bookkeeping for `stopped_members`.
- If this starts requiring a new lifecycle enum or pushes alias parsing into world methods, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
