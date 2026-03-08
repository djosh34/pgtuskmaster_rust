Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Revise an existing reference page so it stays strictly in Diataxis reference form.

[Page path]
- docs/src/reference/shared-state.md

[Page goal]
- Reference the shared identifier wrappers, versioned snapshot types, worker status and error enums, and watch-channel publisher/subscriber API.

[Audience]
- Operators and contributors who need accurate repo-backed facts while working with pgtuskmaster.

[User need]
- Consult the machinery surface, data model, constraints, constants, and behavior without being taught procedures or background explanations.

[mdBook context]
- This is an mdBook page under docs/src/reference/.
- Keep headings and lists suitable for mdBook.
- Do not add verification notes, scratch notes, or commentary about how the page was produced.

[Diataxis guidance]
- This page must stay in the reference quadrant: cognition plus application.
- Describe and only describe.
- Structure the page to mirror the machinery, not a guessed workflow.
- Use neutral, technical language.
- Examples are allowed only when they illustrate the surface concisely.
- Do not include step-by-step operations, recommendations, rationale, or explanations of why the design exists.
- If action or explanation seems necessary, keep the page neutral and mention the boundary without turning the page into a how-to or explanation article.

[Required structure]
- Overview\n- Module surface\n- Identifier and scalar wrapper types\n- Versioned snapshot and worker status types\n- Error enums\n- Watch-channel constructor and handles\n- Verified behaviors from tests when directly supported

[Output requirements]
- Preserve or improve the title so it matches the machinery.
- Prefer compact tables where they clarify enums, fields, constants, routes, or error variants.
- Include only facts supported by the supplied source excerpts.
- If the current page contains unsupported claims, remove or correct them rather than hedging.

[Existing page to revise]

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

[Repo facts and source excerpts]

--- BEGIN FILE: src/state/mod.rs ---
pub mod errors;
pub mod ids;
pub mod time;
pub mod watch_state;

pub use errors::{StatePublishError, StateRecvError, WorkerError};
pub use ids::{ClusterName, JobId, MemberId, SwitchoverRequestId, TimelineId, WalLsn};
pub use time::{UnixMillis, Version, Versioned, WorkerStatus};
pub use watch_state::{new_state_channel, StatePublisher, StateSubscriber};

--- END FILE: src/state/mod.rs ---

--- BEGIN FILE: src/state/errors.rs ---
use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum WorkerError {
    #[error("{0}")]
    Message(String),
}

