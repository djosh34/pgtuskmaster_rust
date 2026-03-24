use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{
    config_v2::types::{
        ApiAuth, ApiConfig, ApiTransport, BinariesConfig, ConfigErrorV2, DcsAuth, DcsConfig,
        DcsEndpoint, LoggingConfig, OperatorConfigV2, PgtmApiTransportExpectation, PostgresConfig,
        RoleConfig, RuntimeConfigV2, Secret, TimingConfig, TlsConfig,
    },
    pginfo::conninfo::{PgClientTls, PgSslMode},
    state::{ApiRoute, ClusterName, MemberId, PgRoute, ScopeName},
};
use reqwest::Url;

use super::private_schema as raw;

pub fn load_runtime_config(path: &Path) -> Result<RuntimeConfigV2, ConfigErrorV2> {
    let contents = read_config_file(path)?;
    load_runtime_config_contents_at(contents.as_str(), path)
}

pub fn load_operator_config(path: &Path) -> Result<OperatorConfigV2, ConfigErrorV2> {
    let contents = read_config_file(path)?;
    load_operator_config_contents_at(contents.as_str(), path)
}

#[cfg(any(test, feature = "internal-test-support"))]
pub fn load_runtime_config_contents(contents: &str) -> Result<RuntimeConfigV2, ConfigErrorV2> {
    load_runtime_config_contents_at(contents, Path::new("<runtime-config>"))
}

#[cfg(any(test, feature = "internal-test-support"))]
pub fn load_operator_config_contents(contents: &str) -> Result<OperatorConfigV2, ConfigErrorV2> {
    load_operator_config_contents_at(contents, Path::new("<operator-config>"))
}

fn load_runtime_config_contents_at(
    contents: &str,
    path: &Path,
) -> Result<RuntimeConfigV2, ConfigErrorV2> {
    let document = toml::from_str::<raw::RuntimeDocument>(contents)
        .map(raw::RuntimeDocument::normalize)
        .map_err(|source| parse_error(path, source))?;
    map_runtime_document(document, path)
}

fn load_operator_config_contents_at(
    contents: &str,
    path: &Path,
) -> Result<OperatorConfigV2, ConfigErrorV2> {
    let document: toml::Value =
        toml::from_str(contents).map_err(|source| parse_error(path, source))?;
    if looks_like_runtime_operator_source(&document) {
        let runtime_document = document
            .try_into::<raw::RuntimeDocument>()
            .map(raw::RuntimeDocument::normalize)
            .map_err(|source| parse_error(path, source))?;
        return runtime_document
            .pgtm
            .ok_or_else(|| {
                validation_error("pgtm", "missing operator config block in runtime document")
            })
            .and_then(|pgtm| parse_operator_config_value_at(pgtm, path, true));
    }
    parse_operator_config_value_at(document, path, true)
}

