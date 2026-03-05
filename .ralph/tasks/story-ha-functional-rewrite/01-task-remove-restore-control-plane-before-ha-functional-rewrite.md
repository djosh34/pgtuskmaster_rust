---
## Task: Remove restore control plane before HA functional rewrite <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Delete the restore takeover control plane from HA, DCS, API, CLI, debug, and tests before rewriting HA around a functional state-machine design.

**Scope:**
- Edit `src/api/controller.rs`, `src/api/worker.rs`, `src/cli/client.rs`, `src/dcs/{state,keys,store,worker,etcd_store}.rs`, `src/ha/{actions,decide,worker,e2e_multi_node}.rs`, and any debug/test files still exposing restore state.
- Remove restore request/status DCS records and restore-specific API/debug/CLI surfaces.
- Remove restore-only HA actions, restore-phase branching, and the restore takeover scenario from HA tests.

**Context from research:**
- The project already has a broader backup-removal story, but this restore-control-plane slice is the part directly poisoning HA architecture and must move into the HA rewrite story.
- Current HA complexity is inflated by restore takeover branches that are not core HA:
  - `src/ha/decide.rs` branches on restore request/status before normal phase handling.
  - `src/ha/actions.rs` includes restore-only actions such as `RunPgBackRestRestore`, `TakeoverRestoredDataDir`, and `WriteRestoreStatus`.
  - `src/dcs/state.rs` carries `RestorePhase`, `RestoreRequestRecord`, and `RestoreStatusRecord`, which are unrelated to normal leader-election/failover state.
  - `src/ha/e2e_multi_node.rs` contains a long external-repo restore scenario that is not part of the HA core we want to keep.
- We discussed explicitly that the functional rewrite should happen after removing pgBackRest/restore from HA, because otherwise the refactor would spend effort making a deleted feature prettier.

**Expected outcome:**
- HA no longer knows what restore takeover is.
- DCS no longer stores restore request/status keys or records.
- API, CLI, debug, and tests no longer expose restore control-plane state.
- The remaining HA surface is about leader election, fail-safe, replication posture, rewinding/basebackup/bootstrap, and fencing only.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Modify `src/api/controller.rs`, `src/api/worker.rs`, `src/cli/client.rs`, `src/dcs/state.rs`, `src/dcs/keys.rs`, `src/dcs/store.rs`, `src/dcs/worker.rs`, `src/dcs/etcd_store.rs`, `src/ha/actions.rs`, `src/ha/decide.rs`, `src/ha/worker.rs`, `src/ha/e2e_multi_node.rs`, and any related debug/test files still carrying restore control-plane code.
- [ ] Remove restore request/status API types, handlers, routes, auth, and client helpers.
- [ ] Remove `RestorePhase`, `RestoreRequestRecord`, `RestoreStatusRecord`, and restore cache fields from DCS state and refresh logic.
- [ ] Remove restore-only HA actions and all restore branching from the HA decision/worker paths.
- [ ] Remove the restore takeover HA scenario and restore helper scaffolding from HA tests.
- [ ] Confirm by search that `src/`, `tests/`, and docs touched by these files no longer contain `/ha/restore`, `/restore`, `restore/request`, `restore/status`, `RestorePhase`, or `WriteRestoreStatus`.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
