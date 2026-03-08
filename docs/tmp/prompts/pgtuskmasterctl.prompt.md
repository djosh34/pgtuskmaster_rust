Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Revise an existing reference page so it stays strictly in Diataxis reference form.

[Page path]
- docs/src/reference/pgtuskmasterctl.md

[Page goal]
- Reference the pgtuskmasterctl CLI command tree, HTTP client mapping, output rendering, token selection, and exit-code behavior.

[Audience]
- Operators and contributors who need accurate repo-backed facts while working with pgtuskmaster.

[User need]
- Consult the machinery surface, data model, constraints, constants, and behavior without being taught procedures or background explanations.

[mdBook context]
- This is an mdBook page under docs/src/reference/.
- Keep headings and lists suitable for mdBook.
- Do not add verification notes, scratch notes, or commentary about how the page was produced.

[Diataxis guidance]
- This page must stay in the reference quadrant: cognition plus application.
- Describe and only describe.
- Structure the page to mirror the machinery, not a guessed workflow.
- Use neutral, technical language.
- Examples are allowed only when they illustrate the surface concisely.
- Do not include step-by-step operations, recommendations, rationale, or explanations of why the design exists.
- If action or explanation seems necessary, keep the page neutral and mention the boundary without turning the page into a how-to or explanation article.

[Required structure]
- Overview\n- Binary entrypoint\n- Global options\n- Command tree\n- Client construction\n- HTTP mapping\n- Output rendering\n- Exit behavior

[Output requirements]
- Preserve or improve the title so it matches the machinery.
- Prefer compact tables where they clarify enums, fields, constants, routes, or error variants.
- Include only facts supported by the supplied source excerpts.
- If the current page contains unsupported claims, remove or correct them rather than hedging.

[Existing page to revise]

# pgtuskmasterctl Reference

CLI surface, HTTP mapping, output rendering, token selection, and exit behavior for the PGTuskMaster admin tool.

## Binary

| Item | Value |
|---|---|
| Source | `src/bin/pgtuskmasterctl.rs` |
| Runtime | `#[tokio::main(flavor = "current_thread")]` |
| Entry point | `main()` parses `Cli::parse()` and awaits `pgtuskmaster_rust::cli::run(cli)` |
| Success output | `println!` of the returned string |
| Error output | `eprintln!` of the formatted error |
| Exit code | `0` on success, otherwise `err.exit_code()` |

## Global Options

| Option | Type | Default | Environment | Description |
|---|---|---|---|---|
| `--base-url` | `String` | `http://127.0.0.1:8080` | none | HTTP base URL for API requests |
| `--read-token` | `Option<String>` | none | `PGTUSKMASTER_READ_TOKEN` | Token for read operations |
| `--admin-token` | `Option<String>` | none | `PGTUSKMASTER_ADMIN_TOKEN` | Token for admin operations |
| `--timeout-ms` | `u64` | `5000` | none | Request timeout in milliseconds |
| `--output` | `json` or `text` | `json` | none | Output format |

Configured token strings are trimmed. Blank token strings become `None`.

## Command Tree

```text
ha state
ha switchover clear
ha switchover request --requested-by <STRING>
```

The `--requested-by` argument is required for `ha switchover request`.

## Client Construction

`cli::run` calls `CliApiClient::new(base_url, timeout_ms, read_token, admin_token)`, dispatches the selected command, then passes the result to `output::render_output(...)`.

| Step | Behavior |
|---|---|
| Base URL | Trimmed before URL parsing |
| Timeout | `Duration::from_millis(timeout_ms)` |
| HTTP pool | `pool_max_idle_per_host(0)` |
| Token normalization | Trimmed; blanks become `None` |
| URL parse or client build failure | `CliError::RequestBuild(...)` |

## HTTP Mapping

