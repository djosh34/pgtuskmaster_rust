---
## Task: Add cluster restore endpoint and safe takeover orchestration across HA/DCS <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Provide an admin API endpoint that forces a full restore takeover on one node and safely converges the entire cluster onto the restored timeline under normal pgtuskmaster HA control.

**Scope:**
- Add restore intent endpoint(s) and status endpoint(s) in API layer with admin auth enforcement.
- Persist restore intents/locks/status in DCS with strict single-flight coordination and explicit ownership.
- Extend HA decision/worker flow to execute restore takeover sequence:
- fence conflicting primaries safely
- execute restore on selected node
- start postgres recovery with managed config takeover
- transition restored node into normal primary leadership
- force remaining nodes to converge via rewind/basebackup as needed
- Ensure restore orchestration works whether the source backup came from a pgtuskmaster-managed cluster or an external/non-pgtuskmaster cluster.

**Context from research:**
- Existing API surface currently supports switchover/fallback but no restore lifecycle endpoint (`src/api/worker.rs`).
- Existing HA loop already coordinates demote/promote/start/rewind/bootstrap actions, so restore should plug into that action model (`src/ha/actions.rs`, `src/ha/worker.rs`, `src/ha/decide.rs`).
- Existing DCS cache/state model already tracks cluster coordination records and can be extended for restore-intent records (`src/dcs/keys.rs`, `src/dcs/state.rs`, `src/dcs/store.rs`).

**Expected outcome:**
- Operators can trigger a restore takeover via API and see explicit progress/failure state.
- Only one restore flow can run at a time cluster-wide; duplicate/competing requests are rejected safely.
- After successful restore, cluster resumes normal HA behavior and non-executor nodes converge automatically.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] Extend API routes/controllers in `src/api/controller.rs` and `src/api/worker.rs`:
- [ ] add admin endpoint to request restore takeover (e.g. `POST /restore`)
- [ ] add read endpoint to inspect restore status/progress/errors
- [ ] enforce admin auth role and clear input validation errors
- [ ] Extend DCS model/store in `src/dcs/keys.rs`, `src/dcs/state.rs`, `src/dcs/store.rs`, and adapter(s):
- [ ] add restore intent/status records and canonical key paths
- [ ] implement compare-and-set/single-flight semantics for restore ownership
- [ ] ensure watch/cache refresh logic handles restore records correctly across reconnect/resnapshot
- [ ] Extend HA action model in `src/ha/actions.rs`, `src/ha/decide.rs`, `src/ha/worker.rs`, and related state types:
- [ ] define restore orchestration actions/phases and deterministic transition rules
- [ ] enforce fencing-first behavior on conflicting primaries during restore takeover
- [ ] converge non-executor nodes to restored leader via existing rewind/basebackup flows
- [ ] ensure failed restore leaves cluster in explicit safe/diagnosable state
- [ ] Extend debug snapshot/views in `src/debug_api/snapshot.rs` and `src/debug_api/view.rs` to expose restore lifecycle state
- [ ] Add API BDD tests in `tests/bdd_api_http.rs`:
- [ ] auth matrix for restore endpoint(s)
- [ ] bad request and idempotency/conflict responses
- [ ] status endpoint payload checks
- [ ] Add HA integration tests (unit + worker integration) for restore orchestration correctness:
- [ ] competing restore requests cannot run concurrently
- [ ] restore request mid-switchover/failover resolves safely
- [ ] split-brain prevention rules remain intact
- [ ] Add real e2e scenario(s) in HA test suites showing full restore takeover and cluster convergence
- [ ] Document operator restore runbook and rollback guidance in `docs/src/operator/`
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

---

## Plan (detailed; exhaustive file checklist)

### Key design decisions (lock these first)

- **Control-plane style:** match existing switchover design: **API writes intent to DCS**, HA reacts asynchronously via DCS snapshot; no direct “RPC into HA worker” from HTTP handlers.
- **Single-flight semantics:** use DCS **create-if-absent** for an immutable restore *request* so only one restore can be active cluster-wide at a time.
- **Do not couple executor selection to “which node received the HTTP request”:** admin traffic is often load-balanced. Make executor explicit in the request body:
  - `executor_member_id` is **required** in v1.
  - `POST /restore` can be sent to *any* node; the request record names the executor node that must execute it.
