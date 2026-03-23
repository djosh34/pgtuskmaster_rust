use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{
    config_v2::types::{
        ApiAuth, ApiConfig, ApiTransport, BinariesConfig, ConfigErrorV2, DcsAuth, DcsConfig,
        DcsEndpoint, FileSinkMode, LogLevel, LoggingConfig, PostgresConfig, RoleConfig,
        RuntimeConfigV2, Secret, TimingConfig, TlsConfig,
    },
    pginfo::conninfo::PgClientTls,
    state::{ClusterName, MemberId, ScopeName},
};

use super::private_schema as raw;

pub fn load_runtime_config(path: &Path) -> Result<RuntimeConfigV2, ConfigErrorV2> {
    let contents = read_config_file(path)?;
    let document: raw::RuntimeDocument =
        toml::from_str(&contents).map_err(|source| parse_error(path, source))?;

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

    let listen_host = normalized_or_default(
        Some(document.postgres.network.listen_host.clone()),
        DEFAULT_POSTGRES_LISTEN_HOST,
    );
    let listen_port = nonzero_or_default(
        document.postgres.network.listen_port,
        DEFAULT_POSTGRES_LISTEN_PORT,
    );
    let advertise_port = document
        .postgres
        .network
        .advertise_port
        .unwrap_or(listen_port);
    let connect_timeout_s = nonzero_or_default(
        document.postgres.connect_timeout_s,
        DEFAULT_POSTGRES_CONNECT_TIMEOUT_S,
    );

    let postgres = PostgresConfig {
        data_dir: data_dir.clone(),
        socket_dir,
        log_file,
        listen_host,
        listen_port,
        advertise_port,
        connect_timeout: Duration::from_secs(u64::from(connect_timeout_s)),
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
        postgres: resolve_binary_path(
            "process.binaries.overrides.postgres",
            "postgres",
            document.process.binaries.overrides.postgres,
        )?,
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
        psql: resolve_binary_path(
            "process.binaries.overrides.psql",
            "psql",
            document.process.binaries.overrides.psql,
        )?,
    };

    let postgres_log_dir = document
        .logging
        .postgres
        .log_dir
        .clone()
        .unwrap_or_else(|| working_root.join("logs/postgres"));

    let logging = LoggingConfig {
        level: map_log_level(document.logging.level),
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
        file_mode: map_file_sink_mode(document.logging.sinks.file.mode),
        postgres_logs_enabled: document.logging.postgres.enabled,
        postgres_log_dir: postgres_log_dir.clone(),
        postgres_pg_ctl_log: document
            .logging
            .postgres
            .pg_ctl_log_file
            .clone()
            .unwrap_or_else(|| postgres_log_dir.join("pg_ctl.log")),
        postgres_log_poll_interval: Duration::from_millis(nonzero_or_default(
            document.logging.postgres.poll_interval_ms,
            DEFAULT_LOGGING_POSTGRES_POLL_INTERVAL_MS,
        )),
        postgres_log_cleanup_enabled: document.logging.postgres.cleanup.enabled,
        postgres_log_cleanup_max_files: nonzero_or_default(
            document.logging.postgres.cleanup.max_files,
            DEFAULT_LOGGING_CLEANUP_MAX_FILES,
        ),
        postgres_log_cleanup_max_age: Duration::from_secs(nonzero_or_default(
            document.logging.postgres.cleanup.max_age_seconds,
            DEFAULT_LOGGING_CLEANUP_MAX_AGE_SECONDS,
        )),
        postgres_log_cleanup_protect_recent: Duration::from_secs(nonzero_or_default(
            document.logging.postgres.cleanup.protect_recent_seconds,
            DEFAULT_LOGGING_CLEANUP_PROTECT_RECENT_SECONDS,
        )),
    };

    Ok(RuntimeConfigV2 {
        cluster_name: ClusterName(document.cluster.name),
        scope: ScopeName(document.cluster.scope),
        member_id: MemberId(document.cluster.member_id),
        postgres,
        dcs,
        timing: TimingConfig {
            ha_loop_interval: Duration::from_millis(nonzero_or_default(
                document.ha.loop_interval_ms,
                DEFAULT_HA_LOOP_INTERVAL_MS,
            )),
            ha_lease_ttl: Duration::from_millis(nonzero_or_default(
                document.ha.lease_ttl_ms,
                DEFAULT_HA_LEASE_TTL_MS,
            )),
            bootstrap_timeout: Duration::from_millis(nonzero_or_default(
                document.process.timeouts.bootstrap_ms,
                DEFAULT_BOOTSTRAP_TIMEOUT_MS,
            )),
            pg_rewind_timeout: Duration::from_millis(nonzero_or_default(
                document.process.timeouts.pg_rewind_ms,
                DEFAULT_PG_REWIND_TIMEOUT_MS,
            )),
            fencing_timeout: Duration::from_millis(nonzero_or_default(
                document.process.timeouts.fencing_ms,
                DEFAULT_FENCING_TIMEOUT_MS,
            )),
        },
        binaries,
        logging,
        api: ApiConfig {
            listen_addr: document.api.listen_addr,
            transport: map_api_transport(document.api.transport)?,
            auth: map_runtime_api_auth(document.api.auth)?,
        },
    })
}