| Command | Method | Path | Token role | Expected status | Request body |
|---|---|---|---|---|---|
| `ha state` | `GET` | `/ha/state` | read | `200 OK` | none |
| `ha switchover clear` | `DELETE` | `/ha/switchover` | admin | `202 Accepted` | none |
| `ha switchover request` | `POST` | `/switchover` | admin | `202 Accepted` | `{"requested_by":"..."}` |

Read requests use `read_token` when present, otherwise fall back to `admin_token`. Admin requests use `admin_token` only.

Unexpected HTTP status returns `CliError::ApiStatus { status, body }`. If reading the error response body fails, `body` becomes `<failed to read response body: ...>`.

## Output Rendering

`--output json` uses `serde_json::to_string_pretty` over an untagged enum containing `AcceptedResponse` or `HaStateResponse`.

`--output text` renders:

| Payload | Format |
|---|---|
| `AcceptedResponse` | `accepted=<bool>` |
| `HaStateResponse` | newline-separated `key=value` lines for `cluster_name`, `scope`, `self_member_id`, `leader`, `switchover_requested_by`, `member_count`, `dcs_trust`, `ha_phase`, `ha_tick`, `ha_decision`, and `snapshot_sequence` |

Missing `leader` and `switchover_requested_by` render as `<none>`.

`ha_decision` text forms:

```text
no_change
wait_for_postgres(start_requested=..., leader_member_id=...)
wait_for_dcs_trust
attempt_leadership
follow_leader(leader_member_id=...)
become_primary(promote=...)
step_down(reason=..., release_leader_lease=..., clear_switchover=..., fence=...)
recover_replica(strategy=...)
fence_node
release_leader_lease(reason=...)
enter_fail_safe(release_leader_lease=...)
```

## Exit Behavior

| Condition | Exit code |
|---|---|
| Success | `0` |
| Clap usage failure before `cli::run` | `2` |
| `CliError::RequestBuild` | `3` |
| `CliError::Transport` | `3` |
| `CliError::ApiStatus` | `4` |
| `CliError::Decode` | `5` |
| `CliError::Output` | `5` |

The command surface and exit mapping are exercised by `tests/cli_binary.rs`, `src/cli/args.rs`, `src/cli/client.rs`, `src/cli/output.rs`, and `src/cli/mod.rs`.

[Repo facts and source excerpts]

--- BEGIN FILE: src/bin/pgtuskmasterctl.rs ---
use std::process::ExitCode;

use clap::Parser;
use pgtuskmaster_rust::cli::args::Cli;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match pgtuskmaster_rust::cli::run(cli).await {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            err.exit_code()
        }
    }
}

--- END FILE: src/bin/pgtuskmasterctl.rs ---

--- BEGIN FILE: src/cli/args.rs ---
use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    Text,
}

#[derive(Clone, Debug, Parser)]
#[command(name = "pgtuskmasterctl")]
#[command(about = "HA admin CLI for PGTuskMaster API")]
pub struct Cli {
    #[arg(long, default_value = "http://127.0.0.1:8080")]
    pub base_url: String,
    #[arg(long, env = "PGTUSKMASTER_READ_TOKEN")]
    pub read_token: Option<String>,
    #[arg(long, env = "PGTUSKMASTER_ADMIN_TOKEN")]
    pub admin_token: Option<String>,
    #[arg(long, default_value_t = 5_000)]
    pub timeout_ms: u64,
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    pub output: OutputFormat,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    Ha(HaArgs),
}

#[derive(Clone, Debug, Args)]
pub struct HaArgs {
    #[command(subcommand)]
    pub command: HaCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum HaCommand {
    State,
    Switchover(SwitchoverArgs),
}

#[derive(Clone, Debug, Args)]
pub struct SwitchoverArgs {
    #[command(subcommand)]
    pub command: SwitchoverCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum SwitchoverCommand {
    Clear,
    Request(RequestSwitchoverArgs),
}

#[derive(Clone, Debug, Args)]
pub struct RequestSwitchoverArgs {
    #[arg(long)]
    pub requested_by: String,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use crate::cli::args::{Cli, Command, HaCommand, OutputFormat, SwitchoverCommand};

