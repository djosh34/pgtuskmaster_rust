use std::path::Path;

use thiserror::Error;

use super::defaults::{
    default_api_listen_addr, default_debug_config, default_logging_config,
    default_postgres_connect_timeout_s, normalize_process_config,
};
use super::schema::{
    ApiConfig, ApiSecurityConfig, ConfigVersion, InlineOrPath, PgHbaConfig, PgIdentConfig,
    PostgresConnIdentityConfig, PostgresConfig, PostgresRoleConfig, PostgresRolesConfig,
    RuntimeConfig, RuntimeConfigV2Input, RoleAuthConfig, SecretSource,
    TlsServerConfig, TlsServerIdentityConfig,
};

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

    #[derive(serde::Deserialize)]
    struct ConfigEnvelope {
        config_version: Option<ConfigVersion>,
    }

    let envelope: ConfigEnvelope =
        toml::from_str(&contents).map_err(|source| ConfigError::Parse {
            path: path.display().to_string(),
            source,
        })?;

    let config_version = envelope.config_version.ok_or_else(|| ConfigError::Validation {
        field: "config_version",
        message: "missing required field; set config_version = \"v2\" to use the explicit secure schema".to_string(),
    })?;

    match config_version {
        ConfigVersion::V1 => {
            probe_legacy_v1_shape_for_diagnostics(&contents);
            Err(ConfigError::Validation {
                field: "config_version",
                message: "config_version = \"v1\" is no longer supported because it depends on implicit security defaults; migrate to config_version = \"v2\""
                    .to_string(),
            })
        }
        ConfigVersion::V2 => {
            let raw: RuntimeConfigV2Input =
                toml::from_str(&contents).map_err(|source| ConfigError::Parse {
                    path: path.display().to_string(),
                    source,
                })?;
            let cfg = normalize_runtime_config_v2(raw)?;
            validate_runtime_config(&cfg)?;
            Ok(cfg)
        }
    }
}

fn probe_legacy_v1_shape_for_diagnostics(contents: &str) {
    // We keep the legacy v1 deserialization surface "alive" to:
    // - avoid unused-schema drift during the transition
    // - allow future improvements that surface rich TOML diagnostics for v1 migrations
    //
    // This must never override the v1 migration guidance with a parse error.
    let parsed: Result<toml::Value, toml::de::Error> = toml::from_str(contents);
    let Ok(mut value) = parsed else {
        return;
    };

    let Some(table) = value.as_table_mut() else {
        return;
    };

    let _ = table.remove("config_version");

    let _: Result<super::schema::PartialRuntimeConfig, toml::de::Error> = value.try_into();
}

fn normalize_runtime_config_v2(input: RuntimeConfigV2Input) -> Result<RuntimeConfig, ConfigError> {
    if !matches!(input.config_version, ConfigVersion::V2) {
        return Err(ConfigError::Validation {
            field: "config_version",
            message: "expected config_version = \"v2\"".to_string(),
        });
    }

    let postgres = normalize_postgres_config_v2(input.postgres)?;
    let process = normalize_process_config(input.process);
    let logging = input.logging.unwrap_or_else(default_logging_config);
    let api = normalize_api_config_v2(input.api)?;
    let debug = input.debug.unwrap_or_else(default_debug_config);

    Ok(RuntimeConfig {
        cluster: input.cluster,
        postgres,
        dcs: input.dcs,
        ha: input.ha,
        process,
        logging,
        api,
        debug,
    })
}

fn normalize_postgres_config_v2(
    input: super::schema::PostgresConfigV2Input,
) -> Result<PostgresConfig, ConfigError> {
    let connect_timeout_s = input
        .connect_timeout_s
        .unwrap_or_else(default_postgres_connect_timeout_s);

    let local_conn_identity = normalize_postgres_conn_identity_v2(
        "postgres.local_conn_identity",
        input.local_conn_identity,
    )?;
    let rewind_conn_identity = normalize_postgres_conn_identity_v2(
        "postgres.rewind_conn_identity",
        input.rewind_conn_identity,
    )?;

    let tls = normalize_tls_server_config_v2("postgres.tls", input.tls)?;
    let roles = normalize_postgres_roles_v2(input.roles)?;
    let pg_hba = normalize_pg_hba_v2(input.pg_hba)?;
    let pg_ident = normalize_pg_ident_v2(input.pg_ident)?;

    Ok(PostgresConfig {
        data_dir: input.data_dir,
        connect_timeout_s,
        listen_host: input.listen_host,
        listen_port: input.listen_port,
        socket_dir: input.socket_dir,
        log_file: input.log_file,
        rewind_source_host: input.rewind_source_host,
        rewind_source_port: input.rewind_source_port,
        local_conn_identity,
        rewind_conn_identity,
        tls,
        roles,
        pg_hba,
        pg_ident,
    })
}

