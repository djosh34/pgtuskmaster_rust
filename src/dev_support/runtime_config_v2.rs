use std::{fs, path::PathBuf, time::Duration};

use crate::{
    config::{
        resolve_inline_or_path_bytes, resolve_inline_or_path_string, resolve_secret_string,
        ApiAuthConfig, ApiTransportConfig, DcsTlsConfig, FileSinkMode as LegacyFileSinkMode,
        InlineOrPath, PostgresBinaryName, PostgresClientTransportConfig, PostgresRoleConfig,
        RoleAuthConfig, RuntimeConfig, TlsServerConfig,
    },
    config_v2::types::{
        ApiAuth, ApiConfig, ApiTransport, BinariesConfig, DcsAuth, DcsConfig, DcsEndpoint,
        FileSinkMode, LogLevel, LoggingConfig, PostgresConfig, RoleConfig, RuntimeConfigV2, Secret,
        TimingConfig, TlsConfig,
    },
    pginfo::conninfo::PgClientTls,
};

pub(crate) fn from_legacy_runtime_config(cfg: RuntimeConfig) -> Result<RuntimeConfigV2, String> {
    let postgres_tls = map_postgres_tls(&cfg)?;
    Ok(RuntimeConfigV2 {
        cluster_name: crate::state::ClusterName(cfg.cluster.name.clone()),
        scope: crate::state::ScopeName(cfg.cluster.scope.clone()),
        member_id: crate::state::MemberId(cfg.cluster.member_id.clone()),
        postgres: PostgresConfig {
            data_dir: cfg.postgres.paths.data_dir.clone(),
            socket_dir: cfg.postgres_socket_dir(),
            log_file: cfg.postgres_log_file(),
            listen_host: cfg.postgres.network.listen_host.clone(),
            listen_port: cfg.postgres.network.listen_port,
            advertise_port: cfg
                .postgres
                .network
                .advertise_port
                .unwrap_or(cfg.postgres.network.listen_port),
            connect_timeout: Duration::from_secs(u64::from(cfg.postgres.connect_timeout_s)),
            local_database: cfg.postgres.local_database.clone(),
            source_client_tls: map_postgres_source_client_tls(&cfg)?,
            superuser: map_role(
                "postgres.roles.mandatory.superuser.auth.password",
                &cfg.postgres.roles.mandatory.superuser,
            )?,
            replicator: map_role(
                "postgres.roles.mandatory.replicator.auth.password",
                &cfg.postgres.roles.mandatory.replicator,
            )?,
            rewinder: map_role(
                "postgres.roles.mandatory.rewinder.auth.password",
                &cfg.postgres.roles.mandatory.rewinder,
            )?,
            pg_hba_file: cfg.postgres.paths.data_dir.join("pgtm.pg_hba.conf"),
            pg_ident_file: cfg.postgres.paths.data_dir.join("pgtm.pg_ident.conf"),
            pg_hba_contents: resolve_inline_or_path_string(
                "postgres.access.hba",
                &cfg.postgres.access.hba,
            )
            .map_err(|err| err.to_string())?,
            pg_ident_contents: resolve_inline_or_path_string(
                "postgres.access.ident",
                &cfg.postgres.access.ident,
            )
            .map_err(|err| err.to_string())?,
            extra_gucs: cfg.postgres.extra_gucs.clone(),
            tls: postgres_tls,
        },
        dcs: DcsConfig {
            endpoints: cfg
                .dcs
                .endpoints
                .iter()
                .map(|endpoint| DcsEndpoint::new(endpoint.to_string()))
                .collect(),
            auth: map_dcs_auth(&cfg)?,
            tls: map_dcs_tls(&cfg),
        },
        timing: TimingConfig {
            ha_loop_interval: Duration::from_millis(cfg.ha.loop_interval_ms),
            ha_lease_ttl: Duration::from_millis(cfg.ha.lease_ttl_ms),
            bootstrap_timeout: Duration::from_millis(cfg.process.timeouts.bootstrap_ms),
            pg_rewind_timeout: Duration::from_millis(cfg.process.timeouts.pg_rewind_ms),
            fencing_timeout: Duration::from_millis(cfg.process.timeouts.fencing_ms),
        },
        binaries: BinariesConfig {
            postgres: resolve_binary_path(&cfg, PostgresBinaryName::Postgres)?,
            pg_ctl: resolve_binary_path(&cfg, PostgresBinaryName::PgCtl)?,
            initdb: resolve_binary_path(&cfg, PostgresBinaryName::Initdb)?,
            pg_rewind: resolve_binary_path(&cfg, PostgresBinaryName::PgRewind)?,
            pg_basebackup: resolve_binary_path(&cfg, PostgresBinaryName::PgBasebackup)?,
            psql: resolve_binary_path(&cfg, PostgresBinaryName::Psql)?,
        },
        logging: LoggingConfig {
            level: map_log_level(cfg.logging.level),
            capture_subprocess_output: cfg.logging.capture_subprocess_output,
            stderr_enabled: cfg.logging.sinks.stderr.enabled,
            file_enabled: cfg.logging.sinks.file.enabled,
            file_path: cfg
                .logging
                .sinks
                .file
                .path
                .clone()
                .unwrap_or_else(|| cfg.process.working_root.join("runtime.jsonl")),
            file_mode: match cfg.logging.sinks.file.mode {
                LegacyFileSinkMode::Append => FileSinkMode::Append,
                LegacyFileSinkMode::Truncate => FileSinkMode::Truncate,
            },
            postgres_logs_enabled: cfg.logging.postgres.enabled,
            postgres_log_dir: cfg
                .logging
                .postgres
                .log_dir
                .clone()
                .unwrap_or_else(|| cfg.process.working_root.join("logs/postgres")),
            postgres_pg_ctl_log: cfg
                .logging
                .postgres
                .pg_ctl_log_file
                .clone()
                .unwrap_or_else(|| cfg.postgres_log_file()),
            postgres_log_poll_interval: Duration::from_millis(
                cfg.logging.postgres.poll_interval_ms,
            ),
            postgres_log_cleanup_enabled: cfg.logging.postgres.cleanup.enabled,
            postgres_log_cleanup_max_files: cfg.logging.postgres.cleanup.max_files,
            postgres_log_cleanup_max_age: Duration::from_secs(
                cfg.logging.postgres.cleanup.max_age_seconds,
            ),
            postgres_log_cleanup_protect_recent: Duration::from_secs(
                cfg.logging.postgres.cleanup.protect_recent_seconds,
            ),
        },
        api: ApiConfig {
            listen_addr: cfg.api.listen_addr,
            transport: match &cfg.api.transport {
                ApiTransportConfig::Http => ApiTransport::Http,
                ApiTransportConfig::Https { .. } => ApiTransport::Http,
            },
            auth: match &cfg.api.auth {
                ApiAuthConfig::Disabled => ApiAuth::Disabled,
                ApiAuthConfig::RoleTokens(tokens) => ApiAuth::Tokens {
                    read_token: Secret::new(
                        resolve_secret_string("api.auth.read_token", &tokens.read_token)
                            .map_err(|err| err.to_string())?,
                    ),
                    admin_token: Secret::new(
                        resolve_secret_string("api.auth.admin_token", &tokens.admin_token)
                            .map_err(|err| err.to_string())?,
                    ),
                },
            },
        },
    })
}

