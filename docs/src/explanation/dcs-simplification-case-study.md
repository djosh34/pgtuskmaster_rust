# DCS Simplification Case Study: From Worker Loop to On-Demand Fetch

## Executive Summary

The current DCS module uses a dedicated worker loop (1,988 lines across 6 files) that maintains a persistent etcd connection, an in-memory cache fed by an etcd watch stream, and a command inbox driven by an async `tokio::select!` loop. This report investigates whether the worker-loop architecture is necessary, or whether DCS could be replaced by an **on-demand fetch** model — where consumers call `give_me_latest_dcs_state()` and mutations go through a thin handle that writes directly to etcd.

**Verdict**: The worker loop is not architecturally necessary. An on-demand model is feasible, would eliminate the watcher, the cache, the reconnect state machine, and the command channel — collapsing roughly **~750 lines** of infrastructure code. The trade-off is slightly higher per-call latency (one etcd round-trip per consumer poll) and loss of instant change notification. For the current consumer patterns — HA polls every `poll_interval`, API handles requests on demand — this is acceptable.

---

## 1. Current Architecture

### 1.1 Module Layout

| File | Lines | Responsibility |
|------|------:|----------------|
| `worker.rs` | 877 | Event loop, etcd watch, reconnect logic, command dispatch |
| `state.rs` | 779 | DcsView/ClusterView types, DcsCache, cache→view conversion |
| `command.rs` | 75 | DcsHandle + DcsCommand mpsc channel |
| `log_event.rs` | 152 | Structured logging for DCS events |
| `startup.rs` | 94 | Bootstrap wiring |
| `mod.rs` | 11 | Re-exports |
| **Total** | **1,988** | |

### 1.2 Data Flow

```
┌─────────────────────────────────────────────────────────┐
│                   DCS Worker Loop                       │
│                                                         │
│  ┌──────────┐    ┌──────────┐    ┌──────────────────┐  │
│  │  etcd     │───>│ DcsCache │───>│ build_dcs_view() │  │
│  │  watcher  │    │ (local)  │    │   ↓               │  │
│  └──────────┘    └──────────┘    │ StatePublisher    │  │
│                                   └────────┬─────────┘  │
│  ┌──────────────┐                          │            │
│  │ DcsCommand   │  (mpsc inbox)            │            │
│  │  Inbox       │──> etcd put/delete       │            │
│  └──────────────┘                          │            │
└────────────────────────────────────────────┼────────────┘
                                             │
              tokio::sync::watch broadcast   │
          ┌──────────────┬───────────────────┤
          ▼              ▼                   ▼
    HA Worker        API Worker       Process Worker
    .changed()       .latest()        .latest()
    .latest()
```

### 1.3 The Worker Loop in Detail

The worker in `worker.rs:101-257` is a `loop {}` with two modes:

**Connected mode** (`tokio::select!` on 5 branches):
1. `tick.tick()` → re-publish local member to etcd
2. `pg.changed()` → re-publish local member (pg state changed)
3. `command_inbox.recv()` → execute leadership/switchover commands on etcd
4. `watch_stream.message()` → apply remote changes to local cache
5. `keepalive deadline` → send etcd lease keepalive

**Disconnected mode** (`tokio::select!` on 4 branches):
1. `sleep_until(reconnect_at)` → attempt reconnection
2. `tick.tick()` → no-op
3. `pg.changed()` → no-op
4. `command_inbox.recv()` → drop command silently

On every successful step, `publish_current_view()` builds a fresh `DcsView` from the cache and pushes it through `StatePublisher<DcsView>`.

### 1.4 The Watch/Cache System

The cache (`DcsCache`) mirrors exactly three key types from etcd:

```
/{scope}/member/{member_id}  →  MemberRecord
/{scope}/leader              →  LeadershipRecord
/{scope}/switchover          →  SwitchoverRecord
```

The cache is populated on connect via `load_snapshot()` (full prefix GET) and then kept in sync via `apply_watch_response()` which processes etcd watch events.

### 1.5 How Consumers Actually Use DcsView

**HA worker** (`ha/worker.rs`):
```rust
// Waits for change notification, then grabs latest
tokio::select! {
    changed = ctx.observed.dcs.changed() => { /* ... */ }
    // ...other branches...
}
// In observe():
let dcs = ctx.observed.dcs.latest();
```

