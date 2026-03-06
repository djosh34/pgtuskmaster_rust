---
## Task: Remove restore control plane before HA functional rewrite <status>done</status> <passes>true</passes>

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

**Story test policy:**
- Validate the full required gate set for the final tree, including `make test-long`.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Modify `src/api/controller.rs`, `src/api/worker.rs`, `src/cli/client.rs`, `src/dcs/state.rs`, `src/dcs/keys.rs`, `src/dcs/store.rs`, `src/dcs/worker.rs`, `src/dcs/etcd_store.rs`, `src/ha/actions.rs`, `src/ha/decide.rs`, `src/ha/worker.rs`, `src/ha/e2e_multi_node.rs`, and any related debug/test files still carrying restore control-plane code.
- [x] Remove restore request/status API types, handlers, routes, auth, and client helpers.
- [x] Remove `RestorePhase`, `RestoreRequestRecord`, `RestoreStatusRecord`, and restore cache fields from DCS state and refresh logic.
- [x] Remove restore-only HA actions and all restore branching from the HA decision/worker paths.
- [x] Remove the restore takeover HA scenario and restore helper scaffolding from HA tests.
- [x] Confirm by search that `src/`, `tests/`, and docs touched by these files no longer contain `/ha/restore`, `/restore`, `restore/request`, `restore/status`, `RestorePhase`, or `WriteRestoreStatus`.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly on the final tree
</acceptance_criteria>

<plan>
## Execution plan

1. Lock the removal boundary before editing anything.
   - This task removes the HA restore control plane only.
   - Preserve restore machinery that still belongs to non-HA bootstrap or recovery flows, especially `src/process/*`, backup configuration, WAL restore ingest, recovery-bootstrap docs, and the startup/runtime restore-bootstrap path in `src/runtime/node.rs` unless compilation proves they are now dead because of the HA control-plane removal.
   - Treat the acceptance-criteria search strings as applying to the HA control-plane files and related docs/tests touched by this task, not as a mandate to delete every remaining `restore` string from the repository.

2. Remove the API and client-facing restore surfaces first so the public contract is narrowed early.
   - In `src/api/controller.rs`, delete restore request/response structs, restore path helpers, `post_restore`, `delete_restore`, and `get_restore_status`.
   - Keep switchover and HA state endpoints intact.
   - In `src/api/worker.rs`, remove restore imports, route wiring, and restore-specific auth handling while preserving the rest of the API worker.
   - In `src/cli/client.rs`, delete restore-only request/response helper types and the test-only restore helper methods.
   - Update `tests/bdd_api_http.rs` in the same slice by deleting the restore endpoint contract/auth cases instead of waiting for later cleanup; those tests currently enforce the removed public contract and will fail immediately if left behind.
   - Update any API endpoint lists in debug payloads or tests that still advertise `/restore` or `/ha/restore`.

3. Remove restore records from DCS modeling and refresh logic as one coherent slice.
   - In `src/dcs/state.rs`, delete `RestorePhase`, `RestoreRequestRecord`, `RestoreStatusRecord`, and the `restore_request` / `restore_status` cache fields.
   - Update every `DcsCache` constructor and test fixture to stop populating restore fields.
   - In `src/dcs/keys.rs`, remove `RestoreRequest` and `RestoreStatus` key variants and path parsing tests.
   - In `src/dcs/worker.rs`, remove restore value variants plus cache put/delete handling for restore keys.
   - In `src/dcs/store.rs`, remove restore decoding, cache reset handling, and restore-specific refresh tests.
   - In `src/dcs/etcd_store.rs`, remove reconnect/reset expectations and snapshot rebuild tests for restore keys.
   - After this step, DCS should model only steady-state HA coordination data such as members, leader, switchover, config, and init lock.

4. Collapse HA back to non-restore behavior.
   - In `src/ha/actions.rs`, remove restore-only `ActionId` and `HaAction` variants (`PrepareDataDirForRestore`, `RunPgBackRestRestore`, `TakeoverRestoredDataDir`, `WriteRestoreStatus`).
   - In `src/ha/decide.rs`, delete the restore guard path entirely, including executor/non-executor branching, orphan handling, restore heartbeat logic, restore-phase progression, and restore-specific tests.
   - Reconnect the normal HA decision flow so leadership, fail-safe, rewind, basebackup, bootstrap, fencing, and switchover decisions operate without restore-request suppression.
   - In `src/ha/worker.rs`, remove restore-status key writes, restore dispatch branches, and restore-only dispatch tests.
   - Be skeptical about any helper function that becomes trivial or dead after the restore branch removal; delete dead code instead of leaving compatibility shims.

