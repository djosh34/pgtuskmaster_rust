use std::path::{Path, PathBuf};

use thiserror::Error;

use super::defaults::{
    default_api_listen_addr, default_debug_config, default_logging_config,
    default_postgres_connect_timeout_s, normalize_process_config,
};
use super::endpoint::DcsEndpoint;
use super::schema::{
    ApiConfig, ApiSecurityConfig, DcsConfig, DcsConfigInput, InlineOrPath, PgHbaConfig,
    PgIdentConfig, PgtmApiClientConfig, PgtmConfig, PgtmConfigInput, PgtmPostgresClientConfig,
    PostgresConfig, PostgresConnIdentityConfig, PostgresRoleConfig, PostgresRolesConfig,
    RoleAuthConfig, RoleAuthConfigInput, RuntimeConfig, RuntimeConfigInput, SecretSource,
    TlsServerConfig, TlsServerIdentityConfig,
};
use crate::postgres_managed_conf::{validate_extra_guc_entry, ManagedPostgresConfError};

const MIN_TIMEOUT_MS: u64 = 1;
const MAX_TIMEOUT_MS: u64 = 86_400_000;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse config file {path}: {source}")]
    Parse {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("invalid config field `{field}`: {message}")]
    Validation {
        field: &'static str,
        message: String,
    },
}

pub fn load_runtime_config(path: &Path) -> Result<RuntimeConfig, ConfigError> {
    let contents = std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.display().to_string(),
        source,
    })?;

    let raw: RuntimeConfigInput =
        toml::from_str(&contents).map_err(|source| ConfigError::Parse {
            path: path.display().to_string(),
            source,
        })?;
    let cfg = normalize_runtime_config(raw)?;
    validate_runtime_config(&cfg)?;
    Ok(cfg)
}

fn normalize_runtime_config(input: RuntimeConfigInput) -> Result<RuntimeConfig, ConfigError> {
    let postgres = normalize_postgres_config(input.postgres)?;
    let dcs = normalize_dcs_config(input.dcs)?;
    let process = normalize_process_config(input.process)?;
    let logging = input.logging.unwrap_or_else(default_logging_config);
    let api = normalize_api_config(input.api)?;
    let pgtm = input.pgtm.map(normalize_pgtm_config).transpose()?;
    let debug = input.debug.unwrap_or_else(default_debug_config);

    Ok(RuntimeConfig {
        cluster: input.cluster,
        postgres,
        dcs,
        ha: input.ha,
        process,
        logging,
        api,
        pgtm,
        debug,
    })
}

fn normalize_postgres_config(
    input: super::schema::PostgresConfigInput,
) -> Result<PostgresConfig, ConfigError> {
    let connect_timeout_s = input
        .connect_timeout_s
        .unwrap_or_else(default_postgres_connect_timeout_s);

    let local_conn_identity = normalize_postgres_conn_identity(
        "postgres.local_conn_identity",
        input.local_conn_identity,
    )?;
    let rewind_conn_identity = normalize_postgres_conn_identity(
        "postgres.rewind_conn_identity",
        input.rewind_conn_identity,
    )?;

    let tls = normalize_tls_server_config("postgres.tls", input.tls)?;
    let roles = normalize_postgres_roles(input.roles)?;
    let pg_hba = normalize_pg_hba(input.pg_hba)?;
    let pg_ident = normalize_pg_ident(input.pg_ident)?;

    Ok(PostgresConfig {
        data_dir: input.data_dir,
        connect_timeout_s,
        listen_host: input.listen_host,
        listen_port: input.listen_port,
        advertise_port: input.advertise_port,
        socket_dir: input.socket_dir,
        log_file: input.log_file,
        local_conn_identity,
        rewind_conn_identity,
        tls,
        roles,
        pg_hba,
        pg_ident,
        extra_gucs: normalize_postgres_extra_gucs(input.extra_gucs)?,
    })
}

fn normalize_postgres_extra_gucs(
    input: Option<std::collections::BTreeMap<String, String>>,
) -> Result<std::collections::BTreeMap<String, String>, ConfigError> {
    let extra_gucs = input.unwrap_or_default();
    for (key, value) in &extra_gucs {
        validate_extra_guc_for_config(key.as_str(), value.as_str())?;
    }
    Ok(extra_gucs)
}

fn normalize_postgres_conn_identity(
    field_prefix: &'static str,
    input: Option<super::schema::PostgresConnIdentityConfigInput>,
) -> Result<PostgresConnIdentityConfig, ConfigError> {
    let identity = input.ok_or_else(|| ConfigError::Validation {
        field: field_prefix,
        message: "missing required secure config block".to_string(),
    })?;

    let user_field = match field_prefix {
        "postgres.local_conn_identity" => "postgres.local_conn_identity.user",
        "postgres.rewind_conn_identity" => "postgres.rewind_conn_identity.user",
        _ => field_prefix,
    };
    let dbname_field = match field_prefix {
        "postgres.local_conn_identity" => "postgres.local_conn_identity.dbname",
        "postgres.rewind_conn_identity" => "postgres.rewind_conn_identity.dbname",
        _ => field_prefix,
    };
    let ssl_mode_field = match field_prefix {
        "postgres.local_conn_identity" => "postgres.local_conn_identity.ssl_mode",
        "postgres.rewind_conn_identity" => "postgres.rewind_conn_identity.ssl_mode",
        _ => field_prefix,
    };
    let ca_cert_field = match field_prefix {
        "postgres.local_conn_identity" => "postgres.local_conn_identity.ca_cert",
        "postgres.rewind_conn_identity" => "postgres.rewind_conn_identity.ca_cert",
        _ => field_prefix,
    };

    let user = identity.user.ok_or_else(|| ConfigError::Validation {
        field: user_field,
        message: "missing required secure field".to_string(),
    })?;
    validate_non_empty(user_field, user.as_str())?;

    let dbname = identity.dbname.ok_or_else(|| ConfigError::Validation {
        field: dbname_field,
        message: "missing required secure field".to_string(),
    })?;
    validate_non_empty(dbname_field, dbname.as_str())?;

    let ssl_mode = identity.ssl_mode.ok_or_else(|| ConfigError::Validation {
        field: ssl_mode_field,
        message: "missing required secure field".to_string(),
    })?;

    let ca_cert = normalize_conn_identity_ca_cert(ca_cert_field, identity.ca_cert.as_ref())?;

    Ok(PostgresConnIdentityConfig {
        user,
        dbname,
        ssl_mode,
        ca_cert,
    })
}

fn normalize_conn_identity_ca_cert(
    field: &'static str,
    input: Option<&InlineOrPath>,
) -> Result<Option<PathBuf>, ConfigError> {
    match input {
        None => Ok(None),
        Some(InlineOrPath::Path(path)) | Some(InlineOrPath::PathConfig { path }) => {
            validate_non_empty_path(field, path)?;
            Ok(Some(path.clone()))
        }
        Some(InlineOrPath::Inline { .. }) => Err(ConfigError::Validation {
            field,
            message: "must be a path-backed CA bundle; inline content is not supported".to_string(),
        }),
    }
}

fn normalize_postgres_roles(
    input: Option<super::schema::PostgresRolesConfigInput>,
) -> Result<PostgresRolesConfig, ConfigError> {
    let roles = input.ok_or_else(|| ConfigError::Validation {
        field: "postgres.roles",
        message: "missing required secure config block".to_string(),
    })?;

    let superuser = normalize_postgres_role("postgres.roles.superuser", roles.superuser)?;
    let replicator = normalize_postgres_role("postgres.roles.replicator", roles.replicator)?;
    let rewinder = normalize_postgres_role("postgres.roles.rewinder", roles.rewinder)?;

    Ok(PostgresRolesConfig {
        superuser,
        replicator,
        rewinder,
    })
}

fn normalize_postgres_role(
    field_prefix: &'static str,
    input: Option<super::schema::PostgresRoleConfigInput>,
) -> Result<PostgresRoleConfig, ConfigError> {
    let role = input.ok_or_else(|| ConfigError::Validation {
        field: field_prefix,
        message: "missing required secure config block".to_string(),
    })?;

    let username_field = match field_prefix {
        "postgres.roles.superuser" => "postgres.roles.superuser.username",
        "postgres.roles.replicator" => "postgres.roles.replicator.username",
        "postgres.roles.rewinder" => "postgres.roles.rewinder.username",
        _ => field_prefix,
    };
    let auth_field = match field_prefix {
        "postgres.roles.superuser" => "postgres.roles.superuser.auth",
        "postgres.roles.replicator" => "postgres.roles.replicator.auth",
        "postgres.roles.rewinder" => "postgres.roles.rewinder.auth",
        _ => field_prefix,
    };

    let username = role.username.ok_or_else(|| ConfigError::Validation {
        field: username_field,
        message: "missing required secure field".to_string(),
    })?;
    validate_non_empty(username_field, username.as_str())?;

    let auth = role.auth.ok_or_else(|| ConfigError::Validation {
        field: auth_field,
        message: "missing required secure field".to_string(),
    })?;

    let auth = normalize_role_auth_config(auth_field, auth)?;

    Ok(PostgresRoleConfig { username, auth })
}

