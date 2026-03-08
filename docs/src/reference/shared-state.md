# Shared State Reference

The `pgtuskmaster_rust::state` module provides the shared identifier, versioning, worker-status, and watch-channel primitives that runtime workers publish and consume across the system.

## Module Surface

| Path | Contents |
|---|---|
| `src/state/mod.rs` | module definition and public re-exports |
| `src/state/errors.rs` | worker and channel error enums |
| `src/state/ids.rs` | identifier, WAL, and timeline newtypes |
| `src/state/time.rs` | timestamp, version, snapshot, and worker-status types |
| `src/state/watch_state.rs` | state-channel constructor, publisher, and subscriber |

`src/state/mod.rs` re-exports:

| Re-export | Kind |
|---|---|
| `new_state_channel` | constructor function |
| `StatePublisher` | publisher handle |
| `StateSubscriber` | subscriber handle |
| `StatePublishError` | publish error enum |
| `StateRecvError` | receive error enum |
| `WorkerError` | worker error enum |
| `WorkerStatus` | worker status enum |
| `Versioned` | snapshot wrapper struct |
| `ClusterName` | `String` newtype |
| `JobId` | `String` newtype |
| `MemberId` | `String` newtype |
| `SwitchoverRequestId` | `String` newtype |
| `TimelineId` | `u32` newtype |
| `WalLsn` | `u64` newtype |
| `UnixMillis` | `u64` newtype |
| `Version` | `u64` newtype |

## Wrapper Types

The identifier and scalar wrappers are tuple structs.

| Type | Inner type | Derived traits |
|---|---|---|
| `MemberId` | `String` | `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize` |
| `ClusterName` | `String` | `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize` |
| `SwitchoverRequestId` | `String` | `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize` |
| `JobId` | `String` | `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize` |
| `WalLsn` | `u64` | `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize` |
| `TimelineId` | `u32` | `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize` |
| `UnixMillis` | `u64` | `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize` |
| `Version` | `u64` | `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize` |

## Snapshot And Worker Types

### `Versioned<T>`

| Field | Type |
|---|---|
| `version` | `Version` |
| `updated_at` | `UnixMillis` |
| `value` | `T` |

Constructor: `Versioned::new(version, updated_at, value)`

`Versioned<T>` derives `Clone`, `Debug`, `PartialEq`, and `Eq`.

### `WorkerStatus`

| Variant | Payload |
|---|---|
| `Starting` | none |
| `Running` | none |
| `Stopping` | none |
| `Stopped` | none |
| `Faulted` | `WorkerError` |

`WorkerStatus` derives `Clone`, `Debug`, `PartialEq`, and `Eq`.

## Error Types

### `WorkerError`

| Variant | Payload |
|---|---|
| `Message` | `String` |

`impl From<crate::test_harness::HarnessError> for WorkerError` maps to `WorkerError::Message(format!("test harness error: {value}"))`.

### `StatePublishError`

| Variant | Meaning |
|---|---|
| `ChannelClosed` | the watch channel is closed |
| `VersionOverflow` | the next version cannot be represented |

### `StateRecvError`

| Variant | Meaning |
|---|---|
| `ChannelClosed` | the watch channel is closed |

## Watch Channel API

`new_state_channel(initial, now)` requires `T: Clone`, seeds the channel with `Versioned::new(Version(0), now, initial)`, and returns `(StatePublisher<T>, StateSubscriber<T>)`.

### `StatePublisher<T>`

`StatePublisher<T>` wraps `tokio::sync::watch::Sender<Versioned<T>>` and implements `Clone`.

| Method | Behavior |
|---|---|
| `publish(next, now)` | reads the current version, increments it by exactly `1` with checked addition, returns `StatePublishError::VersionOverflow` on overflow, sends the updated snapshot, maps send failure to `StatePublishError::ChannelClosed`, and returns the new `Version` |
| `latest()` | clones the latest sender-visible `Versioned<T>` snapshot |

### `StateSubscriber<T>`

`StateSubscriber<T>` wraps `tokio::sync::watch::Receiver<Versioned<T>>` and implements `Clone`.

| Method | Behavior |
|---|---|
| `latest()` | clones the latest receiver-visible `Versioned<T>` snapshot |
| `changed().await` | waits for a watch change, maps closure to `StateRecvError::ChannelClosed`, and returns the latest snapshot |

## Verified Behaviors

Unit tests in `src/state/watch_state.rs` verify:

- the initial snapshot uses `Version(0)` and the supplied timestamp
- `publish` increments version and updates `updated_at`
- `changed()` returns the latest snapshot after a publish
- `changed()` returns `StateRecvError::ChannelClosed` after the publisher drops
- publisher-side and subscriber-side `latest()` snapshots match

`tests/bdd_state_watch.rs` verifies the public flow from the initial `Version(0)` snapshot at `UnixMillis(1)` through one publish to `Version(1)` at `UnixMillis(2)`, then closure handling after the publisher drops.
