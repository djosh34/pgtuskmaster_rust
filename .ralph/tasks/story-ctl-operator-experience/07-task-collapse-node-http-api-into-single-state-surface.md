## Task: Collapse The Node HTTP API Into One `/state` Surface And Remove Debug Snapshotting <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Replace the current split HTTP read surfaces (`/ha/state`, `/debug/*`, `/fallback/*`) with one read endpoint, `GET /state`, that returns a single serializable `NodeState` document containing the node's current state and DCS-backed cluster state. Remove the debug snapshot/history subsystem entirely, remove generic versioned state envelopes entirely, and make `POST /switchover` plus `DELETE /switchover` the only remaining control endpoints.

**Original user shift / motivation:** The user explicitly called out that the current `pgtm` and API design feels overcomplicated, weirdly split, and full of duplicate logic. They want the product to stop feeling like "many shared subsystems plus a smart client" and instead feel like a simple node API with one very verbose state document. They specifically asked for one `/state` endpoint that "literally prints all info available" with all filtering client-side, for `/ha` to disappear, for `/switchover` to use both `POST` and `DELETE`, for debug/history/version metadata to go away, and for the cross-node API fanout/proxy idea to be rejected in favor of DCS-only state.

**Higher-order goal:** Shrink the operator/control-plane architecture to one source-backed read model and one source-backed control model. Delete the parallel debug API/read-model/snapshot path, delete the generic versioning/history substrate that only exists to support it, stop making `pgtm` compare peer self-reports over separate API calls, and make the node API reflect the actual runtime state types as directly as possible instead of maintaining parallel `*Response` / `*View` trees.

**Scope:**
- Collapse the read API in `src/api/` to one `GET /state` route and two switchover control routes: `POST /switchover` and `DELETE /switchover`.
- Remove `src/debug_api/` completely and remove the old debug snapshot/history pipeline from runtime startup, API routing, tests, docs, and CLI.
- Remove the generic `Versioned<T>` state envelope and the debug-history/event timeline model from production code, tests, and docs. The user explicitly does not want history or generic version metadata.
- Introduce one top-level serializable `NodeState` output type for `/state`; avoid parallel response/view trees. Existing real state types should be serialized directly wherever feasible, using targeted serde skipping only where it materially reduces noise or avoids leaking internals/secrets.
- Remove the embedded `RuntimeConfig` copy from `DcsState` / `DcsCache`; `DcsState` must no longer carry full config.
- Rework `pgtm` so it stops faning out to peer `api_url` values for status/debug discovery and instead uses one seed node's `/state` response plus DCS-backed cluster data only.
- Fully clean up stale endpoints, stale DTOs, stale docs, stale CLI commands, stale tests, and stale code paths left behind by the old design.
- This task covers the API/CLI/docs cleanup needed to land the single-surface operator model. If any follow-up work is required, it must be captured explicitly in a separate named task rather than left implicit.

