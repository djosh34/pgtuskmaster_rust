use crate::{
    cli::{
        args::Cli,
        client::{CliApiClientConfig, CliAuthConfig, CliTlsConfig},
        error::CliError,
    },
    config::SecretSource,
    config_v2::{load_operator_config, OperatorConfigV2, PgtmApiTransportExpectation},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OperatorContext {
    pub(crate) api_client: CliApiClientConfig,
    pub(crate) postgres_client_tls: CliTlsConfig,
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

    let base_url = resolve_api_url(cli.base_url.as_deref(), config)?;
    let (config_read_token, config_admin_token, api_auth_enabled) = resolve_config_auth(config)?;
    let read_token = normalize_optional_token(cli.read_token.as_deref()).or(config_read_token);
    let admin_token = normalize_optional_token(cli.admin_token.as_deref()).or(config_admin_token);

    let api_client_tls = if base_url.scheme() == "https" {
        resolve_client_tls(config)?
    } else {
        CliTlsConfig::default()
    };
    let postgres_client_tls = resolve_client_tls(config)?;

    Ok(OperatorContext {
        api_client: CliApiClientConfig {
            base_url,
            timeout_ms: cli.timeout_ms,
            auth: CliAuthConfig {
                read_token: match read_token {
                    Some(value) => SecretSource::String { value },
                    None => SecretSource::None,
                },
                admin_token: match admin_token {
                    Some(value) => SecretSource::String { value },
                    None => SecretSource::None,
                },
            },
            tls: api_client_tls,
            resolve_to: config.and_then(|cfg| cfg.api_resolve_to),
        },
        postgres_client_tls,
        api_auth_enabled,
    })
}

fn resolve_api_url(
    override_base_url: Option<&str>,
    config: Option<&OperatorConfigV2>,
) -> Result<reqwest::Url, CliError> {
    if let Some(raw) = override_base_url {
        let url = reqwest::Url::parse(raw.trim())
            .map_err(|err| CliError::RequestBuild(format!("invalid --base-url value: {err}")))?;
        validate_expected_transport(&url, config)?;
        return Ok(url);
    }

    let Some(config) = config else {
        return Err(CliError::Config(
            "either `-c <PATH>` or `--base-url <URL>` must be provided".to_string(),
        ));
    };

    let base_url = config.api_base_url.clone().ok_or_else(|| {
        CliError::Config(
            "set `api.base_url` in the operator config or pass `--base-url <URL>`".to_string(),
        )
    })?;

    let url = reqwest::Url::parse(base_url.as_str())
        .map_err(|err| CliError::Config(format!("invalid `api.base_url` in operator config: {err}")))?;
    validate_expected_transport(&url, Some(config))?;
    Ok(url)
}

fn validate_expected_transport(
    url: &reqwest::Url,
    config: Option<&OperatorConfigV2>,
) -> Result<(), CliError> {
    let Some(expected_transport) = config.and_then(|cfg| cfg.expected_transport) else {
        return Ok(());
    };

    let scheme_matches = match expected_transport {
        PgtmApiTransportExpectation::Http => url.scheme() == "http",
        PgtmApiTransportExpectation::Https => url.scheme() == "https",
    };

    if scheme_matches {
        return Ok(());
    }

    let expected_scheme = match expected_transport {
        PgtmApiTransportExpectation::Http => "http",
        PgtmApiTransportExpectation::Https => "https",
    };

    Err(CliError::Config(format!(
        "operator config expects `{expected_scheme}` API transport, but resolved base URL uses `{}`",
        url.scheme()
    )))
}

fn resolve_config_auth(
    config: Option<&OperatorConfigV2>,
) -> Result<(Option<String>, Option<String>, bool), CliError> {
    let Some(config) = config else {
        return Ok((None, None, false));
    };

    Ok((
        config
            .api_auth
            .read_token
            .as_ref()
            .map(|token| token.as_str().to_string())
            .and_then(|token| normalize_optional_token(Some(token.as_str()))),
        config
            .api_auth
            .admin_token
            .as_ref()
            .map(|token| token.as_str().to_string())
            .and_then(|token| normalize_optional_token(Some(token.as_str()))),
        config.api_auth_enabled(),
    ))
}

fn resolve_client_tls(config: Option<&OperatorConfigV2>) -> Result<CliTlsConfig, CliError> {
    let Some(config) = config else {
        return Ok(CliTlsConfig::default());
    };
    let tls = &config.client_tls;

    Ok(CliTlsConfig {
        ca_cert_pem: None,
        client_cert_pem: None,
        client_key_pem: None,
        ca_cert_path: tls.ca_cert.clone(),
        client_cert_path: tls
            .identity
            .as_ref()
            .map(|identity| identity.cert.clone()),
        client_key_path: tls
            .identity
            .as_ref()
            .map(|identity| identity.key.clone()),
    })
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
    use std::path::PathBuf;

    use super::resolve_operator_context;
    use crate::cli::args::{Cli, Command};

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
        let ca_path = std::env::temp_dir().join(format!(
            "pgtm-api-ca-{}-{}.pem",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|err| err.to_string())?
                .as_nanos()
        ));
        std::fs::write(&ca_path, "ca-cert").map_err(|err| err.to_string())?;
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
            .as_string()
            .ok_or_else(|| "read token missing".to_string())?;
        if read_token != "read-token" {
            return Err("read token did not resolve".to_string());
        }
        let admin_token = ctx
            .api_client
            .auth
            .admin_token
            .as_string()
            .ok_or_else(|| "admin token missing".to_string())?;
        if admin_token != "admin-token" {
            return Err("admin token did not resolve".to_string());
        }
        if ctx.api_client.tls.ca_cert_path.is_none() {
            return Err("ca cert path did not resolve".to_string());
        }
        Ok(())
    }

    #[test]
    fn resolve_context_preserves_postgres_tls_paths() -> Result<(), String> {
        let ca_path = std::env::temp_dir().join(format!(
            "pgtm-postgres-ca-{}-{}.pem",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|err| err.to_string())?
                .as_nanos()
        ));
        let identity_cert_path = std::env::temp_dir().join(format!(
            "pgtm-postgres-cert-{}-{}.pem",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|err| err.to_string())?
                .as_nanos()
        ));
        let identity_key_path = std::env::temp_dir().join(format!(
            "pgtm-postgres-key-{}-{}.pem",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|err| err.to_string())?
                .as_nanos()
        ));
        std::fs::write(&ca_path, "ca-cert").map_err(|err| err.to_string())?;
        std::fs::write(&identity_cert_path, "client-cert").map_err(|err| err.to_string())?;
        std::fs::write(&identity_key_path, "client-key").map_err(|err| err.to_string())?;

        let path = write_temp_config(format!(
            r#"
[api]
base_url = "https://127.0.0.1:8443"

[api.tls]
ca_cert = {{ path = "{}" }}
identity = {{ cert = {{ path = "{}" }}, key = {{ type = "file", path = "{}" }} }}
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

        if ctx.postgres_client_tls.ca_cert_path.is_none() {
            return Err("expected postgres CA path to be preserved".to_string());
        }
        if ctx.postgres_client_tls.client_cert_path.is_none() {
            return Err("expected postgres client cert path to be preserved".to_string());
        }
        if ctx.postgres_client_tls.client_key_path.is_none() {
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
            Err(err) if err
                .to_string()
                .contains("expects `https` API transport") =>
            {
                Ok(())
            }
            Err(err) => Err(format!("unexpected error: {err}")),
            Ok(_) => Err("expected transport mismatch failure".to_string()),
        }
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
