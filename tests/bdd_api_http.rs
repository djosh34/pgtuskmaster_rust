use std::sync::{Arc, Mutex};

use pgtuskmaster_rust::{
    api::worker::ApiWorkerCtx,
    config::{
        ApiConfig, BinaryPaths, ClusterConfig, DcsConfig, DebugConfig, HaConfig, PostgresConfig,
        ProcessConfig, RuntimeConfig, SecurityConfig,
    },
    dcs::store::{DcsStore, DcsStoreError, WatchEvent},
    state::{new_state_channel, UnixMillis, WorkerError},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Clone, Default)]
struct RecordingStore {
    writes: Arc<Mutex<Vec<(String, String)>>>,
}

impl RecordingStore {
    fn writes(&self) -> Result<Vec<(String, String)>, WorkerError> {
        let guard = self
            .writes
            .lock()
            .map_err(|_| WorkerError::Message("writes lock poisoned".to_string()))?;
        Ok(guard.clone())
    }
}

impl DcsStore for RecordingStore {
    fn healthy(&self) -> bool {
        true
    }

    fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
        let mut guard = self
            .writes
            .lock()
            .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
        guard.push((path.to_string(), value));
        Ok(())
    }

    fn delete_path(&mut self, _path: &str) -> Result<(), DcsStoreError> {
        Ok(())
    }

    fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
        Ok(Vec::new())
    }
}

fn sample_runtime_config(auth_token: Option<String>) -> RuntimeConfig {
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
            listen_addr: "127.0.0.1:0".to_string(),
        },
        debug: DebugConfig { enabled: true },
        security: SecurityConfig {
            tls_enabled: false,
            auth_token,
        },
    }
}

