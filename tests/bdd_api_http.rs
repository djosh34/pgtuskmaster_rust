use std::sync::{Arc, Mutex};
use std::time::Duration;

use pgtuskmaster_rust::{
    api::worker::ApiWorkerCtx,
    config::{ApiAuthConfig, ApiRoleTokensConfig, RuntimeConfig, SecretSource},
    dcs::store::{DcsStore, DcsStoreError, WatchEvent},
    state::{new_state_channel, UnixMillis, WorkerError},
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};

#[derive(Clone, Default)]
struct RecordingStore {
    writes: Arc<Mutex<Vec<(String, String)>>>,
    deletes: Arc<Mutex<Vec<String>>>,
    kv: Arc<Mutex<std::collections::BTreeMap<String, String>>>,
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

    fn read_path(&mut self, path: &str) -> Result<Option<String>, DcsStoreError> {
        let guard = self
            .kv
            .lock()
            .map_err(|_| DcsStoreError::Io("kv lock poisoned".to_string()))?;
        Ok(guard.get(path).cloned())
    }

    fn snapshot_prefix(&mut self, path_prefix: &str) -> Result<Vec<WatchEvent>, DcsStoreError> {
        let guard = self
            .kv
            .lock()
            .map_err(|_| DcsStoreError::Io("kv lock poisoned".to_string()))?;
        let mut events = vec![WatchEvent {
            op: pgtuskmaster_rust::dcs::store::WatchOp::Reset,
            path: path_prefix.to_string(),
            value: None,
            revision: 0,
        }];
        events.extend(
            guard
                .iter()
                .filter(|(path, _)| path.starts_with(path_prefix))
                .map(|(path, value)| WatchEvent {
                    op: pgtuskmaster_rust::dcs::store::WatchOp::Put,
                    path: path.clone(),
                    value: Some(value.clone()),
                    revision: 0,
                }),
        );
        Ok(events)
    }

    fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
        {
            let mut guard = self
                .kv
                .lock()
                .map_err(|_| DcsStoreError::Io("kv lock poisoned".to_string()))?;
            guard.insert(path.to_string(), value.clone());
        }
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
        {
            let mut guard = self
                .kv
                .lock()
                .map_err(|_| DcsStoreError::Io("kv lock poisoned".to_string()))?;
            if guard.contains_key(path) {
                return Ok(false);
            }
            guard.insert(path.to_string(), value.clone());
        }
        let mut guard = self
            .writes
            .lock()
            .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
        guard.push((path.to_string(), value));
        Ok(true)
    }

    fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError> {
        {
            let mut guard = self
                .kv
                .lock()
                .map_err(|_| DcsStoreError::Io("kv lock poisoned".to_string()))?;
            guard.remove(path);
        }
        let mut guard = self
            .deletes
            .lock()
            .map_err(|_| DcsStoreError::Io("deletes lock poisoned".to_string()))?;
        guard.push(path.to_string());
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
    body: Vec<u8>,
}

#[derive(Debug)]
struct ParsedHttpHead {
    status_code: u16,
    content_length: usize,
    body_start: usize,
}

fn parse_http_response_head(raw: &[u8], header_end: usize) -> Result<ParsedHttpHead, WorkerError> {
    let head = raw.get(..header_end).ok_or_else(|| {
        WorkerError::Message("response header end offset out of bounds".to_string())
    })?;

    let status_line_end = head
        .windows(2)
        .position(|window| window == b"\r\n")
        .ok_or_else(|| WorkerError::Message("response missing status line".to_string()))?;

    let status_line_bytes = head.get(..status_line_end).ok_or_else(|| {
        WorkerError::Message("response status line offset out of bounds".to_string())
    })?;
    let status_line = std::str::from_utf8(status_line_bytes)
        .map_err(|err| WorkerError::Message(format!("response status line not utf8: {err}")))?;

    let mut status_parts = status_line.split_whitespace();
    let http_version = status_parts.next().ok_or_else(|| {
        WorkerError::Message("response status line missing http version".to_string())
    })?;
    if http_version != "HTTP/1.1" {
        return Err(WorkerError::Message(format!(
            "unexpected http version in response: {http_version}"
        )));
    }
    let status_str = status_parts.next().ok_or_else(|| {
        WorkerError::Message("response status line missing status code".to_string())
    })?;
    if status_str.len() != 3 || !status_str.bytes().all(|b| b.is_ascii_digit()) {
        return Err(WorkerError::Message(format!(
            "response status code must be 3 digits, got: {status_str}"
        )));
    }
    let status_code = status_str
        .parse::<u16>()
        .map_err(|err| WorkerError::Message(format!("response status code parse failed: {err}")))?;
    if !(100..=599).contains(&status_code) {
        return Err(WorkerError::Message(format!(
            "response status code out of range: {status_code}"
        )));
    }

    let header_text = head
        .get(status_line_end + 2..)
        .ok_or_else(|| WorkerError::Message("response header offset out of bounds".to_string()))?;
    let header_text = std::str::from_utf8(header_text)
        .map_err(|err| WorkerError::Message(format!("response headers not utf8: {err}")))?;

    let mut content_length: Option<usize> = None;
    for line in header_text.split("\r\n") {
        if line.is_empty() {
            continue;
        }
        let (name, value) = line.split_once(':').ok_or_else(|| {
            WorkerError::Message(format!(
                "invalid response header line (missing ':'): {line}"
            ))
        })?;
        let name = name.trim();
        let value = value.trim();

        if name.eq_ignore_ascii_case("Content-Length") {
            if content_length.is_some() {
                return Err(WorkerError::Message(
                    "response contains multiple Content-Length headers".to_string(),
                ));
            }
            let parsed = value.parse::<usize>().map_err(|err| {
                WorkerError::Message(format!("response Content-Length parse failed: {err}"))
            })?;
            content_length = Some(parsed);
        }
    }

    let content_length = content_length.ok_or_else(|| {
        WorkerError::Message("response missing Content-Length header".to_string())
    })?;

    let body_start = header_end
        .checked_add(4)
        .ok_or_else(|| WorkerError::Message("response body offset overflow".to_string()))?;

    Ok(ParsedHttpHead {
        status_code,
        content_length,
        body_start,
    })
}