fn map_role(field: &str, role: &PostgresRoleConfig) -> Result<RoleConfig, String> {
    let RoleAuthConfig::Password { password } = &role.auth;
    Ok(RoleConfig {
        username: role.username.as_str().to_string(),
        password: Secret::new(
            resolve_secret_string(field, password).map_err(|err| err.to_string())?,
        ),
    })
}

fn map_postgres_source_client_tls(cfg: &RuntimeConfig) -> Result<PgClientTls, String> {
    map_legacy_postgres_client_transport(
        "postgres.rewind.transport.ca_cert",
        cfg.postgres.rewind.transport.clone(),
        cfg.postgres.paths.data_dir.join("pgtm.test.source-ca.crt"),
    )
}

fn map_postgres_tls(cfg: &RuntimeConfig) -> Result<Option<TlsConfig>, String> {
    match &cfg.postgres.tls {
        TlsServerConfig::Disabled => Ok(None),
        TlsServerConfig::Enabled {
            identity,
            client_auth,
        } => Ok(Some(TlsConfig {
            cert: materialize_path_or_inline(
                "postgres.tls.identity.cert_chain",
                &identity.cert_chain,
                cfg.postgres
                    .paths
                    .data_dir
                    .join("pgtm.test.source.server.crt"),
            )?,
            key: materialize_path_or_inline(
                "postgres.tls.identity.private_key",
                &identity.private_key,
                cfg.postgres
                    .paths
                    .data_dir
                    .join("pgtm.test.source.server.key"),
            )?,
            ca_cert: client_auth
                .as_ref()
                .map(|client_auth| {
                    materialize_path_or_inline(
                        "postgres.tls.client_auth.client_ca",
                        &client_auth.client_ca,
                        cfg.postgres
                            .paths
                            .data_dir
                            .join("pgtm.test.source.client-ca.crt"),
                    )
                })
                .transpose()?,
        })),
    }
}