fn normalize_role_auth_config(
    field_prefix: &'static str,
    input: RoleAuthConfigInput,
) -> Result<RoleAuthConfig, ConfigError> {
    match input {
        RoleAuthConfigInput::Tls => Ok(RoleAuthConfig::Tls),
        RoleAuthConfigInput::Password { password } => {
            let password_field = match field_prefix {
                "postgres.roles.superuser.auth" => "postgres.roles.superuser.auth.password",
                "postgres.roles.replicator.auth" => "postgres.roles.replicator.auth.password",
                "postgres.roles.rewinder.auth" => "postgres.roles.rewinder.auth.password",
                _ => field_prefix,
            };

            let password = password.ok_or_else(|| ConfigError::Validation {
                field: password_field,
                message: "missing required secure field".to_string(),
            })?;

            Ok(RoleAuthConfig::Password { password })
        }
    }
}

fn normalize_pg_hba(
    input: Option<super::schema::PgHbaConfigInput>,
) -> Result<PgHbaConfig, ConfigError> {
    let cfg = input.ok_or_else(|| ConfigError::Validation {
        field: "postgres.pg_hba",
        message: "missing required secure config block".to_string(),
    })?;
    let source = cfg.source.ok_or_else(|| ConfigError::Validation {
        field: "postgres.pg_hba.source",
        message: "missing required secure field".to_string(),
    })?;
    Ok(PgHbaConfig { source })
}

fn normalize_pg_ident(
    input: Option<super::schema::PgIdentConfigInput>,
) -> Result<PgIdentConfig, ConfigError> {
    let cfg = input.ok_or_else(|| ConfigError::Validation {
        field: "postgres.pg_ident",
        message: "missing required secure config block".to_string(),
    })?;
    let source = cfg.source.ok_or_else(|| ConfigError::Validation {
        field: "postgres.pg_ident.source",
        message: "missing required secure field".to_string(),
    })?;
    Ok(PgIdentConfig { source })
}

fn normalize_api_config(input: super::schema::ApiConfigInput) -> Result<ApiConfig, ConfigError> {
    let listen_addr = normalize_api_listen_addr(
        "api.listen_addr",
        input
            .listen_addr
            .unwrap_or_else(default_api_listen_addr)
            .as_str(),
    )?;

    let security = input.security.ok_or_else(|| ConfigError::Validation {
        field: "api.security",
        message: "missing required secure config block".to_string(),
    })?;

    let tls = normalize_tls_server_config("api.security.tls", security.tls)?;
    let auth = security.auth.ok_or_else(|| ConfigError::Validation {
        field: "api.security.auth",
        message: "missing required secure field".to_string(),
    })?;

    Ok(ApiConfig {
        listen_addr,
        security: ApiSecurityConfig { tls, auth },
    })
}

fn normalize_dcs_config(input: DcsConfigInput) -> Result<DcsConfig, ConfigError> {
    let endpoints = input
        .endpoints
        .into_iter()
        .map(|endpoint| normalize_dcs_endpoint("dcs.endpoints", endpoint.as_str()))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(DcsConfig {
        endpoints,
        scope: input.scope,
        init: input.init,
    })
}

fn normalize_pgtm_config(input: PgtmConfigInput) -> Result<PgtmConfig, ConfigError> {
    Ok(PgtmConfig {
        api_url: normalize_optional_string("pgtm.api_url", input.api_url)?,
        api_client: input
            .api_client
            .map(normalize_pgtm_api_client_config)
            .transpose()?,
        postgres_client: input
            .postgres_client
            .map(normalize_pgtm_postgres_client_config)
            .transpose()?,
    })
}

fn normalize_pgtm_api_client_config(
    input: super::schema::PgtmApiClientConfigInput,
) -> Result<PgtmApiClientConfig, ConfigError> {
    Ok(PgtmApiClientConfig {
        ca_cert: input.ca_cert,
        client_cert: input.client_cert,
        client_key: input.client_key,
    })
}

fn normalize_pgtm_postgres_client_config(
    input: super::schema::PgtmPostgresClientConfigInput,
) -> Result<PgtmPostgresClientConfig, ConfigError> {
    Ok(PgtmPostgresClientConfig {
        ca_cert: input.ca_cert,
        client_cert: input.client_cert,
        client_key: input.client_key,
    })
}