**Context from research:**
- The node runtime currently starts seven long-lived workers in `src/runtime/node.rs`: `pginfo`, `dcs`, `process`, `logging::postgres_ingest`, `ha`, `debug_api`, and `api`. The split is wired in `run_workers(...)` in `src/runtime/node.rs`.
- The current stable read surface is split: `GET /ha/state` is served from local `DcsState` and `HaState` in `src/api/worker.rs` and `src/api/controller.rs`, while `GET /debug/verbose` and `GET /debug/snapshot` are served from a separate debug snapshot publisher built in `src/debug_api/worker.rs` and `src/debug_api/snapshot.rs`.
- The current debug subsystem maintains local-only in-memory history (`changes`, `timeline`) with a separate `SystemSnapshot` model and a separate JSON payload mapper in `src/debug_api/view.rs`. The user explicitly rejected keeping that history/versioning machinery.
- `pgtm status` currently seeds from one node's `/ha/state`, then fans out directly to every advertised peer `api_url` from the CLI in `src/cli/status.rs` `build_sampled_cluster_snapshot(...)` and `sample_peer_states(...)`. The user explicitly rejected keeping cross-node API talk or introducing server-side proxying.
- The current peer API fanout is not mainly about raw replication facts. DCS already carries cluster member routing and PostgreSQL observations in `src/dcs/state.rs` (`MemberSlot`, `MemberRouting`, `MemberPostgresView`, WAL vectors, replica upstream). The peer API fanout mainly adds per-node local trust/authority/role/process/debug self-reporting, which the user wants to stop depending on.
- `WorldView` in `src/ha/types.rs` is the HA engine's derived view of current local and global facts. It is a good candidate to ship inside `/state`, but it is not a complete replacement for the raw current states because it does not contain config, raw DCS cache contents, raw process state, or all raw pginfo details.
- `HaWorkerCtx`, `ProcessWorkerCtx`, and other `*Ctx` types are runtime wiring objects containing subscribers, senders, trait objects, closures, and log handles. They must not become API models. The aggregate state concept that already exists is closer to `DebugSnapshotCtx` / `SystemSnapshot` in `src/debug_api/snapshot.rs`, but the approved direction is to keep a single `NodeState` response and delete the separate snapshot subsystem.
- The current `DcsState` / `DcsCache` in `src/dcs/state.rs` embeds `RuntimeConfig`. That is both an architectural smell and an API blocker because it drags full config into a DCS state object. The approved direction is that `DcsState` must no longer contain config.
- The current API path design is inconsistent: `POST /switchover` but `DELETE /ha/switchover`. The user explicitly wants `/ha` dropped and `/switchover` used for both methods.
- The current fallback endpoints in `src/api/fallback.rs` are not part of the approved future model. `POST /fallback/heartbeat` is effectively a no-op validator today.
- The user explicitly pushed back on overcomplicating the read model with `NodeState` plus multiple extra wrapper/view types. The approved design direction is: one top-level `NodeState`, direct serialization of real state/config/domain types where feasible, targeted serde skips instead of proliferating parallel `*View` trees, and no new response-DTO hierarchy unless strictly unavoidable.
- The user explicitly rejected keeping history and generic version metadata. That means no `snapshot_sequence`, no `channel_version`, no `previous_version` / `current_version`, no debug `since` cursor, and no `Versioned<T>`-shaped API output.
- The user explicitly rejected the proxy idea. This task must not add a cluster aggregation proxy, cross-node API fetches from the server, or continued peer fanout from `pgtm`.

**Expected outcome:**
- Operators and automation can hit one endpoint, `GET /state`, and receive one verbose JSON document that contains the node's current runtime state, the DCS-backed cluster/member facts, and the HA engine's current derived worldview, with no separate debug/status split.
- `pgtm` no longer needs `debug` subcommands or peer API sampling to synthesize cluster state; it can consume one seed `/state` document and do any display/filtering client-side from that single payload.
- The codebase no longer contains the debug snapshot/history/version pipeline, no longer contains `/ha/state` or `/debug/*`, no longer contains `/ha/switchover`, and no longer contains DCS state objects that embed full config.
- The API surface and docs become dramatically smaller and more coherent: one read endpoint and one control noun.

</description>