fn map_runtime_document(
    document: raw::RuntimeDocument,
    path: &Path,
) -> Result<RuntimeConfigV2, ConfigErrorV2> {
    validate_non_empty("cluster.name", document.cluster.name.as_str())?;
    validate_non_empty("cluster.scope", document.cluster.scope.as_str())?;
    validate_non_empty("cluster.member_id", document.cluster.member_id.as_str())?;

    if document.dcs.endpoints.is_empty() {
        return Err(validation_error(
            "dcs.endpoints",
            "at least one endpoint is required",
        ));
    }

    if !document.postgres.roles.extra.is_empty() {
        return Err(validation_error(
            "postgres.roles.extra",
            "managed extra roles are not supported by config_v2",
        ));
    }
    if document.dcs.init.is_some() {
        return Err(validation_error(
            "dcs.init",
            "is not supported by config_v2",
        ));
    }
    let _debug_enabled = document.debug.enabled;
    let _rewind_database = document.postgres.rewind.database.clone();

    let working_root = if document.process.working_root.as_os_str().is_empty() {
        PathBuf::from("/tmp/pgtuskmaster")
    } else {
        document.process.working_root.clone()
    };
    let data_dir = document.postgres.paths.data_dir.clone();
    let socket_dir = document
        .postgres
        .paths
        .socket_dir
        .clone()
        .unwrap_or_else(|| working_root.join("socket"));
    let log_file = document
        .postgres
        .paths
        .log_file
        .clone()
        .unwrap_or_else(|| working_root.join("logs/postgres.log"));

    let listen_host = document.postgres.network.listen_host.clone();
    let listen_port = document.postgres.network.listen_port;
    let cluster_advertise = map_postgres_advertise(
        "postgres.network.cluster_advertise",
        document
            .postgres
            .network
            .cluster_advertise
            .unwrap_or(raw::PostgresAdvertiseConfig {
                host: listen_host.clone(),
                port: listen_port,
                hostaddr: None,
            }),
    )?;
    let operator_advertise = document
        .postgres
        .network
        .operator_advertise
        .map(|advertise| map_postgres_advertise("postgres.network.operator_advertise", advertise))
        .transpose()?;
    let postgres = PostgresConfig {
        data_dir: data_dir.clone(),
        socket_dir,
        log_file,
        listen_host,
        listen_port,
        cluster_advertise,
        operator_advertise,
        connect_timeout: Duration::from_secs(u64::from(document.postgres.connect_timeout_s)),
        local_database: non_empty_owned(
            "postgres.local_database",
            document.postgres.local_database,
        )?,
        source_client_tls: map_postgres_client_tls(document.postgres.rewind.transport)?,
        superuser: map_postgres_role(
            "postgres.roles.mandatory.superuser",
            document.postgres.roles.mandatory.superuser,
        )?,
        replicator: map_postgres_role(
            "postgres.roles.mandatory.replicator",
            document.postgres.roles.mandatory.replicator,
        )?,
        rewinder: map_postgres_role(
            "postgres.roles.mandatory.rewinder",
            document.postgres.roles.mandatory.rewinder,
        )?,
        pg_hba_file: data_dir.join("pgtm.pg_hba.conf"),
        pg_ident_file: data_dir.join("pgtm.pg_ident.conf"),
        pg_hba_contents: resolve_inline_or_path_string(
            "postgres.access.hba",
            document.postgres.access.hba,
        )?,
        pg_ident_contents: resolve_inline_or_path_string(
            "postgres.access.ident",
            document.postgres.access.ident,
        )?,
        extra_gucs: document.postgres.extra_gucs,
        tls: map_postgres_tls(document.postgres.tls)?,
    };

    let dcs_tls = map_dcs_tls(document.dcs.client.tls)?;
    if document
        .dcs
        .endpoints
        .iter()
        .any(|endpoint| endpoint.trim_start().starts_with("https://"))
        && dcs_tls.is_none()
    {
        return Err(validation_error(
            "dcs.client.tls",
            "https DCS endpoints require `dcs.client.tls` to be configured",
        ));
    }

    let dcs = DcsConfig {
        endpoints: document
            .dcs
            .endpoints
            .into_iter()
            .map(|endpoint| DcsEndpoint::new(endpoint.trim().to_string()))
            .collect(),
        auth: map_dcs_auth(document.dcs.client.auth)?,
        tls: dcs_tls,
    };

    let binaries = BinariesConfig {
        pg_ctl: resolve_binary_path(
            "process.binaries.overrides.pg_ctl",
            "pg_ctl",
            document.process.binaries.overrides.pg_ctl,
        )?,
        initdb: resolve_binary_path(
            "process.binaries.overrides.initdb",
            "initdb",
            document.process.binaries.overrides.initdb,
        )?,
        pg_rewind: resolve_binary_path(
            "process.binaries.overrides.pg_rewind",
            "pg_rewind",
            document.process.binaries.overrides.pg_rewind,
        )?,
        pg_basebackup: resolve_binary_path(
            "process.binaries.overrides.pg_basebackup",
            "pg_basebackup",
            document.process.binaries.overrides.pg_basebackup,
        )?,
    };

    let postgres_log_dir = document
        .logging
        .postgres
        .log_dir
        .clone()
        .unwrap_or_else(|| working_root.join("logs/postgres"));

    let logging = LoggingConfig {
        level: document.logging.level,
        capture_subprocess_output: document.logging.capture_subprocess_output,
        stderr_enabled: document.logging.sinks.stderr.enabled,
        file_enabled: document.logging.sinks.file.enabled,
        file_path: document
            .logging
            .sinks
            .file
            .path
            .clone()
            .unwrap_or_else(|| working_root.join("runtime.jsonl")),
        file_mode: document.logging.sinks.file.mode,
        postgres_logs_enabled: document.logging.postgres.enabled,
        postgres_log_dir: postgres_log_dir.clone(),
        postgres_pg_ctl_log: document
            .logging
            .postgres
            .pg_ctl_log_file
            .clone()
            .unwrap_or_else(|| postgres_log_dir.join("pg_ctl.log")),
        postgres_log_poll_interval: Duration::from_millis(
            document.logging.postgres.poll_interval_ms,
        ),
        postgres_log_cleanup_enabled: document.logging.postgres.cleanup.enabled,
        postgres_log_cleanup_max_files: document.logging.postgres.cleanup.max_files,
        postgres_log_cleanup_max_age: Duration::from_secs(
            document.logging.postgres.cleanup.max_age_seconds,
        ),
        postgres_log_cleanup_protect_recent: Duration::from_secs(
            document.logging.postgres.cleanup.protect_recent_seconds,
        ),
    };
    let operator_advertise = document
        .pgtm
        .map(|pgtm| parse_operator_config_value_at(pgtm, path, false))
        .transpose()?
        .and_then(|config| config.advertised_url);

    Ok(RuntimeConfigV2 {
        cluster_name: ClusterName(document.cluster.name),
        scope: ScopeName(document.cluster.scope),
        member_id: MemberId(document.cluster.member_id),
        postgres,
        dcs,
        timing: TimingConfig {
            ha_loop_interval: Duration::from_millis(document.ha.loop_interval_ms),
            ha_lease_ttl: Duration::from_millis(document.ha.lease_ttl_ms),
            bootstrap_timeout: Duration::from_millis(document.process.timeouts.bootstrap_ms),
            pg_rewind_timeout: Duration::from_millis(document.process.timeouts.pg_rewind_ms),
            fencing_timeout: Duration::from_millis(document.process.timeouts.fencing_ms),
        },
        binaries,
        logging,
        api: ApiConfig {
            listen_addr: document.api.listen_addr,
            transport: map_api_transport(document.api.transport)?,
            auth: map_runtime_api_auth(document.api.auth)?,
            advertise: operator_advertise,
        },
    })
}

#[cfg(any(test, feature = "internal-test-support"))]
pub fn validate_runtime_document_contents(contents: &str) -> Result<(), ConfigErrorV2> {
    let _ = toml::from_str::<raw::RuntimeDocument>(contents)
        .map(raw::RuntimeDocument::normalize)
        .map_err(|source| parse_error(Path::new("<runtime-config>"), source))?;
    Ok(())
}

#[cfg(any(test, feature = "internal-test-support"))]
pub fn runtime_test_config() -> Result<RuntimeConfigV2, ConfigErrorV2> {
    load_runtime_test_config_from_paths(PathBuf::from("/tmp/pgdata"), "scope-a")
}

#[cfg(any(test, feature = "internal-test-support"))]
pub fn runtime_test_config_with_data_dir(
    data_dir: impl Into<PathBuf>,
) -> Result<RuntimeConfigV2, ConfigErrorV2> {
    load_runtime_test_config_from_paths(data_dir.into(), "scope-a")
}

#[cfg(any(test, feature = "internal-test-support"))]
pub fn managed_postgres_test_config(
    data_dir: impl Into<PathBuf>,
) -> Result<RuntimeConfigV2, ConfigErrorV2> {
    let config = load_runtime_test_config_from_paths(data_dir.into(), "cluster-a")?;
    Ok(RuntimeConfigV2 {
        timing: TimingConfig {
            ha_loop_interval: Duration::from_millis(500),
            ha_lease_ttl: Duration::from_secs(5),
            bootstrap_timeout: Duration::from_secs(30),
            pg_rewind_timeout: Duration::from_secs(30),
            fencing_timeout: Duration::from_secs(10),
        },
        ..config
    })
}

