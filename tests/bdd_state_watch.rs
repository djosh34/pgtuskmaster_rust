use pgtuskmaster_rust::state::{new_state_channel, StateRecvError};

#[tokio::test(flavor = "current_thread")]
async fn bdd_state_watch_channel_flow() -> Result<(), Box<dyn std::error::Error>> {
    let (publisher, mut subscriber) = new_state_channel("starting".to_string());

    let initial = subscriber.latest();
    assert_eq!(initial, "starting");

    publisher.publish("running".to_string())?;

    let changed = subscriber.changed().await?;
    assert_eq!(changed, "running");

    drop(publisher);
    let closed = subscriber.changed().await;
    assert_eq!(closed, Err(StateRecvError::ChannelClosed));
    Ok(())
}