- **Safety during restore:** while a restore record is active:
  - Non-executor nodes must **not** acquire leadership (`AcquireLeaderLease`) or promote.
  - Any node that is primary while not executor must **demote + release leader lease** (fencing-first posture).
- **Progress visibility + safety against “intent clobber”:** split DCS into **immutable request** + **mutable status**:
  - `/{scope}/restore/request` (written once via put-if-absent; never overwritten)
  - `/{scope}/restore/status` (written by the executor as it progresses; includes heartbeat + last_error + phase)
- **Orphan handling (must avoid cluster deadlock):** `restore/status` contains `heartbeat_at_ms` updated by the executor every HA tick while non-terminal. If the heartbeat is stale beyond a policy threshold:
  - HA must treat restore as **orphaned** and stop blocking leadership forever.
  - API status must surface the orphaned state clearly (so operators can `DELETE /ha/restore` and retry with a new executor).
  - v1 does **not** auto-reassign executors (too risky without a real member liveness/TTL model); it only unblocks normal HA once orphaned is detected.

### API contract (admin endpoint + status endpoint)

- `POST /restore` (admin-only; mirrors `POST /switchover`)
  - Creates `/{scope}/restore/request` if absent (single-flight).
  - Also seeds `/{scope}/restore/status` (phase = `Requested`, heartbeat = now).
  - Returns `202 Accepted` with `{ accepted: true, restore_id }`.
  - If restore request already exists: return **`409 Conflict`** and include existing `{ restore_id, phase, executor_member_id, heartbeat_at_ms }` so operator action is obvious.
  - Input should be strict and stable:
    - `requested_by: MemberId` (required, non-empty; mirrors switchover)
    - `executor_member_id: MemberId` (required; the only node that may execute)
    - `reason: Option<String>` (optional, useful for audit/runbooks)
    - `idempotency_token: Option<String>` (optional; if present and matches the existing request, return 202 with the same restore_id instead of 409)
    - **No restore parameter overrides** in v1: use existing `backup.pgbackrest.*` config for restore specs
- `GET /ha/restore` (read-only)
  - Returns `{ request, status, derived }` where `derived` includes:
    - `is_executor`, `heartbeat_stale`, `local_ha_phase`, `process.active_job_kind/id`, `snapshot_sequence`.
  - If no restore request exists: return a stable “idle” payload (not 404) so operators can poll.
- `DELETE /ha/restore` (admin-only; mirrors `DELETE /ha/switchover`)
  - Deletes both `/{scope}/restore/request` and `/{scope}/restore/status` (idempotent; missing keys are ok).
  - v1 does **not** attempt to kill an in-flight process job; delete is a control-plane clear. Docs must warn: “clearing intent does not forcibly stop pgBackRest if it is already running on the executor”.

### DCS record schema (stored at `/{scope}/restore/*`)

Define two typed records (JSON-serialized) to avoid “intent clobbered by progress updates”.

**Request record** at `/{scope}/restore/request` (immutable once created):
- `restore_id: String` (unique per request; server-generated)
- `requested_by: MemberId`
- `requested_at_ms: UnixMillis`
- `executor_member_id: MemberId`
- `reason: Option<String>`
- `idempotency_token: Option<String>`

**Status record** at `/{scope}/restore/status` (executor-owned; mutable):
- `restore_id: String` (must match request)
- `phase: RestorePhase` (string/enum)
- `heartbeat_at_ms: UnixMillis` (updated by executor each HA tick while non-terminal)
- `running_job_id: Option<String>` (process job correlation)
- `last_error: Option<String>`
- `updated_at_ms: UnixMillis`

Suggested `RestorePhase` set (minimally sufficient for operators/tests):
- `Requested`
- `FencingPrimaries` (cluster quiesce)
- `Restoring` (pgBackRest restore job running)
- `TakeoverManagedConfig` (managed takeover step)
- `StartingPostgres`
- `WaitingPrimary` (waiting for PgInfo to report `Primary` + `Healthy`)
- `Completed`
- `Failed`
- `Cancelled`
- `Orphaned` (heartbeat stale; restore no longer blocks HA forever)

### HA orchestration model