#[cfg(any(test, feature = "internal-test-support"))]
pub fn trace_logging_test_config() -> Result<RuntimeConfigV2, ConfigErrorV2> {
    let config = runtime_test_config()?;
    Ok(RuntimeConfigV2 {
        logging: LoggingConfig {
            level: crate::config_v2::types::LogLevel::Trace,
            postgres_log_poll_interval: Duration::from_millis(50),
            postgres_log_cleanup_enabled: false,
            ..config.logging
        },
        ..config
    })
}

#[cfg(any(test, feature = "internal-test-support"))]
pub fn load_runtime_timing_values(
    path: &Path,
) -> Result<(Duration, Duration, Duration, Duration), ConfigErrorV2> {
    let contents = read_config_file(path)?;
    let document = toml::from_str::<raw::RuntimeDocument>(&contents)
        .map(raw::RuntimeDocument::normalize)
        .map_err(|source| parse_error(path, source))?;
    Ok((
        Duration::from_millis(document.ha.loop_interval_ms),
        Duration::from_millis(document.ha.lease_ttl_ms),
        Duration::from_millis(document.process.timeouts.bootstrap_ms),
        Duration::from_millis(document.process.timeouts.pg_rewind_ms),
    ))
}

#[cfg(any(test, feature = "internal-test-support"))]
fn load_runtime_test_config_from_paths(
    data_dir: PathBuf,
    scope: &str,
) -> Result<RuntimeConfigV2, ConfigErrorV2> {
    let socket_dir = Path::new("/tmp/pgtuskmaster/socket");
    let log_file = Path::new("/tmp/pgtuskmaster/postgres.log");
    let contents = raw::render_runtime_test_config_toml(
        "cluster-a",
        scope,
        "node-a",
        (data_dir.as_path(), socket_dir, log_file),
        ["http://127.0.0.1:2379"],
        std::iter::empty::<&str>(),
    )
    .map_err(|message| validation_error("runtime_test_config", message))?;
    let mut config = load_runtime_config_contents(contents.as_str())?;
    let password = Secret::new("secret-password".to_string());
    config.postgres.superuser.password = password.clone();
    config.postgres.replicator.password = password.clone();
    config.postgres.rewinder.password = password;
    Ok(config)
}

pub(super) fn read_config_file(path: &Path) -> Result<String, ConfigErrorV2> {
    std::fs::read_to_string(path).map_err(|source| ConfigErrorV2::Io {
        path: path.display().to_string(),
        source,
    })
}

pub(super) fn parse_error(path: &Path, source: toml::de::Error) -> ConfigErrorV2 {
    ConfigErrorV2::Parse {
        path: path.display().to_string(),
        source,
    }
}

pub(super) fn validation_error(field: &'static str, message: impl Into<String>) -> ConfigErrorV2 {
    ConfigErrorV2::Validation {
        field,
        message: message.into(),
    }
}

pub(super) fn validate_non_empty(field: &'static str, value: &str) -> Result<(), ConfigErrorV2> {
    if value.trim().is_empty() {
        return Err(validation_error(field, "must not be empty"));
    }
    Ok(())
}

fn non_empty_owned(field: &'static str, value: String) -> Result<String, ConfigErrorV2> {
    validate_non_empty(field, value.as_str())?;
    Ok(value)
}

pub(super) fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub(super) fn resolve_secret_optional(
    field: &'static str,
    source: Option<raw::SecretSource>,
) -> Result<Option<Secret>, ConfigErrorV2> {
    source
        .map(|source| resolve_secret_required(field, source))
        .transpose()
}

pub(super) fn resolve_secret_required(
    field: &'static str,
    source: raw::SecretSource,
) -> Result<Secret, ConfigErrorV2> {
    let value = match source {
        raw::SecretSource::PathConfig { path } => {
            std::fs::read_to_string(&path).map_err(|source| ConfigErrorV2::Io {
                path: path.display().to_string(),
                source,
            })?
        }
        raw::SecretSource::Tagged(raw::TaggedSecretSource::None) => String::new(),
        raw::SecretSource::Tagged(raw::TaggedSecretSource::Env { env }) => std::env::var(&env)
            .map_err(|err| {
                validation_error(
                    field,
                    match err {
                        std::env::VarError::NotPresent => {
                            format!("environment variable `{env}` is not set")
                        }
                        std::env::VarError::NotUnicode(_) => {
                            format!("environment variable `{env}` is not valid unicode")
                        }
                    },
                )
            })?,
        raw::SecretSource::Tagged(raw::TaggedSecretSource::File { path }) => {
            std::fs::read_to_string(&path).map_err(|source| ConfigErrorV2::Io {
                path: path.display().to_string(),
                source,
            })?
        }
        raw::SecretSource::Tagged(raw::TaggedSecretSource::String { value }) => value,
    };

    let value = value.trim_end_matches(['\n', '\r']).to_string();
    if value.trim().is_empty() {
        return Err(validation_error(field, "must not be empty"));
    }
    Ok(Secret::new(value))
}

pub(super) fn resolve_path_only(
    field: &'static str,
    source: raw::PathSource,
) -> Result<PathBuf, ConfigErrorV2> {
    match source {
        raw::PathSource::Path(path) | raw::PathSource::PathConfig { path } => {
            if path.as_os_str().is_empty() {
                return Err(validation_error(field, "must not be empty"));
            }
            Ok(path)
        }
    }
}

fn resolve_inline_or_path_string(
    _field: &'static str,
    source: raw::PathOrInline,
) -> Result<String, ConfigErrorV2> {
    match source {
        raw::PathOrInline::Path(path) | raw::PathOrInline::PathConfig { path } => {
            std::fs::read_to_string(&path).map_err(|source| ConfigErrorV2::Io {
                path: path.display().to_string(),
                source,
            })
        }
        raw::PathOrInline::Inline { content } => Ok(content),
    }
}