<acceptance_criteria>
- [ ] `src/api/worker.rs` exposes exactly one read endpoint, `GET /state`, and only two control endpoints, `POST /switchover` and `DELETE /switchover`; `/ha/state`, `/ha/switchover`, `/debug/*`, and `/fallback/*` are removed from production code and docs
- [ ] A single top-level serializable `NodeState` type is the HTTP output for `/state`; the read path no longer uses `HaStateResponse`, `DebugVerbosePayload`, `SystemSnapshot`, or any parallel read-model/response hierarchy for current state
- [ ] `src/debug_api/` is fully removed and runtime startup in `src/runtime/node.rs` no longer creates or depends on `debug_api` workers, publishers, subscribers, or routes
- [ ] Generic `Versioned<T>` state envelopes and the debug history/timeline subsystem are fully removed from production code, tests, docs, and API output; no snapshot/version/history fields remain in the shipped `/state` surface
- [ ] `src/dcs/state.rs` no longer embeds `RuntimeConfig` inside `DcsState` / `DcsCache`; DCS state contains only DCS/domain data needed for publication and HA decisions
- [ ] `NodeState` includes the real current subsystem state needed for operators, including DCS-backed cluster member state and HA-derived `WorldView`, and it does so primarily by serializing real internal types directly rather than introducing a new parallel tree of `*View` / `*Response` types
- [ ] The implementation uses targeted serde derives / field skipping where appropriate, but runtime `*Ctx` wiring types are not serialized and no misleading "serializable context" surrogate is introduced
- [ ] `pgtm` stops faning out to peer `api_url` values and stops depending on peer `/debug/verbose` or peer `/ha/state` reads; status/primary/replicas/debug-related logic is updated to use the single seed `/state` payload and DCS-only cluster data
- [ ] The CLI no longer exposes the old debug command path or stale status/debug split behavior; docs and help output reflect the single `/state` model and the new `/switchover` method pair
- [ ] Old endpoints, old debug/history/version types, old CLI fanout code, old docs/tutorials, and old tests are fully cleaned from the repo; repo-wide verification confirms no accidental stale occurrences remain outside explicitly allowed historical task text
- [ ] docs are updated with the new single-surface API/CLI model, and stale debug/status split docs are deleted or rewritten rather than left misleadingly in place
- [ ] `make check` passes cleanly
- [ ] `make test` passes cleanly
- [ ] `make test-long` passes cleanly
- [ ] `make lint` passes cleanly
- [ ] `<passes>true</passes>` is set only after every acceptance-criteria item and every required implementation-plan checkbox is complete
</acceptance_criteria>

## Detailed implementation plan

### Phase 1: Define the single read model and remove the split contract
- [ ] Add a new top-level `NodeState` type under `src/api/` to represent the full `/state` payload. This must be the one HTTP read struct and the one serde output contract for current state.
- [ ] Update `src/api/mod.rs` so the old `HaStateResponse`, `AcceptedResponse`, and related stable read DTO hierarchy are either removed or reduced to only the control-plane request/response pieces still needed for switchover.
- [ ] Decide the exact `NodeState` field set and encode it directly in code, including:
  - current node/app lifecycle state,
  - current config state (serialized directly where safe and with targeted serde skipping where needed),
  - current raw pg state,
  - current raw DCS state,
  - current raw process state,
  - current raw HA state,
  - current derived `WorldView`.
- [ ] Prefer deriving `Serialize` on the real internal state types used by `NodeState` instead of introducing a second tree of read-only DTOs. Audit `src/pginfo/state.rs`, `src/process/state.rs`, `src/ha/state.rs`, `src/ha/types.rs`, `src/dcs/state.rs`, `src/state/time.rs`, and adjacent nested types and add serde derives where the type is already a real state model.
- [ ] Use targeted `#[serde(skip_serializing)]` only where a small number of fields are truly internal or sensitive. Do not solve the problem by inventing per-subsystem `*View` wrappers unless a concrete blocker is discovered.
- [ ] Do not serialize any `*Ctx` runtime wiring types such as `HaWorkerCtx`, `ProcessWorkerCtx`, `PgInfoWorkerCtx`, `DcsWorkerCtx`, or `ApiWorkerCtx`.

### Phase 2: Remove generic version/history/snapshot infrastructure
- [ ] Remove the generic `Versioned<T>` envelope from `src/state/time.rs`, `src/state/watch_state.rs`, and all state-channel publishers/subscribers. State channels should carry raw current state values rather than generic version/timestamp wrappers.
- [ ] Update every current publisher/subscriber user in `src/runtime/node.rs`, `src/pginfo/`, `src/dcs/`, `src/process/`, `src/ha/`, and `src/api/` so they no longer depend on `latest().version`, `latest().updated_at`, or `Versioned<T>` wrappers.
- [ ] Remove the entire debug history/timeline model:
  - `src/debug_api/snapshot.rs` `SystemSnapshot`, `DebugChangeEvent`, `DebugTimelineEntry`, `DebugSnapshotCtx`
  - `src/debug_api/worker.rs` history buffers, signature comparison, and snapshot publishing
  - all `since` cursor behavior and all history/version fields in docs/tests/output.
