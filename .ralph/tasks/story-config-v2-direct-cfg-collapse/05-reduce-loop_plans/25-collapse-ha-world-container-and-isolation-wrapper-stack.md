## Plan: Collapse HA World Container And Isolation Wrapper Stack

### Why this reduction target

`tests/ha/support/world/mod.rs` is still 1,300+ lines and the current action boundary is fragmented by thin wrappers that mostly rename one concrete operation:

- `kill_node(...)`, `start_node(...)`, `stop_service(...)`, and `start_service(...)` all do the same three-step flow: resolve a container id, emit a timeline note, then call one Docker lifecycle method.
- `apply_dcs_services(...)` exists only to feed `Self::stop_service` or `Self::start_service` into a tiny forwarding loop.
- `isolate_member_from_peer_on_path(...)` has one real caller, `isolate_member_from_all_peers_on_path(...)`.
- `cut_member_off_from_dcs(...)`, `isolate_member_from_observer_on_api(...)`, and `isolate_member_from_observer_on_postgres(...)` are convenience wrappers over `block_member_path_to_host(...)` or `block_member_path_to_address(...)`, but the owning workflows already live in `HaWorld::fully_isolate_node_from_cluster(...)`, `HaWorld::isolate_node_from_observer_api_access(...)`, and the step `i_cut_the_node_named_off_from_dcs(...)`.

That means one file currently spreads "apply a concrete container/network action and record it" across too many local helper layers.

### Current overlap already verified

- `HaWorld::kill_nodes(...)` and `HaWorld::start_nodes(...)` each forward straight into one Harness lifecycle wrapper per member.
- `HarnessShared::stop_dcs_services(...)` and `HarnessShared::start_dcs_services(...)` differ only by which single-service wrapper they pass to `apply_dcs_services(...)`.
- `HarnessShared::isolate_member_from_all_peers_on_path(...)` only delegates pairwise work into `isolate_member_from_peer_on_path(...)`, which immediately expands back into two `block_member_path_to_host(...)` calls.
- The observer/DCS isolation wrappers do not add reusable state; they only pre-fill one path/peer/gateway argument before calling the concrete block helpers.

### Execution plan

1. Collapse the container lifecycle wrapper stack.
   - Keep one concrete helper for "resolve container id, record the phase/detail, invoke one Docker lifecycle action".
   - Rebuild `kill_node(...)`, `start_node(...)`, `stop_dcs_services(...)`, and `start_dcs_services(...)` directly on that shared helper.
   - Remove `stop_service(...)`, `start_service(...)`, and `apply_dcs_services(...)`.

2. Collapse the peer-isolation wrapper stack.
   - Inline `isolate_member_from_peer_on_path(...)` into `isolate_member_from_all_peers_on_path(...)`.
   - Keep `block_member_path_to_host(...)`, `unblock_member_path_to_host(...)`, and `block_member_path_to_address(...)` as the actual reusable boundaries because they still own the concrete host/address resolution work.

3. Inline the single-purpose observer and DCS convenience wrappers into their owners.
   - Inline `cut_member_off_from_dcs(...)` where it is used by `HaWorld::fully_isolate_node_from_cluster(...)` and `i_cut_the_node_named_off_from_dcs(...)`.
   - Inline `isolate_member_from_observer_on_api(...)` into `HaWorld::fully_isolate_node_from_cluster(...)` and `HaWorld::isolate_node_from_observer_api_access(...)`.
   - Inline `isolate_member_from_observer_on_postgres(...)` into `HaWorld::fully_isolate_node_from_cluster(...)`.
   - Preserve the current timeline note strings and fault semantics exactly.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the reduction improves beyond the current `+5230 -7487 diff: -2257` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Run the validation gates sequentially, not in parallel.

### Guardrails

- Keep the refactor local to `tests/ha/support/world/mod.rs` and the directly affected step call site in `tests/ha/support/steps/mod.rs`.
- Do not add a new enum or struct just to describe start-vs-stop or observer-vs-DCS; reuse existing call shapes and function pointers/closures instead.
- If removing the observer/DCS wrappers makes the owning call sites materially harder to read, keep at most one helper there and still delete the container lifecycle stack plus the single-use peer wrapper.
- If the shared lifecycle helper starts growing action-specific branching, switch this plan back to `TO BE VERIFIED`.

NOW EXECUTE