    #[test]
    fn parse_ha_state_with_defaults() -> Result<(), String> {
        let cli = Cli::try_parse_from(["pgtuskmasterctl", "ha", "state"])
            .map_err(|err| format!("parse should succeed: {err}"))?;

        assert_eq!(cli.base_url, "http://127.0.0.1:8080");
        assert_eq!(cli.timeout_ms, 5_000);
        assert_eq!(cli.output, OutputFormat::Json);

        match cli.command {
            Command::Ha(ha) => match ha.command {
                HaCommand::State => Ok(()),
                _ => Err("expected ha state command".to_string()),
            },
        }
    }

    #[test]
    fn parse_requires_requested_by_for_switchover_request() {
        let parsed = Cli::try_parse_from(["pgtuskmasterctl", "ha", "switchover", "request"]);
        assert!(parsed.is_err(), "requested-by is required");
    }

    #[test]
    fn parse_full_switchover_write_command() -> Result<(), String> {
        let cli = Cli::try_parse_from([
            "pgtuskmasterctl",
            "--base-url",
            "https://cluster.example",
            "--timeout-ms",
            "1234",
            "--output",
            "text",
            "ha",
            "switchover",
            "request",
            "--requested-by",
            "node-a",
        ])
        .map_err(|err| format!("parse should succeed: {err}"))?;

        assert_eq!(cli.base_url, "https://cluster.example");
        assert_eq!(cli.timeout_ms, 1234);
        assert_eq!(cli.output, OutputFormat::Text);

        match cli.command {
            Command::Ha(ha) => match ha.command {
                HaCommand::Switchover(switchover) => match switchover.command {
                    SwitchoverCommand::Request(request) => {
                        assert_eq!(request.requested_by, "node-a");
                        Ok(())
                    }
                    _ => Err("expected switchover request".to_string()),
                },
                _ => Err("expected switchover command".to_string()),
            },
        }
    }

    #[test]
    fn parse_switchover_request() -> Result<(), String> {
        let cli = Cli::try_parse_from([
            "pgtuskmasterctl",
            "ha",
            "switchover",
            "request",
            "--requested-by",
            "node-b",
        ])
        .map_err(|err| format!("parse should succeed: {err}"))?;

        match cli.command {
            Command::Ha(ha) => match ha.command {
                HaCommand::Switchover(switchover) => match switchover.command {
                    SwitchoverCommand::Request(request) => {
                        assert_eq!(request.requested_by, "node-b");
                        Ok(())
                    }
                    _ => Err("expected switchover request".to_string()),
                },
                _ => Err("expected switchover command".to_string()),
            },
        }
    }