Add restore orchestration as an overlay on the existing HA phase machine (do not fork the entire HA phase graph in v1). The restore-specific lifecycle is represented in DCS `restore/status.phase`, and HA uses it to gate/trigger actions.

High-level behavior:

1) **Global restore guard (applies on all nodes when restore request exists and status is non-terminal and not orphaned):**
- If DCS trust is not `FullQuorum`, keep existing fail-safe behavior (no promotions under uncertainty).
- If restore request exists:
  - If `restore/status.phase` is terminal (`Completed|Failed|Cancelled`) OR `Orphaned`: do not block normal HA (but surface status; operators must clear request).
  - Else (active restore):
    - If `self != executor_member_id`:
      - Suppress `AcquireLeaderLease` and `PromoteToPrimary`.
      - If currently primary + leader: emit `DemoteToReplica` + `ReleaseLeaderLease` (optionally `FenceNode` if Postgres must be forced down).
    - If `self == executor_member_id`:
      - Executor runs the restore sequence and updates `restore/status` (including heartbeat).

2) **Executor restore sequence (only on `executor_member_id`):**
- Ensure Postgres is stopped before touching the data directory:
  - if running as primary: `DemoteToReplica` + `ReleaseLeaderLease` (and then `FenceNode` if still running).
  - add explicit preflight: if `postmaster.pid` exists / pg process is running, do not run takeover; keep fencing until stopped.
- Phase machine (driven by observed world state + process state; each phase update is written to `restore/status`):
  - `Requested` → `FencingPrimaries` once executor begins
  - `FencingPrimaries` → `Restoring` once executor is fenced and no other leader exists
  - `Restoring`: enqueue `ProcessJobKind::PgBackRestRestore(...)` via HA dispatch into the process worker (not startup synchronous runner)
  - On restore job success: `TakeoverManagedConfig` (call `postgres_managed::takeover_restored_data_dir(...)` **only after Postgres is confirmed stopped**)
  - Then `StartingPostgres` (existing `StartPostgres`)
  - Then `WaitingPrimary` until `PgInfoState::Primary` and `SqlStatus::Healthy`
  - Then `Completed`
- On restore job failure or takeover/start failure: set `Failed` with `last_error` and keep node fenced (explicitly diagnosable).

3) **Cluster convergence (non-executor nodes):**
- After executor becomes leader/primary, other nodes should converge automatically.
- Make timeline-based resync **explicit (not optional)**:
  - If a node observes `self.timeline != leader.timeline` (both known) while following, it must not continue as a normal replica.
  - Trigger `StartRewind` first; on rewind failure, fall back to `pg_basebackup` (requires adding a basebackup HA action if not already present).
  - Add unit + e2e coverage for this rule so “cluster converges” is deterministic after restore.

### Debug + operator visibility

- Extend debug snapshot and verbose view to expose restore request/status + local execution hints:
  - restore request from DCS (`restore_id`, `executor_member_id`, `requested_by`, `requested_at_ms`)
  - restore status from DCS (`phase`, `heartbeat_at_ms`, `running_job_id`, `last_error`, `updated_at_ms`)
  - derived `heartbeat_stale` / `orphaned` flag
  - local HA phase and process active job kind/id for correlation
- Ensure debug timeline/change logs include restore transitions (at least on executor).

---

## Exhaustive file/module checklist (what to change and why)

### API layer

- [ ] `src/api/controller.rs`
  - [ ] Add DTOs:
    - [ ] `ClusterRestoreRequestInput` with `#[serde(deny_unknown_fields)]`
    - [ ] `ClusterRestoreStatusResponse` (stable JSON contract)
  - [ ] Add handlers:
    - [ ] `post_restore(scope, store, now, input)`:
      - [ ] validate `requested_by` non-empty
      - [ ] validate `executor_member_id` non-empty
      - [ ] generate `restore_id`
      - [ ] atomically create DCS restore request (put-if-absent at `/{scope}/restore/request`)
      - [ ] seed restore status (write `/{scope}/restore/status` with phase `Requested` + heartbeat)
      - [ ] map “already exists” to `ApiError::Conflict` (or accept as idempotent if `idempotency_token` matches)
    - [ ] `get_ha_restore(snapshot)`:
      - [ ] project restore request/status + local derived fields (process/ha + heartbeat stale)
    - [ ] `delete_ha_restore(scope, store)`:
      - [ ] clear `/{scope}/restore/request` and `/{scope}/restore/status` (idempotent)
  - [ ] Add unit tests mirroring existing switchover tests:
    - [ ] deny unknown fields
    - [ ] reject empty `requested_by`
    - [ ] reject empty `executor_member_id`
    - [ ] conflict mapping when restore exists