fn normalize_optional_string(
    field: &'static str,
    value: Option<String>,
) -> Result<Option<String>, ConfigError> {
    match value {
        Some(value) => {
            validate_non_empty(field, value.as_str())?;
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

fn normalize_api_listen_addr(
    field: &'static str,
    value: &str,
) -> Result<std::net::SocketAddr, ConfigError> {
    value
        .parse::<std::net::SocketAddr>()
        .map_err(|err| ConfigError::Validation {
            field,
            message: format!("must be a valid socket address: {err}"),
        })
}

fn normalize_dcs_endpoint(field: &'static str, value: &str) -> Result<DcsEndpoint, ConfigError> {
    DcsEndpoint::parse(value).map_err(|err| ConfigError::Validation {
        field,
        message: err.to_string(),
    })
}

fn normalize_tls_server_config(
    field_prefix: &'static str,
    input: Option<super::schema::TlsServerConfigInput>,
) -> Result<TlsServerConfig, ConfigError> {
    let tls = input.ok_or_else(|| ConfigError::Validation {
        field: field_prefix,
        message: "missing required secure config block".to_string(),
    })?;

    let mode_field = match field_prefix {
        "postgres.tls" => "postgres.tls.mode",
        "api.security.tls" => "api.security.tls.mode",
        _ => field_prefix,
    };
    let identity_field = match field_prefix {
        "postgres.tls" => "postgres.tls.identity",
        "api.security.tls" => "api.security.tls.identity",
        _ => field_prefix,
    };

    let mode = tls.mode.ok_or_else(|| ConfigError::Validation {
        field: mode_field,
        message: "missing required secure field".to_string(),
    })?;

    let identity = match tls.identity {
        None => None,
        Some(identity) => Some(normalize_tls_server_identity(identity_field, identity)?),
    };

    Ok(TlsServerConfig {
        mode,
        identity,
        client_auth: tls.client_auth,
    })
}

fn normalize_tls_server_identity(
    field_prefix: &'static str,
    input: super::schema::TlsServerIdentityConfigInput,
) -> Result<TlsServerIdentityConfig, ConfigError> {
    let cert_chain_field = match field_prefix {
        "postgres.tls.identity" => "postgres.tls.identity.cert_chain",
        "api.security.tls.identity" => "api.security.tls.identity.cert_chain",
        _ => field_prefix,
    };
    let private_key_field = match field_prefix {
        "postgres.tls.identity" => "postgres.tls.identity.private_key",
        "api.security.tls.identity" => "api.security.tls.identity.private_key",
        _ => field_prefix,
    };

    let cert_chain = input.cert_chain.ok_or_else(|| ConfigError::Validation {
        field: cert_chain_field,
        message: "missing required secure field".to_string(),
    })?;
    let private_key = input.private_key.ok_or_else(|| ConfigError::Validation {
        field: private_key_field,
        message: "missing required secure field".to_string(),
    })?;

    Ok(TlsServerIdentityConfig {
        cert_chain,
        private_key,
    })
}

fn validate_absolute_path(field: &'static str, path: &Path) -> Result<(), ConfigError> {
    if !path.is_absolute() {
        return Err(ConfigError::Validation {
            field,
            message: "must be an absolute path".to_string(),
        });
    }
    Ok(())
}

fn normalize_path_lexical(path: &Path) -> PathBuf {
    use std::path::Component;

    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

pub fn validate_runtime_config(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    validate_non_empty_path("postgres.data_dir", &cfg.postgres.data_dir)?;
    validate_non_empty("postgres.listen_host", cfg.postgres.listen_host.as_str())?;
    validate_port("postgres.listen_port", cfg.postgres.listen_port)?;
    if let Some(advertise_port) = cfg.postgres.advertise_port {
        validate_port("postgres.advertise_port", advertise_port)?;
    }
    validate_non_empty_path("postgres.socket_dir", &cfg.postgres.socket_dir)?;
    validate_non_empty_path("postgres.log_file", &cfg.postgres.log_file)?;

    validate_non_empty(
        "postgres.local_conn_identity.user",
        cfg.postgres.local_conn_identity.user.as_str(),
    )?;
    validate_non_empty(
        "postgres.local_conn_identity.dbname",
        cfg.postgres.local_conn_identity.dbname.as_str(),
    )?;
    validate_non_empty(
        "postgres.rewind_conn_identity.user",
        cfg.postgres.rewind_conn_identity.user.as_str(),
    )?;
    validate_non_empty(
        "postgres.rewind_conn_identity.dbname",
        cfg.postgres.rewind_conn_identity.dbname.as_str(),
    )?;

    validate_non_empty(
        "postgres.roles.superuser.username",
        cfg.postgres.roles.superuser.username.as_str(),
    )?;
    validate_non_empty(
        "postgres.roles.replicator.username",
        cfg.postgres.roles.replicator.username.as_str(),
    )?;
    validate_non_empty(
        "postgres.roles.rewinder.username",
        cfg.postgres.roles.rewinder.username.as_str(),
    )?;
    validate_distinct_role_usernames(cfg)?;

    if cfg.postgres.local_conn_identity.user != cfg.postgres.roles.superuser.username {
        return Err(ConfigError::Validation {
            field: "postgres.local_conn_identity.user",
            message: format!(
                "must match postgres.roles.superuser.username (got `{}`, expected `{}`)",
                cfg.postgres.local_conn_identity.user, cfg.postgres.roles.superuser.username
            ),
        });
    }
    if cfg.postgres.rewind_conn_identity.user != cfg.postgres.roles.rewinder.username {
        return Err(ConfigError::Validation {
            field: "postgres.rewind_conn_identity.user",
            message: format!(
                "must match postgres.roles.rewinder.username (got `{}`, expected `{}`)",
                cfg.postgres.rewind_conn_identity.user, cfg.postgres.roles.rewinder.username
            ),
        });
    }
    validate_distinct_postgres_role_usernames(cfg)?;

    validate_postgres_auth_tls_invariants(cfg)?;

    validate_role_auth(
        "postgres.roles.superuser.auth.password.path",
        "postgres.roles.superuser.auth.password.content",
        "postgres.roles.superuser.auth.password.env",
        &cfg.postgres.roles.superuser.auth,
    )?;
    validate_role_auth(
        "postgres.roles.replicator.auth.password.path",
        "postgres.roles.replicator.auth.password.content",
        "postgres.roles.replicator.auth.password.env",
        &cfg.postgres.roles.replicator.auth,
    )?;
    validate_role_auth(
        "postgres.roles.rewinder.auth.password.path",
        "postgres.roles.rewinder.auth.password.content",
        "postgres.roles.rewinder.auth.password.env",
        &cfg.postgres.roles.rewinder.auth,
    )?;

    validate_tls_server_config(
        "postgres.tls.identity",
        "postgres.tls.identity.cert_chain",
        "postgres.tls.identity.private_key",
        &cfg.postgres.tls,
    )?;
    validate_tls_client_auth_config(
        "postgres.tls.client_auth",
        "postgres.tls.client_auth.client_ca",
        &cfg.postgres.tls,
    )?;

    validate_inline_or_path_non_empty(
        "postgres.pg_hba.source",
        &cfg.postgres.pg_hba.source,
        false,
    )?;
    validate_inline_or_path_non_empty(
        "postgres.pg_ident.source",
        &cfg.postgres.pg_ident.source,
        false,
    )?;
    for (key, value) in &cfg.postgres.extra_gucs {
        validate_extra_guc_for_config(key.as_str(), value.as_str())?;
    }

    validate_non_empty_path("process.binaries.postgres", &cfg.process.binaries.postgres)?;
    validate_absolute_path("process.binaries.postgres", &cfg.process.binaries.postgres)?;
    validate_non_empty_path("process.binaries.pg_ctl", &cfg.process.binaries.pg_ctl)?;
    validate_absolute_path("process.binaries.pg_ctl", &cfg.process.binaries.pg_ctl)?;
    validate_non_empty_path(
        "process.binaries.pg_rewind",
        &cfg.process.binaries.pg_rewind,
    )?;
    validate_absolute_path(
        "process.binaries.pg_rewind",
        &cfg.process.binaries.pg_rewind,
    )?;
    validate_non_empty_path("process.binaries.initdb", &cfg.process.binaries.initdb)?;
    validate_absolute_path("process.binaries.initdb", &cfg.process.binaries.initdb)?;
    validate_non_empty_path(
        "process.binaries.pg_basebackup",
        &cfg.process.binaries.pg_basebackup,
    )?;
    validate_absolute_path(
        "process.binaries.pg_basebackup",
        &cfg.process.binaries.pg_basebackup,
    )?;
    validate_non_empty_path("process.binaries.psql", &cfg.process.binaries.psql)?;
    validate_absolute_path("process.binaries.psql", &cfg.process.binaries.psql)?;

    validate_timeout(
        "process.pg_rewind_timeout_ms",
        cfg.process.pg_rewind_timeout_ms,
    )?;
    validate_timeout(
        "process.bootstrap_timeout_ms",
        cfg.process.bootstrap_timeout_ms,
    )?;
    validate_timeout("process.fencing_timeout_ms", cfg.process.fencing_timeout_ms)?;

    validate_timeout(
        "logging.postgres.poll_interval_ms",
        cfg.logging.postgres.poll_interval_ms,
    )?;
    if let Some(path) = cfg.logging.postgres.pg_ctl_log_file.as_ref() {
        validate_non_empty_path("logging.postgres.pg_ctl_log_file", path)?;
        validate_absolute_path("logging.postgres.pg_ctl_log_file", path)?;
    }
    if let Some(path) = cfg.logging.postgres.log_dir.as_ref() {
        validate_non_empty_path("logging.postgres.log_dir", path)?;
        validate_absolute_path("logging.postgres.log_dir", path)?;
    }
    if cfg.logging.postgres.cleanup.enabled {
        if cfg.logging.postgres.cleanup.max_files == 0 {
            return Err(ConfigError::Validation {
                field: "logging.postgres.cleanup.max_files",
                message: "must be greater than zero when cleanup is enabled".to_string(),
            });
        }
        if cfg.logging.postgres.cleanup.max_age_seconds == 0 {
            return Err(ConfigError::Validation {
                field: "logging.postgres.cleanup.max_age_seconds",
                message: "must be greater than zero when cleanup is enabled".to_string(),
            });
        }
        if cfg.logging.postgres.cleanup.protect_recent_seconds == 0 {
            return Err(ConfigError::Validation {
                field: "logging.postgres.cleanup.protect_recent_seconds",
                message: "must be greater than zero when cleanup is enabled".to_string(),
            });
        }
    }

    if let Some(path) = cfg.logging.sinks.file.path.as_ref() {
        validate_non_empty_path("logging.sinks.file.path", path)?;
    }

    if cfg.logging.sinks.file.enabled && cfg.logging.sinks.file.path.is_none() {
        return Err(ConfigError::Validation {
            field: "logging.sinks.file.path",
            message: "must be configured when logging.sinks.file.enabled is true".to_string(),
        });
    }

    validate_non_empty_path("postgres.log_file", &cfg.postgres.log_file)?;
    validate_absolute_path("postgres.log_file", &cfg.postgres.log_file)?;

    if cfg.logging.sinks.file.enabled {
        if let Some(path) = cfg.logging.sinks.file.path.as_ref() {
            validate_absolute_path("logging.sinks.file.path", path)?;
        }
    }

    validate_logging_path_ownership_invariants(cfg)?;

    if cfg.dcs.endpoints.is_empty() {
        return Err(ConfigError::Validation {
            field: "dcs.endpoints",
            message: "must contain at least one endpoint".to_string(),
        });
    }

    if cfg.dcs.scope.trim().is_empty() {
        return Err(ConfigError::Validation {
            field: "dcs.scope",
            message: "must not be empty".to_string(),
        });
    }

    if cfg.ha.loop_interval_ms == 0 {
        return Err(ConfigError::Validation {
            field: "ha.loop_interval_ms",
            message: "must be greater than zero".to_string(),
        });
    }

    if cfg.ha.lease_ttl_ms == 0 {
        return Err(ConfigError::Validation {
            field: "ha.lease_ttl_ms",
            message: "must be greater than zero".to_string(),
        });
    }

    if cfg.ha.lease_ttl_ms <= cfg.ha.loop_interval_ms {
        return Err(ConfigError::Validation {
            field: "ha.lease_ttl_ms",
            message: "must be greater than ha.loop_interval_ms".to_string(),
        });
    }

    match &cfg.api.security.auth {
        crate::config::ApiAuthConfig::Disabled => {}
        crate::config::ApiAuthConfig::RoleTokens(tokens) => {
            validate_optional_secret_source_non_empty(
                "api.security.auth.role_tokens.read_token.path",
                "api.security.auth.role_tokens.read_token.content",
                "api.security.auth.role_tokens.read_token.env",
                tokens.read_token.as_ref(),
            )?;
            validate_optional_secret_source_non_empty(
                "api.security.auth.role_tokens.admin_token.path",
                "api.security.auth.role_tokens.admin_token.content",
                "api.security.auth.role_tokens.admin_token.env",
                tokens.admin_token.as_ref(),
            )?;
            if tokens.read_token.is_none() && tokens.admin_token.is_none() {
                return Err(ConfigError::Validation {
                    field: "api.security.auth.role_tokens",
                    message: "at least one of read_token or admin_token must be configured"
                        .to_string(),
                });
            }
        }
    }

    validate_tls_server_config(
        "api.security.tls.identity",
        "api.security.tls.identity.cert_chain",
        "api.security.tls.identity.private_key",
        &cfg.api.security.tls,
    )?;
    validate_tls_client_auth_config(
        "api.security.tls.client_auth",
        "api.security.tls.client_auth.client_ca",
        &cfg.api.security.tls,
    )?;
    validate_pgtm_config(cfg)?;

    validate_dcs_init_config(cfg)?;

    Ok(())
}

fn validate_distinct_postgres_role_usernames(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    let roles = [
        ("postgres.roles.superuser.username", cfg.postgres.roles.superuser.username.as_str()),
        ("postgres.roles.replicator.username", cfg.postgres.roles.replicator.username.as_str()),
        ("postgres.roles.rewinder.username", cfg.postgres.roles.rewinder.username.as_str()),
    ];
    for (current_index, (current_field, current_username)) in roles.iter().enumerate() {
        for (other_field, other_username) in roles.iter().skip(current_index + 1) {
            if current_username == other_username {
                return Err(ConfigError::Validation {
                    field: other_field,
                    message: format!(
                        "must differ from {current_field} (both were `{current_username}`)"
                    ),
                });
            }
        }
    }
    Ok(())
}

fn validate_distinct_role_usernames(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    if cfg.postgres.roles.superuser.username == cfg.postgres.roles.replicator.username {
        return Err(ConfigError::Validation {
            field: "postgres.roles.replicator.username",
            message: format!(
                "must differ from postgres.roles.superuser.username (`{}`) so replication does not reuse the superuser role",
                cfg.postgres.roles.superuser.username
            ),
        });
    }
    if cfg.postgres.roles.superuser.username == cfg.postgres.roles.rewinder.username {
        return Err(ConfigError::Validation {
            field: "postgres.roles.rewinder.username",
            message: format!(
                "must differ from postgres.roles.superuser.username (`{}`) so rewind does not reuse the superuser role",
                cfg.postgres.roles.superuser.username
            ),
        });
    }
    if cfg.postgres.roles.replicator.username == cfg.postgres.roles.rewinder.username {
        return Err(ConfigError::Validation {
            field: "postgres.roles.rewinder.username",
            message: format!(
                "must differ from postgres.roles.replicator.username (`{}`); replication and rewind use separate roles",
                cfg.postgres.roles.replicator.username
            ),
        });
    }
    Ok(())
}

fn validate_extra_guc_for_config(key: &str, value: &str) -> Result<(), ConfigError> {
    validate_extra_guc_entry(key, value).map_err(|err| match err {
        ManagedPostgresConfError::InvalidExtraGuc { key, message } => ConfigError::Validation {
            field: "postgres.extra_gucs",
            message: format!("entry `{key}` invalid: {message}"),
        },
        ManagedPostgresConfError::ReservedExtraGuc { key } => ConfigError::Validation {
            field: "postgres.extra_gucs",
            message: format!("entry `{key}` is reserved by pgtuskmaster"),
        },
        ManagedPostgresConfError::InvalidPrimarySlotName { slot, message } => {
            ConfigError::Validation {
                field: "postgres.extra_gucs",
                message: format!(
                    "unexpected replica slot validation while checking extra gucs `{slot}`: {message}"
                ),
            }
        }
    })
}

fn validate_logging_path_ownership_invariants(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    let Some(sink_path) = cfg.logging.sinks.file.path.as_ref() else {
        return Ok(());
    };
    if !cfg.logging.sinks.file.enabled {
        return Ok(());
    }

    let effective_pg_ctl_log_file = match cfg.logging.postgres.pg_ctl_log_file.as_ref() {
        Some(path) => path,
        None => &cfg.postgres.log_file,
    };

    let sink_path = normalize_path_lexical(sink_path);
    let postgres_log_file = normalize_path_lexical(&cfg.postgres.log_file);
    let effective_pg_ctl_log_file = normalize_path_lexical(effective_pg_ctl_log_file);

    let tailed_files: [(&'static str, &PathBuf); 2] = [
        ("postgres.log_file", &postgres_log_file),
        (
            "logging.postgres.pg_ctl_log_file",
            &effective_pg_ctl_log_file,
        ),
    ];

    for (field, path) in tailed_files {
        if &sink_path == path {
            return Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                message: format!("must not equal tailed postgres input {field}"),
            });
        }
    }

    if let Some(log_dir) = cfg.logging.postgres.log_dir.as_ref() {
        let log_dir = normalize_path_lexical(log_dir);
        if sink_path.starts_with(&log_dir) {
            return Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                message: "must not be inside logging.postgres.log_dir (would self-ingest)"
                    .to_string(),
            });
        }
    }

    Ok(())
}

fn validate_non_empty_path(field: &'static str, path: &Path) -> Result<(), ConfigError> {
    if path.as_os_str().is_empty() {
        return Err(ConfigError::Validation {
            field,
            message: "must not be empty".to_string(),
        });
    }
    Ok(())
}

fn validate_timeout(field: &'static str, value: u64) -> Result<(), ConfigError> {
    if !(MIN_TIMEOUT_MS..=MAX_TIMEOUT_MS).contains(&value) {
        return Err(ConfigError::Validation {
            field,
            message: format!("must be between {MIN_TIMEOUT_MS} and {MAX_TIMEOUT_MS} ms"),
        });
    }
    Ok(())
}

fn validate_port(field: &'static str, value: u16) -> Result<(), ConfigError> {
    if value == 0 {
        return Err(ConfigError::Validation {
            field,
            message: "must be greater than zero".to_string(),
        });
    }
    Ok(())
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), ConfigError> {
    if value.trim().is_empty() {
        return Err(ConfigError::Validation {
            field,
            message: "must not be empty".to_string(),
        });
    }
    Ok(())
}

fn validate_optional_secret_source_non_empty(
    path_field: &'static str,
    content_field: &'static str,
    env_field: &'static str,
    value: Option<&SecretSource>,
) -> Result<(), ConfigError> {
    if let Some(secret) = value {
        validate_secret_source_non_empty(path_field, content_field, env_field, secret)?;
    }
    Ok(())
}

fn validate_role_auth(
    password_path_field: &'static str,
    password_content_field: &'static str,
    password_env_field: &'static str,
    auth: &RoleAuthConfig,
) -> Result<(), ConfigError> {
    match auth {
        RoleAuthConfig::Tls => Ok(()),
        RoleAuthConfig::Password { password } => validate_secret_source_non_empty(
            password_path_field,
            password_content_field,
            password_env_field,
            password,
        ),
    }
}

fn validate_postgres_auth_tls_invariants(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    validate_postgres_role_auth_supported(
        "postgres.roles.superuser.auth",
        &cfg.postgres.roles.superuser.auth,
    )?;
    validate_postgres_role_auth_supported(
        "postgres.roles.replicator.auth",
        &cfg.postgres.roles.replicator.auth,
    )?;
    validate_postgres_role_auth_supported(
        "postgres.roles.rewinder.auth",
        &cfg.postgres.roles.rewinder.auth,
    )?;

    validate_postgres_conn_identity_ssl_mode_supported(
        "postgres.local_conn_identity.ssl_mode",
        &cfg.postgres.local_conn_identity,
        cfg.postgres.tls.mode,
    )?;
    validate_postgres_conn_identity_ssl_mode_supported(
        "postgres.rewind_conn_identity.ssl_mode",
        &cfg.postgres.rewind_conn_identity,
        cfg.postgres.tls.mode,
    )?;

    Ok(())
}

fn validate_postgres_role_auth_supported(
    field: &'static str,
    auth: &RoleAuthConfig,
) -> Result<(), ConfigError> {
    match auth {
        RoleAuthConfig::Tls => Err(ConfigError::Validation {
            field,
            message:
                "postgresql role TLS client auth is not implemented; use type = \"password\" for now"
                    .to_string(),
        }),
        RoleAuthConfig::Password { .. } => Ok(()),
    }
}

fn validate_postgres_conn_identity_ssl_mode_supported(
    field: &'static str,
    identity: &PostgresConnIdentityConfig,
    tls_mode: crate::config::ApiTlsMode,
) -> Result<(), ConfigError> {
    let ssl_mode = identity.ssl_mode;
    if matches!(tls_mode, crate::config::ApiTlsMode::Disabled)
        && postgres_ssl_mode_requires_server_tls(ssl_mode)
    {
        return Err(ConfigError::Validation {
            field,
            message: format!(
                "must not require server TLS when postgres.tls.mode is disabled (got `{}`)",
                ssl_mode.as_str()
            ),
        });
    }

    if postgres_ssl_mode_requires_root_cert(ssl_mode) && identity.ca_cert.is_none() {
        return Err(ConfigError::Validation {
            field,
            message: format!(
                "must configure the matching `{}.ca_cert` path when ssl_mode is `{}`",
                field.trim_end_matches(".ssl_mode"),
                ssl_mode.as_str()
            ),
        });
    }

    Ok(())
}

fn postgres_ssl_mode_requires_server_tls(ssl_mode: crate::pginfo::conninfo::PgSslMode) -> bool {
    matches!(
        ssl_mode,
        crate::pginfo::conninfo::PgSslMode::Require
            | crate::pginfo::conninfo::PgSslMode::VerifyCa
            | crate::pginfo::conninfo::PgSslMode::VerifyFull
    )
}

fn postgres_ssl_mode_requires_root_cert(ssl_mode: crate::pginfo::conninfo::PgSslMode) -> bool {
    matches!(
        ssl_mode,
        crate::pginfo::conninfo::PgSslMode::VerifyCa
            | crate::pginfo::conninfo::PgSslMode::VerifyFull
    )
}

fn validate_tls_server_config(
    identity_field: &'static str,
    cert_chain_field: &'static str,
    private_key_field: &'static str,
    cfg: &TlsServerConfig,
) -> Result<(), ConfigError> {
    if matches!(cfg.mode, crate::config::ApiTlsMode::Disabled) {
        return Ok(());
    }

    let identity = cfg
        .identity
        .as_ref()
        .ok_or_else(|| ConfigError::Validation {
            field: identity_field,
            message: "tls identity must be configured when tls.mode is optional or required"
                .to_string(),
        })?;

    validate_inline_or_path_non_empty(cert_chain_field, &identity.cert_chain, false)?;
    validate_inline_or_path_non_empty(private_key_field, &identity.private_key, false)?;
    Ok(())
}

fn validate_tls_client_auth_config(
    client_auth_field: &'static str,
    client_ca_field: &'static str,
    cfg: &TlsServerConfig,
) -> Result<(), ConfigError> {
    let Some(client_auth) = cfg.client_auth.as_ref() else {
        return Ok(());
    };

    if matches!(cfg.mode, crate::config::ApiTlsMode::Disabled) {
        return Err(ConfigError::Validation {
            field: client_auth_field,
            message: "must not be configured when tls.mode is disabled".to_string(),
        });
    }

    validate_inline_or_path_non_empty(client_ca_field, &client_auth.client_ca, false)?;
    Ok(())
}

fn validate_dcs_init_config(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    let Some(init) = cfg.dcs.init.as_ref() else {
        return Ok(());
    };

    validate_non_empty("dcs.init.payload_json", init.payload_json.as_str())?;

    let _: serde_json::Value = serde_json::from_str(init.payload_json.as_str()).map_err(|err| {
        ConfigError::Validation {
            field: "dcs.init.payload_json",
            message: format!("must be valid JSON: {err}"),
        }
    })?;

    let _: RuntimeConfig = serde_json::from_str(init.payload_json.as_str()).map_err(|err| {
        ConfigError::Validation {
            field: "dcs.init.payload_json",
            message: format!("must decode as a RuntimeConfig JSON document: {err}"),
        }
    })?;

    Ok(())
}

fn validate_pgtm_config(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    let Some(pgtm) = cfg.pgtm.as_ref() else {
        return Ok(());
    };

    if let Some(api_url) = pgtm.api_url.as_deref() {
        validate_pgtm_api_url(api_url, cfg.api.security.tls.mode)?;
    }

    if let Some(api_client) = pgtm.api_client.as_ref() {
        validate_pgtm_client_material(
            PgtmClientMaterialFields {
                ca_cert: "pgtm.api_client.ca_cert",
                client_cert: "pgtm.api_client.client_cert",
                client_key_path: "pgtm.api_client.client_key.path",
                client_key_content: "pgtm.api_client.client_key.content",
                client_key_env: "pgtm.api_client.client_key.env",
            },
            api_client.ca_cert.as_ref(),
            api_client.client_cert.as_ref(),
            api_client.client_key.as_ref(),
        )?;

        if matches!(
            cfg.api.security.tls.mode,
            crate::config::ApiTlsMode::Disabled
        ) {
            return Err(ConfigError::Validation {
                field: "pgtm.api_client",
                message: "must not be configured when api.security.tls.mode is disabled"
                    .to_string(),
            });
        }
    }

    if let Some(postgres_client) = pgtm.postgres_client.as_ref() {
        validate_pgtm_client_material(
            PgtmClientMaterialFields {
                ca_cert: "pgtm.postgres_client.ca_cert",
                client_cert: "pgtm.postgres_client.client_cert",
                client_key_path: "pgtm.postgres_client.client_key.path",
                client_key_content: "pgtm.postgres_client.client_key.content",
                client_key_env: "pgtm.postgres_client.client_key.env",
            },
            postgres_client.ca_cert.as_ref(),
            postgres_client.client_cert.as_ref(),
            postgres_client.client_key.as_ref(),
        )?;
    }

    if matches!(
        cfg.api.security.tls.mode,
        crate::config::ApiTlsMode::Required
    ) {
        if let Some(api_url) = pgtm.api_url.as_deref() {
            let parsed = reqwest::Url::parse(api_url).map_err(|err| ConfigError::Validation {
                field: "pgtm.api_url",
                message: format!("must be a valid absolute http or https URL: {err}"),
            })?;
            if parsed.scheme() != "https" {
                return Err(ConfigError::Validation {
                    field: "pgtm.api_url",
                    message: "must use https when api.security.tls.mode is required".to_string(),
                });
            }
        }
        if cfg
            .api
            .security
            .tls
            .client_auth
            .as_ref()
            .is_some_and(|auth| auth.require_client_cert)
        {
            let has_client_cert = pgtm
                .api_client
                .as_ref()
                .and_then(|client| client.client_cert.as_ref())
                .is_some();
            let has_client_key = pgtm
                .api_client
                .as_ref()
                .and_then(|client| client.client_key.as_ref())
                .is_some();
            if !has_client_cert || !has_client_key {
                return Err(ConfigError::Validation {
                    field: "pgtm.api_client",
                    message:
                        "must provide client_cert and client_key when api.security.tls.client_auth.require_client_cert is true"
                            .to_string(),
                });
            }
        }
    }

    if let Some(api_url) = pgtm.api_url.as_deref() {
        let parsed = reqwest::Url::parse(api_url).map_err(|err| ConfigError::Validation {
            field: "pgtm.api_url",
            message: format!("must be a valid absolute http or https URL: {err}"),
        })?;
        if parsed.scheme() == "http" && pgtm.api_client.is_some() {
            return Err(ConfigError::Validation {
                field: "pgtm.api_client",
                message: "must not be configured when pgtm.api_url uses http".to_string(),
            });
        }
    }

    Ok(())
}

fn validate_pgtm_api_url(
    api_url: &str,
    tls_mode: crate::config::ApiTlsMode,
) -> Result<(), ConfigError> {
    let parsed = reqwest::Url::parse(api_url).map_err(|err| ConfigError::Validation {
        field: "pgtm.api_url",
        message: format!("must be a valid absolute http or https URL: {err}"),
    })?;
    match parsed.scheme() {
        "http" => {
            if matches!(tls_mode, crate::config::ApiTlsMode::Disabled) {
                return Ok(());
            }
            Ok(())
        }
        "https" => {
            if matches!(tls_mode, crate::config::ApiTlsMode::Disabled) {
                return Err(ConfigError::Validation {
                    field: "pgtm.api_url",
                    message: "must not use https when api.security.tls.mode is disabled"
                        .to_string(),
                });
            }
            Ok(())
        }
        _ => Err(ConfigError::Validation {
            field: "pgtm.api_url",
            message: "must use http or https".to_string(),
        }),
    }
}

struct PgtmClientMaterialFields {
    ca_cert: &'static str,
    client_cert: &'static str,
    client_key_path: &'static str,
    client_key_content: &'static str,
    client_key_env: &'static str,
}

fn validate_pgtm_client_material(
    fields: PgtmClientMaterialFields,
    ca_cert: Option<&InlineOrPath>,
    client_cert: Option<&InlineOrPath>,
    client_key: Option<&SecretSource>,
) -> Result<(), ConfigError> {
    if let Some(ca_cert) = ca_cert {
        validate_inline_or_path_non_empty(fields.ca_cert, ca_cert, false)?;
    }
    if let Some(client_cert) = client_cert {
        validate_inline_or_path_non_empty(fields.client_cert, client_cert, false)?;
    }
    if let Some(client_key) = client_key {
        validate_secret_source_non_empty(
            fields.client_key_path,
            fields.client_key_content,
            fields.client_key_env,
            client_key,
        )?;
    }

    if client_cert.is_some() && client_key.is_none() {
        return Err(ConfigError::Validation {
            field: fields.client_key_path,
            message: "must be configured when client_cert is set".to_string(),
        });
    }
    if client_key.is_some() && client_cert.is_none() {
        return Err(ConfigError::Validation {
            field: fields.client_cert,
            message: "must be configured when client_key is set".to_string(),
        });
    }

    Ok(())
}

fn validate_secret_source_non_empty(
    path_field: &'static str,
    content_field: &'static str,
    env_field: &'static str,
    secret: &SecretSource,
) -> Result<(), ConfigError> {
    match secret {
        SecretSource::Path(path) => validate_non_empty_path(path_field, path),
        SecretSource::PathConfig { path } => validate_non_empty_path(path_field, path),
        SecretSource::Inline { content } => validate_non_empty(content_field, content.as_str()),
        SecretSource::Env { env } => validate_non_empty(env_field, env.as_str()),
    }
}

fn validate_inline_or_path_non_empty(
    field: &'static str,
    value: &InlineOrPath,
    allow_empty_inline: bool,
) -> Result<(), ConfigError> {
    match value {
        InlineOrPath::Path(path) => validate_non_empty_path(field, path),
        InlineOrPath::PathConfig { path } => validate_non_empty_path(field, path),
        InlineOrPath::Inline { content } => {
            if allow_empty_inline {
                Ok(())
            } else {
                validate_non_empty(field, content.as_str())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::config::schema::{
        ApiAuthConfig, ApiConfig, ApiRoleTokensConfig, ApiSecurityConfig, ApiTlsMode, BinaryPaths,
        ClusterConfig, DcsConfig, DebugConfig, FileSinkConfig, FileSinkMode, HaConfig,
        InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig, LoggingSinksConfig, PgHbaConfig,
        PgIdentConfig, PostgresConfig, PostgresConnIdentityConfig, PostgresLoggingConfig,
        PostgresRoleConfig, PostgresRolesConfig, ProcessConfig, RoleAuthConfig, RuntimeConfig,
        StderrSinkConfig, TlsServerConfig,
    };
    use crate::pginfo::conninfo::PgSslMode;

    fn sample_password_auth() -> RoleAuthConfig {
        RoleAuthConfig::Password {
            password: crate::config::SecretSource::Inline {
                content: "secret-password".to_string(),
            },
        }
    }

    fn expect_validation_error(
        result: Result<(), ConfigError>,
        expected_field: &'static str,
        expected_message_fragment: &str,
    ) -> Result<(), String> {
        match result {
            Err(ConfigError::Validation { field, message }) => {
                if field != expected_field {
                    return Err(format!(
                        "expected validation field {expected_field}, got {field}"
                    ));
                }
                if !message.contains(expected_message_fragment) {
                    return Err(format!(
                        "expected validation message to contain {expected_message_fragment:?}, got {message:?}"
                    ));
                }
                Ok(())
            }
            other => Err(format!(
                "expected validation error for {expected_field}, got {other:?}"
            )),
        }
    }

    fn base_runtime_config() -> RuntimeConfig {
        RuntimeConfig {
            cluster: ClusterConfig {
                name: "cluster-a".to_string(),
                member_id: "member-a".to_string(),
            },
            postgres: PostgresConfig {
                data_dir: PathBuf::from("/var/lib/postgresql/data"),
                connect_timeout_s: 5,
                listen_host: "127.0.0.1".to_string(),
                listen_port: 5432,
                advertise_port: None,
                socket_dir: PathBuf::from("/tmp/pgtuskmaster/socket"),
                log_file: PathBuf::from("/tmp/pgtuskmaster/postgres.log"),
                local_conn_identity: PostgresConnIdentityConfig {
                    user: "postgres".to_string(),
                    dbname: "postgres".to_string(),
                    ssl_mode: PgSslMode::Prefer,
                    ca_cert: None,
                },
                rewind_conn_identity: PostgresConnIdentityConfig {
                    user: "rewinder".to_string(),
                    dbname: "postgres".to_string(),
                    ssl_mode: PgSslMode::Prefer,
                    ca_cert: None,
                },
                tls: TlsServerConfig {
                    mode: ApiTlsMode::Disabled,
                    identity: None,
                    client_auth: None,
                },
                roles: PostgresRolesConfig {
                    superuser: PostgresRoleConfig {
                        username: "postgres".to_string(),
                        auth: sample_password_auth(),
                    },
                    replicator: PostgresRoleConfig {
                        username: "replicator".to_string(),
                        auth: sample_password_auth(),
                    },
                    rewinder: PostgresRoleConfig {
                        username: "rewinder".to_string(),
                        auth: sample_password_auth(),
                    },
                },
                pg_hba: PgHbaConfig {
                    source: InlineOrPath::Inline {
                        content: "local all all trust\n".to_string(),
                    },
                },
                pg_ident: PgIdentConfig {
                    source: InlineOrPath::Inline {
                        content: "# empty\n".to_string(),
                    },
                },
                extra_gucs: std::collections::BTreeMap::new(),
            },
            dcs: DcsConfig {
                endpoints: vec![crate::config::DcsEndpoint::from_socket_addr(
                    std::net::SocketAddr::from(([127, 0, 0, 1], 2379)),
                )],
                scope: "scope-a".to_string(),
                init: None,
            },
            ha: HaConfig {
                loop_interval_ms: 1_000,
                lease_ttl_ms: 10_000,
            },
            process: ProcessConfig {
                pg_rewind_timeout_ms: 120_000,
                bootstrap_timeout_ms: 300_000,
                fencing_timeout_ms: 30_000,
                binaries: BinaryPaths {
                    postgres: PathBuf::from("/usr/bin/postgres"),
                    pg_ctl: PathBuf::from("/usr/bin/pg_ctl"),
                    pg_rewind: PathBuf::from("/usr/bin/pg_rewind"),
                    initdb: PathBuf::from("/usr/bin/initdb"),
                    pg_basebackup: PathBuf::from("/usr/bin/pg_basebackup"),
                    psql: PathBuf::from("/usr/bin/psql"),
                },
            },
            logging: LoggingConfig {
                level: LogLevel::Info,
                capture_subprocess_output: true,
                postgres: PostgresLoggingConfig {
                    enabled: true,
                    pg_ctl_log_file: None,
                    log_dir: None,
                    poll_interval_ms: 200,
                    cleanup: LogCleanupConfig {
                        enabled: true,
                        max_files: 10,
                        max_age_seconds: 60,
                        protect_recent_seconds: 300,
                    },
                },
                sinks: LoggingSinksConfig {
                    stderr: StderrSinkConfig { enabled: true },
                    file: FileSinkConfig {
                        enabled: false,
                        path: None,
                        mode: FileSinkMode::Append,
                    },
                },
            },
            api: ApiConfig {
                listen_addr: std::net::SocketAddr::from(([127, 0, 0, 1], 8080)),
                security: ApiSecurityConfig {
                    tls: TlsServerConfig {
                        mode: ApiTlsMode::Disabled,
                        identity: None,
                        client_auth: None,
                    },
                    auth: ApiAuthConfig::Disabled,
                },
            },
            pgtm: None,
            debug: DebugConfig { enabled: false },
        }
    }

    #[test]
    fn validate_runtime_config_accepts_valid_config() {
        let cfg = base_runtime_config();
        assert!(validate_runtime_config(&cfg).is_ok());
    }

    #[test]
    fn validate_runtime_config_rejects_postgres_role_tls_auth() -> Result<(), String> {
        let mut superuser_cfg = base_runtime_config();
        superuser_cfg.postgres.roles.superuser.auth = RoleAuthConfig::Tls;
        expect_validation_error(
            validate_runtime_config(&superuser_cfg),
            "postgres.roles.superuser.auth",
            "type = \"password\"",
        )?;

        let mut replicator_cfg = base_runtime_config();
        replicator_cfg.postgres.roles.replicator.auth = RoleAuthConfig::Tls;
        expect_validation_error(
            validate_runtime_config(&replicator_cfg),
            "postgres.roles.replicator.auth",
            "type = \"password\"",
        )?;

        let mut rewinder_cfg = base_runtime_config();
        rewinder_cfg.postgres.roles.rewinder.auth = RoleAuthConfig::Tls;
        expect_validation_error(
            validate_runtime_config(&rewinder_cfg),
            "postgres.roles.rewinder.auth",
            "type = \"password\"",
        )
    }

    #[test]
    fn validate_runtime_config_rejects_local_conn_ssl_mode_requiring_tls_when_postgres_tls_disabled(
    ) -> Result<(), String> {
        let mut cfg = base_runtime_config();
        cfg.postgres.local_conn_identity.ssl_mode = PgSslMode::Require;

        expect_validation_error(
            validate_runtime_config(&cfg),
            "postgres.local_conn_identity.ssl_mode",
            "postgres.tls.mode is disabled",
        )
    }

    #[test]
    fn validate_runtime_config_rejects_rewind_conn_ssl_mode_requiring_tls_when_postgres_tls_disabled(
    ) -> Result<(), String> {
        let mut cfg = base_runtime_config();
        cfg.postgres.rewind_conn_identity.ssl_mode = PgSslMode::VerifyFull;

        expect_validation_error(
            validate_runtime_config(&cfg),
            "postgres.rewind_conn_identity.ssl_mode",
            "postgres.tls.mode is disabled",
        )
    }

    #[test]
    fn validate_runtime_config_rejects_empty_binary_path() {
        let mut cfg = base_runtime_config();
        cfg.process.binaries.pg_ctl = PathBuf::new();

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "process.binaries.pg_ctl",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_non_absolute_binary_paths() {
        let mut cfg = base_runtime_config();
        cfg.process.binaries.pg_ctl = PathBuf::from("pg_ctl");
        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "process.binaries.pg_ctl",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_bad_timeout() {
        let mut cfg = base_runtime_config();
        cfg.process.bootstrap_timeout_ms = 0;

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "process.bootstrap_timeout_ms",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_invalid_postgres_runtime_fields() {
        let mut cfg = base_runtime_config();
        cfg.postgres.listen_host = " ".to_string();
        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.listen_host",
                ..
            })
        ));

        let mut cfg = base_runtime_config();
        cfg.postgres.listen_port = 0;
        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.listen_port",
                ..
            })
        ));

        let mut cfg = base_runtime_config();
        cfg.postgres.advertise_port = Some(0);
        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.advertise_port",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_missing_dcs_and_ha_invariants() {
        let mut cfg = base_runtime_config();
        cfg.dcs.endpoints.clear();

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "dcs.endpoints",
                ..
            })
        ));

        let mut cfg = base_runtime_config();
        cfg.ha.lease_ttl_ms = cfg.ha.loop_interval_ms;

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "ha.lease_ttl_ms",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_blank_api_tokens() {
        let mut cfg = base_runtime_config();
        cfg.api.security.auth = ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
            read_token: Some(crate::config::SecretSource::Inline {
                content: " ".to_string(),
            }),
            admin_token: None,
        });

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "api.security.auth.role_tokens.read_token.content",
                ..
            })
        ));

        let mut cfg = base_runtime_config();
        cfg.api.security.auth = ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
            read_token: None,
            admin_token: Some(crate::config::SecretSource::Inline {
                content: "\t".to_string(),
            }),
        });

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "api.security.auth.role_tokens.admin_token.content",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_blank_env_secret_name() {
        let mut cfg = base_runtime_config();
        cfg.api.security.auth = ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
            read_token: Some(crate::config::SecretSource::Env {
                env: "   ".to_string(),
            }),
            admin_token: None,
        });

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "api.security.auth.role_tokens.read_token.env",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_https_pgtm_api_url_when_api_tls_disabled() {
        let mut cfg = base_runtime_config();
        cfg.pgtm = Some(crate::config::PgtmConfig {
            api_url: Some("https://cluster.example:8443".to_string()),
            api_client: None,
            postgres_client: None,
        });

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "pgtm.api_url",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_requires_pgtm_client_key_when_client_cert_present() {
        let mut cfg = base_runtime_config();
        cfg.api.security.tls.mode = ApiTlsMode::Required;
        cfg.api.security.tls.identity = Some(crate::config::TlsServerIdentityConfig {
            cert_chain: InlineOrPath::Inline {
                content: "-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n"
                    .to_string(),
            },
            private_key: InlineOrPath::Inline {
                content: "-----BEGIN PRIVATE KEY-----\nMIIB\n-----END PRIVATE KEY-----\n"
                    .to_string(),
            },
        });
        cfg.pgtm = Some(crate::config::PgtmConfig {
            api_url: Some("https://cluster.example:8443".to_string()),
            api_client: Some(crate::config::PgtmApiClientConfig {
                ca_cert: None,
                client_cert: Some(InlineOrPath::Inline {
                    content: "-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n"
                        .to_string(),
                }),
                client_key: None,
            }),
            postgres_client: None,
        });

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "pgtm.api_client.client_key.path",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_enabled_without_path() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = None;

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_empty_path() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::new());

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_accepts_file_sink_enabled_with_path() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::from("/tmp/pgtuskmaster.jsonl"));

        assert!(validate_runtime_config(&cfg).is_ok());
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_equal_to_tailed_log_via_dot_segments() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::from("/tmp/pgtuskmaster/./postgres.log"));

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_equal_to_tailed_log_via_parent_segments() {
        let mut cfg = base_runtime_config();
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::from("/tmp/pgtuskmaster/tmp/../postgres.log"));

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn validate_runtime_config_rejects_file_sink_inside_log_dir_via_dot_segments() {
        let mut cfg = base_runtime_config();
        cfg.logging.postgres.log_dir = Some(PathBuf::from("/tmp/pgtuskmaster/log_dir"));
        cfg.logging.sinks.file.enabled = true;
        cfg.logging.sinks.file.path = Some(PathBuf::from("/tmp/pgtuskmaster/log_dir/./out.jsonl"));

        let err = validate_runtime_config(&cfg);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "logging.sinks.file.path",
                ..
            })
        ));
    }

    #[test]
    fn load_runtime_config_missing_required_sections_is_rejected(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-{unique}.toml"));

        let toml = r#"
[cluster]
name = "cluster-a"
member_id = "member-a"
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(err, Err(ConfigError::Parse { .. })));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_rejects_unknown_top_level_version_field(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-invalid-{unique}.toml"));

        let toml = r#"
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(err, Err(ConfigError::Parse { .. })));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_rejects_unknown_fields() -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-invalid-{unique}.toml"));

        let toml = r#"