fn map_dcs_auth(cfg: &RuntimeConfig) -> Result<Option<DcsAuth>, String> {
    match &cfg.dcs.client.auth {
        crate::config::DcsAuthConfig::Disabled => Ok(None),
        crate::config::DcsAuthConfig::Basic { username, password } => Ok(Some(DcsAuth {
            username: username.clone(),
            password: Secret::new(
                resolve_secret_string("dcs.client.auth.password", password)
                    .map_err(|err| err.to_string())?,
            ),
        })),
    }
}

fn map_dcs_tls(cfg: &RuntimeConfig) -> Option<crate::config_v2::types::TlsConfig> {
    match &cfg.dcs.client.tls {
        DcsTlsConfig::Disabled => None,
        DcsTlsConfig::Enabled { .. } => None,
    }
}

fn resolve_binary_path(
    cfg: &RuntimeConfig,
    binary: PostgresBinaryName,
) -> Result<std::path::PathBuf, String> {
    cfg.process.binaries.resolve_binary_path(binary)
}

fn map_log_level(level: crate::config::LogLevel) -> LogLevel {
    match level {
        crate::config::LogLevel::Trace => LogLevel::Trace,
        crate::config::LogLevel::Debug => LogLevel::Debug,
        crate::config::LogLevel::Info => LogLevel::Info,
        crate::config::LogLevel::Warn => LogLevel::Warn,
        crate::config::LogLevel::Error => LogLevel::Error,
        crate::config::LogLevel::Fatal => LogLevel::Fatal,
    }
}

fn materialize_path_or_inline(
    field: &str,
    source: &InlineOrPath,
    target: PathBuf,
) -> Result<PathBuf, String> {
    match source {
        InlineOrPath::Path(path) | InlineOrPath::PathConfig { path } => Ok(path.clone()),
        InlineOrPath::Inline { .. } => {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .map_err(|err| format!("create {} failed: {err}", parent.display()))?;
            }
            let bytes =
                resolve_inline_or_path_bytes(field, source).map_err(|err| err.to_string())?;
            fs::write(&target, bytes)
                .map_err(|err| format!("write {} failed: {err}", target.display()))?;
            Ok(target)
        }
    }
}

fn map_legacy_postgres_client_transport(
    ca_field: &str,
    transport: PostgresClientTransportConfig,
    inline_target: PathBuf,
) -> Result<PgClientTls, String> {
    Ok(PgClientTls {
        mode: transport.ssl_mode,
        root_cert: transport
            .ca_cert
            .as_ref()
            .map(|ca_cert| materialize_path_or_inline(ca_field, ca_cert, inline_target))
            .transpose()?,
        client_cert: None,
        client_key: None,
    })
}
