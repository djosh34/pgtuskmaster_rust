use tokio::sync::watch;

use super::errors::StateRecvError;

#[derive(Debug)]
pub struct StatePublisher<T: Clone> {
    tx: watch::Sender<T>,
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
    rx: watch::Receiver<T>,
}

impl<T: Clone> Clone for StateSubscriber<T> {
    fn clone(&self) -> Self {
        Self {
            rx: self.rx.clone(),
        }
    }
}

pub fn new_state_channel<T: Clone>(initial: T) -> (StatePublisher<T>, StateSubscriber<T>) {
    let (tx, rx) = watch::channel(initial);
    (StatePublisher { tx }, StateSubscriber { rx })
}

impl<T: Clone> StatePublisher<T> {
    pub fn publish(&self, next: T) -> Result<(), StateRecvError> {
        self.tx
            .send(next)
            .map_err(|_| StateRecvError::ChannelClosed)
    }

    pub fn latest(&self) -> T {
        self.tx.borrow().clone()
    }
}

impl<T: Clone> StateSubscriber<T> {
    pub fn latest(&self) -> T {
        self.rx.borrow().clone()
    }

    pub async fn changed(&mut self) -> Result<T, StateRecvError> {
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
    async fn initial_snapshot_matches_input() {
        let (_publisher, subscriber) = new_state_channel("booting".to_string());
        let latest = subscriber.latest();
        assert_eq!(latest, "booting");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn publish_replaces_value() -> Result<(), Box<dyn std::error::Error>> {
        let (publisher, subscriber) = new_state_channel("a".to_string());

        publisher.publish("b".to_string())?;
        let latest = subscriber.latest();
        assert_eq!(latest, "b");

        publisher.publish("c".to_string())?;
        let latest = subscriber.latest();
        assert_eq!(latest, "c");
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn changed_returns_latest_after_publish() -> Result<(), Box<dyn std::error::Error>> {
        let (publisher, mut subscriber) = new_state_channel("ready".to_string());
        publisher.publish("running".to_string())?;

        let changed = subscriber.changed().await?;
        assert_eq!(changed, "running");
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn changed_propagates_closed_channel_error() {
        let (publisher, mut subscriber) = new_state_channel("ready".to_string());
        drop(publisher);

        let changed = subscriber.changed().await;
        assert_eq!(changed, Err(StateRecvError::ChannelClosed));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn latest_matches_between_publisher_and_subscriber(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (publisher, subscriber) = new_state_channel("ready".to_string());
        publisher.publish("running".to_string())?;

        assert_eq!(publisher.latest(), subscriber.latest());
        Ok(())
    }
}
