use std::{net::IpAddr, path::PathBuf};

use reqwest::Url;

use crate::{
    cli::{
        args::Cli,
        client::{CliApiClientConfig, CliAuthConfig, CliTlsConfig},
        error::CliError,
    },
    config::{
        load_operator_config, resolve_inline_or_path_bytes, resolve_secret_string, ApiAuthConfig,
        ApiClientAuthConfig, ApiTransportConfig, InlineOrPath, PgtmApiAuthConfig,
        PgtmApiTransportExpectation, PgtmConfig, PgtmPrimaryTargetConfig, RuntimeConfig,
        SecretSource,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OperatorContext {
    pub(crate) api_client: CliApiClientConfig,
    pub(crate) postgres_client_tls: CliTlsConfig,
    pub(crate) api_auth_enabled: bool,
    pub(crate) primary_target: Option<PgtmPrimaryTargetConfig>,
}

#[derive(Clone, Debug)]
struct OperatorConfigSource {
    runtime: Option<RuntimeConfig>,
    operator: Option<PgtmConfig>,
}

pub(crate) fn resolve_operator_context(cli: &Cli) -> Result<OperatorContext, CliError> {
    let config_source = cli
        .config
        .as_ref()
        .map(|path| load_operator_config_source(path.as_path()))
        .transpose()?;

    let base_url = resolve_api_url(cli.base_url.as_deref(), config_source.as_ref())?;
    validate_effective_api_url(&base_url, config_source.as_ref())?;

    let (config_read_token, config_admin_token, api_auth_enabled) =
        resolve_config_auth(config_source.as_ref())?;
    let read_token = normalize_optional_token(cli.read_token.as_deref()).or(config_read_token);
    let admin_token = normalize_optional_token(cli.admin_token.as_deref()).or(config_admin_token);

    let api_client_tls = if base_url.scheme() == "https" {
        resolve_api_client_tls(config_source.as_ref())?
    } else {
        CliTlsConfig::default()
    };
    let postgres_client_tls = resolve_postgres_client_tls(config_source.as_ref())?;
    let primary_target = config_source
        .as_ref()
        .and_then(|source| source.operator.as_ref())
        .and_then(|operator| operator.primary_target.clone());

    Ok(OperatorContext {
        api_client: CliApiClientConfig {
            base_url,
            timeout_ms: cli.timeout_ms,
            auth: CliAuthConfig {
                read_token,
                admin_token,
            },
            tls: api_client_tls,
        },
        postgres_client_tls,
        api_auth_enabled,
        primary_target,
    })
}

fn load_operator_config_source(path: &std::path::Path) -> Result<OperatorConfigSource, CliError> {
    let contents = std::fs::read_to_string(path).map_err(|err| {
        CliError::Config(format!(
            "failed to read config file {}: {err}",
            path.display()
        ))
    })?;

    if let Ok(runtime) = toml::from_str::<RuntimeConfig>(&contents) {
        return Ok(OperatorConfigSource {
            operator: runtime.pgtm.clone(),
            runtime: Some(runtime),
        });
    }

    let operator = load_operator_config(path).map_err(|err| CliError::Config(err.to_string()))?;
    Ok(OperatorConfigSource {
        runtime: None,
        operator: Some(operator),
    })
}

fn resolve_api_url(
    override_base_url: Option<&str>,
    config_source: Option<&OperatorConfigSource>,
) -> Result<Url, CliError> {
    if let Some(raw) = override_base_url {
        return Url::parse(raw.trim())
            .map_err(|err| CliError::RequestBuild(format!("invalid --base-url value: {err}")));
    }

    let source = config_source.ok_or_else(|| {
        CliError::Config("either `-c <PATH>` or `--base-url <URL>` must be provided".to_string())
    })?;

    if let Some(operator) = source.operator.as_ref() {
        if let Some(api_url) = operator
            .api
            .base_url
            .as_deref()
            .or(operator.api.advertised_url.as_deref())
        {
            return Url::parse(api_url)
                .map_err(|err| CliError::Config(format!("invalid `pgtm.api.base_url`: {err}")));
        }
    }

    let cfg = source.runtime.as_ref().ok_or_else(|| {
        CliError::Config(
            "set `pgtm.api.base_url` in the operator config or pass `--base-url <URL>`".to_string(),
        )
    })?;

    match cfg.api.listen_addr.ip() {
        IpAddr::V4(ip) if ip.is_unspecified() => Err(CliError::Config(
            "`api.listen_addr` uses 0.0.0.0; set `pgtm.api.base_url` to an operator-reachable address"
                .to_string(),
        )),
        IpAddr::V6(ip) if ip.is_unspecified() => Err(CliError::Config(
            "`api.listen_addr` uses [::]; set `pgtm.api.base_url` to an operator-reachable address"
                .to_string(),
        )),
        _ => {
            let scheme = match cfg.api.transport {
                ApiTransportConfig::Http => "http",
                ApiTransportConfig::Https { .. } => "https",
            };
            Url::parse(format!("{scheme}://{}", cfg.api.listen_addr).as_str()).map_err(|err| {
                CliError::Config(format!(
                    "failed to derive API URL from `api.listen_addr`: {err}"
                ))
            })
        }
    }
}

fn validate_effective_api_url(
    base_url: &Url,
    config_source: Option<&OperatorConfigSource>,
) -> Result<(), CliError> {
    let Some(source) = config_source else {
        return Ok(());
    };

    if let Some(expected_transport) = source
        .operator
        .as_ref()
        .and_then(|operator| operator.api.expected_transport)
    {
        return match (expected_transport, base_url.scheme()) {
            (PgtmApiTransportExpectation::Http, "https") => Err(CliError::Config(
                "API URL must not use https when `pgtm.api.expected_transport = \"http\"`"
                    .to_string(),
            )),
            (PgtmApiTransportExpectation::Https, "http") => Err(CliError::Config(
                "API URL must use https when `pgtm.api.expected_transport = \"https\"`".to_string(),
            )),
            _ => Ok(()),
        };
    }

    let Some(cfg) = source.runtime.as_ref() else {
        return Ok(());
    };

    match (&cfg.api.transport, base_url.scheme()) {
        (ApiTransportConfig::Http, "https") => Err(CliError::Config(
            "API URL must not use https when `api.transport = \"http\"`".to_string(),
        )),
        (ApiTransportConfig::Https { .. }, "http") => Err(CliError::Config(
            "API URL must use https when `api.transport = \"https\"`".to_string(),
        )),
        _ => Ok(()),
    }
}

fn resolve_config_auth(
    config_source: Option<&OperatorConfigSource>,
) -> Result<(Option<String>, Option<String>, bool), CliError> {
    let Some(source) = config_source else {
        return Ok((None, None, false));
    };

    if let Some(operator) = source.operator.as_ref() {
        return match &operator.api.auth {
            PgtmApiAuthConfig::Disabled => Ok((None, None, false)),
            PgtmApiAuthConfig::RoleTokens {
                read_token,
                admin_token,
            } => Ok((
                resolve_optional_secret("pgtm.api.auth.read_token", read_token.as_ref())?,
                resolve_optional_secret("pgtm.api.auth.admin_token", admin_token.as_ref())?,
                true,
            )),
        };
    }

    let Some(cfg) = source.runtime.as_ref() else {
        return Ok((None, None, false));
    };

    match &cfg.api.auth {
        ApiAuthConfig::Disabled => Ok((None, None, false)),
        ApiAuthConfig::RoleTokens(tokens) => Ok((
            resolve_optional_secret("api.auth.read_token", tokens.read_token.as_ref())?,
            resolve_optional_secret("api.auth.admin_token", tokens.admin_token.as_ref())?,
            true,
        )),
    }
}

fn resolve_api_client_tls(
    config_source: Option<&OperatorConfigSource>,
) -> Result<CliTlsConfig, CliError> {
    let Some(source) = config_source else {
        return Ok(CliTlsConfig::default());
    };
    let Some(api_client) = source.operator.as_ref().map(|operator| &operator.api.tls) else {
        return Ok(CliTlsConfig::default());
    };

    if api_requires_client_cert(source) && api_client.identity.is_none() {
        return Err(CliError::Config(
            "`pgtm.api.tls.identity` is required when API client certificates are mandatory"
                .to_string(),
        ));
    }

    Ok(CliTlsConfig {
        ca_cert_pem: api_client
            .ca_cert
            .as_ref()
            .map(|source| resolve_inline_or_path_bytes("pgtm.api.tls.ca_cert", source))
            .transpose()
            .map_err(|err| CliError::Config(err.to_string()))?,
        client_cert_pem: api_client
            .identity
            .as_ref()
            .map(|identity| {
                resolve_inline_or_path_bytes("pgtm.api.tls.identity.cert", &identity.cert)
            })
            .transpose()
            .map_err(|err| CliError::Config(err.to_string()))?,
        client_key_pem: api_client
            .identity
            .as_ref()
            .map(|identity| resolve_secret_string("pgtm.api.tls.identity.key", &identity.key))
            .transpose()
            .map(|result| result.map(String::into_bytes))
            .map_err(|err| CliError::Config(err.to_string()))?,
        ca_cert_path: api_client
            .ca_cert
            .as_ref()
            .and_then(inline_or_path_to_path_buf),
        client_cert_path: api_client
            .identity
            .as_ref()
            .and_then(|identity| inline_or_path_to_path_buf(&identity.cert)),
        client_key_path: api_client
            .identity
            .as_ref()
            .and_then(|identity| secret_to_path_buf(&identity.key)),
    })
}

fn resolve_postgres_client_tls(
    config_source: Option<&OperatorConfigSource>,
) -> Result<CliTlsConfig, CliError> {
    let Some(source) = config_source else {
        return Ok(CliTlsConfig::default());
    };
    let Some(operator) = source.operator.as_ref() else {
        return Ok(CliTlsConfig::default());
    };
    let ca_cert = operator
        .postgres
        .tls
        .ca_cert
        .as_ref()
        .or(operator.api.tls.ca_cert.as_ref());
    let identity = operator
        .postgres
        .tls
        .identity
        .as_ref()
        .or(operator.api.tls.identity.as_ref());

    Ok(CliTlsConfig {
        ca_cert_pem: ca_cert
            .map(|source| resolve_inline_or_path_bytes("pgtm.postgres.tls.ca_cert", source))
            .transpose()
            .map_err(|err| CliError::Config(err.to_string()))?,
        client_cert_pem: identity
            .map(|identity| {
                resolve_inline_or_path_bytes("pgtm.postgres.tls.identity.cert", &identity.cert)
            })
            .transpose()
            .map_err(|err| CliError::Config(err.to_string()))?,
        client_key_pem: identity
            .map(|identity| resolve_secret_string("pgtm.postgres.tls.identity.key", &identity.key))
            .transpose()
            .map(|result| result.map(String::into_bytes))
            .map_err(|err| CliError::Config(err.to_string()))?,
        ca_cert_path: ca_cert.and_then(inline_or_path_to_path_buf),
        client_cert_path: identity.and_then(|identity| inline_or_path_to_path_buf(&identity.cert)),
        client_key_path: identity.and_then(|identity| secret_to_path_buf(&identity.key)),
    })
}

fn inline_or_path_to_path_buf(source: &InlineOrPath) -> Option<PathBuf> {
    match source {
        InlineOrPath::Path(path) | InlineOrPath::PathConfig { path } => Some(path.clone()),
        InlineOrPath::Inline { .. } => None,
    }
}

fn secret_to_path_buf(source: &SecretSource) -> Option<PathBuf> {
    match source {
        SecretSource::Path(path) | SecretSource::PathConfig { path } => Some(path.clone()),
        SecretSource::Inline { .. } | SecretSource::Env { .. } => None,
    }
}

fn resolve_optional_secret(
    field: &str,
    value: Option<&crate::config::SecretSource>,
) -> Result<Option<String>, CliError> {
    value
        .map(|source| resolve_secret_string(field, source))
        .transpose()
        .map(|value| value.and_then(|token| normalize_optional_token(Some(token.as_str()))))
        .map_err(|err| CliError::Config(err.to_string()))
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

fn api_requires_client_cert(source: &OperatorConfigSource) -> bool {
    source
        .runtime
        .as_ref()
        .map(|cfg| match &cfg.api.transport {
            ApiTransportConfig::Http => false,
            ApiTransportConfig::Https { tls } => {
                matches!(tls.client_auth, ApiClientAuthConfig::Required { .. })
            }
        })
        .unwrap_or(false)
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
    fn resolve_context_requires_override_for_unspecified_listen_addr() -> Result<(), String> {
        let path = write_temp_config(
            r##"
[cluster]
name = "cluster-a"
scope = "scope-a"
member_id = "node-a"

[postgres]
paths = { data_dir = "/tmp/pgdata" }
network = { listen_host = "127.0.0.1", listen_port = 5432 }
access = { hba = { content = "local all all trust" }, ident = { content = "# empty" } }
local_database = "postgres"
rewind = { database = "postgres", transport = { ssl_mode = "prefer" } }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]

[process]
[process.timeouts]
pg_rewind_ms = 1000
bootstrap_ms = 1000
fencing_ms = 1000

[process.binaries.overrides]
postgres = "/usr/bin/postgres"
pg_ctl = "/usr/bin/pg_ctl"
pg_rewind = "/usr/bin/pg_rewind"
initdb = "/usr/bin/initdb"
pg_basebackup = "/usr/bin/pg_basebackup"
psql = "/usr/bin/psql"

[api]
listen_addr = "0.0.0.0:8080"
transport = { transport = "http" }
auth = { type = "disabled" }
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
            Err(err) if err.to_string().contains("set `pgtm.api.base_url`") => Ok(()),
            Err(err) => Err(format!("unexpected error: {err}")),
            Ok(_) => Err("expected resolution failure".to_string()),
        }
    }

    #[test]
    fn resolve_context_loads_tokens_and_tls_from_config() -> Result<(), String> {
        let path = write_temp_config(
            r##"
[api]
base_url = "https://127.0.0.1:8443"

[api.auth]
type = "role_tokens"
read_token = { content = "read-token" }
admin_token = { content = "admin-token" }

[api.tls]
ca_cert = { content = "-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n" }
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
        let ctx = resolve_operator_context(&cli).map_err(|err| err.to_string())?;
        let _ = std::fs::remove_file(path);
        if ctx.api_client.auth.read_token.as_deref() != Some("read-token") {
            return Err("read token did not resolve".to_string());
        }
        if ctx.api_client.auth.admin_token.as_deref() != Some("admin-token") {
            return Err("admin token did not resolve".to_string());
        }
        if ctx.api_client.tls.ca_cert_pem.is_none() {
            return Err("ca cert did not resolve".to_string());
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
                .map_err(|err| format!("system time error: {err}"))?
                .as_nanos()
        ));
        let cert_path = std::env::temp_dir().join(format!(
            "pgtm-postgres-cert-{}-{}.pem",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|err| format!("system time error: {err}"))?
                .as_nanos()
        ));
        let key_path = std::env::temp_dir().join(format!(
            "pgtm-postgres-key-{}-{}.pem",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|err| format!("system time error: {err}"))?
                .as_nanos()
        ));
        std::fs::write(&ca_path, "ca").map_err(|err| format!("write ca failed: {err}"))?;
        std::fs::write(&cert_path, "cert").map_err(|err| format!("write cert failed: {err}"))?;
        std::fs::write(&key_path, "key").map_err(|err| format!("write key failed: {err}"))?;

        let path = write_temp_config(
            format!(
                r##"
[api]
base_url = "http://127.0.0.1:8080"

[postgres.tls]
ca_cert = {{ path = "{}" }}
identity = {{ cert = {{ path = "{}" }}, key = {{ path = "{}" }} }}
"##,
                ca_path.display(),
                cert_path.display(),
                key_path.display()
            )
            .as_str(),
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
        let ctx = resolve_operator_context(&cli).map_err(|err| err.to_string())?;

        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(ca_path.clone());
        let _ = std::fs::remove_file(cert_path.clone());
        let _ = std::fs::remove_file(key_path.clone());

        if ctx.postgres_client_tls.ca_cert_path != Some(ca_path) {
            return Err("postgres client CA path did not resolve".to_string());
        }
        if ctx.postgres_client_tls.client_cert_path != Some(cert_path) {
            return Err("postgres client cert path did not resolve".to_string());
        }
        if ctx.postgres_client_tls.client_key_path != Some(key_path) {
            return Err("postgres client key path did not resolve".to_string());
        }
        Ok(())
    }

    #[test]
    fn resolve_context_falls_back_to_api_client_tls_paths() -> Result<(), String> {
        let ca_path = std::env::temp_dir().join(format!(
            "pgtm-api-ca-{}-{}.pem",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|err| format!("system time error: {err}"))?
                .as_nanos()
        ));
        std::fs::write(&ca_path, "ca").map_err(|err| format!("write ca failed: {err}"))?;

        let path = write_temp_config(
            format!(
                r##"
[api]
base_url = "https://127.0.0.1:8443"

[api.tls]
ca_cert = {{ path = "{}" }}
"##,
                ca_path.display()
            )
            .as_str(),
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
        let ctx = resolve_operator_context(&cli).map_err(|err| err.to_string())?;

        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(ca_path.clone());

        if ctx.postgres_client_tls.ca_cert_path != Some(ca_path) {
            return Err("postgres TLS fallback to api_client did not preserve path".to_string());
        }
        Ok(())
    }

    fn write_temp_config(contents: &str) -> Result<PathBuf, String> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|err| format!("system time error: {err}"))?
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("pgtm-context-{unique}-{}.toml", std::process::id()));
        std::fs::write(&path, contents).map_err(|err| format!("write config failed: {err}"))?;
        Ok(path)
    }
}