- [ ] Remove `tests/bdd_state_watch.rs` or rewrite it against the new raw state-channel contract; the test must no longer assert generic version increments or `updated_at` behavior that is being deleted.
- [ ] Remove or rewrite any production tests that still assume versioned state or snapshot/history output, including the embedded debug/snapshot tests in `src/api/worker.rs`.

### Phase 3: Refactor DCS and HA state for direct serialization
- [ ] Refactor `src/dcs/state.rs` so `DcsCache` / `DcsState` no longer embeds `RuntimeConfig`. Replace that dependency with only the specific DCS/HA-domain fields actually required for trust evaluation and member publication.
- [ ] Update all DCS trust/publication code in `src/dcs/worker.rs`, `src/ha/worker.rs`, and any helper modules so they no longer reach into `dcs.cache.config`.
- [ ] Audit `src/ha/worker.rs` and `src/ha/types.rs` so `WorldView`, `LocalKnowledge`, `GlobalKnowledge`, and nested types can be serialized directly.
- [ ] Revisit misleading HA terms that assumed cross-node API probing. In particular, audit `ApiVisibility` and adjacent peer-knowledge fields in `src/ha/types.rs` / `src/ha/worker.rs` so shipped `/state` semantics reflect the DCS-only design and do not claim liveness/reachability that is no longer measured.
- [ ] Ensure `NodeState` contains both raw DCS member facts and the derived `WorldView`, so the endpoint exposes both facts and interpretation without requiring extra read models.

### Phase 4: Collapse routing into `/state` and `/switchover`
- [ ] Rewrite `src/api/worker.rs` route handling so only these production routes remain:
  - `GET /state`
  - `POST /switchover`
  - `DELETE /switchover`
- [ ] Remove the old `GET /ha/state`, `DELETE /ha/switchover`, `/debug/snapshot`, `/debug/verbose`, `/debug/ui`, `/fallback/cluster`, and `/fallback/heartbeat` paths and their route tests.
- [ ] Remove `src/api/fallback.rs` if it becomes dead after route collapse.
- [ ] Replace the old `get_ha_state(...)` projection path in `src/api/controller.rs` with a `build_node_state(...)` assembly path that constructs the single `NodeState` directly from the latest current states and derived `WorldView`.
- [ ] Keep the switchover control logic, but move it fully to `/switchover` for both `POST` and `DELETE`, cleaning up the old path split and the CLI client methods accordingly.

### Phase 5: Delete `debug_api` from runtime and keep only one API subsystem
- [ ] Remove `src/debug_api/` from the crate module tree and update `src/lib.rs` accordingly.
- [ ] Update `src/runtime/node.rs` so runtime startup no longer creates debug snapshot publishers/subscribers or runs the `debug_api` worker.
- [ ] Remove all remaining debug snapshot wiring from `ApiWorkerCtx` and adjacent APIs in `src/api/worker.rs`.
- [ ] Remove any dead code that existed only to support the old debug UI or snapshot pipeline, including embedded HTML/JS UI content in `src/api/worker.rs`.