    #[test]
    fn parse_env_token_fallbacks() -> Result<(), String> {
        let read_var = "PGTUSKMASTER_READ_TOKEN";
        let admin_var = "PGTUSKMASTER_ADMIN_TOKEN";

        std::env::set_var(read_var, "reader");
        std::env::set_var(admin_var, "admin");

        let parsed = Cli::try_parse_from(["pgtuskmasterctl", "ha", "state"])
            .map_err(|err| format!("parse should succeed: {err}"));

        std::env::remove_var(read_var);
        std::env::remove_var(admin_var);

        let cli = parsed?;
        assert_eq!(cli.read_token.as_deref(), Some("reader"));
        assert_eq!(cli.admin_token.as_deref(), Some("admin"));
        Ok(())
    }
}

--- END FILE: src/cli/args.rs ---

--- BEGIN FILE: src/cli/client.rs ---
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
    requested_by: String,
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
        requested_by: String,
    ) -> Result<AcceptedResponse, CliError> {
        let body = SwitchoverRequestInput { requested_by };
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
        let response_body = r#"{"cluster_name":"cluster-a","scope":"scope-a","self_member_id":"node-a","leader":null,"switchover_requested_by":null,"member_count":1,"dcs_trust":"full_quorum","ha_phase":"primary","ha_tick":1,"ha_decision":{"kind":"become_primary","promote":true},"snapshot_sequence":10}"#;
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
        let response_body = r#"{"cluster_name":"cluster-a","scope":"scope-a","self_member_id":"node-a","leader":null,"switchover_requested_by":null,"member_count":1,"dcs_trust":"full_quorum","ha_phase":"primary","ha_tick":1,"ha_decision":{"kind":"become_primary","promote":true},"snapshot_sequence":10}"#;
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

--- END FILE: src/cli/client.rs ---

--- BEGIN FILE: src/cli/output.rs ---
use serde::Serialize;

use crate::cli::{
    args::OutputFormat,
    client::{AcceptedResponse, HaDecisionResponse, HaStateResponse},
    error::CliError,
    CommandOutput,
};

pub fn render_output(
    command_output: &CommandOutput,
    format: OutputFormat,
) -> Result<String, CliError> {
    match format {
        OutputFormat::Json => render_json(command_output),
        OutputFormat::Text => Ok(render_text(command_output)),
    }
}

fn render_json(command_output: &CommandOutput) -> Result<String, CliError> {
    #[derive(Serialize)]
    #[serde(untagged)]
    enum OutputRef<'a> {
        State(&'a HaStateResponse),
        Accepted(&'a AcceptedResponse),
    }

    let payload = match command_output {
        CommandOutput::HaState(value) => OutputRef::State(value.as_ref()),
        CommandOutput::Accepted(value) => OutputRef::Accepted(value),
    };

    serde_json::to_string_pretty(&payload)
        .map_err(|err| CliError::Output(format!("json encode failed: {err}")))
}

fn render_text(command_output: &CommandOutput) -> String {
    match command_output {
        CommandOutput::Accepted(value) => format!("accepted={}", value.accepted),
        CommandOutput::HaState(value) => {
            let value = value.as_ref();
            let leader = value.leader.as_deref().unwrap_or("<none>");
            let switchover = value.switchover_requested_by.as_deref().unwrap_or("<none>");
            [
                format!("cluster_name={}", value.cluster_name),
                format!("scope={}", value.scope),
                format!("self_member_id={}", value.self_member_id),
                format!("leader={leader}"),
                format!("switchover_requested_by={switchover}"),
                format!("member_count={}", value.member_count),
                format!("dcs_trust={}", value.dcs_trust),
                format!("ha_phase={}", value.ha_phase),
                format!("ha_tick={}", value.ha_tick),
                format!("ha_decision={}", render_decision_text(&value.ha_decision)),
                format!("snapshot_sequence={}", value.snapshot_sequence),
            ]
            .join("\n")
        }
    }
}

fn render_decision_text(value: &HaDecisionResponse) -> String {
    match value {
        HaDecisionResponse::NoChange => "no_change".to_string(),
        HaDecisionResponse::WaitForPostgres {
            start_requested,
            leader_member_id,
        } => {
            let leader_detail = leader_member_id.as_deref().unwrap_or("none");
            format!(
                "wait_for_postgres(start_requested={start_requested}, leader_member_id={leader_detail})"
            )
        }
        HaDecisionResponse::WaitForDcsTrust => "wait_for_dcs_trust".to_string(),
        HaDecisionResponse::AttemptLeadership => "attempt_leadership".to_string(),
        HaDecisionResponse::FollowLeader { leader_member_id } => {
            format!("follow_leader(leader_member_id={leader_member_id})")
        }
        HaDecisionResponse::BecomePrimary { promote } => {
            format!("become_primary(promote={promote})")
        }
        HaDecisionResponse::StepDown {
            reason,
            release_leader_lease,
            clear_switchover,
            fence,
        } => format!(
            "step_down(reason={reason}, release_leader_lease={release_leader_lease}, clear_switchover={clear_switchover}, fence={fence})"
        ),
        HaDecisionResponse::RecoverReplica { strategy } => {
            format!("recover_replica(strategy={strategy})")
        }
        HaDecisionResponse::FenceNode => "fence_node".to_string(),
        HaDecisionResponse::ReleaseLeaderLease { reason } => {
            format!("release_leader_lease(reason={reason})")
        }
        HaDecisionResponse::EnterFailSafe {
            release_leader_lease,
        } => format!("enter_fail_safe(release_leader_lease={release_leader_lease})"),
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::{
        args::OutputFormat,
        client::{AcceptedResponse, HaDecisionResponse, HaStateResponse},
        output::render_output,
        CommandOutput,
    };

    #[test]
    fn text_output_renders_state_lines() {
        let output = render_output(
            &CommandOutput::HaState(Box::new(HaStateResponse {
                cluster_name: "cluster-a".to_string(),
                scope: "scope-a".to_string(),
                self_member_id: "node-a".to_string(),
                leader: Some("node-a".to_string()),
                switchover_requested_by: None,
                member_count: 3,
                dcs_trust: crate::api::DcsTrustResponse::FullQuorum,
                ha_phase: crate::api::HaPhaseResponse::Primary,
                ha_tick: 9,
                ha_decision: HaDecisionResponse::BecomePrimary { promote: true },
                snapshot_sequence: 77,
            })),
            OutputFormat::Text,
        );
        assert!(output.is_ok(), "text render should succeed");
        let rendered = output.unwrap_or_default();
        assert!(!rendered.is_empty(), "rendered text should not be empty");
        assert!(rendered.contains("cluster_name=cluster-a"));
        assert!(rendered.contains("leader=node-a"));
        assert!(rendered.contains("switchover_requested_by=<none>"));
        assert!(rendered.contains("ha_decision=become_primary(promote=true)"));
    }

    #[test]
    fn json_output_renders_accepted_payload() {
        let output = render_output(
            &CommandOutput::Accepted(AcceptedResponse { accepted: true }),
            OutputFormat::Json,
        );
        assert!(output.is_ok(), "json render should succeed");
        let rendered = output.unwrap_or_default();
        assert!(!rendered.is_empty(), "rendered json should not be empty");
        assert!(rendered.contains("\"accepted\": true"));
    }
}

--- END FILE: src/cli/output.rs ---

--- BEGIN FILE: src/cli/mod.rs ---
pub mod args;
pub mod client;
pub mod error;
pub mod output;

use args::{Cli, Command, HaCommand, SwitchoverCommand};
use client::{AcceptedResponse, CliApiClient, HaStateResponse};
use error::CliError;

pub enum CommandOutput {
    HaState(Box<HaStateResponse>),
    Accepted(AcceptedResponse),
}

pub async fn run(cli: Cli) -> Result<String, CliError> {
    let output_format = cli.output;
    let command = cli.command;
    let client = CliApiClient::new(
        cli.base_url,
        cli.timeout_ms,
        cli.read_token,
        cli.admin_token,
    )?;

    let command_output = match command {
        Command::Ha(ha) => match ha.command {
            HaCommand::State => CommandOutput::HaState(Box::new(client.get_ha_state().await?)),
            HaCommand::Switchover(switchover) => match switchover.command {
                SwitchoverCommand::Clear => {
                    CommandOutput::Accepted(client.delete_switchover().await?)
                }
                SwitchoverCommand::Request(input) => {
                    CommandOutput::Accepted(client.post_switchover(input.requested_by).await?)
                }
            },
        },
    };

    output::render_output(&command_output, output_format)
}

#[cfg(test)]
mod tests {
    use crate::cli::error::CliError;

    #[test]
    fn exit_code_mapping_is_stable() {
        assert_eq!(CliError::Transport("x".to_string()).exit_code(), 3.into());
        assert_eq!(
            CliError::ApiStatus {
                status: 500,
                body: "x".to_string()
            }
            .exit_code(),
            4.into()
        );
        assert_eq!(CliError::Decode("x".to_string()).exit_code(), 5.into());
    }
}

--- END FILE: src/cli/mod.rs ---

--- BEGIN FILE: src/cli/error.rs ---
use std::process::ExitCode;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("transport error: {0}")]
    Transport(String),
    #[error("api request failed with status {status}: {body}")]
    ApiStatus { status: u16, body: String },
    #[error("response decode failed: {0}")]
    Decode(String),
    #[error("request build failed: {0}")]
    RequestBuild(String),
    #[error("output serialization failed: {0}")]
    Output(String),
}

impl CliError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::Transport(_) | Self::RequestBuild(_) => ExitCode::from(3),
            Self::ApiStatus { .. } => ExitCode::from(4),
            Self::Decode(_) | Self::Output(_) => ExitCode::from(5),
        }
    }
}