#[cfg(any(test, feature = "internal-test-support"))]
pub(crate) fn validate_runtime_document(path: &Path) -> Result<(), ConfigErrorV2> {
    let contents = read_config_file(path)?;
    let _: raw::RuntimeDocument =
        toml::from_str(&contents).map_err(|source| parse_error(path, source))?;
    Ok(())
}

#[cfg(any(test, feature = "internal-test-support"))]
pub(crate) fn load_runtime_timing_values(
    path: &Path,
) -> Result<(Duration, Duration, Duration, Duration), ConfigErrorV2> {
    let contents = read_config_file(path)?;
    let document: raw::RuntimeDocument =
        toml::from_str(&contents).map_err(|source| parse_error(path, source))?;
    Ok((
        Duration::from_millis(document.ha.loop_interval_ms),
        Duration::from_millis(document.ha.lease_ttl_ms),
        Duration::from_millis(document.process.timeouts.bootstrap_ms),
        Duration::from_millis(document.process.timeouts.pg_rewind_ms),
    ))
}

const DEFAULT_POSTGRES_CONNECT_TIMEOUT_S: u32 = 5;
const DEFAULT_POSTGRES_LISTEN_HOST: &str = "127.0.0.1";
const DEFAULT_POSTGRES_LISTEN_PORT: u16 = 5432;
const DEFAULT_HA_LOOP_INTERVAL_MS: u64 = 1_000;
const DEFAULT_HA_LEASE_TTL_MS: u64 = 10_000;
const DEFAULT_PG_REWIND_TIMEOUT_MS: u64 = 120_000;
const DEFAULT_BOOTSTRAP_TIMEOUT_MS: u64 = 300_000;
const DEFAULT_FENCING_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_LOGGING_POSTGRES_POLL_INTERVAL_MS: u64 = 200;
const DEFAULT_LOGGING_CLEANUP_MAX_FILES: u64 = 50;
const DEFAULT_LOGGING_CLEANUP_MAX_AGE_SECONDS: u64 = 7 * 24 * 60 * 60;
const DEFAULT_LOGGING_CLEANUP_PROTECT_RECENT_SECONDS: u64 = 300;

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

fn map_log_level(level: raw::LogLevel) -> LogLevel {
    match level {
        raw::LogLevel::Trace => LogLevel::Trace,
        raw::LogLevel::Debug => LogLevel::Debug,
        raw::LogLevel::Info => LogLevel::Info,
        raw::LogLevel::Warn => LogLevel::Warn,
        raw::LogLevel::Error => LogLevel::Error,
        raw::LogLevel::Fatal => LogLevel::Fatal,
    }
}

fn map_file_sink_mode(mode: raw::FileSinkMode) -> FileSinkMode {
    match mode {
        raw::FileSinkMode::Append => FileSinkMode::Append,
        raw::FileSinkMode::Truncate => FileSinkMode::Truncate,
    }
}

