use std::time::Duration;

use pgtuskmaster_rust::{
    api::worker::{step_once, ApiWorkerCtx},
    config::{
        ApiConfig, BinaryPaths, ClusterConfig, DcsConfig, DebugConfig, HaConfig, PostgresConfig,
        ProcessConfig, RuntimeConfig, SecurityConfig,
    },
    dcs::store::{DcsStore, DcsStoreError, WatchEvent},
    state::{new_state_channel, UnixMillis, WorkerError},
};

struct SmokeStore;

impl DcsStore for SmokeStore {
    fn healthy(&self) -> bool {
        true
    }

    fn write_path(&mut self, _path: &str, _value: String) -> Result<(), DcsStoreError> {
        Ok(())
    }

    fn delete_path(&mut self, _path: &str) -> Result<(), DcsStoreError> {
        Ok(())
    }

    fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
        Ok(Vec::new())
    }
}

fn sample_runtime_config() -> RuntimeConfig {
    RuntimeConfig {
        cluster: ClusterConfig {
            name: "cluster-a".to_string(),
            member_id: "node-a".to_string(),
        },
        postgres: PostgresConfig {
            data_dir: "/tmp/pgdata".into(),
            connect_timeout_s: 5,
        },
        dcs: DcsConfig {
            endpoints: vec!["http://127.0.0.1:2379".to_string()],
            scope: "scope-a".to_string(),
        },
        ha: HaConfig {
            loop_interval_ms: 1000,
            lease_ttl_ms: 10_000,
        },
        process: ProcessConfig {
            pg_rewind_timeout_ms: 1000,
            bootstrap_timeout_ms: 1000,
            fencing_timeout_ms: 1000,
            binaries: BinaryPaths {
                postgres: "/usr/bin/postgres".into(),
                pg_ctl: "/usr/bin/pg_ctl".into(),
                pg_rewind: "/usr/bin/pg_rewind".into(),
                initdb: "/usr/bin/initdb".into(),
                psql: "/usr/bin/psql".into(),
            },
        },
        api: ApiConfig {
            listen_addr: "127.0.0.1:18080".to_string(),
        },
        debug: DebugConfig { enabled: true },
        security: SecurityConfig {
            tls_enabled: false,
            auth_token: None,
        },
    }
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
