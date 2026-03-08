# Shared State Reference

The `pgtuskmaster_rust::state` module provides shared identifier primitives, versioning infrastructure, worker-status reporting, and watch-channel communication.

## Overview

The module organizes its public API into four submodules and re-exports their items at the module root. The surface consists of identifier newtypes, versioning primitives, a watch-channel constructor with publisher and subscriber handles, and error enums for worker and channel operations.

## Module surface

| Path | Contents |
|---|---|
| `src/state/mod.rs` | Module definition and public re-exports |
| `src/state/errors.rs` | Worker and channel error enums |
| `src/state/ids.rs` | Identifier, WAL, and timeline newtypes |
| `src/state/time.rs` | Timestamp, version, snapshot, and worker-status types |
| `src/state/watch_state.rs` | State-channel constructor, publisher, and subscriber |

Re-exports from `src/state/mod.rs`:

| Re-export | Kind |
|---|---|
| `new_state_channel` | Constructor function |
| `StatePublisher` | Publisher handle |
| `StateSubscriber` | Subscriber handle |
| `StatePublishError` | Publish error enum |
| `StateRecvError` | Receive error enum |
| `WorkerError` | Worker error enum |
| `WorkerStatus` | Worker status enum |
| `Versioned` | Snapshot wrapper struct |
| `ClusterName` | `String` newtype |
| `JobId` | `String` newtype |
| `MemberId` | `String` newtype |
| `SwitchoverRequestId` | `String` newtype |
| `TimelineId` | `u32` newtype |
| `WalLsn` | `u64` newtype |
| `UnixMillis` | `u64` newtype |
| `Version` | `u64` newtype |

## Identifier and scalar wrapper types

Tuple structs with derived traits:

| Type | Inner | Derived traits |
|---|---|---|
| `MemberId` | `String` | `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize` |
| `ClusterName` | `String` | `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize` |
| `SwitchoverRequestId` | `String` | `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize` |
| `JobId` | `String` | `Clone`, `Debug`, `PartialEq`, `Eq`, `Hash`, `Serialize`, `Deserialize` |
| `WalLsn` | `u64` | `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize` |
| `TimelineId` | `u32` | `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize` |
| `UnixMillis` | `u64` | `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize` |
| `Version` | `u64` | `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, `Deserialize` |

## Versioned snapshot and worker status types

### `Versioned<T>`

Fields:
- `version: Version`
- `updated_at: UnixMillis`
- `value: T`

| Constructor | Signature |
|---|---|
| `new` | `Versioned::new(version, updated_at, value)` |

Derived: `Clone`, `Debug`, `PartialEq`, `Eq`

### `WorkerStatus`

Variants:
- `Starting`
- `Running`
- `Stopping`
- `Stopped`
- `Faulted(WorkerError)`

Derived: `Clone`, `Debug`, `PartialEq`, `Eq`

## Error enums

### `WorkerError`

Variants:
- `Message(String)`

`impl From<crate::test_harness::HarnessError> for WorkerError` produces `WorkerError::Message(format!("test harness error: {value}"))`.

### `StatePublishError`

Variants:
- `ChannelClosed`
- `VersionOverflow`

### `StateRecvError`

Variants:
- `ChannelClosed`

## Watch-channel constructor and handles

### `new_state_channel`

`new_state_channel<T: Clone>(initial, now) -> (StatePublisher<T>, StateSubscriber<T>)` creates a watch channel seeded with `Versioned::new(Version(0), now, initial)`.

### `StatePublisher<T>`

Wraps `tokio::sync::watch::Sender<Versioned<T>>`. Derived: `Clone`.

Methods:
- `publish(next, now) -> Result<Version, StatePublishError>`: Publishes a new state and returns the assigned `Version`
- `latest() -> Versioned<T>`: Returns a clone of the sender-visible snapshot

### `StateSubscriber<T>`

Wraps `tokio::sync::watch::Receiver<Versioned<T>>`. Derived: `Clone`.

Methods:
- `latest() -> Versioned<T>`: Returns a clone of the receiver-visible snapshot
- `changed() -> impl Future<Output = Result<Versioned<T>, StateRecvError>> + '_`: Waits for a change and returns the latest snapshot

## Verified behaviors from tests when directly supported

`src/state/watch_state.rs` tests verify:

- The initial snapshot uses `Version(0)` and the supplied timestamp
- Version increments by exactly `1` with each publish
- Publish updates `updated_at` to the supplied timestamp
- `changed()` returns the latest snapshot after a publish
- `changed()` returns `StateRecvError::ChannelClosed` after the publisher drops
- Publisher and subscriber `latest()` snapshots match

`tests/bdd_state_watch.rs` verifies:

- Channel flow from initial `Version(0)` at `UnixMillis(1)` through one publish to `Version(1)` at `UnixMillis(2)`
- Closure handling after publisher drop