**API worker** (`api/worker.rs`, `api/controller.rs`):
```rust
// Grabs latest snapshot per HTTP request
let dcs = ctx.observed.dcs.latest();
```

**Process worker** (`process/worker.rs`):
```rust
// Grabs latest when materializing intents
let dcs = ctx.observed.dcs.latest();
```

Key observation: **every consumer only ever calls `.latest()`**. The HA worker also uses `.changed()` but only to wake up from sleep — it immediately calls `.latest()` afterward.

---

## 2. The On-Demand Alternative

### 2.1 Core Idea

Replace the worker loop + cache + watcher with:

1. **`DcsState`**: A single typed struct representing the full cluster state
2. **`dcs_fetch_latest()`**: Fetches current state directly from etcd (one GET with prefix)
3. **`DcsHandle`**: Keeps the same interface but executes etcd writes inline (no command channel needed)

```
┌───────────────────────────────────────┐
│            Shared EtcdPool            │
│  (connection + reconnect managed by   │
│   etcd_client internally)             │
└──────────┬────────────┬───────────────┘
           │            │
     ┌─────┴──┐   ┌────┴──────┐
     │ fetch  │   │ DcsHandle │
     │ latest │   │ .acquire  │
     │        │   │ .release  │
     └────┬───┘   │ .publish  │
          │       │ .clear    │
          │       └───────────┘
          │
    HA loop calls it once per tick
    API calls it once per request
    Process calls it once per poll
```

### 2.2 The New DcsState Struct

Instead of the current split between `DcsCache` (internal) → `DcsView` (public), there would be one struct:

```rust
/// The complete DCS cluster state, fetched on demand from etcd.
/// This is the only type consumers interact with.
pub struct DcsState {
    pub mode: DcsMode,
    pub members: BTreeMap<MemberId, ClusterMemberView>,
    pub leadership: LeadershipObservation,
    pub switchover: SwitchoverView,
}

pub enum DcsMode {
    NotTrusted,  // etcd unreachable
    Degraded,    // reachable but quorum/self-presence insufficient
    Coordinated, // full quorum
}
```

No `NotTrustedView` wrapper, no separate `ClusterView`, no `DcsCache`, no `MemberRecord` ↔ `MemberPostgresView` conversion layer.

### 2.3 The Fetch Function

```rust
pub struct DcsClient {
    client: etcd_client::Client,
    identity: DcsNodeIdentity,
    cadence: DcsCadence,
}

impl DcsClient {
    /// Fetch the latest full DCS state from etcd.
    /// Called by each consumer once per loop/request.
    pub async fn fetch_latest(&self) -> DcsState {
        let prefix = scope_prefix(&self.identity.scope);
        match timeout_etcd(
            "etcd get",
            self.client.get(prefix.as_str(), Some(GetOptions::new().with_prefix())),
        ).await {
            Ok(response) => {
                let cache = parse_response_into_cache(
                    &self.identity.scope,
                    &response,
                );
                let mode = evaluate_mode(true, &cache, &self.identity.self_id);
                build_dcs_state(mode, &cache)
            }
            Err(_) => DcsState {
                mode: DcsMode::NotTrusted,
                members: BTreeMap::new(),
                leadership: LeadershipObservation::Open,
                switchover: SwitchoverView::None,
            },
        }
    }
}

fn parse_response_into_cache(scope: &str, response: &GetResponse) -> ParsedSnapshot {
    // Same logic as current load_snapshot(), but returns owned data
    // without writing to a persistent cache
    let mut members = BTreeMap::new();
    let mut leader = None;
    let mut switchover = None;
    for kv in response.kvs() {
        // parse_key + deserialize, same as apply_key_value()
    }
    ParsedSnapshot { members, leader, switchover }
}
```

### 2.4 The Handle (Mutations)

The handle would hold a cloneable `Arc<DcsClient>` and call etcd directly:

```rust
#[derive(Clone)]
pub struct DcsHandle {
    client: Arc<tokio::sync::Mutex<etcd_client::Client>>,
    identity: DcsNodeIdentity,
    cadence: DcsCadence,
    leader_lease: Arc<tokio::sync::Mutex<Option<OwnedLeaderLease>>>,
}

impl DcsHandle {
    pub async fn acquire_leadership(&self) -> Result<(), DcsError> {
        // Same transactional CAS logic as current acquire_local_leadership()
        // but called inline by HA worker
    }

    pub async fn release_leadership(&self) -> Result<(), DcsError> {
        // Same as current release_local_leadership()
    }

    pub async fn publish_switchover(&self, target: SwitchoverTarget) -> Result<(), DcsError> {
        // Same as current publish_switchover() - direct etcd PUT
    }

    pub async fn clear_switchover(&self) -> Result<(), DcsError> {
        // Same as current clear_switchover() - direct etcd DELETE
    }

    pub async fn sync_local_member(
        &self,
        pg_state: &PgInfoState,
    ) -> Result<(), DcsError> {
        // Same as current sync_local_member() - PUT member record with lease
    }
}
```

### 2.5 Consumer Changes

**HA worker** — instead of subscribing:
```rust
// Before (current):
loop {
    tokio::select! {
        changed = ctx.observed.dcs.changed() => { }
        _ = interval.tick() => {}
    }
    let dcs = ctx.observed.dcs.latest();  // from cache
    // ...decide...
}

// After (proposed):
loop {
    interval.tick().await;
    let dcs = ctx.dcs_client.fetch_latest().await;  // from etcd
    ctx.dcs_handle.sync_local_member(&pg).await?;
    // ...decide...
}
```

**API worker** — per-request:
```rust
// Before:
let dcs = ctx.observed.dcs.latest();  // from watch-fed cache

// After:
let dcs = ctx.dcs_client.fetch_latest().await;  // from etcd
```

---

## 3. What Gets Eliminated

### 3.1 Code Removal Breakdown

| What | Lines | Why it's gone |
|------|------:|---------------|
| `worker.rs`: main event loop (connected + disconnected modes) | ~160 | No loop needed |
| `worker.rs`: `connect_session()`, watch setup | ~25 | No watcher |
| `worker.rs`: `apply_watch_response()`, `apply_key_value()`, `apply_delete()` | ~50 | No cache to update |
| `worker.rs`: reconnect backoff logic | ~40 | etcd_client handles reconnect |
| `worker.rs`: `handle_connected_failure()`, `handle_initial_connect_failure()` | ~25 | Errors returned inline |
| `worker.rs`: `handle_connected_command()`, `handle_disconnected_command()` | ~40 | Commands execute inline |
| `state.rs`: `DcsCache`, `DcsStateChannel`, `DcsControlPlane` | ~30 | No cache |
| `state.rs`: `DcsWorkerCtx`, `DcsRuntime` | ~15 | No worker context |
| `state.rs`: `NotTrustedView` wrapper | ~15 | Unified `DcsState` |
| `command.rs`: `DcsCommand` enum, mpsc channel | ~50 | Commands go direct |
| `startup.rs`: worker bootstrap wiring | ~40 | Simpler init |
| `log_event.rs`: `ConnectedStepFailed`, `InitialConnectFailed` | ~60 | Inline error handling |
| **Estimated removal** | **~550-750** | |

### 3.2 What Stays

| What | Lines | Why |
|------|------:|-----|
| `state.rs`: `DcsView`/`ClusterView`/`MemberPostgresView` types | ~280 | Core domain types (possibly merged/simplified) |
| `state.rs`: `evaluate_mode()`, `build_dcs_view()`, `build_member_view()` | ~80 | Still needed for mode evaluation |
| `state.rs`: `build_local_member_record()` | ~60 | Still needed for self-advertisement |
| `worker.rs`: etcd connection, TLS, auth setup | ~70 | Still need to connect to etcd |
| `worker.rs`: key path functions | ~30 | Still need key layout |
| `worker.rs`: leadership transaction logic | ~80 | CAS + lease keepalive stays |
| `worker.rs`: member sync + switchover PUT/DELETE | ~60 | Direct write operations stay |

### 3.3 Net Effect

- **Before**: 1,988 lines across 6 files
- **After (estimate)**: ~1,000-1,200 lines across 3-4 files
- **Removed**: ~750-950 lines (38-48% reduction)
- **Eliminated concepts**: watch stream, DcsCache, command channel/inbox, DcsWorkerCtx, reconnect state machine, connected/disconnected mode enum, publish_current_view loop