[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
connect_timeout_s = 5
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
unknown = 10

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
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[logging]
level = "info"
capture_subprocess_output = true
postgres = { enabled = true, poll_interval_ms = 200, cleanup = { enabled = true, max_files = 10, max_age_seconds = 60 } }
sinks = { stderr = { enabled = true }, file = { enabled = false, mode = "append" } }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(err, Err(ConfigError::Parse { .. })));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_happy_path_with_safe_defaults() -> Result<(), Box<dyn std::error::Error>>
    {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-{unique}.toml"));

        let toml = r#"
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
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#;

        std::fs::write(&path, toml)?;
        let cfg = load_runtime_config(&path)?;
        assert_eq!(cfg.postgres.connect_timeout_s, 5);
        assert_eq!(cfg.process.pg_rewind_timeout_ms, 120_000);
        assert_eq!(cfg.process.bootstrap_timeout_ms, 300_000);
        assert_eq!(cfg.process.fencing_timeout_ms, 30_000);
        assert_eq!(
            cfg.api.listen_addr,
            std::net::SocketAddr::from(([127, 0, 0, 1], 8080))
        );
        assert!(!cfg.debug.enabled);

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_missing_secure_fields_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-missing-{unique}.toml"));

        // Intentionally omit `postgres.local_conn_identity`.
        let toml = r#"
[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
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
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.local_conn_identity",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_missing_process_binaries_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("runtime-config-missing-binaries-{unique}.toml"));

        // Intentionally omit `process.binaries`.
        let toml = r#"
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
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "process.binaries",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_password_auth_missing_password_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-missing-auth-password-{unique}.toml"
        ));

        // Intentionally omit `postgres.roles.superuser.auth.password`.
        let toml = r#"
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
roles = { superuser = { username = "postgres", auth = { type = "password" } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
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
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles.superuser.auth.password",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_missing_postgres_roles_block_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-missing-roles-{unique}.toml"));

        // Intentionally omit `postgres.roles`.
        let toml = r#"
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
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_missing_replicator_role_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-missing-replicator-role-{unique}.toml"
        ));

        // Intentionally omit `postgres.roles.replicator`.
        let toml = r#"
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
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
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
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles.replicator",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn validate_runtime_config_rejects_replicator_reusing_superuser_username() -> Result<(), String> {
        let mut cfg = base_runtime_config();
        cfg.postgres.roles.replicator.username = cfg.postgres.roles.superuser.username.clone();
        expect_validation_error(
            validate_runtime_config(&cfg),
            "postgres.roles.replicator.username",
            "must differ from postgres.roles.superuser.username",
        )
    }

    #[test]
    fn validate_runtime_config_rejects_rewinder_reusing_superuser_username() -> Result<(), String> {
        let mut cfg = base_runtime_config();
        cfg.postgres.roles.rewinder.username = cfg.postgres.roles.superuser.username.clone();
        expect_validation_error(
            validate_runtime_config(&cfg),
            "postgres.roles.rewinder.username",
            "must differ from postgres.roles.superuser.username",
        )
    }

    #[test]
    fn validate_runtime_config_rejects_rewinder_reusing_replicator_username() -> Result<(), String> {
        let mut cfg = base_runtime_config();
        cfg.postgres.roles.rewinder.username = cfg.postgres.roles.replicator.username.clone();
        expect_validation_error(
            validate_runtime_config(&cfg),
            "postgres.roles.rewinder.username",
            "must differ from postgres.roles.replicator.username",
        )
    }

    #[test]
    fn load_runtime_config_missing_replicator_username_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-missing-replicator-username-{unique}.toml"
        ));

        // Intentionally omit `postgres.roles.replicator.username`.
        let toml = r#"
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
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
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
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles.replicator.username",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_missing_replicator_auth_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-missing-replicator-auth-{unique}.toml"
        ));

        // Intentionally omit `postgres.roles.replicator.auth`.
        let toml = r#"
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
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator" }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
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
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles.replicator.auth",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_rejects_conn_identity_role_mismatch(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-conn-identity-mismatch-{unique}.toml"
        ));

        // Intentionally set local_conn_identity.user to a different user than roles.superuser.username.
        let toml = r#"
