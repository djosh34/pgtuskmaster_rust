---
## Task: Implement API and Debug API workers with typed contracts <status>done</status> <passes>true</passes> <priority>high</priority>

<blocked_by>05-task-dcs-worker-trust-cache-watch-member-publish,08-task-ha-worker-select-loop-and-action-dispatch</blocked_by>

<description>
**Goal:** Implement typed API endpoints and debug snapshot visibility without bypassing system ownership rules.

**Scope:**
- Implement `src/api/controller.rs`, `src/api/fallback.rs`, `src/api/worker.rs`, `src/api/mod.rs`.
- Implement `src/debug_api/snapshot.rs`, `src/debug_api/worker.rs`, `src/debug_api/mod.rs`.
- Add `post_switchover`, `get_fallback_cluster`, `post_fallback_heartbeat`, `build_snapshot`, and worker loops.

**Context from research:**
- API must write switchover requests through DCS adapter, not direct HA mutation.

**Expected outcome:**
- Controller/fallback endpoints and debug snapshots are typed, tested, and aligned to worker ownership boundaries.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] API request/response models are typed and validated.
- [x] Debug snapshot includes app/config/pg/dcs/process/ha versioned states.
- [x] Integration tests verify API requests influence HA only via DCS state changes.
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make lint`.
- [x] Run `make test`.
- [x] If failing, create `$add-bug` tasks with endpoint payload and response evidence.
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan (Draft)

### 0) Preconditions + guardrails (do before coding)
- [x] Confirm blocker tasks are already in `done` + `<passes>true</passes>`:
  - [x] `05-task-dcs-worker-trust-cache-watch-member-publish`
  - [x] `08-task-ha-worker-select-loop-and-action-dispatch`
- [x] Capture a baseline `make check` before edits to isolate regressions from this task.
- [x] Keep runtime code free of `unwrap`/`expect`/`panic`/`todo!`/`unimplemented!` (crate root denies these outside `cfg(test)`).
- [x] Prefer small, typed helper functions that are unit-testable without spinning servers.

### 1) Decide the “API worker” boundary (HTTP transport vs typed business logic)
- [x] Keep typed endpoint logic in `src/api/controller.rs` and `src/api/fallback.rs` as pure functions that:
  - [x] validate inputs
  - [x] perform only allowed side effects (DCS writes for switchover)
  - [x] return typed responses
- [x] Keep HTTP transport + routing in `src/api/worker.rs` only.
- [x] Enforce ownership rule: API must not mutate HA state directly; switchover must be expressed as a typed write to DCS.

### 2) Add serde-typed request/response models (API contract)
- [x] Update `src/api/controller.rs`:
  - [x] Add `serde::{Serialize, Deserialize}` derives to `SwitchoverRequestInput` and `AcceptedResponse`.
  - [x] Use typed IDs where possible:
    - [x] Prefer `requested_by: MemberId` (serde newtype already exists), not raw `String`.
  - [x] Add `#[serde(deny_unknown_fields)]` to reject payload typos.
  - [x] Add explicit validation helpers (e.g., “requested_by must not be empty”) returning an `ApiError`.
- [x] Update `src/api/fallback.rs` similarly:
  - [x] `FallbackClusterView` derives `Serialize` (and `Deserialize` only if needed for tests).
  - [x] `FallbackHeartbeatInput` derives `Deserialize` + `deny_unknown_fields`, and validates `source` is non-empty.

### 3) Implement typed endpoint behavior (no HTTP yet)
- [x] Implement `post_switchover` (in `src/api/controller.rs`):
  - [x] Build a typed `crate::dcs::state::SwitchoverRequest { requested_by: MemberId }`.
  - [x] Encode to JSON with `serde_json::to_string`.
  - [x] Write to the *single* allowed path: `/{scope}/switchover` via `crate::dcs::store::DcsStore::write_path`.
  - [x] Return `AcceptedResponse { accepted: true }` on success.
  - [x] Map store/serde failures into a typed `ApiError` without panicking.
