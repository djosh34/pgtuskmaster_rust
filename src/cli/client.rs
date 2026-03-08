use std::time::Duration;

use reqwest::{Method, StatusCode, Url};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub(crate) use crate::api::{AcceptedResponse, HaDecisionResponse, HaStateResponse};
use crate::cli::error::CliError;

#[derive(Clone, Debug)]
pub struct CliApiClient {
    base_url: Url,
    http: reqwest::Client,
    read_token: Option<String>,
    admin_token: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AuthRole {
    Read,
    Admin,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct SwitchoverRequestInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    switchover_to: Option<String>,
}

impl CliApiClient {
    pub fn new(
        base_url: String,
        timeout_ms: u64,
        read_token: Option<String>,
        admin_token: Option<String>,
    ) -> Result<Self, CliError> {
        let base_url = Url::parse(base_url.trim())
            .map_err(|err| CliError::RequestBuild(format!("invalid --base-url value: {err}")))?;
        let http = reqwest::Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .pool_max_idle_per_host(0)
            .build()
            .map_err(|err| CliError::RequestBuild(format!("build http client failed: {err}")))?;

        Ok(Self {
            base_url,
            http,
            read_token: normalize_token(read_token),
            admin_token: normalize_token(admin_token),
        })
    }

    pub async fn get_ha_state(&self) -> Result<HaStateResponse, CliError> {
        self.send_json_no_body(Method::GET, "/ha/state", AuthRole::Read, StatusCode::OK)
            .await
    }

    pub async fn delete_switchover(&self) -> Result<AcceptedResponse, CliError> {
        self.send_json_no_body(
            Method::DELETE,
            "/ha/switchover",
            AuthRole::Admin,
            StatusCode::ACCEPTED,
        )
        .await
    }

    pub async fn post_switchover(
        &self,
        switchover_to: Option<String>,
    ) -> Result<AcceptedResponse, CliError> {
        let body = SwitchoverRequestInput { switchover_to };
        self.send_json_with_body(
            Method::POST,
            "/switchover",
            AuthRole::Admin,
            &body,
            StatusCode::ACCEPTED,
        )
        .await
    }

    async fn send_json_no_body<T>(
        &self,
        method: Method,
        path: &str,
        role: AuthRole,
        expected_status: StatusCode,
    ) -> Result<T, CliError>
    where
        T: DeserializeOwned,
    {
        let url = self
            .base_url
            .join(path)
            .map_err(|err| CliError::RequestBuild(format!("compose URL failed: {err}")))?;
        let mut request = self.http.request(method, url);
        if let Some(token) = self.token_for(role) {
            request = request.bearer_auth(token);
        }

        let response = request
            .send()
            .await
            .map_err(|err| CliError::Transport(err.to_string()))?;

        read_json_response(response, expected_status).await
    }

    async fn send_json_with_body<T, B>(
        &self,
        method: Method,
        path: &str,
        role: AuthRole,
        body: &B,
        expected_status: StatusCode,
    ) -> Result<T, CliError>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let url = self
            .base_url
            .join(path)
            .map_err(|err| CliError::RequestBuild(format!("compose URL failed: {err}")))?;
        let mut request = self.http.request(method, url);
        if let Some(token) = self.token_for(role) {
            request = request.bearer_auth(token);
        }

        let response = request
            .json(body)
            .send()
            .await
            .map_err(|err| CliError::Transport(err.to_string()))?;

        read_json_response(response, expected_status).await
    }

    fn token_for(&self, role: AuthRole) -> Option<&str> {
        match role {
            AuthRole::Read => self.read_token.as_deref().or(self.admin_token.as_deref()),
            AuthRole::Admin => self.admin_token.as_deref(),
        }
    }
}

async fn read_json_response<T>(
    response: reqwest::Response,
    expected_status: StatusCode,
) -> Result<T, CliError>
where
    T: DeserializeOwned,
{
    let status = response.status();
    if status != expected_status {
        let body = match response.text().await {
            Ok(value) => value,
            Err(err) => format!("<failed to read response body: {err}>"),
        };
        return Err(CliError::ApiStatus {
            status: status.as_u16(),
            body,
        });
    }

    response
        .json::<T>()
        .await
        .map_err(|err| CliError::Decode(err.to_string()))
}