fn map_postgres_role(
    field_prefix: &'static str,
    role: raw::PostgresRoleConfig,
) -> Result<RoleConfig, ConfigErrorV2> {
    validate_non_empty(role_username_field(field_prefix), role.username.as_str())?;
    let raw::RoleAuthConfig::Password { password } = role.auth;
    Ok(RoleConfig {
        username: role.username,
        password: resolve_secret_required(role_password_field(field_prefix), password)?,
    })
}

fn map_postgres_client_tls(
    transport: raw::PostgresClientTransportConfig,
) -> Result<PgClientTls, ConfigErrorV2> {
    Ok(PgClientTls {
        mode: transport.ssl_mode,
        root_cert: transport
            .ca_cert
            .map(|ca_cert| resolve_path_only("postgres.rewind.transport.ca_cert", ca_cert))
            .transpose()?,
        client_cert: None,
        client_key: None,
    })
}

fn role_username_field(field_prefix: &'static str) -> &'static str {
    match field_prefix {
        "postgres.roles.mandatory.superuser" => "postgres.roles.mandatory.superuser.username",
        "postgres.roles.mandatory.replicator" => "postgres.roles.mandatory.replicator.username",
        "postgres.roles.mandatory.rewinder" => "postgres.roles.mandatory.rewinder.username",
        _ => field_prefix,
    }
}

fn role_password_field(field_prefix: &'static str) -> &'static str {
    match field_prefix {
        "postgres.roles.mandatory.superuser" => "postgres.roles.mandatory.superuser.auth.password",
        "postgres.roles.mandatory.replicator" => {
            "postgres.roles.mandatory.replicator.auth.password"
        }
        "postgres.roles.mandatory.rewinder" => "postgres.roles.mandatory.rewinder.auth.password",
        _ => field_prefix,
    }
}

fn map_postgres_tls(tls: raw::TlsServerConfig) -> Result<Option<TlsConfig>, ConfigErrorV2> {
    match tls {
        raw::TlsServerConfig::Disabled => Ok(None),
        raw::TlsServerConfig::Enabled {
            identity,
            client_auth,
        } => Ok(Some(TlsConfig {
            cert: resolve_path_only("postgres.tls.identity.cert_chain", identity.cert_chain)?,
            key: resolve_path_only("postgres.tls.identity.private_key", identity.private_key)?,
            ca_cert: client_auth
                .map(|client_auth| {
                    let _client_certificate_mode = client_auth.client_certificate;
                    resolve_path_only("postgres.tls.client_auth.client_ca", client_auth.client_ca)
                })
                .transpose()?,
        })),
    }
}

fn map_dcs_auth(auth: raw::DcsAuthConfig) -> Result<Option<DcsAuth>, ConfigErrorV2> {
    match auth {
        raw::DcsAuthConfig::Disabled => Ok(None),
        raw::DcsAuthConfig::Basic { username, password } => {
            validate_non_empty("dcs.client.auth.username", username.as_str())?;
            Ok(Some(DcsAuth {
                username,
                password: resolve_secret_required("dcs.client.auth.password", password)?,
            }))
        }
    }
}

fn map_dcs_tls(tls: raw::DcsTlsConfig) -> Result<Option<TlsConfig>, ConfigErrorV2> {
    match tls {
        raw::DcsTlsConfig::Disabled => Ok(None),
        raw::DcsTlsConfig::Enabled {
            ca_cert,
            identity,
            server_name,
        } => {
            if server_name.is_some() {
                return Err(validation_error(
                    "dcs.client.tls.server_name",
                    "is not supported by config_v2",
                ));
            }
            let Some(identity) = identity else {
                return Err(validation_error(
                    "dcs.client.tls.identity",
                    "enabled DCS TLS currently requires a client identity",
                ));
            };
            Ok(Some(TlsConfig {
                cert: resolve_path_only("dcs.client.tls.identity.cert", identity.cert)?,
                key: resolve_path_only("dcs.client.tls.identity.key", identity.key)?,
                ca_cert: ca_cert
                    .map(|ca_cert| resolve_path_only("dcs.client.tls.ca_cert", ca_cert))
                    .transpose()?,
            }))
        }
    }
}

fn map_postgres_advertise(
    field: &'static str,
    advertise: raw::PostgresAdvertiseConfig,
) -> Result<PgRoute, ConfigErrorV2> {
    let host = non_empty_owned(field, advertise.host)?;
    if advertise.port == 0 {
        return Err(validation_error(field, "port must not be zero"));
    }
    PgRoute::tcp_hostaddr(host, advertise.port, advertise.hostaddr)
        .map_err(|message| validation_error(field, message))
}

fn looks_like_runtime_operator_source(document: &toml::Value) -> bool {
    document.as_table().is_some_and(|table| {
        [
            "cluster", "dcs", "ha", "process", "logging", "debug", "pgtm",
        ]
        .into_iter()
        .any(|field| table.contains_key(field))
    })
}

fn parse_operator_config_value_at(
    value: toml::Value,
    path: &Path,
    resolve_auth_tokens: bool,
) -> Result<OperatorConfigV2, ConfigErrorV2> {
    let document: raw::OperatorDocument = value
        .try_into()
        .map_err(|source| parse_error(path, source))?;
    map_operator_document(document, resolve_auth_tokens)
}

fn map_operator_document(
    document: raw::OperatorDocument,
    resolve_auth_tokens: bool,
) -> Result<OperatorConfigV2, ConfigErrorV2> {
    let raw::OperatorDocument { api, postgres } = document;
    let raw::OperatorApiConfig {
        base_url,
        advertised_url,
        expected_transport,
        resolve_to,
        auth,
        tls: api_tls,
    } = api;
    let (read_token_source, admin_token_source) = take_token_sources(auth.clone());
    let (read_token, admin_token) = match (resolve_auth_tokens, token_auth_mode(&auth)) {
        (_, TokenAuthMode::Disabled) | (false, TokenAuthMode::RoleTokens) => (None, None),
        (true, TokenAuthMode::RoleTokens) => (
            resolve_secret_optional("pgtm.api.auth.read_token", read_token_source)?,
            resolve_secret_optional("pgtm.api.auth.admin_token", admin_token_source)?,
        ),
    };
    let api_tls = resolve_operator_client_tls(
        api_tls,
        "pgtm.api.tls.ca_cert",
        "pgtm.api.tls.identity.cert",
        "pgtm.api.tls.identity.key",
    )?;
    let postgres_tls = resolve_operator_client_tls(
        postgres.tls,
        "pgtm.postgres.tls.ca_cert",
        "pgtm.postgres.tls.identity.cert",
        "pgtm.postgres.tls.identity.key",
    )?;

    Ok(OperatorConfigV2 {
        base_url: parse_operator_url("pgtm.api.base_url", base_url, expected_transport)?,
        advertised_url: parse_operator_url(
            "pgtm.api.advertised_url",
            advertised_url,
            expected_transport,
        )?
        .map(|url| {
            ApiRoute::from_url(url).map_err(|err| validation_error("pgtm.api.advertised_url", err))
        })
        .transpose()?,
        expected_transport,
        resolve_to,
        client_tls: merge_operator_client_tls(api_tls, postgres_tls)?,
        read_token,
        admin_token,
    })
}

fn resolve_operator_client_tls(
    tls: raw::OperatorClientTlsInput,
    ca_field: &'static str,
    cert_field: &'static str,
    key_field: &'static str,
) -> Result<Option<PgClientTls>, ConfigErrorV2> {
    let root_cert = tls
        .ca_cert
        .map(|ca_cert| resolve_path_only(ca_field, ca_cert))
        .transpose()?;
    let identity = tls
        .identity
        .map(|identity| {
            Ok((
                resolve_path_only(cert_field, identity.cert)?,
                resolve_path_only(key_field, identity.key)?,
            ))
        })
        .transpose()?;
    let (client_cert, client_key) = match identity {
        Some((cert, key)) => (Some(cert), Some(key)),
        None => (None, None),
    };

    Ok(
        (root_cert.is_some() || client_cert.is_some() || client_key.is_some()).then_some(
            PgClientTls {
                mode: PgSslMode::VerifyFull,
                root_cert,
                client_cert,
                client_key,
            },
        ),
    )
}

fn parse_operator_url(
    field: &'static str,
    value: Option<String>,
    expected_transport: Option<PgtmApiTransportExpectation>,
) -> Result<Option<Url>, ConfigErrorV2> {
    normalize_optional_string(value)
        .map(|value| {
            let url = Url::parse(value.as_str())
                .map_err(|err| validation_error(field, format!("must be a valid URL: {err}")))?;
            validate_expected_transport(field, &url, expected_transport)?;
            Ok(url)
        })
        .transpose()
}

fn validate_expected_transport(
    field: &'static str,
    url: &Url,
    expected_transport: Option<PgtmApiTransportExpectation>,
) -> Result<(), ConfigErrorV2> {
    let Some(expected_transport) = expected_transport else {
        return Ok(());
    };

    if expected_transport.matches_url(url) {
        return Ok(());
    }

    Err(validation_error(
        field,
        format!(
            "operator config expects `{}` API transport, but resolved base URL uses `{}`",
            expected_transport.scheme(),
            url.scheme()
        ),
    ))
}

fn merge_optional_path(
    left_field: &'static str,
    left: Option<PathBuf>,
    right_field: &'static str,
    right: Option<PathBuf>,
    merged_field: &'static str,
) -> Result<Option<PathBuf>, ConfigErrorV2> {
    match (left, right) {
        (Some(left), Some(right)) if left != right => Err(validation_error(
            merged_field,
            format!("`{left_field}` and `{right_field}` must match when both are configured"),
        )),
        (Some(path), Some(_)) | (Some(path), None) | (None, Some(path)) => Ok(Some(path)),
        (None, None) => Ok(None),
    }
}

fn merge_operator_client_tls(
    left: Option<PgClientTls>,
    right: Option<PgClientTls>,
) -> Result<Option<PgClientTls>, ConfigErrorV2> {
    match (left, right) {
        (Some(left), Some(right)) => {
            let left_identity = left.client_cert.as_ref().zip(left.client_key.as_ref());
            let right_identity = right.client_cert.as_ref().zip(right.client_key.as_ref());
            if let (Some((left_cert, left_key)), Some((right_cert, right_key))) =
                (left_identity, right_identity)
            {
                if left_cert != right_cert || left_key != right_key {
                    return Err(validation_error(
                        "pgtm.client_tls.identity",
                        "`pgtm.api.tls.identity` and `pgtm.postgres.tls.identity` must match when both are configured",
                    ));
                }
            }

            Ok(Some(PgClientTls {
                mode: PgSslMode::VerifyFull,
                root_cert: merge_optional_path(
                    "pgtm.api.tls.ca_cert",
                    left.root_cert,
                    "pgtm.postgres.tls.ca_cert",
                    right.root_cert,
                    "pgtm.client_tls.ca_cert",
                )?,
                client_cert: left.client_cert.or(right.client_cert),
                client_key: left.client_key.or(right.client_key),
            }))
        }
        (Some(tls), None) | (None, Some(tls)) => Ok(Some(tls)),
        (None, None) => Ok(None),
    }
}

fn map_api_transport(transport: raw::ApiTransportConfig) -> Result<ApiTransport, ConfigErrorV2> {
    match transport {
        raw::ApiTransportConfig::Http => Ok(ApiTransport::Http),
        raw::ApiTransportConfig::Https { tls } => {
            let (client_ca, client_cert_required, allowed_client_common_names) =
                match tls.client_auth {
                    raw::ApiClientAuthConfig::Disabled => (None, false, Vec::new()),
                    raw::ApiClientAuthConfig::Optional { client_ca } => (
                        Some(resolve_path_only(
                            "api.transport.tls.client_auth.client_ca",
                            client_ca,
                        )?),
                        false,
                        Vec::new(),
                    ),
                    raw::ApiClientAuthConfig::Required {
                        client_ca,
                        allowed_common_names,
                    } => (
                        Some(resolve_path_only(
                            "api.transport.tls.client_auth.client_ca",
                            client_ca,
                        )?),
                        true,
                        allowed_common_names,
                    ),
                };

            Ok(ApiTransport::Https {
                tls: TlsConfig {
                    cert: resolve_path_only(
                        "api.transport.tls.identity.cert_chain",
                        tls.identity.cert_chain,
                    )?,
                    key: resolve_path_only(
                        "api.transport.tls.identity.private_key",
                        tls.identity.private_key,
                    )?,
                    ca_cert: None,
                },
                client_ca,
                client_cert_required,
                allowed_client_common_names,
            })
        }
    }
}

fn map_runtime_api_auth(auth: raw::TokenAuthConfig) -> Result<ApiAuth, ConfigErrorV2> {
    let mode = token_auth_mode(&auth);
    let (read_token, admin_token) = take_token_sources(auth);
    match mode {
        TokenAuthMode::Disabled => Ok(ApiAuth::Disabled),
        TokenAuthMode::RoleTokens => Ok(ApiAuth::Tokens {
            read_token: resolve_secret_required(
                "api.auth.read_token",
                read_token.ok_or_else(|| {
                    validation_error("api.auth.read_token", "is required when auth is enabled")
                })?,
            )?,
            admin_token: resolve_secret_required(
                "api.auth.admin_token",
                admin_token.ok_or_else(|| {
                    validation_error("api.auth.admin_token", "is required when auth is enabled")
                })?,
            )?,
        }),
    }
}

pub(super) fn take_token_sources(
    auth: raw::TokenAuthConfig,
) -> (Option<raw::SecretSource>, Option<raw::SecretSource>) {
    let mut read_token = auth.read_token;
    let mut admin_token = auth.admin_token;
    if let Some(tokens) = auth.tokens {
        if read_token.is_none() {
            read_token = tokens.read_token;
        }
        if admin_token.is_none() {
            admin_token = tokens.admin_token;
        }
    }
    (read_token, admin_token)
}

pub(super) fn token_auth_mode(auth: &raw::TokenAuthConfig) -> TokenAuthMode {
    match auth.kind.as_deref() {
        Some("disabled") | None
            if auth.read_token.is_none() && auth.admin_token.is_none() && auth.tokens.is_none() =>
        {
            TokenAuthMode::Disabled
        }
        Some("disabled") => TokenAuthMode::Disabled,
        Some("role_tokens") | None => TokenAuthMode::RoleTokens,
        Some(_) => TokenAuthMode::RoleTokens,
    }
}

pub(super) enum TokenAuthMode {
    Disabled,
    RoleTokens,
}

fn resolve_binary_path(
    field: &'static str,
    executable: &str,
    override_path: Option<PathBuf>,
) -> Result<PathBuf, ConfigErrorV2> {
    if let Some(path) = override_path {
        if !path.is_file() {
            return Err(validation_error(
                "process.binaries",
                format!(
                    "`{field}` points to a missing executable: {}",
                    path.display()
                ),
            ));
        }
        return Ok(path);
    }

    let mut searched = Vec::new();
    if let Some(path_env) = std::env::var_os("PATH") {
        for directory in std::env::split_paths(&path_env) {
            let candidate = directory.join(executable);
            if candidate.is_file() {
                return Ok(candidate);
            }
            searched.push(candidate);
        }
    }

    for directory in conventional_postgres_bin_dirs() {
        let candidate = directory.join(executable);
        if candidate.is_file() {
            return Ok(candidate);
        }
        searched.push(candidate);
    }

    let preview = searched
        .iter()
        .take(6)
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let detail = if preview.is_empty() {
        "no candidate paths were discovered".to_string()
    } else {
        format!("searched {preview}")
    };

    Err(validation_error(
        "process.binaries",
        format!(
            "unable to resolve `{executable}` via PATH or conventional PostgreSQL install locations; {detail}; set `{field}` explicitly if autodiscovery fails"
        ),
    ))
}

fn conventional_postgres_bin_dirs() -> Vec<PathBuf> {
    let mut directories = Vec::new();
    directories.extend(child_bin_dirs_matching(
        Path::new("/usr/lib/postgresql"),
        |_| true,
    ));
    directories.extend(child_bin_dirs_matching(Path::new("/usr"), |name| {
        name.starts_with("pgsql-")
    }));
    directories.extend(child_bin_dirs_matching(
        Path::new("/opt/homebrew/opt"),
        |name| name.starts_with("postgresql@"),
    ));
    directories.extend(child_bin_dirs_matching(
        Path::new("/usr/local/opt"),
        |name| name.starts_with("postgresql@"),
    ));
    directories.push(PathBuf::from("/opt/homebrew/opt/libpq/bin"));
    directories.push(PathBuf::from("/usr/local/opt/libpq/bin"));
    directories
}

fn child_bin_dirs_matching<F>(root: &Path, predicate: F) -> Vec<PathBuf>
where
    F: Fn(&str) -> bool,
{
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };

    let mut directories = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let file_type = entry.file_type().ok()?;
            if !file_type.is_dir() {
                return None;
            }
            let name = entry.file_name();
            let name = name.to_str()?;
            predicate(name).then(|| entry.path().join("bin"))
        })
        .collect::<Vec<_>>();
    directories.sort();
    directories
}

