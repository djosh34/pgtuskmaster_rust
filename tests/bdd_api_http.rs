use std::sync::{Arc, Mutex};
use std::time::Duration;

use pgtuskmaster_rust::{
    api::worker::ApiWorkerCtx,
    config::{ApiAuthConfig, ApiRoleTokensConfig, RuntimeConfig, SecretSource},
    dcs::store::{DcsStore, DcsStoreError, WatchEvent},
    state::{new_state_channel, WorkerError},
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};

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

    fn read_path(&mut self, _path: &str) -> Result<Option<String>, DcsStoreError> {
        Ok(None)
    }

    fn snapshot_prefix(&mut self, _path_prefix: &str) -> Result<Vec<WatchEvent>, DcsStoreError> {
        Ok(Vec::new())
    }

    fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
        let mut guard = self
            .writes
            .lock()
            .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
        guard.push((path.to_string(), value));
        Ok(())
    }

    fn write_path_with_lease(
        &mut self,
        path: &str,
        value: String,
        _lease_ttl_ms: u64,
    ) -> Result<(), DcsStoreError> {
        self.write_path(path, value)
    }

    fn put_path_if_absent(&mut self, path: &str, value: String) -> Result<bool, DcsStoreError> {
        self.write_path(path, value)?;
        Ok(true)
    }

    fn delete_path(&mut self, _path: &str) -> Result<(), DcsStoreError> {
        Ok(())
    }

    fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
        Ok(Vec::new())
    }
}

fn sample_runtime_config(auth_token: Option<String>) -> RuntimeConfig {
    let auth = match auth_token {
        Some(token) => ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
            read_token: Some(SecretSource::Inline {
                content: token.clone(),
            }),
            admin_token: Some(SecretSource::Inline { content: token }),
        }),
        None => ApiAuthConfig::Disabled,
    };

    pgtuskmaster_rust::test_harness::runtime_config::RuntimeConfigBuilder::new()
        .with_api_listen_addr(std::net::SocketAddr::from(([127, 0, 0, 1], 0)))
        .with_api_auth(auth)
        .build()
}

const HEADER_LIMIT: usize = 16 * 1024;
const MAX_BODY_BYTES: usize = 256 * 1024;
const MAX_RESPONSE_BYTES: usize = HEADER_LIMIT + MAX_BODY_BYTES;
const IO_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug)]
struct TestHttpResponse {
    status_code: u16,
}

fn parse_status_code(response: &[u8]) -> Result<(u16, usize), WorkerError> {
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| WorkerError::Message("response missing header terminator".to_string()))?
        + 4;
    let status_line_end = response
        .windows(2)
        .position(|window| window == b"\r\n")
        .ok_or_else(|| WorkerError::Message("response missing status line".to_string()))?;
    let status_line = std::str::from_utf8(&response[..status_line_end])
        .map_err(|err| WorkerError::Message(format!("response status line not utf8: {err}")))?;
    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| WorkerError::Message("response missing status code".to_string()))?
        .parse::<u16>()
        .map_err(|err| WorkerError::Message(format!("invalid status code: {err}")))?;
    Ok((status_code, header_end))
}

async fn read_http_response_framed<S>(stream: &mut S) -> Result<TestHttpResponse, WorkerError>
where
    S: AsyncRead + Unpin,
{
    let task = async {
        let mut buffer = Vec::new();
        let mut scratch = [0_u8; 1024];
        loop {
            let read = stream
                .read(&mut scratch)
                .await
                .map_err(|err| WorkerError::Message(format!("read failed: {err}")))?;
            if read == 0 {
                break;
            }
            buffer.extend_from_slice(&scratch[..read]);
            if buffer.len() > MAX_RESPONSE_BYTES {
                return Err(WorkerError::Message("response exceeded limit".to_string()));
            }
            if let Ok((status_code, header_end)) = parse_status_code(&buffer) {
                let body = buffer[header_end..].to_vec();
                if body.len() <= MAX_BODY_BYTES {
                    return Ok(TestHttpResponse { status_code });
                }
            }
        }
        Err(WorkerError::Message(
            "connection closed before complete response".to_string(),
        ))
    };

    tokio::time::timeout(IO_TIMEOUT, task)
        .await
        .map_err(|_| WorkerError::Message("timed out reading http response".to_string()))?
}

async fn request_once(
    ctx: &mut ApiWorkerCtx,
    request: &str,
) -> Result<TestHttpResponse, WorkerError> {
    let addr = ctx.local_addr()?;
    let mut client = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
    client
        .write_all(request.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("write failed: {err}")))?;

    pgtuskmaster_rust::api::worker::step_once(ctx).await?;
    read_http_response_framed(&mut client).await
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_state_requires_live_state_subscribers() -> Result<(), WorkerError> {
    let cfg = sample_runtime_config(None);
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;

    let store = RecordingStore::default();
    let mut ctx = ApiWorkerCtx::contract_stub(listener, cfg_subscriber, Box::new(store));
    let response = request_once(
        &mut ctx,
        "GET /state HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await?;
    assert_eq!(response.status_code, 503);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_old_debug_and_fallback_routes_are_gone() -> Result<(), WorkerError> {
    let cfg = sample_runtime_config(None);
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;

    let store = RecordingStore::default();
    let mut ctx = ApiWorkerCtx::contract_stub(listener, cfg_subscriber, Box::new(store));

    let debug_response = request_once(
        &mut ctx,
        "GET /debug/verbose HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await?;
    assert_eq!(debug_response.status_code, 404);

    let fallback_response = request_once(
        &mut ctx,
        "GET /fallback/cluster HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await?;
    assert_eq!(fallback_response.status_code, 404);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn bdd_api_auth_token_denies_missing_header() -> Result<(), WorkerError> {
    let cfg = sample_runtime_config(Some("secret".to_string()));
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;

    let store = RecordingStore::default();
    let mut ctx = ApiWorkerCtx::contract_stub(listener, cfg_subscriber, Box::new(store.clone()));
    let response = request_once(
        &mut ctx,
        "GET /state HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
    )
    .await?;
    assert_eq!(response.status_code, 401);
    assert!(store.writes()?.is_empty());
    Ok(())
}