- [ ] `src/api/worker.rs`
  - [ ] Route wiring:
    - [ ] `POST /restore` → controller
    - [ ] `GET /ha/restore` → controller
    - [ ] `DELETE /ha/restore` → controller
  - [ ] Auth role classification:
    - [ ] POST/DELETE are `EndpointRole::Admin`
    - [ ] GET is `EndpointRole::Read` (or admin-only if we decide)
  - [ ] HTTP error mapping:
    - [ ] introduce `409 Conflict` support via `ApiError::Conflict` → HTTP 409
  - [ ] Worker tests:
    - [ ] auth matrix for restore routes (401/403/202)
    - [ ] invalid JSON → 400

- [ ] `src/api/mod.rs`
  - [ ] Extend API response structs if `/ha/state` should surface restore state (recommended).
  - [ ] Update node API interface types accordingly.

### DCS model/store

- [ ] `src/dcs/keys.rs`
  - [ ] Add key variant for restore record:
    - [ ] `/{scope}/restore/request`
    - [ ] `/{scope}/restore/status`
  - [ ] Extend parser + tests so this key is recognized (avoids “unknown key” trust degradation).

- [ ] `src/dcs/state.rs`
  - [ ] Add `RestoreRequestRecord`, `RestoreStatusRecord`, and `RestorePhase`
  - [ ] Extend `DcsCache` with:
    - [ ] `restore_request: Option<RestoreRequestRecord>`
    - [ ] `restore_status: Option<RestoreStatusRecord>`

- [ ] `src/dcs/store.rs`
  - [ ] Extend watch decode/apply:
    - [ ] add `DcsValue::RestoreRequest(RestoreRequestRecord)` decoding from JSON
    - [ ] add `DcsValue::RestoreStatus(RestoreStatusRecord)` decoding from JSON
    - [ ] on `Reset`, clear restore request/status (like switchover/init_lock)
    - [ ] on `Delete`, clear whichever restore key was deleted
  - [ ] Add writer helpers:
    - [ ] `put_restore_request_if_absent(scope, request)` (single-flight create)
    - [ ] `write_restore_status(scope, status)` (normal put; executor heartbeat/phase)
    - [ ] `clear_restore_request(scope)` (delete)
    - [ ] `clear_restore_status(scope)` (delete)
  - [ ] Add unit tests:
    - [ ] put/delete/reset behavior for restore request/status

- [ ] `src/dcs/worker.rs`
  - [ ] Ensure `apply_watch_update` handles restore value and reset semantics.
  - [ ] Ensure trust evaluation still works with restore record present.

- [ ] `src/dcs/etcd_store.rs`
  - [ ] Promote put-if-absent into the `DcsStore` trait so API/controller can use it generically.
  - [ ] Implement trait method by delegating to existing transaction-based `put_path_if_absent`.
  - [ ] Add integration tests (or extend existing ones) proving restore key survives reconnect snapshot/reset properly.

### HA orchestration

- [ ] `src/ha/state.rs`
  - [ ] v1: avoid adding new HA phases; use DCS `restore/status.phase` for restore lifecycle and keep HA phase graph stable.
  - [ ] Ensure `/ha/state` and debug views surface restore request/status so operators can correlate HA state with restore lifecycle.

- [ ] `src/ha/actions.rs`
  - [ ] Add restore-specific actions + action ids (include `restore_id` in IDs to avoid accidental dedupe collisions):
    - [ ] `WriteRestoreStatus { restore_id, phase, running_job_id, last_error }` (also bumps heartbeat)
    - [ ] `RunPgBackRestRestore { restore_id }`
    - [ ] `TakeoverRestoredDataDir { restore_id }`
  - [ ] Add a resync action for other nodes if needed:
    - [ ] `StartBaseBackup` (to satisfy rewind/basebackup convergence)