- [x] Implement fallback endpoints (in `src/api/fallback.rs`):
  - [x] `get_fallback_cluster` returns the configured cluster name (from `RuntimeConfig.cluster.name`) as `FallbackClusterView`.
  - [x] `post_fallback_heartbeat` validates input and returns `AcceptedResponse { accepted: true }` (no DCS writes unless/until a typed DCS key exists for it).

### 4) Implement a minimal HTTP transport in `src/api/worker.rs` (so tests can hit real endpoints)
Rationale: keep dependencies light and preserve a meaningful `step_once()` by serving at most one connection/request per call.

- [x] Add a small HTTP parser dependency (`httparse`) in `Cargo.toml`.
  - [x] If `tokio::io::{AsyncReadExt, AsyncWriteExt}` is used and `io-util` isn’t already enabled, add `io-util` to tokio features.
- [x] Define small internal structs in `src/api/worker.rs`:
  - [x] `HttpRequest { method, path, headers, body }`
  - [x] `HttpResponse { status, content_type, body }`
- [x] Implement request parsing:
  - [x] Read from `TcpStream` up to a hard size limit (e.g., 1 MiB).
  - [x] Parse request line + headers with `httparse`.
  - [x] For `POST` endpoints requiring JSON: require `Content-Length` and read exact body bytes.
  - [x] Return `400` on malformed HTTP or invalid JSON.
- [x] Implement routing:
  - [x] `POST /switchover` -> `post_switchover`
  - [x] `GET /fallback/cluster` -> `get_fallback_cluster`
  - [x] `POST /fallback/heartbeat` -> `post_fallback_heartbeat`
  - [x] `GET /debug/snapshot` -> serve latest debug snapshot (see section 6), gated by `cfg.debug.enabled`
  - [x] Unknown -> `404`
- [x] Implement auth gate (from `RuntimeConfig.security.auth_token`):
  - [x] If `auth_token` is `Some`, require `Authorization: Bearer <token>` for *all* endpoints (including debug).
  - [x] Return `401` on missing/invalid token.
- [x] Update `ApiWorkerCtx` to carry only what transport/handlers need:
  - [x] `listener: tokio::net::TcpListener`
  - [x] `poll_interval: Duration` (avoid busy-loop when there are no connections)
  - [x] `scope: String`
  - [x] `config_subscriber: StateSubscriber<RuntimeConfig>`
  - [x] `dcs_store: Box<dyn DcsStore>` (no `Mutex` needed if `step_once` is single-request, single-threaded)
  - [x] optional `debug_snapshot_subscriber: Option<StateSubscriber<SystemSnapshot>>`
- [x] Implement `step_once(&mut ApiWorkerCtx)` to:
  - [x] **must not block when no client connects**:
    - [x] use a tiny `tokio::time::timeout(...)` around `listener.accept()` and return `Ok(())` on timeout
  - [x] parse one request
  - [x] call the typed handler functions
  - [x] write response
  - [x] close connection
- [x] Implement `run(ApiWorkerCtx)` as an infinite loop calling `step_once()` and then sleeping `poll_interval` (keeps CPU low when idle).

### 5) Implement Debug Snapshot worker (typed visibility without ownership bypass)
- [x] Keep `src/debug_api/snapshot.rs` as the single source of truth for `SystemSnapshot` and `build_snapshot`.
- [x] Update `src/debug_api/worker.rs` so the worker owns a versioned snapshot channel:
  - [x] Add `DebugApiCtx` fields:
    - [x] `app: AppLifecycle`
    - [x] `publisher: StatePublisher<SystemSnapshot>`
    - [x] `config_subscriber: StateSubscriber<RuntimeConfig>`
    - [x] `pg_subscriber: StateSubscriber<PgInfoState>`
    - [x] `dcs_subscriber: StateSubscriber<DcsState>`
    - [x] `process_subscriber: StateSubscriber<ProcessState>`
    - [x] `ha_subscriber: StateSubscriber<HaState>`
    - [x] `poll_interval: Duration`
    - [x] `now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>`
  - [x] Implement `step_once(&mut DebugApiCtx)`:
    - [x] Read `latest()` from each subscriber.
    - [x] Build `DebugSnapshotCtx` and call `build_snapshot`.
    - [x] Publish `SystemSnapshot` via `publisher.publish(snapshot, now?)`.
  - [x] Implement `run(DebugApiCtx)` as a timed loop calling `step_once()` and sleeping `poll_interval`.