--- END FILE: src/cli/error.rs ---

--- BEGIN FILE: tests/cli_binary.rs ---
use std::process::Command;

fn write_temp_config(label: &str, toml: &str) -> Result<std::path::PathBuf, String> {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| format!("system time error: {err}"))?
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "pgtuskmaster-cli-config-{label}-{unique}-{}",
        std::process::id()
    ));

    std::fs::write(&path, toml).map_err(|err| format!("write config failed: {err}"))?;
    Ok(path)
}

fn cli_bin_path() -> Result<std::path::PathBuf, String> {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_pgtuskmasterctl") {
        return Ok(std::path::PathBuf::from(path));
    }

    let current = std::env::current_exe().map_err(|err| format!("current_exe failed: {err}"))?;
    let debug_dir = current
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| "failed to derive target/debug directory".to_string())?;
    let mut candidate = debug_dir.join("pgtuskmasterctl");
    if cfg!(windows) {
        candidate.set_extension("exe");
    }
    if candidate.exists() {
        Ok(candidate)
    } else {
        Err(format!("cli binary not found at {}", candidate.display()))
    }
}

fn node_bin_path() -> Result<std::path::PathBuf, String> {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_pgtuskmaster") {
        return Ok(std::path::PathBuf::from(path));
    }

    let current = std::env::current_exe().map_err(|err| format!("current_exe failed: {err}"))?;
    let debug_dir = current
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| "failed to derive target/debug directory".to_string())?;
    let mut candidate = debug_dir.join("pgtuskmaster");
    if cfg!(windows) {
        candidate.set_extension("exe");
    }
    if candidate.exists() {
        Ok(candidate)
    } else {
        Err(format!("node binary not found at {}", candidate.display()))
    }
}