fn extract_status_and_body(raw: &[u8]) -> Result<(String, Vec<u8>), WorkerError> {
    let raw_str = String::from_utf8_lossy(raw);
    let mut parts = raw_str.splitn(2, "\r\n");
    let status_line = parts
        .next()
        .ok_or_else(|| WorkerError::Message("missing status line".to_string()))?
        .to_string();
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| WorkerError::Message("missing header terminator".to_string()))?;
    Ok((status_line, raw[split + 4..].to_vec()))
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_post_switchover_writes_dcs_key() -> Result<(), WorkerError> {
    let cfg = sample_runtime_config(None);
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg, UnixMillis(1));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;

    let store = RecordingStore::default();
    let store_for_ctx = store.clone();
    let mut ctx = ApiWorkerCtx::contract_stub(listener, cfg_subscriber, Box::new(store_for_ctx));
    let addr = ctx.local_addr()?;

    let mut client = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
    let body = br#"{"requested_by":"node-a"}"#;
    let request = format!(
        "POST /switchover HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    client
        .write_all(request.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("client write header failed: {err}")))?;
    client
        .write_all(body)
        .await
        .map_err(|err| WorkerError::Message(format!("client write body failed: {err}")))?;

    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;

    let mut raw = Vec::new();
    client
        .read_to_end(&mut raw)
        .await
        .map_err(|err| WorkerError::Message(format!("client read failed: {err}")))?;
    let (status_line, body_bytes) = extract_status_and_body(&raw)?;
    assert!(
        status_line.contains("202"),
        "expected 202, got: {status_line}"
    );
    let decoded: serde_json::Value = serde_json::from_slice(&body_bytes)
        .map_err(|err| WorkerError::Message(format!("decode response json failed: {err}")))?;
    assert_eq!(decoded["accepted"], true);

    let writes = store.writes()?;
    assert_eq!(writes.len(), 1);
    assert_eq!(writes[0].0, "/scope-a/switchover");
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_get_fallback_cluster_returns_name() -> Result<(), WorkerError> {
    let cfg = sample_runtime_config(None);
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg, UnixMillis(1));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;

    let store = RecordingStore::default();
    let store_for_ctx = store.clone();
    let mut ctx = ApiWorkerCtx::contract_stub(listener, cfg_subscriber, Box::new(store_for_ctx));
    let addr = ctx.local_addr()?;

    let mut client = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
    let request = "GET /fallback/cluster HTTP/1.1\r\nHost: localhost\r\n\r\n";
    client
        .write_all(request.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("client write failed: {err}")))?;

    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;

    let mut raw = Vec::new();
    client
        .read_to_end(&mut raw)
        .await
        .map_err(|err| WorkerError::Message(format!("client read failed: {err}")))?;
    let (status_line, body_bytes) = extract_status_and_body(&raw)?;
    assert!(
        status_line.contains("200"),
        "expected 200, got: {status_line}"
    );
    let decoded: serde_json::Value = serde_json::from_slice(&body_bytes)
        .map_err(|err| WorkerError::Message(format!("decode response json failed: {err}")))?;
    assert_eq!(decoded["name"], "cluster-a");
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_auth_token_denies_missing_header() -> Result<(), WorkerError> {
    let cfg = sample_runtime_config(Some("secret".to_string()));
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg, UnixMillis(1));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;

    let store = RecordingStore::default();
    let store_for_ctx = store.clone();
    let mut ctx = ApiWorkerCtx::contract_stub(listener, cfg_subscriber, Box::new(store_for_ctx));
    let addr = ctx.local_addr()?;

    let mut client = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
    let request = "GET /fallback/cluster HTTP/1.1\r\nHost: localhost\r\n\r\n";
    client
        .write_all(request.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("client write failed: {err}")))?;

    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;

    let mut raw = Vec::new();
    client
        .read_to_end(&mut raw)
        .await
        .map_err(|err| WorkerError::Message(format!("client read failed: {err}")))?;
    let (status_line, _body_bytes) = extract_status_and_body(&raw)?;
    assert!(
        status_line.contains("401"),
        "expected 401, got: {status_line}"
    );
    let writes = store.writes()?;
    assert_eq!(writes.len(), 0);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_debug_routes_expose_ui_and_verbose_contracts() -> Result<(), WorkerError> {
    let cfg = sample_runtime_config(None);
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg, UnixMillis(1));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;

    let store = RecordingStore::default();
    let store_for_ctx = store.clone();
    let mut ctx = ApiWorkerCtx::contract_stub(listener, cfg_subscriber, Box::new(store_for_ctx));
    let addr = ctx.local_addr()?;

    let mut ui_client = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
    let ui_request = "GET /debug/ui HTTP/1.1\r\nHost: localhost\r\n\r\n";
    ui_client
        .write_all(ui_request.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("ui write failed: {err}")))?;
    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;
    let mut ui_raw = Vec::new();
    ui_client
        .read_to_end(&mut ui_raw)
        .await
        .map_err(|err| WorkerError::Message(format!("ui read failed: {err}")))?;
    let (ui_status, ui_body) = extract_status_and_body(&ui_raw)?;
    assert!(ui_status.contains("200"), "expected 200, got: {ui_status}");
    let ui_html = String::from_utf8(ui_body)
        .map_err(|err| WorkerError::Message(format!("ui body not utf8: {err}")))?;
    assert!(ui_html.contains("id=\"meta-panel\""));
    assert!(ui_html.contains("/debug/verbose"));

    let mut verbose_client = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
    let verbose_request = "GET /debug/verbose HTTP/1.1\r\nHost: localhost\r\n\r\n";
    verbose_client
        .write_all(verbose_request.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("verbose write failed: {err}")))?;
    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;
    let mut verbose_raw = Vec::new();
    verbose_client
        .read_to_end(&mut verbose_raw)
        .await
        .map_err(|err| WorkerError::Message(format!("verbose read failed: {err}")))?;
    let (verbose_status, verbose_body) = extract_status_and_body(&verbose_raw)?;
    assert!(
        verbose_status.contains("503"),
        "expected 503, got: {verbose_status}"
    );
    let verbose_text = String::from_utf8(verbose_body)
        .map_err(|err| WorkerError::Message(format!("verbose body not utf8: {err}")))?;
    assert!(verbose_text.contains("snapshot unavailable"));
    Ok(())
}
