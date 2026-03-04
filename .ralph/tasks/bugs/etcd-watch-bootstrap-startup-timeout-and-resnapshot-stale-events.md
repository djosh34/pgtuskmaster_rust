---
## Bug: Etcd watch bootstrap can hang startup and resnapshot can replay stale events <status>done</status> <passes>true</passes>

<description>
The etcd DCS store watch worker has subtle correctness issues in bootstrap/reconnect handling.

Detected during code audit of `src/dcs/etcd_store.rs`:
- `EtcdDcsStore::connect` waits only `COMMAND_TIMEOUT` for worker startup and then `join()`s the worker thread on timeout. If bootstrap (`connect + get + watch`) takes longer than that timeout, the join can block indefinitely while the worker continues running.
- On watch reconnect/resnapshot, bootstrap snapshot events are appended to the existing queue without clearing/draining stale pre-disconnect events. This can replay stale PUT events that should have been superseded by deletes included in the snapshot state.

Please explore and research existing DCS/watch semantics in the codebase first, then fix implementation and tests.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

---

## Plan (Verified)

### 0) Ground Truth / Semantics (read-only)
- [x] Re-read `src/dcs/etcd_store.rs` end-to-end, focusing on:
  - [x] `EtcdDcsStore::connect_with_worker_bootstrap_timeout` timeout branch (startup wait + shutdown + join)
  - [x] worker bootstrap path: `run_worker_loop` → `establish_watch_session` → `connect_client` + `bootstrap_snapshot` + `create_watch_stream`
  - [x] reconnect signal: `had_successful_session` being passed as `is_reconnect`
- [x] Re-read consumer semantics in `src/dcs/store.rs`:
  - [x] `WatchOp::Reset` meaning
  - [x] `refresh_from_etcd_watch` reset behavior (what is cleared vs preserved)

### 1) Fix: `connect()` must not block on worker join after startup timeout

**Problem recap**
- `connect_with_worker_bootstrap_timeout` waits `worker_bootstrap_timeout` for `startup_rx`.
- On timeout, it currently sends `WorkerCommand::Shutdown`, drops `command_tx`, and then calls `worker_handle.join()`.
- If the worker is still in bootstrap (or hung in a non-yielding call), `join()` can exceed the caller’s timeout (or block indefinitely), turning a bounded startup timeout into an unbounded `connect()` call.

**Implementation approach (bounded caller latency; best-effort cleanup)**
- [x] In `src/dcs/etcd_store.rs`, in the `Err(recv_timeout_err)` branch:
  - [x] Send `WorkerCommand::Shutdown` (best-effort; ignore send failure).
  - [x] Drop `command_tx` to close the channel (so the worker can observe disconnect once it begins polling).
  - [x] **Do not join inline** on the main thread.
  - [x] Prefer detaching by dropping `worker_handle` in this error path instead of spawning a reaper thread.
    - [x] Rationale: a reaper can itself block forever if bootstrap is stuck; dropping `JoinHandle` cleanly detaches without creating another potentially stuck thread.
  - [x] Return `DcsStoreError::Io(...)` indicating startup timed out.
- [x] Distinguish `recv_timeout` errors:
  - [x] `Timeout` => startup timeout error message.
  - [x] `Disconnected` => worker exited before signaling startup (more accurate than timeout text).
- [x] Update inline comments to match detached cleanup behavior.

**Notes / constraints**
- No `unwrap()`, `expect()`, or `panic!()` anywhere in changes.
- Keep the existing per-operation timeout behavior (`timeout_etcd` using `COMMAND_TIMEOUT`) unchanged for now; this plan focuses on bounding caller latency independently.

### 2) Verify-first: resnapshot/reconnect stale replay semantics

**Current intended semantics**
- On reconnect/resnapshot: synthesize `WatchOp::Reset`, then enqueue snapshot `Put`s, and ensure any pre-reconnect queued events are dropped so deleted keys cannot be resurrected.

**Work**
- [x] Confirm reconnect path always uses `replace_watch_events` (clear+extend) and that the `Reset` marker is first in the queue.
- [x] If this is already true (current code suggests it is), **do not change logic**; keep scope focused on timeout/join bug.
- [x] If any path still uses append-on-reconnect, switch it to replace semantics. (N/A: already uses replace-on-reconnect)
- [x] Keep the event queue mutation under the existing mutex; do not introduce lock ordering or additional shared state unless needed.

### 3) Tests: make failures deterministic and cover the critical invariants

#### 3a) Strengthen the startup-timeout test to catch join-induced blocking
- [x] Update `etcd_store_connect_timeout_returns_and_does_not_hang` in `src/dcs/etcd_store.rs` to assert **latency is bounded near `worker_bootstrap_timeout`**, not merely “< 3s”.
  - [x] Use `EstablishDelayGuard` with a **much larger delay** (e.g. multiple seconds) so inline-join regressions are unambiguous.
  - [x] Measure elapsed wall time and assert with a **buffered bound** (e.g. `< worker_bootstrap_timeout + 1s`), avoiding fragile sub-500ms thresholds.
  - [x] Keep an outer timeout as a hard stop; set it tighter than establish delay so join regressions fail deterministically.

#### 3b) Add reconnect test for non-empty snapshot (authoritative rebuild)
- [x] Add a new test in `src/dcs/etcd_store.rs` that proves reconnect snapshot is authoritative when it contains data:
  - [x] Arrange: write a known leader record into real etcd.
  - [x] Force reconnect by restarting etcd (fixture helper restarts without deleting the data dir, so the snapshot is guaranteed non-empty).
  - [x] Assert: the drained queue contains a `Reset` marker, followed by snapshot `Put` events for the expected keys.
  - [x] Apply via `refresh_from_etcd_watch` and assert cache matches the snapshot (and does not preserve any pre-reconnect in-memory “stale” record).

#### 3c) Store-level ordering regression test (required)
- [x] In `src/dcs/store.rs` unit tests, add an ordering regression case:
  - [x] Given `[Put(stale), Reset, Put(fresh)]`, after `refresh_from_etcd_watch`, the cache must reflect only the post-reset state.
  - [x] This is a cheap, deterministic guardrail for the “no resurrection” rule.

### 4) Validation (must be 100% green)
- [x] `make check`
- [x] `make test`
- [x] `make test-long`
- [x] `make lint`

NOW EXECUTE