#[test]
fn help_exits_success() -> Result<(), String> {
    let bin = cli_bin_path()?;
    let output = Command::new(&bin)
        .arg("--help")
        .output()
        .map_err(|err| format!("failed to run --help: {err}"))?;

    assert!(
        output.status.success(),
        "--help should exit successfully, got {:?}",
        output.status.code()
    );

    let stdout = String::from_utf8(output.stdout)
        .map_err(|err| format!("stdout utf8 decode failed: {err}"))?;
    assert!(
        stdout.contains("ha"),
        "help output should include ha command"
    );
    Ok(())
}

#[test]
fn missing_required_subcommand_arg_exits_usage_code() -> Result<(), String> {
    let bin = cli_bin_path()?;
    let output = Command::new(&bin)
        .args(["ha", "leader", "set"])
        .output()
        .map_err(|err| format!("failed to run command: {err}"))?;

    assert_eq!(
        output.status.code(),
        Some(2),
        "clap usage failures should exit with code 2"
    );
    Ok(())
}

#[test]
fn state_command_maps_connection_refused_to_exit_3() -> Result<(), String> {
    let bin = cli_bin_path()?;
    let listener =
        std::net::TcpListener::bind("127.0.0.1:0").map_err(|err| format!("bind failed: {err}"))?;
    let addr = listener
        .local_addr()
        .map_err(|err| format!("local_addr failed: {err}"))?;
    drop(listener);

    let output = Command::new(&bin)
        .args([
            "--base-url",
            &format!("http://{addr}"),
            "--timeout-ms",
            "50",
            "ha",
            "state",
        ])
        .output()
        .map_err(|err| format!("failed to run state command: {err}"))?;

    assert_eq!(
        output.status.code(),
        Some(3),
        "transport errors should map to exit code 3"
    );

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("transport error"),
        "stderr should mention transport error"
    );
    Ok(())
}

