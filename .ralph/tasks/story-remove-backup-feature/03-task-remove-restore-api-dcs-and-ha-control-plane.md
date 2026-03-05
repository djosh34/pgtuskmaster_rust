---
## Task: Remove the restore request API, DCS state, and HA restore orchestration <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Delete the restore takeover control plane completely so there is no remaining API, DCS keyspace, HA action, or debug surface for cluster restore orchestration.

**Scope:**
- Remove `POST /restore`, `GET /ha/restore`, and `DELETE /ha/restore`, along with any client-side restore helpers.
- Remove DCS restore keys, restore request/status records, and restore-specific cache/state plumbing.
- Remove HA restore phases/actions/status writes and the restore takeover e2e scenario.

**Context from research:**
- The restore control plane landed in commit `80c6940` on 2026-03-05 and touches API, DCS, HA, debug, CLI, and tests.
- Current API types and handlers are in `src/api/controller.rs` and `src/api/worker.rs`.
- Current DCS surface is in `src/dcs/state.rs`, `src/dcs/keys.rs`, `src/dcs/store.rs`, `src/dcs/worker.rs`, and `src/dcs/etcd_store.rs`.
- Current HA surface is in `src/ha/actions.rs`, `src/ha/decide.rs`, `src/ha/worker.rs`, and restore-oriented parts of `src/ha/e2e_multi_node.rs`.
- Current debug and CLI exposure is in `src/debug_api/view.rs` and `src/cli/client.rs`.
- Switchover, HA state inspection, and ordinary HA failover/fencing behavior must remain intact.

**Expected outcome:**
- The node API no longer accepts or reports restore requests.
- DCS no longer stores or parses restore request/status keys.
- HA no longer branches on restore phases or writes restore status records.
- Debug and CLI surfaces no longer mention restore state.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Remove `ClusterRestoreRequestInput`, `ClusterRestoreAcceptedResponse`, `ClusterRestoreDerivedView`, `ClusterRestoreStatusResponse`, `post_restore`, `get_restore_status`, and `delete_restore` from `src/api/controller.rs`.
- [ ] Remove `/restore` and `/ha/restore` route handling, auth matrix, and serialization paths from `src/api/worker.rs`.
- [ ] Remove restore-specific helper methods and response structs from `src/cli/client.rs`.
- [ ] Remove `RestorePhase`, `RestoreRequestRecord`, `RestoreStatusRecord`, and restore cache fields from `src/dcs/state.rs`.
- [ ] Remove restore keys from `src/dcs/keys.rs`.
- [ ] Remove restore put/read/delete/refresh handling from `src/dcs/store.rs`, `src/dcs/worker.rs`, and `src/dcs/etcd_store.rs`.
- [ ] Remove `RunPgBackRestRestore`, `TakeoverRestoredDataDir`, `WriteRestoreStatus`, and any other restore-only actions from `src/ha/actions.rs`.
- [ ] Remove all restore guard logic, restore phase progression, restore status generation, and restore-specific blocking/fencing behavior from `src/ha/decide.rs`.
- [ ] Remove restore-specific action dispatching and DCS writes from `src/ha/worker.rs`.
- [ ] Remove restore state exposure from `src/debug_api/view.rs` and any debug worker fixtures that depend on it.
- [ ] Remove the restore takeover scenario and restore helper scaffolding from `src/ha/e2e_multi_node.rs`, while keeping non-restore HA scenarios intact.
- [ ] Remove restore-specific BDD/API coverage from `tests/bdd_api_http.rs`.
- [ ] Confirm by search that `src/`, `tests/`, and `docs/src/interfaces/node-api.md` no longer contain `/restore`, `restore/status`, `restore/request`, `RestorePhase`, or `WriteRestoreStatus`.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