[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "not-postgres", dbname = "postgres", ssl_mode = "prefer" }
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
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.local_conn_identity.user",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_rejects_blank_password_secret() -> Result<(), Box<dyn std::error::Error>>
    {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-blank-password-secret-{unique}.toml"
        ));

        // Intentionally set password secret content to empty.
        let toml = r#"
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
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
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
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.roles.superuser.auth.password.content",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_rejects_tls_required_without_identity(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-required-tls-no-identity-{unique}.toml"
        ));

        // Intentionally omit `postgres.tls.identity` while requiring TLS.
        let toml = r#"
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
tls = { mode = "required" }
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
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.tls.identity",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_rejects_client_auth_with_tls_disabled(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-client-auth-with-tls-disabled-{unique}.toml"
        ));

        // Intentionally configure client auth while TLS is disabled.
        let toml = r#"
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
tls = { mode = "disabled", client_auth = { client_ca = { content = "client-ca" }, require_client_cert = false } }
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
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "postgres.tls.client_auth",
                ..
            })
        ));

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_rejects_postgres_role_tls_auth_with_actionable_error(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-postgres-role-tls-auth-{unique}.toml"
        ));

        let toml = r#"
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
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        let mapped = match err {
            Err(ConfigError::Validation { field, message }) => {
                if field != "postgres.roles.superuser.auth" {
                    Err(format!(
                        "expected validation field postgres.roles.superuser.auth, got {field}"
                    ))
                } else if !message.contains("type = \"password\"") {
                    Err(format!(
                        "expected validation message to mention password auth, got {message:?}"
                    ))
                } else {
                    Ok(())
                }
            }
            other => Err(format!("expected validation error, got {other:?}")),
        };
        mapped.map_err(std::io::Error::other)?;

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_rejects_ssl_mode_requiring_tls_when_postgres_tls_disabled(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-postgres-ssl-mode-requires-tls-{unique}.toml"
        ));

        let toml = r#"
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
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "verify-full" }
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
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        let mapped = match err {
            Err(ConfigError::Validation { field, message }) => {
                if field != "postgres.local_conn_identity.ssl_mode" {
                    Err(format!(
                        "expected validation field postgres.local_conn_identity.ssl_mode, got {field}"
                    ))
                } else if !message.contains("postgres.tls.mode is disabled") {
                    Err(format!(
                        "expected validation message to mention disabled postgres TLS, got {message:?}"
                    ))
                } else {
                    Ok(())
                }
            }
            other => Err(format!("expected validation error, got {other:?}")),
        };
        mapped.map_err(std::io::Error::other)?;

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_rejects_verify_full_without_ca_cert(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-postgres-verify-full-missing-ca-cert-{unique}.toml"
        ));

        let toml = r#"