### Phase 6: Simplify `pgtm` to consume one seed `/state`
- [ ] Refactor `src/cli/client.rs` so the read client uses `GET /state` and the control client uses `POST /switchover` / `DELETE /switchover`. Remove stale `/ha/state`, `/ha/switchover`, and `/debug/verbose` client methods.
- [ ] Remove the old debug CLI flow in `src/cli/debug.rs`, `src/cli/mod.rs`, `src/cli/args.rs`, and `src/cli/output.rs`.
- [ ] Rewrite `src/cli/status.rs` so it no longer seeds from one node and then fan-outs to peer APIs. Delete `sample_peer_states(...)`, `build_sampled_cluster_snapshot(...)` peer API polling behavior, and any `JoinSet`-based peer sampling that only existed for cross-node API reads.
- [ ] Rework status/primary/replicas resolution so they use the single seed `/state` response and only DCS-backed cluster member data from that payload.
- [ ] Remove debug-observation handling and stale error branches that existed only because status tried to enrich itself with peer `/debug/verbose` calls.
- [ ] Keep or simplify `pgtm status`, `pgtm primary`, `pgtm replicas`, and `pgtm switchover` as appropriate, but ensure none of them depend on peer API calls or the removed debug/status split.

### Phase 7: Tests and docs for the single-surface model
- [ ] Update `tests/bdd_api_http.rs` to assert the new endpoint and route behavior:
  - `GET /state`
  - `POST /switchover`
  - `DELETE /switchover`
  - removal of old debug/ha/fallback routes
- [ ] Update or replace CLI tests in `tests/cli_binary.rs` and `src/cli/*` unit tests so they assert the new `/state`-driven behavior and the removal of peer fanout / debug commands.
- [ ] Update any HA support code that still calls the old API shape, including `tests/ha/support/observer/pgtm.rs` and any feature helpers that assume `/ha/state` or debug endpoints.
- [ ] Rewrite `docs/src/reference/http-api.md` around the final `/state` and `/switchover` model; merge or delete the old debug API reference so there is no longer a separate debug read-surface document.
- [ ] Update or remove `docs/src/reference/debug-api.md`, `docs/src/reference/overview.md`, `docs/src/reference/pgtm-cli.md`, `docs/src/how-to/debug-cluster-issues.md`, `docs/src/tutorial/debug-api-usage.md`, and any other operator docs that still teach `/debug/*`, `/ha/state`, or peer API fanout.
- [ ] Ensure docs do not preserve stale examples of `snapshot_sequence`, `channel_version`, `since`, `/debug/verbose`, `/debug/ui`, `/ha/state`, or `/ha/switchover`.

### Phase 8: Repo-wide stale cleanup verification and closeout
- [ ] Run repo-wide verification for removed endpoint paths and confirm only explicitly allowed historical/task references remain:
  - `rg -n "(/ha/state|/ha/switchover|/debug/|/fallback/|snapshot_sequence|channel_version|schema_version|previous_version|current_version|history_changes|history_timeline)" src tests docs .ralph/tasks`
- [ ] Run repo-wide verification for removed implementation types/patterns and confirm only explicitly allowed historical/task references remain:
  - `rg -n "(Versioned<|SystemSnapshot|DebugApiCtx|DebugSnapshotCtx|DebugChangeEvent|DebugTimelineEntry|build_sampled_cluster_snapshot|sample_peer_states|JoinSet)" src tests docs .ralph/tasks`
- [ ] Run repo-wide verification that `DcsState` no longer embeds config:
  - `rg -n "cache\\.config|RuntimeConfig" src/dcs src/ha src/api tests`
- [ ] Run targeted verification that `pgtm` no longer performs peer API fanout:
  - `rg -n "(api_url.*Url::parse|with_base_url|get_debug_verbose|get_ha_state\\(\\).*await.*JoinSet|sample_peer_states)" src/cli tests`
- [ ] Run `make check`
- [ ] Run `make test`
- [ ] Run `make test-long`
- [ ] Run `make lint`
- [ ] Update docs/task notes if required by the task
- [ ] Only after all required checkboxes are complete, set `<passes>true</passes>`
- [ ] Run `/bin/bash .ralph/task_switch.sh`
- [ ] Commit all required files, including `.ralph/` updates, with a task-finished commit message that includes verification evidence
- [ ] Push with `git push`

TO BE VERIFIED