---

## 4. What About Leases?

> "Leased keys are still leased, but etcd can handle that for you right?"

**Yes, with a nuance.** Two types of leases exist:

### 4.1 Member Leases (TTL-based self-advertisement)

Current behavior: Each `sync_local_member()` call creates a short-lived etcd lease, PUTs the member record under that lease, and the record auto-expires when the lease TTL elapses (if not refreshed).

In the on-demand model: **This works identically**. The HA loop calls `sync_local_member()` every `poll_interval` (typically 2-5s). Each call creates a fresh lease and PUTs the member record. If the node dies, the lease expires, and etcd auto-deletes the member record. No watcher needed for this — it's fire-and-forget from the writer side.

### 4.2 Leader Lease (CAS + keepalive)

Current behavior: `acquire_local_leadership()` uses a transaction (CAS) to atomically create the leader key under a lease. A `LeaseKeeper` + `LeaseKeepAliveStream` runs inside the worker loop to periodically ping etcd and extend the lease.

In the on-demand model: **The keepalive needs a home**. Options:

**Option A: Piggyback on HA loop tick**
```rust
impl DcsHandle {
    pub async fn tick_keepalive(&self) -> Result<(), DcsError> {
        let mut lease = self.leader_lease.lock().await;
        if let Some(ref mut lease) = *lease {
            if Instant::now() >= lease.next_keepalive_at {
                lease.keeper.keep_alive().await?;
                let resp = lease.stream.message().await?;
                // update next_keepalive_at
            }
        }
        Ok(())
    }
}

// In HA loop:
loop {
    interval.tick().await;
    ctx.dcs_handle.tick_keepalive().await?;
    let dcs = ctx.dcs_client.fetch_latest().await;
    // ...
}
```

This works because the HA loop already ticks every `poll_interval` (2-5s), and leader keepalive needs to fire roughly every `ttl/3` seconds (typically 10s). The HA loop tick is frequent enough.

**Option B: Spawn a small keepalive task**

A lightweight `tokio::spawn` that only does lease keepalive. Much simpler than the full worker loop. This would be about 20 lines of code.

Both options are drastically simpler than the current full worker loop.

---

## 5. Trade-offs

### 5.1 Latency

| Scenario | Current | On-Demand |
|----------|---------|-----------|
| HA loop reads DCS | ~0μs (read from local `watch::Receiver`) | ~1-5ms (etcd GET with prefix) |
| API request reads DCS | ~0μs | ~1-5ms |
| Reacting to leader change | Instant via watch event | Up to `poll_interval` delay |

For HA with a 2-5 second poll interval, an extra 1-5ms per tick is negligible. The HA loop already sleeps for seconds between ticks.

For the API, 1-5ms extra latency per request is negligible for an operator-facing control surface.

### 5.2 Reactivity

The current watcher gives **instant** notification when another node's membership changes or a leader lease expires. In the on-demand model, the HA loop would only see the change on its next tick.

**Is this a problem?** In practice, no:
- The HA loop already makes decisions on a tick interval. Even with the watcher, it doesn't react mid-sleep — it waits for `select!` to yield, then processes.
- Failover detection already depends on lease TTL expiry (seconds), not sub-millisecond reactivity.
- The difference between "detect 0ms after change" and "detect up to `poll_interval` after change" is small relative to the already-configured TTL windows.

### 5.3 etcd Load

Current: 1 persistent watch connection + periodic PUTs.
On-demand: 1 GET-with-prefix per consumer per tick + periodic PUTs.

With 3 consumers polling at 2s intervals, that's ~1.5 GETs/second. etcd handles thousands of reads/second trivially. This is a non-issue.

### 5.4 Simplicity

| Dimension | Current | On-Demand |
|-----------|---------|-----------|
| State machine | 2 modes (connected/disconnected) | None |
| Async coordination | mpsc channel + watch channel | Direct async calls |
| Error recovery | Reconnect backoff loop | Return error, caller retries on next tick |
| Cache consistency | Watch stream + snapshot on reconnect | Always consistent (fresh read) |
| Code volume | 1,988 lines | ~1,000-1,200 lines |
| Testability | Need to mock watch stream, channels | Mock a single etcd GET response |

