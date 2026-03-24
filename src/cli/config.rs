use crate::{
    cli::{
        args::Cli,
        client::{CliApiClientConfig, CliAuthConfig},
        error::CliError,
    },
    config_v2::{
        load_operator_config, types::OperatorClientTlsConfig, OperatorConfigV2,
        PgtmApiTransportExpectation,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OperatorContext {
    pub(crate) api_client: CliApiClientConfig,
    pub(crate) postgres_client_tls: OperatorClientTlsConfig,
    pub(crate) api_auth_enabled: bool,
}

pub(crate) fn resolve_operator_context(cli: &Cli) -> Result<OperatorContext, CliError> {
    let config = cli
        .config
        .as_ref()
        .map(|path| {
            load_operator_config(path.as_path()).map_err(|err| CliError::Config(err.to_string()))
        })
        .transpose()?;
    let config = config.as_ref();

    let base_url = resolve_base_url(cli.base_url.as_deref(), config)?;
    let read_token = normalize_optional_token(cli.read_token.as_deref()).or_else(|| {
        config.and_then(|config| {
            config
                .read_token
                .as_ref()
                .map(|token| token.as_str().to_string())
        })
    });
    let admin_token = normalize_optional_token(cli.admin_token.as_deref()).or_else(|| {
        config.and_then(|config| {
            config
                .admin_token
                .as_ref()
                .map(|token| token.as_str().to_string())
        })
    });
    let api_auth_enabled = config
        .map(OperatorConfigV2::api_auth_enabled)
        .unwrap_or(false);

    let api_client_tls = if base_url.scheme() == "https" {
        resolve_client_tls(config)
    } else {
        OperatorClientTlsConfig::default()
    };
    let postgres_client_tls = resolve_client_tls(config);

    Ok(OperatorContext {
        api_client: CliApiClientConfig {
            base_url,
            timeout_ms: cli.timeout_ms,
            auth: CliAuthConfig {
                read_token,
                admin_token,
            },
            tls: api_client_tls,
            resolve_to: config.and_then(|config| config.resolve_to),
        },
        postgres_client_tls,
        api_auth_enabled,
    })
}

fn resolve_base_url(
    override_base_url: Option<&str>,
    config: Option<&OperatorConfigV2>,
) -> Result<reqwest::Url, CliError> {
    if let Some(raw) = override_base_url {
        let url = reqwest::Url::parse(raw.trim())
            .map_err(|err| CliError::RequestBuild(format!("invalid --base-url value: {err}")))?;
        validate_expected_transport(&url, config.and_then(|config| config.expected_transport))?;
        return Ok(url);
    }

    match config {
        Some(config) => config.base_url.clone().ok_or_else(|| {
            CliError::Config(
                "set `api.base_url` in the operator config or pass `--base-url <URL>`".to_string(),
            )
        }),
        None => Err(CliError::Config(
            "either `-c <PATH>` or `--base-url <URL>` must be provided".to_string(),
        )),
    }
}

fn validate_expected_transport(
    url: &reqwest::Url,
    expected_transport: Option<PgtmApiTransportExpectation>,
) -> Result<(), CliError> {
    let Some(expected_transport) = expected_transport else {
        return Ok(());
    };

    if expected_transport.matches_url(url) {
        return Ok(());
    }

    Err(CliError::Config(format!(
        "operator config expects `{}` API transport, but resolved base URL uses `{}`",
        expected_transport.scheme(),
        url.scheme()
    )))
}

fn resolve_client_tls(config: Option<&OperatorConfigV2>) -> OperatorClientTlsConfig {
    config
        .map(|config| config.client_tls.clone())
        .unwrap_or_default()
}

fn normalize_optional_token(value: Option<&str>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::Duration};

    use axum::{routing::get, Json, Router};
    use axum_server::tls_rustls::RustlsConfig;

    use super::resolve_operator_context;
    use crate::api::NodeState;
    use crate::cli::args::{Cli, Command};
    use crate::cli::client::CliApiClient;
    use crate::dcs::DcsSnapshot;
    use crate::dev_support::{
        test_fs::{unique_test_dir, write_text_file},
        tls::{
            build_server_config_with_client_auth, generate_ca, generate_leaf_cert,
            TestSubjectAltName,
        },
    };
    use crate::ha::state::HaState;
    use crate::pginfo::state::PgInfoState;
    use crate::process::state::ProcessState;
    use crate::state::{ClusterName, MemberId, NodeIdentity, ScopeName, WorkerStatus};

    fn base_cli() -> Cli {
        Cli {
            config: None,
            base_url: Some("http://127.0.0.1:18081".to_string()),
            read_token: None,
            admin_token: None,
            timeout_ms: 5_000,
            json: false,
            verbose: false,
            watch: false,
            command: Some(Command::Status),
        }
    }

    #[test]
    fn resolve_context_uses_cli_overrides_without_config() -> Result<(), String> {
        let cli = base_cli();
        let ctx = resolve_operator_context(&cli).map_err(|err| err.to_string())?;
        if ctx.api_client.base_url.as_str() != "http://127.0.0.1:18081/" {
            return Err(format!("unexpected base URL {}", ctx.api_client.base_url));
        }
        Ok(())
    }

    #[test]
    fn resolve_context_requires_base_url_when_config_omits_it() -> Result<(), String> {
        let path = write_temp_config(
            r##"
[api]
"##,
        )?;
        let cli = Cli {
            config: Some(path.clone()),
            base_url: None,
            read_token: None,
            admin_token: None,
            timeout_ms: 5_000,
            json: false,
            verbose: false,
            watch: false,
            command: Some(Command::Status),
        };
        let err = resolve_operator_context(&cli);
        let _ = std::fs::remove_file(path);
        match err {
            Err(err) if err.to_string().contains("set `api.base_url`") => Ok(()),
            Err(err) => Err(format!("unexpected error: {err}")),
            Ok(_) => Err("expected resolution failure".to_string()),
        }
    }

    #[test]
    fn resolve_context_loads_tokens_and_tls_from_config() -> Result<(), String> {
        let dir = unique_test_dir("cli-config", "api-tls")?;
        let ca_path = write_text_file(dir.as_path(), "api-ca.pem", "ca-cert")?;
        let path = write_temp_config(format!(
            r##"
[api]
base_url = "https://127.0.0.1:8443"

[api.auth]
type = "role_tokens"
read_token = {{ type = "string", value = "read-token" }}
admin_token = {{ type = "string", value = "admin-token" }}

[api.tls]
ca_cert = {{ path = "{}" }}
"##,
            ca_path.display()
        ))?;
        let cli = Cli {
            config: Some(path.clone()),
            base_url: None,
            read_token: None,
            admin_token: None,
            timeout_ms: 5_000,
            json: false,
            verbose: false,
            watch: false,
            command: Some(Command::Status),
        };
        let ctx = resolve_operator_context(&cli).map_err(|err| err.to_string())?;
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(ca_path);
        let read_token = ctx
            .api_client
            .auth
            .read_token
            .as_deref()
            .map(str::to_string)
            .ok_or_else(|| "read token missing".to_string())?;
        if read_token != "read-token" {
            return Err("read token did not resolve".to_string());
        }
        let admin_token = ctx
            .api_client
            .auth
            .admin_token
            .as_deref()
            .map(str::to_string)
            .ok_or_else(|| "admin token missing".to_string())?;
        if admin_token != "admin-token" {
            return Err("admin token did not resolve".to_string());
        }
        if ctx.api_client.tls.ca_cert.is_none() {
            return Err("ca cert path did not resolve".to_string());
        }
        Ok(())
    }

    #[test]
    fn resolve_context_preserves_postgres_tls_paths() -> Result<(), String> {
        let dir = unique_test_dir("cli-config", "postgres-tls")?;
        let ca_path = write_text_file(dir.as_path(), "postgres-ca.pem", "ca-cert")?;
        let identity_cert_path =
            write_text_file(dir.as_path(), "postgres-cert.pem", "client-cert")?;
        let identity_key_path = write_text_file(dir.as_path(), "postgres-key.pem", "client-key")?;

        let path = write_temp_config(format!(
            r#"
[api]
base_url = "https://127.0.0.1:8443"

[api.tls]
ca_cert = {{ path = "{}" }}
identity = {{ cert = {{ path = "{}" }}, key = {{ path = "{}" }} }}
"#,
            ca_path.display(),
            identity_cert_path.display(),
            identity_key_path.display()
        ))?;
        let cli = Cli {
            config: Some(path.clone()),
            base_url: None,
            read_token: None,
            admin_token: None,
            timeout_ms: 5_000,
            json: false,
            verbose: false,
            watch: false,
            command: Some(Command::Status),
        };
        let ctx = resolve_operator_context(&cli).map_err(|err| err.to_string())?;

        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(ca_path);
        let _ = std::fs::remove_file(identity_cert_path);
        let _ = std::fs::remove_file(identity_key_path);

        if ctx.postgres_client_tls.ca_cert.is_none() {
            return Err("expected postgres CA path to be preserved".to_string());
        }
        if ctx
            .postgres_client_tls
            .identity
            .as_ref()
            .map(|tls| &tls.cert)
            .is_none()
        {
            return Err("expected postgres client cert path to be preserved".to_string());
        }
        if ctx
            .postgres_client_tls
            .identity
            .as_ref()
            .map(|tls| &tls.key)
            .is_none()
        {
            return Err("expected postgres client key path to be preserved".to_string());
        }
        Ok(())
    }

    #[test]
    fn resolve_context_rejects_base_url_that_violates_expected_transport() -> Result<(), String> {
        let path = write_temp_config(
            r#"
[api]
base_url = "http://127.0.0.1:8443"
expected_transport = "https"
"#,
        )?;
        let cli = Cli {
            config: Some(path.clone()),
            base_url: None,
            read_token: None,
            admin_token: None,
            timeout_ms: 5_000,
            json: false,
            verbose: false,
            watch: false,
            command: Some(Command::Status),
        };

        let err = resolve_operator_context(&cli);
        let _ = std::fs::remove_file(path);

        match err {
            Err(err) if err.to_string().contains("expects `https` API transport") => Ok(()),
            Err(err) => Err(format!("unexpected error: {err}")),
            Ok(_) => Err("expected transport mismatch failure".to_string()),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn resolve_context_uses_https_resolve_to_for_non_resolvable_host() -> Result<(), String> {
        let server_ca = generate_ca("server-ca").map_err(|err| err.to_string())?;
        let client_ca = generate_ca("client-ca").map_err(|err| err.to_string())?;
        let server_cert = generate_leaf_cert(
            "node-b",
            &[TestSubjectAltName::Dns("node-b".to_string())],
            false,
            server_ca.issuer(),
            false,
        )
        .map_err(|err| err.to_string())?;
        let client_cert = generate_leaf_cert(
            "observer",
            &[TestSubjectAltName::Dns("localhost".to_string())],
            false,
            client_ca.issuer(),
            true,
        )
        .map_err(|err| err.to_string())?;

        let dir = unique_test_dir("cli-config", "https-resolve-to")?;
        let ca_cert = write_text_file(dir.as_path(), "ca.crt", server_ca.cert.cert_pem.as_str())?;
        let client_cert_path =
            write_text_file(dir.as_path(), "client.crt", client_cert.cert_pem.as_str())?;
        let client_key_path =
            write_text_file(dir.as_path(), "client.key", client_cert.key_pem.as_str())?;

        let server_config =
            build_server_config_with_client_auth(&server_cert, &server_ca.cert, &client_ca.cert)
                .map_err(|err| err.to_string())?;
        let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .map_err(|err| format!("bind test listener failed: {err}"))?;
        let listen_addr = listener
            .local_addr()
            .map_err(|err| format!("read test listener addr failed: {err}"))?;
        drop(listener);
        let server = tokio::spawn(async move {
            let state = NodeState {
                identity: NodeIdentity {
                    cluster_name: ClusterName("cluster-a".to_string()),
                    scope: ScopeName("scope-a".to_string()),
                    member_id: MemberId("node-b".to_string()),
                },
                pg: PgInfoState::starting(),
                process: ProcessState::starting(),
                dcs: DcsSnapshot::starting(),
                ha: HaState::initial(WorkerStatus::Starting),
            };
            axum_server::bind_rustls(listen_addr, RustlsConfig::from_config(server_config))
                .serve(
                    Router::new()
                        .route(
                            "/state",
                            get(move || {
                                let state = state.clone();
                                async move { Json(state) }
                            }),
                        )
                        .into_make_service(),
                )
                .await
        });

        let path = write_temp_config(format!(
            r#"
[api]
base_url = "https://node-b:{port}"
expected_transport = "https"
resolve_to = "127.0.0.1:{port}"

[api.tls]
ca_cert = {{ path = "{ca_path}" }}
identity = {{ cert = {{ path = "{client_cert_path}" }}, key = {{ path = "{client_key_path}" }} }}
"#,
            port = listen_addr.port(),
            ca_path = ca_cert.display(),
            client_cert_path = client_cert_path.display(),
            client_key_path = client_key_path.display(),
        ))?;
        let cli = Cli {
            config: Some(path.clone()),
            base_url: None,
            read_token: None,
            admin_token: None,
            timeout_ms: 5_000,
            json: false,
            verbose: false,
            watch: false,
            command: Some(Command::Status),
        };

        let ctx = resolve_operator_context(&cli).map_err(|err| err.to_string())?;
        let client = CliApiClient::from_config(ctx.api_client).map_err(|err| err.to_string())?;
        let state = {
            let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
            loop {
                match client.get_state().await {
                    Ok(state) => break state,
                    Err(_) if tokio::time::Instant::now() < deadline => {
                        tokio::time::sleep(Duration::from_millis(25)).await;
                    }
                    Err(err) => return Err(err.to_string()),
                }
            }
        };

        server.abort();
        let _ = server.await;
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_dir_all(dir);

        if state.identity.member_id.0 != "node-b" {
            return Err(format!(
                "expected state identity for node-b, got {}",
                state.identity.member_id.0
            ));
        }

        Ok(())
    }

    fn write_temp_config(contents: impl AsRef<str>) -> Result<PathBuf, String> {
        let path = std::env::temp_dir().join(format!(
            "pgtm-config-{}-{}.toml",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|err| err.to_string())?
                .as_nanos()
        ));
        std::fs::write(&path, contents.as_ref()).map_err(|err| err.to_string())?;
        Ok(path)
    }
}
