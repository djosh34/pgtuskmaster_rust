use std::{path::PathBuf, time::Duration};

use reqwest::{Method, StatusCode, Url};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub(crate) use crate::api::{AcceptedResponse, NodeState as NodeStateResponse};
use crate::cli::error::CliError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliAuthConfig {
    pub read_token: Option<String>,
    pub admin_token: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CliTlsConfig {
    pub ca_cert_pem: Option<Vec<u8>>,
    pub client_cert_pem: Option<Vec<u8>>,
    pub client_key_pem: Option<Vec<u8>>,
    pub ca_cert_path: Option<PathBuf>,
    pub client_cert_path: Option<PathBuf>,
    pub client_key_path: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliApiClientConfig {
    pub base_url: Url,
    pub timeout_ms: u64,
    pub auth: CliAuthConfig,
    pub tls: CliTlsConfig,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
struct SwitchoverRequestInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    switchover_to: Option<String>,
}

impl CliApiClient {
    pub fn from_config(config: CliApiClientConfig) -> Result<Self, CliError> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .pool_max_idle_per_host(0);
        let http = apply_tls_config(http, &config.tls)?;
        let http = http
            .build()
            .map_err(|err| CliError::RequestBuild(format!("build http client failed: {err}")))?;

        Ok(Self {
            base_url: config.base_url,
            http,
            read_token: normalize_token(config.auth.read_token),
            admin_token: normalize_token(config.auth.admin_token),
        })
    }

    pub(crate) async fn get_state(&self) -> Result<NodeStateResponse, CliError> {
        self.send_json_no_body(Method::GET, "/state", AuthRole::Read, StatusCode::OK)
            .await
    }

    pub async fn delete_switchover(&self) -> Result<AcceptedResponse, CliError> {
        self.send_json_no_body(
            Method::DELETE,
            "/switchover",
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

    pub fn base_url(&self) -> &Url {
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
    config: &CliTlsConfig,
) -> Result<reqwest::ClientBuilder, CliError> {
    let builder = if let Some(ca_cert_pem) = config.ca_cert_pem.as_ref() {
        let certificate = reqwest::Certificate::from_pem(ca_cert_pem)
            .map_err(|err| CliError::RequestBuild(format!("parse CA certificate failed: {err}")))?;
        builder.add_root_certificate(certificate)
    } else {
        builder
    };

    if config.client_cert_pem.is_none() && config.client_key_pem.is_none() {
        return Ok(builder);
    }

    let client_cert_pem = config
        .client_cert_pem
        .as_ref()
        .ok_or_else(|| CliError::RequestBuild("client certificate missing".to_string()))?;
    let client_key_pem = config
        .client_key_pem
        .as_ref()
        .ok_or_else(|| CliError::RequestBuild("client key missing".to_string()))?;
    let mut client_identity_pem =
        Vec::with_capacity(client_cert_pem.len().saturating_add(client_key_pem.len()));
    client_identity_pem.extend_from_slice(client_cert_pem);
    client_identity_pem.extend_from_slice(client_key_pem);
    let identity = reqwest::Identity::from_pem(&client_identity_pem)
        .map_err(|err| CliError::RequestBuild(format!("parse client identity failed: {err}")))?;
    Ok(builder.identity(identity))
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

    serde_json::from_str(&body).map_err(|err| CliError::Decode(err.to_string()))
}

fn normalize_token(raw: Option<String>) -> Option<String> {
    raw.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}