#[cfg(test)]
mod tests {
    use super::{
        load_operator_config_contents, load_runtime_config_contents, load_runtime_timing_values,
        validate_runtime_document_contents,
    };
    use crate::{
        config_v2::{
            render_operator_test_config_toml, render_runtime_test_config_toml, toml_path_source,
            toml_string_secret, ConfigErrorV2, PgtmApiTransportExpectation,
        },
        dev_support::test_fs::unique_test_dir,
        pginfo::conninfo::PgSslMode,
    };
    use std::{fs, net::SocketAddr, os::unix::fs::PermissionsExt, path::Path, time::Duration};

    fn runtime_config_contents_with_zero_runtime_defaults(root: &Path) -> Result<String, String> {
        render_runtime_test_config_toml(
            "cluster-a",
            "scope-a",
            "node-a",
            (
                root.join("data").as_path(),
                Path::new("/tmp/pgtm-socket"),
                Path::new("/tmp/pgtm.log"),
            ),
            ["http://127.0.0.1:2379"],
            [r#"postgres.connect_timeout_s = 0
postgres.network.listen_host = "   "
postgres.network.listen_port = 0
ha.loop_interval_ms = 0
ha.lease_ttl_ms = 0
process.timeouts.pg_rewind_ms = 0
process.timeouts.bootstrap_ms = 0
process.timeouts.fencing_ms = 0
process.binaries.overrides.pg_ctl = "/bin/true"
process.binaries.overrides.initdb = "/bin/true"
process.binaries.overrides.pg_rewind = "/bin/true"
process.binaries.overrides.pg_basebackup = "/bin/true"
logging.postgres.poll_interval_ms = 0
logging.postgres.cleanup.max_files = 0
logging.postgres.cleanup.max_age_seconds = 0
logging.postgres.cleanup.protect_recent_seconds = 0"#],
        )
    }

    #[test]
    fn load_runtime_config_preserves_runtime_tls_and_operator_api_fields() -> Result<(), String> {
        let root = unique_test_dir("load-config", "runtime-config-v2-runtime-fields")?;
        let ca_cert = root.join("source-ca.crt");
        let config = load_runtime_config_contents(
            render_runtime_test_config_toml(
                "cluster-a",
                "scope-a",
                "node-a",
                (
                    root.join("data").as_path(),
                    Path::new("/tmp/pgtm-socket"),
                    Path::new("/tmp/pgtm.log"),
                ),
                ["http://127.0.0.1:2379"],
                [format!(
                    r#"[postgres.rewind.transport]
ssl_mode = "verify-full"
ca_cert = {}

[process.binaries.overrides]
pg_ctl = "/bin/true"
initdb = "/bin/true"
pg_rewind = "/bin/true"
pg_basebackup = "/bin/true"

[pgtm.api]
advertised_url = "https://127.0.0.1:18081"
expected_transport = "{}""#,
                    toml_path_source(ca_cert.as_path()),
                    PgtmApiTransportExpectation::Https.scheme(),
                )],
            )
            .map_err(|err| err.to_string())?
            .as_str(),
        )
        .map_err(|err| err.to_string())?;

        assert_eq!(
            config.postgres.source_client_tls.mode,
            PgSslMode::VerifyFull
        );
        assert_eq!(
            config.postgres.source_client_tls.root_cert,
            Some(ca_cert.clone())
        );
        assert_eq!(config.postgres.replicator.username, "replicator");
        assert_eq!(config.postgres.rewinder.username, "rewinder");
        if config
            .api
            .advertise
            .as_ref()
            .map(crate::state::ApiRoute::as_str)
            != Some("https://127.0.0.1:18081/")
        {
            return Err(format!(
                "unexpected runtime advertised API route: {:?}",
                config.api.advertise
            ));
        }

        Ok(())
    }

    #[test]
    fn load_runtime_config_and_timing_values_normalize_zero_runtime_fields() -> Result<(), String> {
        let root = unique_test_dir("load-config", "runtime-config-v2-zero-defaults")?;
        let contents = runtime_config_contents_with_zero_runtime_defaults(root.as_path())?;
        let config =
            load_runtime_config_contents(contents.as_str()).map_err(|err| err.to_string())?;

        assert_eq!(
            (
                config.postgres.listen_host.as_str(),
                config.postgres.listen_port,
                config.postgres.cluster_advertise.host(),
                config.postgres.cluster_advertise.port(),
                config.postgres.connect_timeout,
                config.timing.bootstrap_timeout,
                config.logging.postgres_log_poll_interval,
            ),
            (
                "127.0.0.1",
                5432,
                "127.0.0.1",
                5432,
                Duration::from_secs(5),
                Duration::from_millis(300_000),
                Duration::from_millis(200),
            )
        );
        assert_eq!(config.logging.postgres_log_cleanup_max_files, 50);
        let config_path = root.join("runtime.toml");
        fs::write(config_path.as_path(), contents).map_err(|err| err.to_string())?;

        let (loop_interval, lease_ttl, bootstrap_timeout, pg_rewind_timeout) =
            load_runtime_timing_values(config_path.as_path()).map_err(|err| err.to_string())?;

        assert_eq!(
            (
                loop_interval,
                lease_ttl,
                bootstrap_timeout,
                pg_rewind_timeout
            ),
            (
                Duration::from_millis(1_000),
                Duration::from_millis(10_000),
                Duration::from_millis(300_000),
                Duration::from_millis(120_000)
            )
        );
        Ok(())
    }

    #[test]
    fn load_runtime_config_does_not_require_readable_operator_auth_tokens() -> Result<(), String> {
        let root = unique_test_dir("load-config", "runtime-config-v2-operator-auth")?;
        let unreadable_token = root.join("operator-admin-token");
        fs::write(unreadable_token.as_path(), "secret\n").map_err(|err| err.to_string())?;
        fs::set_permissions(
            unreadable_token.as_path(),
            fs::Permissions::from_mode(0o000),
        )
        .map_err(|err| err.to_string())?;
        let result = load_runtime_config_contents(
            render_runtime_test_config_toml(
                "cluster-a",
                "scope-a",
                "node-a",
                (
                    root.join("data").as_path(),
                    Path::new("/tmp/pgtm-socket"),
                    Path::new("/tmp/pgtm.log"),
                ),
                ["http://127.0.0.1:2379"],
                [format!(
                    r#"[pgtm.api]
base_url = "https://127.0.0.1:8443"

[pgtm.api.auth]
type = "role_tokens"

[pgtm.api.auth.tokens]
admin_token = {{ type = "file", path = "{}" }}"#,
                    unreadable_token.display()
                )],
            )
            .map_err(|err| err.to_string())?
            .as_str(),
        );

        fs::set_permissions(
            unreadable_token.as_path(),
            fs::Permissions::from_mode(0o600),
        )
        .map_err(|err| err.to_string())?;
        let _ = fs::remove_dir_all(root);

        result.map(|_| ()).map_err(|err| err.to_string())
    }

    #[test]
    fn parse_boundaries_reject_non_path_tls_sources() -> Result<(), String> {
        let root = unique_test_dir("load-config", "runtime-config-v2-inline-tls")?;
        match validate_runtime_document_contents(
            render_runtime_test_config_toml(
                "cluster-a",
                "scope-a",
                "node-a",
                (
                    root.join("data").as_path(),
                    Path::new("/tmp/pgtm-socket"),
                    Path::new("/tmp/pgtm.log"),
                ),
                ["http://127.0.0.1:2379"],
                [r#"[postgres.tls]
mode = "enabled"
identity = { cert_chain = { content = "CERT" }, private_key = { content = "KEY" } }"#],
            )
            .map_err(|err| err.to_string())?
            .as_str(),
        ) {
            Err(ConfigErrorV2::Parse { .. }) => {}
            Err(err) => return Err(format!("expected parse error, got {err}")),
            Ok(()) => return Err("expected inline TLS parse rejection".to_string()),
        }

        match load_operator_config_contents(
            render_operator_test_config_toml(
                Some("https://127.0.0.1:8443"),
                None,
                None,
                None,
                [r#"[api.tls]
identity = { cert = { path = "/tmp/client.crt" }, key = { type = "env", env = "CLIENT_KEY" } }"#],
            )
            .map_err(|err| err.to_string())?
            .as_str(),
        ) {
            Err(ConfigErrorV2::Parse { .. }) => Ok(()),
            Err(err) => Err(format!("expected parse error, got {err}")),
            Ok(_) => Err("expected non-path TLS identity rejection".to_string()),
        }
    }

    #[test]
    fn load_operator_config_preserves_api_routing_fields_for_operator_and_runtime_documents(
    ) -> Result<(), String> {
        let operator = load_operator_config_contents(
            render_operator_test_config_toml(
                Some("https://node-b:8443"),
                Some("https://127.0.0.1:18081"),
                Some("https"),
                Some(SocketAddr::from(([127, 0, 0, 1], 18443))),
                std::iter::empty::<String>(),
            )
            .map_err(|err| err.to_string())?
            .as_str(),
        )
        .map_err(|err| err.to_string())?;

        assert_eq!(
            (
                operator.base_url.as_ref().map(reqwest::Url::as_str),
                operator.expected_transport,
                operator.resolve_to,
                operator
                    .advertised_url
                    .as_ref()
                    .map(crate::state::ApiRoute::as_str),
            ),
            (
                Some("https://node-b:8443/"),
                Some(PgtmApiTransportExpectation::Https),
                Some(SocketAddr::from(([127, 0, 0, 1], 18443))),
                Some("https://127.0.0.1:18081/"),
            )
        );

        let runtime_document = render_runtime_test_config_toml(
            "cluster-a",
            "scope-a",
            "node-a",
            (
                Path::new("/tmp/data"),
                Path::new("/tmp/pgtm-socket"),
                Path::new("/tmp/pgtm.log"),
            ),
            ["http://127.0.0.1:2379"],
            [format!(
                r#"[pgtm.api]
base_url = "https://127.0.0.1:8443"
expected_transport = "{}""#,
                PgtmApiTransportExpectation::Https.scheme(),
            )],
        )
        .map_err(|err| err.to_string())?;

        let runtime = load_runtime_config_contents(runtime_document.as_str())
            .map_err(|err| err.to_string())?;
        assert_eq!(
            runtime
                .api
                .advertise
                .as_ref()
                .map(crate::state::ApiRoute::as_str),
            None
        );

        let operator = load_operator_config_contents(runtime_document.as_str())
            .map_err(|err| err.to_string())?;
        assert_eq!(
            operator.expected_transport,
            Some(PgtmApiTransportExpectation::Https)
        );
        Ok(())
    }

    #[test]
    fn load_operator_config_flattens_tokens_and_merges_client_tls() -> Result<(), String> {
        let dir = unique_test_dir("load-operator-config", "merge")?;
        let api_ca_path = dir.join("api-ca.pem");
        let identity_cert_path = dir.join("client.crt");
        let identity_key_path = dir.join("client.key");
        let config = load_operator_config_contents(
            render_operator_test_config_toml(
                Some("https://127.0.0.1:8443"),
                None,
                None,
                None,
                [format!(
                    r#"[api.auth]
type = "role_tokens"
read_token = {}
admin_token = {}

[api.tls]
ca_cert = {}
identity = {{ cert = {}, key = {} }}

[postgres.tls]
ca_cert = {}
identity = {{ cert = {}, key = {} }}"#,
                    toml_string_secret("read-token"),
                    toml_string_secret("admin-token"),
                    toml_path_source(api_ca_path.as_path()),
                    toml_path_source(identity_cert_path.as_path()),
                    toml_path_source(identity_key_path.as_path()),
                    toml_path_source(api_ca_path.as_path()),
                    toml_path_source(identity_cert_path.as_path()),
                    toml_path_source(identity_key_path.as_path()),
                )],
            )
            .map_err(|err| err.to_string())?
            .as_str(),
        )
        .map_err(|err| err.to_string())?;
        let _ = std::fs::remove_dir_all(dir);

        assert_eq!(
            config.read_token.as_ref().map(|token| token.as_str()),
            Some("read-token")
        );
        assert_eq!(
            config.admin_token.as_ref().map(|token| token.as_str()),
            Some("admin-token")
        );
        assert_eq!(
            config
                .client_tls
                .as_ref()
                .and_then(|tls| tls.root_cert.as_ref()),
            Some(&api_ca_path)
        );
        assert_eq!(
            config
                .client_tls
                .as_ref()
                .and_then(|tls| tls.client_cert.as_ref()),
            Some(&identity_cert_path)
        );
        assert_eq!(
            config
                .client_tls
                .as_ref()
                .and_then(|tls| tls.client_key.as_ref()),
            Some(&identity_key_path)
        );
        Ok(())
    }
}
