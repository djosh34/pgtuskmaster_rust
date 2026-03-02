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