fn normalize_postgres_conn_identity_v2(
    field_prefix: &'static str,
    input: Option<super::schema::PostgresConnIdentityConfigV2Input>,
) -> Result<PostgresConnIdentityConfig, ConfigError> {
    let identity = input.ok_or_else(|| ConfigError::Validation {
        field: field_prefix,
        message: "missing required secure config block for config_version=v2".to_string(),
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

    let user = identity.user.ok_or_else(|| ConfigError::Validation {
        field: user_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    validate_non_empty(user_field, user.as_str())?;

    let dbname = identity.dbname.ok_or_else(|| ConfigError::Validation {
        field: dbname_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    validate_non_empty(dbname_field, dbname.as_str())?;

    let ssl_mode = identity.ssl_mode.ok_or_else(|| ConfigError::Validation {
        field: ssl_mode_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;

    Ok(PostgresConnIdentityConfig {
        user,
        dbname,
        ssl_mode,
    })
}

fn normalize_postgres_roles_v2(
    input: Option<super::schema::PostgresRolesConfigV2Input>,
) -> Result<PostgresRolesConfig, ConfigError> {
    let roles = input.ok_or_else(|| ConfigError::Validation {
        field: "postgres.roles",
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;

    let superuser = normalize_postgres_role_v2("postgres.roles.superuser", roles.superuser)?;
    let replicator = normalize_postgres_role_v2("postgres.roles.replicator", roles.replicator)?;
    let rewinder = normalize_postgres_role_v2("postgres.roles.rewinder", roles.rewinder)?;

    Ok(PostgresRolesConfig {
        superuser,
        replicator,
        rewinder,
    })
}

fn normalize_postgres_role_v2(
    field_prefix: &'static str,
    input: Option<super::schema::PostgresRoleConfigV2Input>,
) -> Result<PostgresRoleConfig, ConfigError> {
    let role = input.ok_or_else(|| ConfigError::Validation {
        field: field_prefix,
        message: "missing required secure config block for config_version=v2".to_string(),
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
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    validate_non_empty(username_field, username.as_str())?;

    let auth = role.auth.ok_or_else(|| ConfigError::Validation {
        field: auth_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;

    Ok(PostgresRoleConfig { username, auth })
}

fn normalize_pg_hba_v2(
    input: Option<super::schema::PgHbaConfigV2Input>,
) -> Result<PgHbaConfig, ConfigError> {
    let cfg = input.ok_or_else(|| ConfigError::Validation {
        field: "postgres.pg_hba",
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;
    let source = cfg.source.ok_or_else(|| ConfigError::Validation {
        field: "postgres.pg_hba.source",
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    Ok(PgHbaConfig { source })
}

fn normalize_pg_ident_v2(
    input: Option<super::schema::PgIdentConfigV2Input>,
) -> Result<PgIdentConfig, ConfigError> {
    let cfg = input.ok_or_else(|| ConfigError::Validation {
        field: "postgres.pg_ident",
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;
    let source = cfg.source.ok_or_else(|| ConfigError::Validation {
        field: "postgres.pg_ident.source",
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    Ok(PgIdentConfig { source })
}

fn normalize_api_config_v2(input: super::schema::ApiConfigV2Input) -> Result<ApiConfig, ConfigError> {
    let listen_addr = input.listen_addr.unwrap_or_else(default_api_listen_addr);

    let security = input.security.ok_or_else(|| ConfigError::Validation {
        field: "api.security",
        message: "missing required secure config block for config_version=v2".to_string(),
    })?;

    let tls = normalize_tls_server_config_v2("api.security.tls", security.tls)?;
    let auth = security.auth.ok_or_else(|| ConfigError::Validation {
        field: "api.security.auth",
        message: "missing required secure field for config_version=v2".to_string(),
    })?;

    Ok(ApiConfig {
        listen_addr,
        security: ApiSecurityConfig { tls, auth },
    })
}

fn normalize_tls_server_config_v2(
    field_prefix: &'static str,
    input: Option<super::schema::TlsServerConfigV2Input>,
) -> Result<TlsServerConfig, ConfigError> {
    let tls = input.ok_or_else(|| ConfigError::Validation {
        field: field_prefix,
        message: "missing required secure config block for config_version=v2".to_string(),
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
        message: "missing required secure field for config_version=v2".to_string(),
    })?;

    let identity = match tls.identity {
        None => None,
        Some(identity) => Some(normalize_tls_server_identity_v2(identity_field, identity)?),
    };

    Ok(TlsServerConfig {
        mode,
        identity,
        client_auth: tls.client_auth,
    })
}

fn normalize_tls_server_identity_v2(
    field_prefix: &'static str,
    input: super::schema::TlsServerIdentityConfigV2Input,
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
        message: "missing required secure field for config_version=v2".to_string(),
    })?;
    let private_key = input.private_key.ok_or_else(|| ConfigError::Validation {
        field: private_key_field,
        message: "missing required secure field for config_version=v2".to_string(),
    })?;

    Ok(TlsServerIdentityConfig {
        cert_chain,
        private_key,
    })
}

pub fn validate_runtime_config(cfg: &RuntimeConfig) -> Result<(), ConfigError> {
    validate_non_empty_path("postgres.data_dir", &cfg.postgres.data_dir)?;
    validate_non_empty("postgres.listen_host", cfg.postgres.listen_host.as_str())?;
    validate_port("postgres.listen_port", cfg.postgres.listen_port)?;
    validate_non_empty_path("postgres.socket_dir", &cfg.postgres.socket_dir)?;
    validate_non_empty_path("postgres.log_file", &cfg.postgres.log_file)?;
    validate_non_empty(
        "postgres.rewind_source_host",
        cfg.postgres.rewind_source_host.as_str(),
    )?;
    validate_port(
        "postgres.rewind_source_port",
        cfg.postgres.rewind_source_port,
    )?;

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

    validate_role_auth(
        "postgres.roles.superuser.auth.password.path",
        "postgres.roles.superuser.auth.password.content",
        &cfg.postgres.roles.superuser.auth,
    )?;
    validate_role_auth(
        "postgres.roles.replicator.auth.password.path",
        "postgres.roles.replicator.auth.password.content",
        &cfg.postgres.roles.replicator.auth,
    )?;
    validate_role_auth(
        "postgres.roles.rewinder.auth.password.path",
        "postgres.roles.rewinder.auth.password.content",
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

    validate_inline_or_path_non_empty("postgres.pg_hba.source", &cfg.postgres.pg_hba.source, false)?;
    validate_inline_or_path_non_empty(
        "postgres.pg_ident.source",
        &cfg.postgres.pg_ident.source,
        false,
    )?;

    validate_non_empty_path("process.binaries.postgres", &cfg.process.binaries.postgres)?;
    validate_non_empty_path("process.binaries.pg_ctl", &cfg.process.binaries.pg_ctl)?;
    validate_non_empty_path(
        "process.binaries.pg_rewind",
        &cfg.process.binaries.pg_rewind,
    )?;
    validate_non_empty_path("process.binaries.initdb", &cfg.process.binaries.initdb)?;
    validate_non_empty_path(
        "process.binaries.pg_basebackup",
        &cfg.process.binaries.pg_basebackup,
    )?;
    validate_non_empty_path("process.binaries.psql", &cfg.process.binaries.psql)?;

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
    }
    if let Some(path) = cfg.logging.postgres.log_dir.as_ref() {
        validate_non_empty_path("logging.postgres.log_dir", path)?;
    }
    if let Some(path) = cfg.logging.postgres.archive_command_log_file.as_ref() {
        validate_non_empty_path("logging.postgres.archive_command_log_file", path)?;
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

    if cfg.dcs.endpoints.is_empty() {
        return Err(ConfigError::Validation {
            field: "dcs.endpoints",
            message: "must contain at least one endpoint".to_string(),
        });
    }

    for endpoint in &cfg.dcs.endpoints {
        if endpoint.trim().is_empty() {
            return Err(ConfigError::Validation {
                field: "dcs.endpoints",
                message: "must not contain empty endpoint values".to_string(),
            });
        }
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
            validate_optional_non_empty(
                "api.security.auth.role_tokens.read_token",
                tokens.read_token.as_deref(),
            )?;
            validate_optional_non_empty(
                "api.security.auth.role_tokens.admin_token",
                tokens.admin_token.as_deref(),
            )?;
            if tokens.read_token.is_none() && tokens.admin_token.is_none() {
                return Err(ConfigError::Validation {
                    field: "api.security.auth.role_tokens",
                    message: "at least one of read_token or admin_token must be configured".to_string(),
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

    validate_dcs_init_config(cfg)?;

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

fn validate_optional_non_empty(
    field: &'static str,
    value: Option<&str>,
) -> Result<(), ConfigError> {
    if let Some(raw) = value {
        if raw.trim().is_empty() {
            return Err(ConfigError::Validation {
                field,
                message: "must not be empty when configured".to_string(),
            });
        }
    }
    Ok(())
}

fn validate_role_auth(
    password_path_field: &'static str,
    password_content_field: &'static str,
    auth: &RoleAuthConfig,
) -> Result<(), ConfigError> {
    match auth {
        RoleAuthConfig::Tls => Ok(()),
        RoleAuthConfig::Password { password } => {
            validate_secret_source_non_empty(password_path_field, password_content_field, password)
        }
    }
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

    let identity = cfg.identity.as_ref().ok_or_else(|| ConfigError::Validation {
        field: identity_field,
        message: "tls identity must be configured when tls.mode is optional or required".to_string(),
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

    let _: serde_json::Value =
        serde_json::from_str(init.payload_json.as_str()).map_err(|err| ConfigError::Validation {
            field: "dcs.init.payload_json",
            message: format!("must be valid JSON: {err}"),
        })?;

    let _: RuntimeConfig =
        serde_json::from_str(init.payload_json.as_str()).map_err(|err| ConfigError::Validation {
            field: "dcs.init.payload_json",
            message: format!("must decode as a RuntimeConfig JSON document: {err}"),
        })?;

    Ok(())
}

fn validate_secret_source_non_empty(
    path_field: &'static str,
    content_field: &'static str,
    secret: &SecretSource,
) -> Result<(), ConfigError> {
    validate_inline_or_path_non_empty_for_secret(path_field, content_field, &secret.0)
}

fn validate_inline_or_path_non_empty_for_secret(
    path_field: &'static str,
    content_field: &'static str,
    value: &InlineOrPath,
) -> Result<(), ConfigError> {
    match value {
        InlineOrPath::Path(path) => validate_non_empty_path(path_field, path),
        InlineOrPath::PathConfig { path } => validate_non_empty_path(path_field, path),
        InlineOrPath::Inline { content } => validate_non_empty(content_field, content.as_str()),
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
            PgIdentConfig, PostgresConnIdentityConfig, PostgresConfig, PostgresLoggingConfig,
            PostgresRoleConfig, PostgresRolesConfig, ProcessConfig, RoleAuthConfig, RuntimeConfig,
            StderrSinkConfig, TlsServerConfig,
        };
        use crate::pginfo::conninfo::PgSslMode;

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
                    socket_dir: PathBuf::from("/tmp/pgtuskmaster/socket"),
                    log_file: PathBuf::from("/tmp/pgtuskmaster/postgres.log"),
                    rewind_source_host: "127.0.0.1".to_string(),
                    rewind_source_port: 5432,
                    local_conn_identity: PostgresConnIdentityConfig {
                        user: "postgres".to_string(),
                        dbname: "postgres".to_string(),
                        ssl_mode: PgSslMode::Prefer,
                    },
                    rewind_conn_identity: PostgresConnIdentityConfig {
                        user: "rewinder".to_string(),
                        dbname: "postgres".to_string(),
                        ssl_mode: PgSslMode::Prefer,
                    },
                    tls: TlsServerConfig {
                        mode: ApiTlsMode::Disabled,
                        identity: None,
                        client_auth: None,
                    },
                    roles: PostgresRolesConfig {
                        superuser: PostgresRoleConfig {
                            username: "postgres".to_string(),
                            auth: RoleAuthConfig::Tls,
                        },
                        replicator: PostgresRoleConfig {
                            username: "replicator".to_string(),
                            auth: RoleAuthConfig::Tls,
                        },
                        rewinder: PostgresRoleConfig {
                            username: "rewinder".to_string(),
                            auth: RoleAuthConfig::Tls,
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
                },
                dcs: DcsConfig {
                    endpoints: vec!["http://127.0.0.1:2379".to_string()],
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
                    archive_command_log_file: None,
                    poll_interval_ms: 200,
                    cleanup: LogCleanupConfig {
                        enabled: true,
                        max_files: 10,
                        max_age_seconds: 60,
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
                    listen_addr: "127.0.0.1:8080".to_string(),
                    security: ApiSecurityConfig {
                        tls: TlsServerConfig {
                            mode: ApiTlsMode::Disabled,
                            identity: None,
                            client_auth: None,
                        },
                        auth: ApiAuthConfig::Disabled,
                    },
                },
                debug: DebugConfig { enabled: false },
            }
        }

    #[test]
    fn validate_runtime_config_accepts_valid_config() {
        let cfg = base_runtime_config();
        assert!(validate_runtime_config(&cfg).is_ok());
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
                read_token: Some(" ".to_string()),
                admin_token: None,
            });

            let err = validate_runtime_config(&cfg);
            assert!(matches!(
                err,
                Err(ConfigError::Validation {
                    field: "api.security.auth.role_tokens.read_token",
                    ..
                })
            ));

            let mut cfg = base_runtime_config();
            cfg.api.security.auth = ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
                read_token: None,
                admin_token: Some("\t".to_string()),
            });

            let err = validate_runtime_config(&cfg);
            assert!(matches!(
                err,
                Err(ConfigError::Validation {
                    field: "api.security.auth.role_tokens.admin_token",
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
    fn load_runtime_config_missing_config_version_is_rejected(
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
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "config_version",
                ..
            })
        ));

        let _ = std::fs::remove_file(path);
        Ok(())
    }

    #[test]
    fn load_runtime_config_config_version_v1_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-invalid-{unique}.toml"));

        let toml = r#"
config_version = "v1"
"#;

        std::fs::write(&path, toml)?;

        let err = load_runtime_config(&path);
        assert!(matches!(
            err,
            Err(ConfigError::Validation {
                field: "config_version",
                ..
            })
        ));

        let _ = std::fs::remove_file(path);
        Ok(())
    }

    #[test]
    fn load_runtime_config_rejects_unknown_fields_in_v2() -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-invalid-{unique}.toml"));

        let toml = r#"
config_version = "v2"

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
rewind_source_host = "127.0.0.1"
rewind_source_port = 5432
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "tls" } }, replicator = { username = "replicator", auth = { type = "tls" } }, rewinder = { username = "rewinder", auth = { type = "tls" } } }
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

        let _ = std::fs::remove_file(path);
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_happy_path_with_safe_defaults() -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-v2-{unique}.toml"));

        let toml = r#"
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
rewind_source_host = "127.0.0.1"
rewind_source_port = 5432
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "tls" } }, replicator = { username = "replicator", auth = { type = "tls" } }, rewinder = { username = "rewinder", auth = { type = "tls" } } }
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
        assert_eq!(cfg.api.listen_addr, "127.0.0.1:8080");
        assert!(!cfg.debug.enabled);

        let _ = std::fs::remove_file(path);
        Ok(())
    }

    #[test]
    fn load_runtime_config_v2_missing_secure_fields_is_actionable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("runtime-config-v2-missing-{unique}.toml"));

        // Intentionally omit `postgres.local_conn_identity`.
        let toml = r#"
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
rewind_source_host = "127.0.0.1"
rewind_source_port = 5432
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "tls" } }, replicator = { username = "replicator", auth = { type = "tls" } }, rewinder = { username = "rewinder", auth = { type = "tls" } } }
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

        let _ = std::fs::remove_file(path);
        Ok(())
    }
}
