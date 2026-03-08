# Verbose context for docs/src/how-to/remove-cluster-node.md

Important scope boundary:
- The requested files do not show a dedicated operator-facing "remove node" API or CLI command.
- The codebase does support some DCS key deletion primitives, but those are focused on leader and switchover keys, not a documented member-decommission workflow.
- The e2e policy test explicitly forbids direct DCS writes/deletes from HA tests after startup, which is a strong hint that docs should not tell operators to manually mutate DCS internals as the normal path.

What DCS deletion support actually exists:
- `src/dcs/store.rs` exposes a generic `delete_path`.
- The `DcsHaWriter` helper provides:
  - `delete_leader(scope)` -> deletes `/{scope}/leader`
  - `clear_switchover(scope)` -> deletes `/{scope}/switchover`
- I did not see a dedicated helper for deleting `/{scope}/member/{member_id}` records in the requested files.
- `src/dcs/keys.rs` confirms the member key shape is exactly `/{scope}/member/{member_id}`.

What the DCS cache model implies:
- Member records live under `/{scope}/member/{member_id}`.
- HA logic consumes the cached member set, leader record, and switchover record.
- If a member disappears from the cache, the decision engine reacts to the changed topology rather than running a special "node removal" procedure.

What the HA decision code suggests about node disappearance:
- The decision engine is driven by current facts such as:
  - active leader member id
  - available primary member id
  - trust state
  - whether PostgreSQL is reachable
- If a former leader disappears and another active leader exists, the local node may step down and fence depending on role and phase.
- If no leader remains, candidate nodes attempt leadership.
- This means "remove node" is mostly an external operational event whose effects are absorbed by trust evaluation and HA reconciliation.

What process/job support exists:
- `src/process/jobs.rs` defines generic jobs such as demote, start, base backup, rewind, promote, and fencing.
- There is no dedicated decommission or unregister job in the requested files.
- Fencing exists as a safety shutdown action, but it is not described in code as a complete node-removal workflow.

What the test harness suggests:
- `tests/ha/support/multi_node.rs` contains fixture-level node shutdown behavior and cluster observation helpers.
- It also treats etcd shutdown as external quorum loss rather than direct DCS steering.
- This supports documenting node removal as an operational process driven by:
  - stopping the node
  - confirming the remaining cluster converges
  - checking trust and leader state
  - only then considering cleanup of stale infrastructure/resources

Safe documentation stance from the sources:
- Prefer "stop or decommission the host/container, then observe the cluster reconcile" over "delete DCS member keys directly."
- If the page mentions DCS cleanup at all, it should be framed carefully:
  - member keys use `/{scope}/member/{member_id}`
  - the requested files do not define a first-class operator API for removing them
  - manual DCS surgery is not shown as the standard supported path
- The how-to should probably distinguish graceful scale-down from failure/removal after the fact, while being explicit that the repo does not currently expose a dedicated removal command.
