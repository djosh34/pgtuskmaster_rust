use std::net::IpAddr;

use reqwest::Url;

use crate::{
    cli::{
        args::Cli,
        client::{CliApiClientConfig, CliAuthConfig, CliTlsConfig},
        error::CliError,
    },
    config::{
        load_runtime_config, resolve_inline_or_path_bytes, resolve_secret_string, ApiAuthConfig,
        ApiTlsMode, RuntimeConfig,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OperatorContext {
    pub(crate) api_client: CliApiClientConfig,
    pub(crate) postgres_client_tls: CliTlsConfig,
    pub(crate) api_auth_enabled: bool,
}

pub(crate) fn resolve_operator_context(cli: &Cli) -> Result<OperatorContext, CliError> {
    let runtime_config = cli
        .config
        .as_ref()
        .map(|path| load_runtime_config(path.as_path()))
        .transpose()
        .map_err(|err| CliError::Config(err.to_string()))?;

    let base_url = resolve_api_url(cli.base_url.as_deref(), runtime_config.as_ref())?;
    validate_effective_api_url(&base_url, runtime_config.as_ref())?;

    let (config_read_token, config_admin_token, api_auth_enabled) =
        resolve_config_auth(runtime_config.as_ref())?;
    let read_token = normalize_optional_token(cli.read_token.as_deref()).or(config_read_token);
    let admin_token = normalize_optional_token(cli.admin_token.as_deref()).or(config_admin_token);

    let api_client_tls = if base_url.scheme() == "https" {
        resolve_api_client_tls(runtime_config.as_ref())?
    } else {
        CliTlsConfig::default()
    };
    let postgres_client_tls = resolve_postgres_client_tls(runtime_config.as_ref())?;

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
    })
}