[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "disable" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "verify-full" }
tls = { mode = "required", identity = { cert_chain = { content = "cert" }, private_key = { content = "key" } } }
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
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        let mapped = match err {
            Err(ConfigError::Validation { field, message }) => {
                if field != "postgres.rewind_conn_identity.ssl_mode" {
                    Err(format!(
                        "expected validation field postgres.rewind_conn_identity.ssl_mode, got {field}"
                    ))
                } else if !message.contains("postgres.rewind_conn_identity.ca_cert") {
                    Err(format!(
                        "expected validation message to mention postgres.rewind_conn_identity.ca_cert, got {message:?}"
                    ))
                } else {
                    Ok(())
                }
            }
            other => Err(format!("expected validation error, got {other:?}")),
        };
        mapped.map_err(std::io::Error::other)?;

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_rejects_inline_conn_identity_ca_cert(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-postgres-inline-conn-identity-ca-cert-{unique}.toml"
        ));

        let toml = r#"
[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "disable" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "verify-full", ca_cert = { content = "ca-pem" } }
tls = { mode = "required", identity = { cert_chain = { content = "cert" }, private_key = { content = "key" } } }
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
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        let mapped = match err {
            Err(ConfigError::Validation { field, message }) => {
                if field != "postgres.rewind_conn_identity.ca_cert" {
                    Err(format!(
                        "expected validation field postgres.rewind_conn_identity.ca_cert, got {field}"
                    ))
                } else if !message.contains("path-backed CA bundle") {
                    Err(format!(
                        "expected validation message to mention path-backed CA bundle, got {message:?}"
                    ))
                } else {
                    Ok(())
                }
            }
            other => Err(format!("expected validation error, got {other:?}")),
        };
        mapped.map_err(std::io::Error::other)?;

        std::fs::remove_file(&path)?;
        Ok(())
    }

    #[test]
    fn load_runtime_config_rejects_duplicate_postgres_role_usernames(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "runtime-config-duplicate-postgres-role-usernames-{unique}.toml"
        ));

        let toml = r#"