#[test]
fn node_help_exits_success() -> Result<(), String> {
    let bin = node_bin_path()?;
    let output = Command::new(&bin)
        .arg("--help")
        .output()
        .map_err(|err| format!("failed to run node --help: {err}"))?;

    assert!(
        output.status.success(),
        "--help should exit successfully, got {:?}",
        output.status.code()
    );

    let stdout = String::from_utf8(output.stdout)
        .map_err(|err| format!("stdout utf8 decode failed: {err}"))?;
    assert!(
        stdout.contains("--config"),
        "help output should include --config option"
    );
    Ok(())
}

#[test]
fn node_missing_config_version_prints_explicit_v2_migration_hint() -> Result<(), String> {
    let bin = node_bin_path()?;
    let path = write_temp_config(
        "missing-config-version",
        r#"
[cluster]
name = "cluster-a"
member_id = "member-a"
"#,
    )?;

    let output = Command::new(&bin)
        .args(["--config", path.to_string_lossy().as_ref()])
        .output()
        .map_err(|err| format!("failed to run node with missing config_version: {err}"))?;

    assert_eq!(
        output.status.code(),
        Some(1),
        "invalid configs should exit with code 1"
    );

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("set config_version = \"v2\""),
        "stderr should include explicit v2 migration hint, got: {stderr}"
    );

    let _ = std::fs::remove_file(path);
    Ok(())
}

#[test]
fn node_missing_secure_field_prints_stable_field_path() -> Result<(), String> {
    let bin = node_bin_path()?;
    let path = write_temp_config(
        "missing-process-binaries",
        r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
pg_rewind_timeout_ms = 120000
bootstrap_timeout_ms = 300000
fencing_timeout_ms = 30000

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#,
    )?;

    let output = Command::new(&bin)
        .args(["--config", path.to_string_lossy().as_ref()])
        .output()
        .map_err(|err| format!("failed to run node with invalid config: {err}"))?;

    assert_eq!(
        output.status.code(),
        Some(1),
        "invalid configs should exit with code 1"
    );

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("`process.binaries`"),
        "stderr should mention stable field path, got: {stderr}"
    );

    let _ = std::fs::remove_file(path);
    Ok(())
}

#[test]
fn node_rejects_postgres_role_tls_auth_with_stable_field_path() -> Result<(), String> {
    let bin = node_bin_path()?;
    let path = write_temp_config(
        "postgres-role-tls-auth",
        r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "tls" } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#,
    )?;

    let output = Command::new(&bin)
        .args(["--config", path.to_string_lossy().as_ref()])
        .output()
        .map_err(|err| format!("failed to run node with invalid config: {err}"))?;

    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("`postgres.roles.superuser.auth`"),
        "stderr should mention stable field path, got: {stderr}"
    );

    let _ = std::fs::remove_file(path);
    Ok(())
}

#[test]
fn node_rejects_ssl_mode_requiring_tls_when_postgres_tls_disabled() -> Result<(), String> {
    let bin = node_bin_path()?;
    let path = write_temp_config(
        "postgres-ssl-mode-requires-tls",
        r#"
config_version = "v2"

[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "require" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#,
    )?;

    let output = Command::new(&bin)
        .args(["--config", path.to_string_lossy().as_ref()])
        .output()
        .map_err(|err| format!("failed to run node with invalid config: {err}"))?;

    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("`postgres.local_conn_identity.ssl_mode`"),
        "stderr should mention stable field path, got: {stderr}"
    );

    let _ = std::fs::remove_file(path);
    Ok(())
}

--- END FILE: tests/cli_binary.rs ---

