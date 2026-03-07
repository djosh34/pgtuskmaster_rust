use std::time::Duration;

use pgtuskmaster_rust::{
    api::worker::{step_once, ApiWorkerCtx},
    config::RuntimeConfig,
    dcs::store::{DcsStore, DcsStoreError, WatchEvent},
    state::{new_state_channel, UnixMillis, WorkerError},
};

struct SmokeStore;

impl DcsStore for SmokeStore {
    fn healthy(&self) -> bool {
        true
    }

    fn read_path(&mut self, _path: &str) -> Result<Option<String>, DcsStoreError> {
        Ok(None)
    }

    fn write_path(&mut self, _path: &str, _value: String) -> Result<(), DcsStoreError> {
        Ok(())
    }

    fn put_path_if_absent(&mut self, _path: &str, _value: String) -> Result<bool, DcsStoreError> {
        Ok(true)
    }

    fn delete_path(&mut self, _path: &str) -> Result<(), DcsStoreError> {
        Ok(())
    }

    fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
        Ok(Vec::new())
    }
}

fn sample_runtime_config() -> RuntimeConfig {
    pgtuskmaster_rust::test_harness::runtime_config::RuntimeConfigBuilder::new()
        .with_api_listen_addr("127.0.0.1:18080")
        .build()
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), WorkerError> {
    let cfg = sample_runtime_config();
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg, UnixMillis(1));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:18080")
        .await
        .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;
    let mut ctx = ApiWorkerCtx::contract_stub(listener, cfg_subscriber, Box::new(SmokeStore));

    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}