fn resolve_api_url(
    override_base_url: Option<&str>,
    runtime_config: Option<&RuntimeConfig>,
) -> Result<Url, CliError> {
    if let Some(raw) = override_base_url {
        return Url::parse(raw.trim())
            .map_err(|err| CliError::RequestBuild(format!("invalid --base-url value: {err}")));
    }

    let cfg = runtime_config.ok_or_else(|| {
        CliError::Config("either `-c <PATH>` or `--base-url <URL>` must be provided".to_string())
    })?;

    if let Some(api_url) = cfg.pgtm.as_ref().and_then(|pgtm| pgtm.api_url.as_deref()) {
        return Url::parse(api_url)
            .map_err(|err| CliError::Config(format!("invalid `pgtm.api_url`: {err}")));
    }

    match cfg.api.listen_addr.ip() {
        IpAddr::V4(ip) if ip.is_unspecified() => Err(CliError::Config(
            "`api.listen_addr` uses 0.0.0.0; set `pgtm.api_url` to an operator-reachable address"
                .to_string(),
        )),
        IpAddr::V6(ip) if ip.is_unspecified() => Err(CliError::Config(
            "`api.listen_addr` uses [::]; set `pgtm.api_url` to an operator-reachable address"
                .to_string(),
        )),
        _ => {
            let scheme = match cfg.api.security.tls.mode {
                ApiTlsMode::Disabled => "http",
                ApiTlsMode::Optional | ApiTlsMode::Required => "https",
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
    runtime_config: Option<&RuntimeConfig>,
) -> Result<(), CliError> {
    let Some(cfg) = runtime_config else {
        return Ok(());
    };

    match cfg.api.security.tls.mode {
        ApiTlsMode::Disabled if base_url.scheme() == "https" => Err(CliError::Config(
            "API URL must not use https when `api.security.tls.mode = \"disabled\"`".to_string(),
        )),
        ApiTlsMode::Required if base_url.scheme() != "https" => Err(CliError::Config(
            "API URL must use https when `api.security.tls.mode = \"required\"`".to_string(),
        )),
        _ => Ok(()),
    }
}

fn resolve_config_auth(
    runtime_config: Option<&RuntimeConfig>,
) -> Result<(Option<String>, Option<String>, bool), CliError> {
    let Some(cfg) = runtime_config else {
        return Ok((None, None, false));
    };

    match &cfg.api.security.auth {
        ApiAuthConfig::Disabled => Ok((None, None, false)),
        ApiAuthConfig::RoleTokens(tokens) => Ok((
            resolve_optional_secret(
                "api.security.auth.role_tokens.read_token",
                tokens.read_token.as_ref(),
            )?,
            resolve_optional_secret(
                "api.security.auth.role_tokens.admin_token",
                tokens.admin_token.as_ref(),
            )?,
            true,
        )),
    }
}

fn resolve_api_client_tls(
    runtime_config: Option<&RuntimeConfig>,
) -> Result<CliTlsConfig, CliError> {
    let Some(cfg) = runtime_config else {
        return Ok(CliTlsConfig::default());
    };
    let Some(api_client) = cfg.pgtm.as_ref().and_then(|pgtm| pgtm.api_client.as_ref()) else {
        return Ok(CliTlsConfig::default());
    };

    if cfg
        .api
        .security
        .tls
        .client_auth
        .as_ref()
        .is_some_and(|auth| auth.require_client_cert)
        && (api_client.client_cert.is_none() || api_client.client_key.is_none())
    {
        return Err(CliError::Config(
            "`pgtm.api_client.client_cert` and `pgtm.api_client.client_key` are required when API client certificates are mandatory"
                .to_string(),
        ));
    }

    Ok(CliTlsConfig {
        ca_cert_pem: api_client
            .ca_cert
            .as_ref()
            .map(|source| resolve_inline_or_path_bytes("pgtm.api_client.ca_cert", source))
            .transpose()
            .map_err(|err| CliError::Config(err.to_string()))?,
        client_cert_pem: api_client
            .client_cert
            .as_ref()
            .map(|source| resolve_inline_or_path_bytes("pgtm.api_client.client_cert", source))
            .transpose()
            .map_err(|err| CliError::Config(err.to_string()))?,
        client_key_pem: api_client
            .client_key
            .as_ref()
            .map(|source| resolve_secret_string("pgtm.api_client.client_key", source))
            .transpose()
            .map(|result| result.map(String::into_bytes))
            .map_err(|err| CliError::Config(err.to_string()))?,
    })
}

fn resolve_postgres_client_tls(
    runtime_config: Option<&RuntimeConfig>,
) -> Result<CliTlsConfig, CliError> {
    let Some(cfg) = runtime_config else {
        return Ok(CliTlsConfig::default());
    };
    let Some(pgtm) = cfg.pgtm.as_ref() else {
        return Ok(CliTlsConfig::default());
    };
    let ca_cert = pgtm
        .postgres_client
        .as_ref()
        .and_then(|client| client.ca_cert.as_ref())
        .or_else(|| {
            pgtm.api_client
                .as_ref()
                .and_then(|client| client.ca_cert.as_ref())
        });
    let client_cert = pgtm
        .postgres_client
        .as_ref()
        .and_then(|client| client.client_cert.as_ref())
        .or_else(|| {
            pgtm.api_client
                .as_ref()
                .and_then(|client| client.client_cert.as_ref())
        });
    let client_key = pgtm
        .postgres_client
        .as_ref()
        .and_then(|client| client.client_key.as_ref())
        .or_else(|| {
            pgtm.api_client
                .as_ref()
                .and_then(|client| client.client_key.as_ref())
        });

    Ok(CliTlsConfig {
        ca_cert_pem: ca_cert
            .map(|source| resolve_inline_or_path_bytes("pgtm.postgres_client.ca_cert", source))
            .transpose()
            .map_err(|err| CliError::Config(err.to_string()))?,
        client_cert_pem: client_cert
            .map(|source| resolve_inline_or_path_bytes("pgtm.postgres_client.client_cert", source))
            .transpose()
            .map_err(|err| CliError::Config(err.to_string()))?,
        client_key_pem: client_key
            .map(|source| resolve_secret_string("pgtm.postgres_client.client_key", source))
            .transpose()
            .map(|result| result.map(String::into_bytes))
            .map_err(|err| CliError::Config(err.to_string()))?,
    })
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
member_id = "node-a"

[postgres]
data_dir = "/tmp/pgdata"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtm/socket"
log_file = "/tmp/pgtm/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "# empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
pg_rewind_timeout_ms = 1000
bootstrap_timeout_ms = 1000
fencing_timeout_ms = 1000
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
listen_addr = "0.0.0.0:8080"
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
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
            Err(err) if err.to_string().contains("set `pgtm.api_url`") => Ok(()),
            Err(err) => Err(format!("unexpected error: {err}")),
            Ok(_) => Err("expected resolution failure".to_string()),
        }
    }

    #[test]
    fn resolve_context_loads_tokens_and_tls_from_config() -> Result<(), String> {
        let path = write_temp_config(
            r##"
[cluster]
name = "cluster-a"
member_id = "node-a"

[postgres]
data_dir = "/tmp/pgdata"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtm/socket"
log_file = "/tmp/pgtm/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "# empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
pg_rewind_timeout_ms = 1000
bootstrap_timeout_ms = 1000
fencing_timeout_ms = 1000
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
listen_addr = "127.0.0.1:8443"
security = { tls = { mode = "required", identity = { cert_chain = { content = "-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n" }, private_key = { content = "-----BEGIN PRIVATE KEY-----\nMIIB\n-----END PRIVATE KEY-----\n" } } }, auth = { type = "role_tokens", read_token = { content = "read-token" }, admin_token = { content = "admin-token" } } }

[pgtm]
api_url = "https://127.0.0.1:8443"

[pgtm.api_client]
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