async fn read_http_response_framed(
    stream: &mut (impl AsyncRead + Unpin),
    timeout: Duration,
) -> Result<TestHttpResponse, WorkerError> {
    let response = tokio::time::timeout(timeout, async {
        let mut raw: Vec<u8> = Vec::new();
        let mut scratch = [0u8; 4096];

        let mut parsed_head: Option<ParsedHttpHead> = None;
        let mut expected_total_len: Option<usize> = None;

        loop {
            if let Some(expected) = expected_total_len {
                if raw.len() == expected {
                    let parsed = parsed_head.ok_or_else(|| {
                        WorkerError::Message("response framing parsed without header".to_string())
                    })?;
                    let body = raw
                        .get(parsed.body_start..expected)
                        .ok_or_else(|| {
                            WorkerError::Message(
                                "response body slice out of bounds after framing".to_string(),
                            )
                        })?
                        .to_vec();
                    return Ok(TestHttpResponse {
                        status_code: parsed.status_code,
                        body,
                    });
                }
                if raw.len() > expected {
                    return Err(WorkerError::Message(format!(
                        "response exceeded expected length (expected {expected} bytes, got {})",
                        raw.len()
                    )));
                }
            } else {
                if raw.len() > HEADER_LIMIT {
                    return Err(WorkerError::Message(format!(
                        "response headers exceeded limit of {HEADER_LIMIT} bytes"
                    )));
                }

                if let Some(header_end) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = parse_http_response_head(&raw, header_end)?;
                    if head.content_length > MAX_BODY_BYTES {
                        return Err(WorkerError::Message(format!(
                            "response body exceeded limit of {MAX_BODY_BYTES} bytes (Content-Length={})",
                            head.content_length
                        )));
                    }
                    let expected = head.body_start.checked_add(head.content_length).ok_or_else(|| {
                        WorkerError::Message("response total length overflow".to_string())
                    })?;
                    if expected > MAX_RESPONSE_BYTES {
                        return Err(WorkerError::Message(format!(
                            "response exceeded limit of {MAX_RESPONSE_BYTES} bytes (expected {expected})"
                        )));
                    }
                    parsed_head = Some(head);
                    expected_total_len = Some(expected);
                    continue;
                }
            }

            let n = stream.read(&mut scratch).await.map_err(|err| {
                WorkerError::Message(format!("client read failed: {err}"))
            })?;
            if n == 0 {
                return Err(WorkerError::Message(format!(
                    "unexpected eof while reading response (read {} bytes so far)",
                    raw.len()
                )));
            }

            let new_len = raw.len().checked_add(n).ok_or_else(|| {
                WorkerError::Message("response length overflow while reading".to_string())
            })?;
            if new_len > MAX_RESPONSE_BYTES {
                return Err(WorkerError::Message(format!(
                    "response exceeded limit of {MAX_RESPONSE_BYTES} bytes while reading (would reach {new_len})"
                )));
            }
            raw.extend_from_slice(&scratch[..n]);
        }
    })
    .await;

    match response {
        Ok(inner) => inner,
        Err(_) => Err(WorkerError::Message(format!(
            "timed out reading framed http response after {}s",
            timeout.as_secs()
        ))),
    }
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
    let request = "GET /fallback/cluster HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    client
        .write_all(request.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("client write failed: {err}")))?;

    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;

    let response = read_http_response_framed(&mut client, IO_TIMEOUT).await?;
    assert_eq!(response.status_code, 200);
    let decoded: serde_json::Value = serde_json::from_slice(&response.body)
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
    let request = "GET /fallback/cluster HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    client
        .write_all(request.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("client write failed: {err}")))?;

    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;

    let response = read_http_response_framed(&mut client, IO_TIMEOUT).await?;
    assert_eq!(response.status_code, 401);
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
    let ui_request = "GET /debug/ui HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    ui_client
        .write_all(ui_request.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("ui write failed: {err}")))?;
    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;
    let ui_response = read_http_response_framed(&mut ui_client, IO_TIMEOUT).await?;
    assert_eq!(ui_response.status_code, 200);
    let ui_html = String::from_utf8(ui_response.body)
        .map_err(|err| WorkerError::Message(format!("ui body not utf8: {err}")))?;
    assert!(ui_html.contains("id=\"meta-panel\""));
    assert!(ui_html.contains("/debug/verbose"));

    let mut verbose_client = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|err| WorkerError::Message(format!("connect failed: {err}")))?;
    let verbose_request =
        "GET /debug/verbose HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    verbose_client
        .write_all(verbose_request.as_bytes())
        .await
        .map_err(|err| WorkerError::Message(format!("verbose write failed: {err}")))?;
    pgtuskmaster_rust::api::worker::step_once(&mut ctx).await?;
    let verbose_response = read_http_response_framed(&mut verbose_client, IO_TIMEOUT).await?;
    assert_eq!(verbose_response.status_code, 503);
    let verbose_text = String::from_utf8(verbose_response.body)
        .map_err(|err| WorkerError::Message(format!("verbose body not utf8: {err}")))?;
    assert!(verbose_text.contains("snapshot unavailable"));
    Ok(())
}