The on-demand model is **inherently consistent** — there is no cache that can go stale or miss a watch event. Every read is a fresh snapshot from etcd.

---

## 6. The Auto-Apply Pattern

> "When DCS changes a value in DcsState, it auto applies the changed keys on etcd. Is that possible?"

This is about making mutations feel like struct field assignments rather than explicit etcd API calls. In Rust, this is achievable with a builder/diff pattern:

```rust
/// Represents a desired mutation to DCS state.
/// Built by the caller, applied atomically.
pub struct DcsMutation {
    acquire_leadership: Option<()>,
    release_leadership: Option<()>,
    set_switchover: Option<SwitchoverTarget>,
    clear_switchover: bool,
    sync_member: Option<MemberRecord>,
}

impl DcsHandle {
    /// Apply all pending mutations in one batch.
    pub async fn apply(&self, mutation: DcsMutation) -> Result<(), DcsError> {
        if let Some(()) = mutation.acquire_leadership {
            self.acquire_leadership_inner().await?;
        }
        if let Some(()) = mutation.release_leadership {
            self.release_leadership_inner().await?;
        }
        if let Some(target) = mutation.set_switchover {
            self.publish_switchover_inner(target).await?;
        }
        if mutation.clear_switchover {
            self.clear_switchover_inner().await?;
        }
        if let Some(record) = mutation.sync_member {
            self.sync_member_inner(record).await?;
        }
        Ok(())
    }
}
```

A true "auto-apply on field change" (like a reactive proxy) is possible but adds complexity without clear benefit. The explicit mutation object is more Rust-idiomatic and makes the etcd round-trips visible at the call site.

However, the current `DcsHandle` with named methods (`acquire_leadership()`, `release_leadership()`, etc.) is already close to this — the only change is that methods become `async` and execute directly instead of sending through a channel.

---

## 7. What Doesn't Change

1. **DcsView types** — `ClusterView`, `ClusterMemberView`, `MemberPostgresView`, `LeadershipObservation`, `SwitchoverView` all stay. They're the domain model.
2. **Mode evaluation** — `evaluate_mode()` logic stays. Trust gating is independent of how data is fetched.
3. **etcd key layout** — `/{scope}/member/{id}`, `/{scope}/leader`, `/{scope}/switchover` stay.
4. **Leadership CAS** — The transactional acquire + lease keepalive logic stays. It just lives in `DcsHandle` methods instead of the worker loop.
5. **Logging** — `DcsLogEvent::CoordinationModeTransition` stays. `ConnectedStepFailed` and `InitialConnectFailed` merge into a simpler `EtcdOperationFailed` event.

---

## 8. Migration Path

If this simplification is pursued:

1. **Make `DcsHandle` methods `async`** — they call etcd directly instead of sending commands through mpsc.
2. **Add `DcsClient::fetch_latest()`** — wraps a single etcd GET-with-prefix + parse + evaluate_mode.
3. **Remove the worker loop** — delete the `run()` function, the `ConnectedSession`, the watch setup, the reconnect state machine.
4. **Remove `DcsCache`** — no persistent cache needed. Parse responses into a temporary struct and build `DcsState` from it.
5. **Remove `StatePublisher<DcsView>`** — consumers call `fetch_latest()` directly. No broadcast channel.
6. **Remove `DcsCommand`/`DcsCommandInbox`** — no command channel. Handle methods execute inline.
7. **Move keepalive** — either piggyback on HA loop tick or spawn a lightweight task.
8. **Simplify startup** — `DcsRuntime` returns `(DcsClient, DcsHandle)` instead of `(StateSubscriber, DcsHandle, DcsWorker)`.

---

## 9. Conclusion

The DCS worker loop exists primarily to serve the watcher-fed cache pattern. Every consumer of DCS state ultimately just wants "give me the latest cluster state" — and the watcher is an optimization that turns an etcd GET into a local memory read. That optimization buys sub-millisecond read latency at the cost of 750+ lines of state machine, reconnect logic, cache synchronization, and dual-channel coordination.

For a system where the fastest consumer polls every 2 seconds and the domain already tolerates multi-second TTL windows, that optimization is not worth the complexity. An on-demand fetch model would be simpler, inherently consistent, easier to test, and roughly half the code.