fn normalize_token(raw: Option<String>) -> Option<String> {
    match raw {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    use crate::cli::client::{CliApiClient, CliError, HaDecisionResponse};

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct RecordedRequest {
        method: String,
        path: String,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    }

    #[tokio::test]
    async fn state_request_uses_read_token_when_configured() -> Result<(), CliError> {
        let response_body = r#"{"cluster_name":"cluster-a","scope":"scope-a","self_member_id":"node-a","leader":null,"switchover_pending":false,"switchover_to":null,"member_count":1,"dcs_trust":"full_quorum","ha_phase":"primary","ha_tick":1,"ha_decision":{"kind":"become_primary","promote":true},"snapshot_sequence":10}"#;
        let (addr, handle) = spawn_server(http_response(200, response_body)).await?;

        let client = CliApiClient::new(
            format!("http://{addr}"),
            5_000,
            Some("read-token".to_string()),
            Some("admin-token".to_string()),
        )?;
        let state = client.get_ha_state().await?;
        assert_eq!(state.cluster_name, "cluster-a");
        assert_eq!(
            state.ha_decision,
            HaDecisionResponse::BecomePrimary { promote: true }
        );

        let request = handle_request(handle).await?;
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/ha/state");
        assert_header(&request.headers, "authorization", "Bearer read-token")?;
        Ok(())
    }

    #[tokio::test]
    async fn state_request_falls_back_to_admin_token_when_read_missing() -> Result<(), CliError> {
        let response_body = r#"{"cluster_name":"cluster-a","scope":"scope-a","self_member_id":"node-a","leader":null,"switchover_pending":false,"switchover_to":null,"member_count":1,"dcs_trust":"full_quorum","ha_phase":"primary","ha_tick":1,"ha_decision":{"kind":"become_primary","promote":true},"snapshot_sequence":10}"#;
        let (addr, handle) = spawn_server(http_response(200, response_body)).await?;

        let client = CliApiClient::new(
            format!("http://{addr}"),
            5_000,
            None,
            Some("admin-token".to_string()),
        )?;
        let _ = client.get_ha_state().await?;

        let request = handle_request(handle).await?;
        assert_header(&request.headers, "authorization", "Bearer admin-token")?;
        Ok(())
    }

    #[tokio::test]
    async fn switchover_clear_uses_delete_endpoint() -> Result<(), CliError> {
        let (addr, handle) = spawn_server(http_response(202, r#"{"accepted":true}"#)).await?;
        let client = CliApiClient::new(
            format!("http://{addr}"),
            5_000,
            Some("reader".to_string()),
            Some("admin".to_string()),
        )?;

        let _ = client.delete_switchover().await?;
        let request = handle_request(handle).await?;
        assert_eq!(request.method, "DELETE");
        assert_eq!(request.path, "/ha/switchover");
        assert_header(&request.headers, "authorization", "Bearer admin")?;
        Ok(())
    }

    #[tokio::test]
    async fn switchover_request_without_target_posts_empty_object() -> Result<(), CliError> {
        let (addr, handle) = spawn_server(http_response(202, r#"{"accepted":true}"#)).await?;
        let client = CliApiClient::new(
            format!("http://{addr}"),
            5_000,
            Some("reader".to_string()),
            Some("admin".to_string()),
        )?;

        let _ = client.post_switchover(None).await?;
        let request = handle_request(handle).await?;
        assert_eq!(request.method, "POST");
        assert_eq!(request.path, "/switchover");
        assert_eq!(String::from_utf8_lossy(&request.body), "{}");
        Ok(())
    }

    #[tokio::test]
    async fn switchover_request_with_target_posts_member_id() -> Result<(), CliError> {
        let (addr, handle) = spawn_server(http_response(202, r#"{"accepted":true}"#)).await?;
        let client = CliApiClient::new(
            format!("http://{addr}"),
            5_000,
            Some("reader".to_string()),
            Some("admin".to_string()),
        )?;

        let _ = client.post_switchover(Some("node-b".to_string())).await?;
        let request = handle_request(handle).await?;
        assert_eq!(request.method, "POST");
        assert_eq!(request.path, "/switchover");
        assert_eq!(
            String::from_utf8_lossy(&request.body),
            r#"{"switchover_to":"node-b"}"#
        );
        Ok(())
    }

    #[tokio::test]
    async fn non_2xx_maps_to_api_status_error() -> Result<(), CliError> {
        let (addr, _handle) = spawn_server(http_response(403, "forbidden")).await?;
        let client = CliApiClient::new(
            format!("http://{addr}"),
            5_000,
            Some("reader".to_string()),
            Some("admin".to_string()),
        )?;

        let result = client.get_ha_state().await;
        match result {
            Err(CliError::ApiStatus { status, body }) => {
                assert_eq!(status, 403);
                assert_eq!(body, "forbidden");
            }
            Err(other) => {
                return Err(CliError::Decode(format!(
                    "expected ApiStatus error, got {other}"
                )));
            }
            Ok(_) => {
                return Err(CliError::Decode(
                    "expected failure for non-2xx response".to_string(),
                ));
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn malformed_json_maps_to_decode_error() -> Result<(), CliError> {
        let (addr, _handle) = spawn_server(http_response(200, "{not-json")).await?;
        let client = CliApiClient::new(
            format!("http://{addr}"),
            5_000,
            Some("reader".to_string()),
            Some("admin".to_string()),
        )?;

        let result = client.get_ha_state().await;
        match result {
            Err(CliError::Decode(_)) => Ok(()),
            Err(other) => Err(CliError::Decode(format!(
                "expected decode error, got {other}"
            ))),
            Ok(_) => Err(CliError::Decode(
                "expected decode failure for malformed json".to_string(),
            )),
        }
    }

    #[tokio::test]
    async fn connection_refused_maps_to_transport_error() -> Result<(), CliError> {
        let addr = reserve_unused_addr().await?;
        let client = CliApiClient::new(
            format!("http://{addr}"),
            200,
            Some("reader".to_string()),
            Some("admin".to_string()),
        )?;

        let result = client.get_ha_state().await;
        match result {
            Err(CliError::Transport(_)) => Ok(()),
            Err(other) => Err(CliError::Decode(format!(
                "expected transport error, got {other}"
            ))),
            Ok(_) => Err(CliError::Decode(
                "expected transport failure on unused port".to_string(),
            )),
        }
    }

    async fn reserve_unused_addr() -> Result<SocketAddr, CliError> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|err| CliError::Transport(format!("bind failed: {err}")))?;
        listener
            .local_addr()
            .map_err(|err| CliError::Transport(format!("local_addr failed: {err}")))
    }

    async fn spawn_server(
        response: String,
    ) -> Result<
        (
            SocketAddr,
            tokio::task::JoinHandle<Result<RecordedRequest, CliError>>,
        ),
        CliError,
    > {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|err| CliError::Transport(format!("bind failed: {err}")))?;
        let addr = listener
            .local_addr()
            .map_err(|err| CliError::Transport(format!("local_addr failed: {err}")))?;

        let handle = tokio::spawn(async move {
            let (mut stream, _peer) = listener
                .accept()
                .await
                .map_err(|err| CliError::Transport(format!("accept failed: {err}")))?;
            let request = read_http_request(&mut stream).await?;
            stream
                .write_all(response.as_bytes())
                .await
                .map_err(|err| CliError::Transport(format!("response write failed: {err}")))?;
            stream
                .shutdown()
                .await
                .map_err(|err| CliError::Transport(format!("shutdown failed: {err}")))?;
            Ok(request)
        });

        Ok((addr, handle))
    }

    async fn handle_request(
        handle: tokio::task::JoinHandle<Result<RecordedRequest, CliError>>,
    ) -> Result<RecordedRequest, CliError> {
        match handle.await {
            Ok(result) => result,
            Err(err) => Err(CliError::Transport(format!("server task failed: {err}"))),
        }
    }

    async fn read_http_request(
        stream: &mut tokio::net::TcpStream,
    ) -> Result<RecordedRequest, CliError> {
        let mut buffer = Vec::new();
        let mut temp = [0u8; 1024];

        loop {
            let read = stream
                .read(&mut temp)
                .await
                .map_err(|err| CliError::Transport(format!("request read failed: {err}")))?;
            if read == 0 {
                break;
            }
            buffer.extend_from_slice(&temp[..read]);

            if let Some(header_end) = find_header_end(&buffer) {
                let content_length = parse_content_length(&buffer[..header_end])?;
                if buffer.len() >= header_end + content_length {
                    break;
                }
            }
        }

        parse_http_request(&buffer)
    }

    fn parse_http_request(buffer: &[u8]) -> Result<RecordedRequest, CliError> {
        let header_end = find_header_end(buffer).ok_or_else(|| {
            CliError::Decode("request parse failed: missing header terminator".to_string())
        })?;

        let header_text = std::str::from_utf8(&buffer[..header_end]).map_err(|err| {
            CliError::Decode(format!("request parse failed: invalid utf8 headers: {err}"))
        })?;
        let mut lines = header_text.split("\r\n");
        let request_line = lines.next().ok_or_else(|| {
            CliError::Decode("request parse failed: missing request line".to_string())
        })?;

        let mut request_parts = request_line.split_whitespace();
        let method = request_parts
            .next()
            .ok_or_else(|| CliError::Decode("missing request method".to_string()))?
            .to_string();
        let path = request_parts
            .next()
            .ok_or_else(|| CliError::Decode("missing request path".to_string()))?
            .to_string();

        let mut headers = Vec::new();
        for line in lines {
            if line.is_empty() {
                continue;
            }
            if let Some((name, value)) = line.split_once(':') {
                headers.push((name.trim().to_string(), value.trim().to_string()));
            }
        }

        let content_length = parse_content_length(&buffer[..header_end])?;
        let body_end = header_end
            .checked_add(content_length)
            .ok_or_else(|| CliError::Decode("request body length overflow".to_string()))?;
        if body_end > buffer.len() {
            return Err(CliError::Decode(
                "request parse failed: body shorter than content-length".to_string(),
            ));
        }

        Ok(RecordedRequest {
            method,
            path,
            headers,
            body: buffer[header_end..body_end].to_vec(),
        })
    }

    fn parse_content_length(headers: &[u8]) -> Result<usize, CliError> {
        let text = std::str::from_utf8(headers)
            .map_err(|err| CliError::Decode(format!("header utf8 decode failed: {err}")))?;
        for line in text.split("\r\n") {
            if let Some((name, value)) = line.split_once(':') {
                if name.eq_ignore_ascii_case("content-length") {
                    let parsed = value.trim().parse::<usize>().map_err(|err| {
                        CliError::Decode(format!("content-length parse failed: {err}"))
                    })?;
                    return Ok(parsed);
                }
            }
        }
        Ok(0)
    }

    fn find_header_end(buffer: &[u8]) -> Option<usize> {
        buffer
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|value| value + 4)
    }

    fn http_response(status_code: u16, body: &str) -> String {
        let reason = match status_code {
            200 => "OK",
            202 => "Accepted",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Status",
        };
        format!(
            "HTTP/1.1 {status_code} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }

    fn assert_header(
        headers: &[(String, String)],
        expected_name: &str,
        expected_value: &str,
    ) -> Result<(), CliError> {
        let found = headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(expected_name))
            .map(|(_, value)| value.as_str());
        match found {
            Some(value) if value == expected_value => Ok(()),
            Some(value) => Err(CliError::Decode(format!(
                "header mismatch for {expected_name}: expected {expected_value}, got {value}"
            ))),
            None => Err(CliError::Decode(format!(
                "missing required header {expected_name}"
            ))),
        }
    }
}
