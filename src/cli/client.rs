use std::{fs, net::SocketAddr, time::Duration};

use reqwest::{Method, StatusCode, Url};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub(crate) use crate::api::{AcceptedResponse, NodeState as NodeStateResponse};
use crate::{
    cli::error::CliError,
    config_v2::types::OperatorClientTlsConfig,
    state::{MemberId, SwitchoverState},
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CliAuthConfig {
    pub(crate) read_token: Option<String>,
    pub(crate) admin_token: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CliApiClientConfig {
    pub(crate) base_url: Url,
    pub(crate) timeout_ms: u64,
    pub(crate) auth: CliAuthConfig,
    pub(crate) tls: OperatorClientTlsConfig,
    pub(crate) resolve_to: Option<SocketAddr>,
}

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

type SwitchoverRequestInput = SwitchoverState;

impl CliApiClient {
    pub(crate) fn from_config(config: CliApiClientConfig) -> Result<Self, CliError> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .pool_max_idle_per_host(0);
        let http = match config.resolve_to {
            Some(resolve_to) => {
                let host = config.base_url.host_str().ok_or_else(|| {
                    CliError::RequestBuild(
                        "API URL did not include a hostname for custom resolution".to_string(),
                    )
                })?;
                http.no_proxy().resolve(host, resolve_to)
            }
            None => http,
        };
        let http = apply_tls_config(http, &config.tls)?;
        let http = http
            .build()
            .map_err(|err| CliError::RequestBuild(format!("build http client failed: {err}")))?;

        Ok(Self {
            base_url: config.base_url,
            http,
            read_token: normalize_token(config.auth.read_token.as_deref()),
            admin_token: normalize_token(config.auth.admin_token.as_deref()),
        })
    }

    pub(crate) async fn get_state(&self) -> Result<NodeStateResponse, CliError> {
        self.send_json_no_body(Method::GET, "/state", AuthRole::Read, StatusCode::OK)
            .await
    }

    pub(crate) async fn delete_switchover(&self) -> Result<AcceptedResponse, CliError> {
        self.send_json_no_body(
            Method::DELETE,
            "/switchover",
            AuthRole::Admin,
            StatusCode::ACCEPTED,
        )
        .await
    }

    pub(crate) async fn post_switchover(
        &self,
        switchover_to: Option<String>,
    ) -> Result<AcceptedResponse, CliError> {
        let body = match switchover_to {
            Some(member_id) => SwitchoverRequestInput::Specific(MemberId(member_id)),
            None => SwitchoverRequestInput::AnyHealthyReplica,
        };
        self.send_json_with_body(
            Method::POST,
            "/switchover",
            AuthRole::Admin,
            &body,
            StatusCode::ACCEPTED,
        )
        .await
    }

    pub(crate) fn base_url(&self) -> &Url {
        &self.base_url
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
        self.send_json_to_url(method, url, role, expected_status)
            .await
    }

    async fn send_json_to_url<T>(
        &self,
        method: Method,
        url: Url,
        role: AuthRole,
        expected_status: StatusCode,
    ) -> Result<T, CliError>
    where
        T: DeserializeOwned,
    {
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

fn apply_tls_config(
    builder: reqwest::ClientBuilder,
    config: &OperatorClientTlsConfig,
) -> Result<reqwest::ClientBuilder, CliError> {
    let builder = if let Some(ca_cert_path) = config.ca_cert.as_ref() {
        let ca_cert_bytes = read_tls_bytes("api.tls.ca_cert", ca_cert_path.as_path())?;
        let certificate = reqwest::Certificate::from_pem(ca_cert_bytes.as_slice())
            .map_err(|err| CliError::RequestBuild(format!("parse CA certificate failed: {err}")))?;
        builder.add_root_certificate(certificate)
    } else {
        builder
    };

    let Some(identity) = config.identity.as_ref() else {
        return Ok(builder);
    };

    let client_cert_bytes = read_tls_bytes("api.tls.identity.cert", identity.cert.as_path())?;
    let client_key_bytes = read_tls_bytes("api.tls.identity.key", identity.key.as_path())?;
    let mut client_identity_pem = Vec::with_capacity(
        client_cert_bytes
            .len()
            .saturating_add(client_key_bytes.len()),
    );
    client_identity_pem.extend_from_slice(client_cert_bytes.as_slice());
    client_identity_pem.extend_from_slice(client_key_bytes.as_slice());
    let identity = reqwest::Identity::from_pem(&client_identity_pem)
        .map_err(|err| CliError::RequestBuild(format!("parse client identity failed: {err}")))?;
    Ok(builder.identity(identity))
}

fn read_tls_bytes(field: &str, path: &std::path::Path) -> Result<Vec<u8>, CliError> {
    fs::read(path).map_err(|err| {
        CliError::RequestBuild(format!(
            "read {field} from {} failed: {err}",
            path.display()
        ))
    })
}

async fn read_json_response<T>(
    response: reqwest::Response,
    expected_status: StatusCode,
) -> Result<T, CliError>
where
    T: DeserializeOwned,
{
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|err| CliError::Transport(err.to_string()))?;

    if status != expected_status {
        return Err(CliError::ApiStatus {
            status: status.as_u16(),
            body,
        });
    }

    serde_json::from_str(&body).map_err(CliError::Decode)
}

fn normalize_token(raw: Option<&str>) -> Option<String> {
    raw.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

#[cfg(test)]
mod tests {
    use std::{
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use axum::{routing::get, Json, Router};
    use axum_server::tls_rustls::RustlsConfig;
    use reqwest::{Method, StatusCode, Url};
    use serde_json::{json, Value};

    use super::{AuthRole, CliApiClient, CliApiClientConfig};
    use crate::{
        cli::client::CliAuthConfig,
        config_v2::types::{OperatorClientTlsConfig, TlsConfig},
        dev_support::tls::{build_adversarial_tls_fixture, build_server_config_with_client_auth},
    };

    fn unique_test_dir(label: &str) -> Result<PathBuf, String> {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("clock error for test dir: {err}"))?
            .as_millis();
        let dir = std::env::temp_dir().join(format!(
            "pgtm-cli-client-{label}-{}-{millis}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir)
            .map_err(|err| format!("create test dir {} failed: {err}", dir.display()))?;
        Ok(dir)
    }

    fn write_pem_file(dir: &Path, name: &str, contents: &str) -> Result<PathBuf, String> {
        let path = dir.join(name);
        std::fs::write(&path, contents)
            .map_err(|err| format!("write {} failed: {err}", path.display()))?;
        Ok(path)
    }

    #[tokio::test(flavor = "current_thread")]
    async fn cli_api_client_loads_tls_material_from_paths() -> Result<(), String> {
        let fixture = build_adversarial_tls_fixture().map_err(|err| err.to_string())?;
        let dir = unique_test_dir("tls-paths")?;
        let ca_cert = write_pem_file(
            dir.as_path(),
            "ca.crt",
            fixture.valid_server_ca.cert.cert_pem.as_str(),
        )?;
        let client_cert = write_pem_file(
            dir.as_path(),
            "client.crt",
            fixture.trusted_client.cert_pem.as_str(),
        )?;
        let client_key = write_pem_file(
            dir.as_path(),
            "client.key",
            fixture.trusted_client.key_pem.as_str(),
        )?;

        let tls = OperatorClientTlsConfig {
            ca_cert: Some(ca_cert),
            identity: Some(TlsConfig {
                cert: client_cert,
                key: client_key,
                ca_cert: None,
            }),
        };

        let server_config = build_server_config_with_client_auth(
            &fixture.valid_server,
            &fixture.valid_server_ca.cert,
            &fixture.trusted_client_ca.cert,
        )
        .map_err(|err| err.to_string())?;
        let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .map_err(|err| format!("bind test listener failed: {err}"))?;
        let listen_addr = listener
            .local_addr()
            .map_err(|err| format!("read test listener addr failed: {err}"))?;
        drop(listener);
        let server = tokio::spawn(async move {
            axum_server::bind_rustls(listen_addr, RustlsConfig::from_config(server_config))
                .serve(
                    Router::new()
                        .route("/state", get(|| async { Json(json!({ "ok": true })) }))
                        .into_make_service(),
                )
                .await
        });

        let client = CliApiClient::from_config(CliApiClientConfig {
            base_url: Url::parse(format!("https://localhost:{}/", listen_addr.port()).as_str())
                .map_err(|err| format!("build test url failed: {err}"))?,
            timeout_ms: 5_000,
            auth: CliAuthConfig {
                read_token: None,
                admin_token: None,
            },
            tls,
            resolve_to: None,
        })
        .map_err(|err| err.to_string())?;

        let response = client
            .send_json_no_body::<Value>(Method::GET, "/state", AuthRole::Read, StatusCode::OK)
            .await
            .map_err(|err| err.to_string())?;
        if response != json!({ "ok": true }) {
            return Err(format!("unexpected response payload: {response}"));
        }

        server.abort();
        let _ = server.await;
        Ok(())
    }
}