5. Remove restore takeover e2e coverage and any test scaffolding that exists only for it.
   - In `src/ha/e2e_multi_node.rs`, delete `e2e_multi_node_restore_takeover_external_repo_converges_cluster`.
   - Remove helper code that is only used by that scenario, such as external restore-repo preparation helpers, restore-only timeline artifact naming, and restore polling helpers.
   - Keep unrelated HA scenarios and generic harness utilities that are still needed elsewhere.
   - In `Makefile`, remove `ha::e2e_multi_node::e2e_multi_node_restore_takeover_external_repo_converges_cluster` from `ULTRA_LONG_TESTS` so the suite classification stays internally consistent once the long suite is re-enabled in the final story task.
   - Update any other test files, contract tests, or BDD API tests that still exercise `/restore`, `/ha/restore`, restore DCS keys, or restore debug payloads.

6. Remove stale debug and documentation surfaces, then rebuild docs if needed.
   - In `src/debug_api/view.rs` and related snapshot/debug files, remove restore sections, restore blocking state, and any endpoint listings that mention restore control-plane routes.
   - In `docs/src/interfaces/node-api.md`, delete the restore takeover endpoint documentation.
   - Delete `docs/src/operator/cluster-restore-takeover-runbook.md` if it becomes wholly stale, and remove any references to it from summary/navigation files.
   - Update any contributor or operator docs that mention the HA restore control plane, while preserving docs for recovery bootstrap, WAL restore ingest, and startup restore-bootstrap behavior that are still valid.
   - Do not commit generated `docs/book` output; if docs are rebuilt locally, keep only source-doc changes tracked.

7. Run targeted searches before full gates to catch stale references cheaply.
   - Search the edited HA/API/DCS/debug/test/docs files for `/restore`, `/ha/restore`, `restore/request`, `restore/status`, `RestorePhase`, `RestoreRequestRecord`, `RestoreStatusRecord`, and `WriteRestoreStatus`.
   - If remaining hits are legitimate bootstrap/recovery uses outside the HA control plane, verify they are outside the touched scope and not dead references to deleted contracts.
   - Remove any stale assertions, endpoint lists, or docs text discovered by search before running the expensive gates.

8. Run the required validation in the exact final sequence and do not mark done until all of it passes.
   - `make check`
   - `make test`
   - `make lint`
   - `make test-long`
   - If docs source changed in a way that can break mdBook structure, run `make docs-build` as an extra safety check even though it is not one of the mandatory final gates.
   - If any gate failure reveals additional stale restore control-plane code, fix it and rerun the affected gates until the full required set is green.

9. Finish the task bookkeeping only after code, docs, and all gates are green.
   - Tick every completed checkbox in this task file.
   - Set `<passes>true</passes>` only after the final gate run succeeds.
   - Run `/bin/bash .ralph/task_switch.sh`.
   - Commit all tracked changes, including `.ralph` updates, with `task finished 01-task-remove-restore-control-plane-before-ha-functional-rewrite: ...` and include evidence of gate success plus any notable implementation constraints.
   - Push with `git push`.
   - Add an AGENTS.md note only if a genuinely reusable learning emerged during implementation.

## Expected tricky points to verify during execution

- `src/process/jobs.rs`, `src/process/state.rs`, and `src/runtime/node.rs` still mention restore/bootstrap behavior; do not remove those paths unless the compile graph proves they are now dead and they are not part of recovery-bootstrap behavior.
- `tests/bdd_api_http.rs` appears to have several restore endpoint cases and will need coordinated removal alongside API route changes.
- `src/debug_api/view.rs` currently exposes restore state and endpoint names, so API surface removal is incomplete until debug payloads are cleaned too.
- `Makefile` currently classifies the restore takeover e2e as ultra-long, so forgetting that edit will leave the long-suite inventory stale when task 06 finally runs `make test-long`.
- The repo ignores generated docs output; doc source cleanup must avoid introducing tracked `docs/book` artifacts.

NOW EXECUTE
</plan>