impl From<crate::test_harness::HarnessError> for WorkerError {
    fn from(value: crate::test_harness::HarnessError) -> Self {
        Self::Message(format!("test harness error: {value}"))
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum StatePublishError {
    #[error("state channel is closed")]
    ChannelClosed,
    #[error("state version overflow")]
    VersionOverflow,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum StateRecvError {
    #[error("state channel is closed")]
    ChannelClosed,
}

--- END FILE: src/state/errors.rs ---

--- BEGIN FILE: src/state/ids.rs ---
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct MemberId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClusterName(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SwitchoverRequestId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub String);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WalLsn(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TimelineId(pub u32);

--- END FILE: src/state/ids.rs ---

--- BEGIN FILE: src/state/time.rs ---
use super::errors::WorkerError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UnixMillis(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version(pub u64);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Versioned<T> {
    pub version: Version,
    pub updated_at: UnixMillis,
    pub value: T,
}

impl<T> Versioned<T> {
    pub fn new(version: Version, updated_at: UnixMillis, value: T) -> Self {
        Self {
            version,
            updated_at,
            value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkerStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Faulted(WorkerError),
}

--- END FILE: src/state/time.rs ---

--- BEGIN FILE: src/state/watch_state.rs ---
use tokio::sync::watch;

use super::{
    errors::{StatePublishError, StateRecvError},
    time::{UnixMillis, Version, Versioned},
};

#[derive(Debug)]
pub struct StatePublisher<T: Clone> {
    tx: watch::Sender<Versioned<T>>,
}

impl<T: Clone> Clone for StatePublisher<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

#[derive(Debug)]
pub struct StateSubscriber<T: Clone> {
    rx: watch::Receiver<Versioned<T>>,
}

impl<T: Clone> Clone for StateSubscriber<T> {
    fn clone(&self) -> Self {
        Self {
            rx: self.rx.clone(),
        }
    }
}

pub fn new_state_channel<T: Clone>(
    initial: T,
    now: UnixMillis,
) -> (StatePublisher<T>, StateSubscriber<T>) {
    let initial_snapshot = Versioned::new(Version(0), now, initial);
    let (tx, rx) = watch::channel(initial_snapshot);
    (StatePublisher { tx }, StateSubscriber { rx })
}

impl<T: Clone> StatePublisher<T> {
    pub fn publish(&self, next: T, now: UnixMillis) -> Result<Version, StatePublishError> {
        let current = self.tx.borrow().version;
        // Checked increment preserves strict +1 semantics and reports overflow explicitly.
        let next_version = Version(
            current
                .0
                .checked_add(1)
                .ok_or(StatePublishError::VersionOverflow)?,
        );
        let updated = Versioned::new(next_version, now, next);
        self.tx
            .send(updated)
            .map_err(|_| StatePublishError::ChannelClosed)?;
        Ok(next_version)
    }

    pub fn latest(&self) -> Versioned<T> {
        self.tx.borrow().clone()
    }
}

impl<T: Clone> StateSubscriber<T> {
    pub fn latest(&self) -> Versioned<T> {
        self.rx.borrow().clone()
    }

    pub async fn changed(&mut self) -> Result<Versioned<T>, StateRecvError> {
        self.rx
            .changed()
            .await
            .map_err(|_| StateRecvError::ChannelClosed)?;
        Ok(self.latest())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "current_thread")]
    async fn initial_snapshot_has_expected_version_and_time() {
        let (_publisher, subscriber) = new_state_channel("booting".to_string(), UnixMillis(123));
        let latest = subscriber.latest();
        assert_eq!(latest.version, Version(0));
        assert_eq!(latest.updated_at, UnixMillis(123));
        assert_eq!(latest.value, "booting");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn publish_increments_version_and_updates_timestamp(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (publisher, subscriber) = new_state_channel("a".to_string(), UnixMillis(100));

        let v1 = publisher.publish("b".to_string(), UnixMillis(200))?;
        assert_eq!(v1, Version(1));
        let latest = subscriber.latest();
        assert_eq!(latest.version, Version(1));
        assert_eq!(latest.updated_at, UnixMillis(200));
        assert_eq!(latest.value, "b");

        let v2 = publisher.publish("c".to_string(), UnixMillis(300))?;
        assert_eq!(v2, Version(2));
        let latest = subscriber.latest();
        assert_eq!(latest.version, Version(2));
        assert_eq!(latest.updated_at, UnixMillis(300));
        assert_eq!(latest.value, "c");
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn changed_returns_latest_after_publish() -> Result<(), Box<dyn std::error::Error>> {
        let (publisher, mut subscriber) = new_state_channel("ready".to_string(), UnixMillis(10));
        publisher.publish("running".to_string(), UnixMillis(20))?;

        let changed = subscriber.changed().await?;
        assert_eq!(changed.version, Version(1));
        assert_eq!(changed.updated_at, UnixMillis(20));
        assert_eq!(changed.value, "running");
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn changed_propagates_closed_channel_error() {
        let (publisher, mut subscriber) = new_state_channel("ready".to_string(), UnixMillis(10));
        drop(publisher);

        let changed = subscriber.changed().await;
        assert_eq!(changed, Err(StateRecvError::ChannelClosed));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn latest_matches_between_publisher_and_subscriber(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (publisher, subscriber) = new_state_channel("ready".to_string(), UnixMillis(10));
        publisher.publish("running".to_string(), UnixMillis(20))?;

        assert_eq!(publisher.latest(), subscriber.latest());
        Ok(())
    }
}

--- END FILE: src/state/watch_state.rs ---

--- BEGIN FILE: tests/bdd_state_watch.rs ---
use pgtuskmaster_rust::state::{new_state_channel, StateRecvError, UnixMillis, Version};

#[tokio::test(flavor = "current_thread")]
async fn bdd_state_watch_channel_flow() -> Result<(), Box<dyn std::error::Error>> {
    let (publisher, mut subscriber) = new_state_channel("starting".to_string(), UnixMillis(1));

    let initial = subscriber.latest();
    assert_eq!(initial.version, Version(0));
    assert_eq!(initial.value, "starting");

    let next_version = publisher.publish("running".to_string(), UnixMillis(2))?;
    assert_eq!(next_version, Version(1));

    let changed = subscriber.changed().await?;
    assert_eq!(changed.version, Version(1));
    assert_eq!(changed.updated_at, UnixMillis(2));
    assert_eq!(changed.value, "running");

    drop(publisher);
    let closed = subscriber.changed().await;
    assert_eq!(closed, Err(StateRecvError::ChannelClosed));
    Ok(())
}

--- END FILE: tests/bdd_state_watch.rs ---