- [x] Ensure this worker only *reads* other workers’ states and publishes its own snapshot state (no DCS/HA/process mutations).

### 6) Wire debug snapshot visibility into API transport (optional but recommended)
- [x] If `RuntimeConfig.debug.enabled` is true and API worker has a `debug_snapshot_subscriber`:
  - [x] `GET /debug/snapshot` returns a stable, easy-to-consume payload.
  - [x] Prefer returning `text/plain` containing `format!("{snapshot:#?}")` to avoid adding `Serialize` derives across many state types in this task.
  - [x] Return `503` if debug is enabled but snapshot channel is unavailable.
- [x] If `debug.enabled` is false, return `404` (no accidental exposure).

### 7) Tests (unit + integration) with strong evidence of ownership boundaries
- [x] Update `src/worker_contract_tests.rs`:
  - [x] Replace `ApiWorkerCtx` and `DebugApiCtx` direct construction with `contract_stub(...)` constructors if needed (similar to `HaWorkerCtx::contract_stub`).
  - [x] Keep the existing “step_once callable” test intact but updated for new ctx fields.
  - [x] While touching this file, convert `expect(...)` calls into `Result`-returning tests (`-> Result<(), WorkerError>`) so we don’t add more `expect`/`unwrap` debt.
- [x] Add focused unit tests under `src/api/`:
  - [x] `post_switchover` writes exactly one DCS write to `/{scope}/switchover` and the JSON decodes as `SwitchoverRequest`.
  - [x] Fallback endpoints reject empty fields and accept valid ones.
- [x] Add an integration test under `tests/` (BDD-style) that hits the real TCP listener:
  - [x] Spawn API worker on `127.0.0.1:0` (ephemeral port) and send raw HTTP requests.
  - [x] Assert HTTP status + response body.
  - [x] Assert DCS writes happened (and that *no other* DCS keys were written).
  - [x] If auth_token is set in config, assert missing token yields `401`.
- [x] Add a debug snapshot worker test:
  - [x] Start snapshot worker ctx with known state channel versions.
  - [x] Run `step_once` and assert subscriber sees a `SystemSnapshot` with expected versions and cloned values.
- [x] Update `Makefile` `test` target to include the new BDD test binary (e.g., `cargo test --test bdd_state_watch --test bdd_api_http`) so CI runs it.

### 8) Verification gates (must be green before marking done)
- [x] Run `make check`
- [x] Run `make test`
- [x] Run `make test` (note: required a `cargo clean` once due to a stale/corrupted `target/` archive error)
- [x] Run `make lint`
- [x] If any fail, create `$add-bug` tasks with (N/A: all gates green):
  - [x] exact repro command
  - [x] failing request payload + response (for API failures)
  - [x] relevant worker snapshot/state evidence

### 9) Task closeout (only after all gates pass)
- [ ] Tick all acceptance criteria checkboxes.
- [ ] Set `<passes>true</passes>`.
- [ ] Run `/bin/bash .ralph/task_switch.sh`.
- [ ] Commit everything (including `.ralph`) with message:
  - [ ] `task finished 09-task-api-debug-workers-and-snapshot-contracts: <summary + evidence + challenges>`
- [ ] Append any learnings/surprises to `AGENTS.md`.
- [ ] Append diary entry to the progress log.
</execution_plan>

NOW EXECUTE
