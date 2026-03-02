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