- [ ] `src/ha/decide.rs`
  - [ ] Add restore guard logic:
    - [ ] non-executor suppression of promotions/lease acquisition during restore
    - [ ] primary demotion + lease release during restore if not executor
  - [ ] Add executor restore phase machine:
    - [ ] phase transitions based on `ProcessState` outcomes
    - [ ] wait until `PgInfoState::Primary` and `SqlStatus::Healthy` before completing
  - [ ] Add resync rule (timeline mismatch → rewind/basebackup) after restore completion
  - [ ] Extend transition-matrix tests to cover:
    - [ ] non-executor does not acquire lease during restore
    - [ ] primary demotes/releases lease during restore
    - [ ] executor runs restore job when requested

- [ ] `src/ha/worker.rs`
  - [ ] Dispatch new actions:
    - [ ] `WriteRestoreStatus` → DCS write `/{scope}/restore/status` (phase, heartbeat_at_ms, updated_at_ms, running_job_id, last_error)
    - [ ] `RunPgBackRestRestore` → enqueue `ProcessJobKind::PgBackRestRestore(...)`
    - [ ] `TakeoverRestoredDataDir` → call `postgres_managed::takeover_restored_data_dir(...)` and surface errors as dispatch failures
    - [ ] `StartBaseBackup` → enqueue `ProcessJobKind::BaseBackup(...)`
  - [ ] NOTE: action retry semantics are currently imperfect due to global `recent_action_ids` dedupe. This is tracked in `.ralph/tasks/bugs/ha-action-deduping-suppresses-retry.md`. Restore actions must be written to be “single-shot but progress-driven” (i.e., the next action is different once state changes). If restore requires retries for correctness, fix dedupe in the bug task (or pull that fix into this task explicitly).
  - [ ] Add unit tests for new dispatch arms.

### Debug API

- [ ] `src/debug_api/snapshot.rs`
  - [ ] Extend snapshot model to include restore request/status (either in DCS section or a new dedicated section).
- [ ] `src/debug_api/view.rs`
  - [ ] Add restore fields to verbose payload and expose:
    - [ ] `restore.request.restore_id`, `restore.request.executor_member_id`
    - [ ] `restore.status.phase`, `restore.status.heartbeat_at_ms`, `restore.status.last_error`
  - [ ] Ensure change/timeline labels include restore transitions.

### Tests (must be real; no skipping)

- [ ] `tests/bdd_api_http.rs`
  - [ ] Add BDD tests for restore endpoints:
    - [ ] auth matrix (401/403/202)
    - [ ] bad request (invalid JSON, unknown fields)
    - [ ] conflict (second `POST /restore` returns 409, no extra DCS writes)
    - [ ] status payload checks (idle vs active restore)
- [ ] HA integration/unit tests
  - [ ] extend `src/ha/decide.rs` matrix tests (restore guard + executor progression)
  - [ ] extend `src/ha/worker.rs` dispatch tests (new actions)
- [ ] DCS watch/cache tests
  - [ ] extend `src/dcs/store.rs` watch refresh tests for restore request/status put/delete/reset
  - [ ] extend `src/dcs/etcd_store.rs` reconnect snapshot tests to include restore request/status keys
- [ ] Real e2e scenario (HA test suite)
  - [ ] Add one end-to-end restore takeover scenario in `src/ha/e2e_multi_node.rs`
  - [ ] Extend e2e harness config (`src/test_harness/ha_e2e/startup.rs` / config) to allow enabling `backup.enabled + backup.bootstrap.enabled` for that scenario only.
  - [ ] Ensure pgBackRest binary + fixtures are provisioned for CI (if missing, install; do not skip).

### Docs

- [ ] `docs/src/interfaces/node-api.md`
  - [ ] Document `POST /restore` and `GET/DELETE /ha/restore` with auth requirements and response shapes.
- [ ] `docs/src/operator/cluster-restore-takeover-runbook.md` (new)
  - [ ] Step-by-step operator runbook for restore takeover and rollback.
  - [ ] Explicitly call out single-flight, orphan detection (heartbeat stale), and safety gates.
- [ ] `docs/src/operator/index.md` + `docs/src/SUMMARY.md`
  - [ ] Link new runbook.

### Validation gates (must be green before marking passing)

- [ ] `make check`
- [ ] `make test`
- [ ] `make test-long`
- [ ] `make lint`

NOW EXECUTE