[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
connect_timeout_s = 5
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "disable" }
rewind_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "require" }
tls = { mode = "required", identity = { cert_chain = { content = "cert" }, private_key = { content = "key" } } }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

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

[logging]
level = "info"
capture_subprocess_output = false
postgres = { enabled = false, poll_interval_ms = 1000, cleanup = { enabled = false, max_files = 1, max_age_seconds = 1, protect_recent_seconds = 1 } }
sinks = { stderr = { enabled = true }, file = { enabled = false, mode = "append", path = "/tmp/runtime.jsonl" } }

[api]
listen_addr = "127.0.0.1:8443"
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }

[debug]
enabled = true
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        let mapped = match err {
            Err(ConfigError::Validation { field, message }) => {
                if field != "postgres.roles.rewinder.username" {
                    Err(format!(
                        "expected validation field postgres.roles.rewinder.username, got {field}"
                    ))
                } else if !message.contains("postgres.roles.superuser.username") {
                    Err(format!(
                        "expected validation message to mention superuser username duplication, got {message:?}"
                    ))
                } else {
                    Ok(())
                }
            }
            Ok(_) => Err("expected duplicate role usernames to fail validation".to_string()),
            Err(other) => Err(format!("expected validation error, got {other}")),
        };
        mapped.map_err(std::io::Error::other)?;

        std::fs::remove_file(&path)?;
        Ok(())
    }
}