fn normalized_or_default(value: Option<String>, default: &str) -> String {
    if let Some(value) = value {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    default.to_string()
}

fn nonzero_or_default<T>(value: T, default: T) -> T
where
    T: Default + PartialEq,
{
    if value == T::default() {
        default
    } else {
        value
    }
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
    use super::{load_runtime_config, validate_runtime_document};
    use crate::{config_v2::ConfigErrorV2, pginfo::conninfo::PgSslMode};
    use std::{
        fs,
        path::{Path, PathBuf},
        time::SystemTime,
    };

    #[test]
    fn load_runtime_config_preserves_shared_source_client_tls() -> Result<(), String> {
        let root = unique_test_dir("runtime-config-v2-source-client-tls")?;
        fs::create_dir_all(root.join("data")).map_err(|err| err.to_string())?;
        let ca_cert = root.join("source-ca.crt");
        fs::write(&ca_cert, "test ca").map_err(|err| err.to_string())?;
        let config_path = root.join("runtime.toml");
        fs::write(
            &config_path,
            format!(
                r#"
[cluster]
name = "cluster-a"
scope = "scope-a"
member_id = "node-a"

[postgres.paths]
data_dir = "{}"

[postgres.rewind.transport]
ssl_mode = "verify_full"
ca_cert = {{ path = "{}" }}

[postgres.roles.mandatory.superuser]
username = "postgres"
auth = {{ type = "password", password = {{ type = "string", value = "postgres" }} }}

[postgres.roles.mandatory.replicator]
username = "replicator"
auth = {{ type = "password", password = {{ type = "string", value = "replicator" }} }}

[postgres.roles.mandatory.rewinder]
username = "rewinder"
auth = {{ type = "password", password = {{ type = "string", value = "rewinder" }} }}

[postgres.access]
hba = {{ content = "host all all 127.0.0.1/32 trust" }}
ident = {{ content = "" }}

[dcs]
endpoints = ["http://127.0.0.1:2379"]

[process.binaries.overrides]
postgres = "/bin/true"
pg_ctl = "/bin/true"
initdb = "/bin/true"
pg_rewind = "/bin/true"
pg_basebackup = "/bin/true"
psql = "/bin/true"
"#,
                root.join("data").display(),
                ca_cert.display()
            ),
        )
        .map_err(|err| err.to_string())?;

        let config = load_runtime_config(config_path.as_path()).map_err(|err| err.to_string())?;

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
        Ok(())
    }

    #[test]
    fn validate_runtime_document_rejects_inline_postgres_tls_sources_at_parse_boundary(
    ) -> Result<(), String> {
        let root = unique_test_dir("runtime-config-v2-inline-tls")?;
        fs::create_dir_all(&root).map_err(|err| err.to_string())?;
        let config_path = root.join("runtime.toml");
        fs::write(
            &config_path,
            format!(
                r#"
[cluster]
name = "cluster-a"
scope = "scope-a"
member_id = "node-a"

[postgres.paths]
data_dir = "{}"

[postgres.tls]
mode = "enabled"
identity = {{ cert_chain = {{ content = "CERT" }}, private_key = {{ content = "KEY" }} }}

[postgres.roles.mandatory.superuser]
username = "postgres"
auth = {{ type = "password", password = {{ type = "string", value = "postgres" }} }}

[postgres.roles.mandatory.replicator]
username = "replicator"
auth = {{ type = "password", password = {{ type = "string", value = "replicator" }} }}

[postgres.roles.mandatory.rewinder]
username = "rewinder"
auth = {{ type = "password", password = {{ type = "string", value = "rewinder" }} }}

[postgres.access]
hba = {{ content = "host all all 127.0.0.1/32 trust" }}
ident = {{ content = "" }}

[dcs]
endpoints = ["http://127.0.0.1:2379"]
"#,
                root.join("data").display(),
            ),
        )
        .map_err(|err| err.to_string())?;

        match validate_runtime_document(config_path.as_path()) {
            Err(ConfigErrorV2::Parse { .. }) => Ok(()),
            Err(err) => Err(format!("expected parse error, got {err}")),
            Ok(()) => Err("expected inline TLS parse rejection".to_string()),
        }
    }

    fn unique_test_dir(label: &str) -> Result<PathBuf, String> {
        let root = std::env::temp_dir().join(format!(
            "pgtm-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|err| err.to_string())?
                .as_nanos()
        ));
        remove_dir_if_exists(root.as_path())?;
        Ok(root)
    }

    fn remove_dir_if_exists(path: &Path) -> Result<(), String> {
        match fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(format!("remove {} failed: {err}", path.display())),
        }
    }
}
