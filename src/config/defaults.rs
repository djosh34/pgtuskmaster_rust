use super::schema::{
    ApiAuthConfig, ApiConfig, ApiRoleTokensConfig, ApiSecurityConfig, ApiTlsMode, DebugConfig,
    FileSinkConfig, FileSinkMode, InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig,
    LoggingSinksConfig, PartialRuntimeConfig, PgHbaConfig, PgIdentConfig,
    PostgresConnIdentityConfig, PostgresConfig, PostgresLoggingConfig, PostgresRoleConfig,
    PostgresRolesConfig, ProcessConfig, RoleAuthConfig, RuntimeConfig, StderrSinkConfig,
    TlsServerConfig,
};

use crate::pginfo::conninfo::PgSslMode;

const DEFAULT_PG_CONNECT_TIMEOUT_S: u32 = 5;
const DEFAULT_PG_LISTEN_HOST: &str = "127.0.0.1";
const DEFAULT_PG_LISTEN_PORT: u16 = 5432;
const DEFAULT_PG_SOCKET_DIR: &str = "/tmp/pgtuskmaster/socket";
const DEFAULT_PG_LOG_FILE: &str = "/tmp/pgtuskmaster/postgres.log";
const DEFAULT_PG_REWIND_SOURCE_HOST: &str = "127.0.0.1";
const DEFAULT_PG_REWIND_SOURCE_PORT: u16 = 5432;
const DEFAULT_PG_LOCAL_USER: &str = "postgres";
const DEFAULT_PG_LOCAL_DBNAME: &str = "postgres";
const DEFAULT_PG_REWIND_TIMEOUT_MS: u64 = 120_000;
const DEFAULT_BOOTSTRAP_TIMEOUT_MS: u64 = 300_000;
const DEFAULT_FENCING_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_API_LISTEN_ADDR: &str = "127.0.0.1:8080";
const DEFAULT_DEBUG_ENABLED: bool = false;
const DEFAULT_LOGGING_LEVEL: LogLevel = LogLevel::Info;
const DEFAULT_LOGGING_CAPTURE_SUBPROCESS_OUTPUT: bool = true;
const DEFAULT_LOGGING_POSTGRES_ENABLED: bool = true;
const DEFAULT_LOGGING_POSTGRES_POLL_INTERVAL_MS: u64 = 200;
const DEFAULT_LOGGING_CLEANUP_ENABLED: bool = true;
const DEFAULT_LOGGING_CLEANUP_MAX_FILES: u64 = 50;
const DEFAULT_LOGGING_CLEANUP_MAX_AGE_SECONDS: u64 = 7 * 24 * 60 * 60;
const DEFAULT_LOGGING_SINK_STDERR_ENABLED: bool = true;
const DEFAULT_LOGGING_SINK_FILE_ENABLED: bool = false;
const DEFAULT_LOGGING_SINK_FILE_MODE: FileSinkMode = FileSinkMode::Append;

fn tls_disabled() -> TlsServerConfig {
    TlsServerConfig {
        mode: ApiTlsMode::Disabled,
        identity: None,
        client_auth: None,
    }
}

fn default_conn_identity() -> PostgresConnIdentityConfig {
    PostgresConnIdentityConfig {
        user: DEFAULT_PG_LOCAL_USER.to_string(),
        dbname: DEFAULT_PG_LOCAL_DBNAME.to_string(),
        ssl_mode: PgSslMode::Prefer,
    }
}

fn default_roles() -> PostgresRolesConfig {
    PostgresRolesConfig {
        superuser: PostgresRoleConfig {
            username: DEFAULT_PG_LOCAL_USER.to_string(),
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
    }
}

fn empty_inline_source() -> InlineOrPath {
    InlineOrPath::Inline {
        content: String::new(),
    }
}

pub fn apply_defaults(raw: PartialRuntimeConfig) -> RuntimeConfig {
    let postgres = PostgresConfig {
        data_dir: raw.postgres.data_dir,
        connect_timeout_s: raw
            .postgres
            .connect_timeout_s
            .unwrap_or(DEFAULT_PG_CONNECT_TIMEOUT_S),
        listen_host: raw
            .postgres
            .listen_host
            .unwrap_or_else(|| DEFAULT_PG_LISTEN_HOST.to_string()),
        listen_port: raw.postgres.listen_port.unwrap_or(DEFAULT_PG_LISTEN_PORT),
        socket_dir: raw
            .postgres
            .socket_dir
            .unwrap_or_else(|| DEFAULT_PG_SOCKET_DIR.into()),
        log_file: raw
            .postgres
            .log_file
            .unwrap_or_else(|| DEFAULT_PG_LOG_FILE.into()),
        rewind_source_host: raw
            .postgres
            .rewind_source_host
            .unwrap_or_else(|| DEFAULT_PG_REWIND_SOURCE_HOST.to_string()),
        rewind_source_port: raw
            .postgres
            .rewind_source_port
            .unwrap_or(DEFAULT_PG_REWIND_SOURCE_PORT),
        local_conn_identity: default_conn_identity(),
        rewind_conn_identity: default_conn_identity(),
        tls: tls_disabled(),
        roles: default_roles(),
        pg_hba: PgHbaConfig {
            source: empty_inline_source(),
        },
        pg_ident: PgIdentConfig {
            source: empty_inline_source(),
        },
    };

    let process = ProcessConfig {
        pg_rewind_timeout_ms: raw
            .process
            .pg_rewind_timeout_ms
            .unwrap_or(DEFAULT_PG_REWIND_TIMEOUT_MS),
        bootstrap_timeout_ms: raw
            .process
            .bootstrap_timeout_ms
            .unwrap_or(DEFAULT_BOOTSTRAP_TIMEOUT_MS),
        fencing_timeout_ms: raw
            .process
            .fencing_timeout_ms
            .unwrap_or(DEFAULT_FENCING_TIMEOUT_MS),
        binaries: raw.process.binaries,
    };

    let logging_raw = raw.logging.as_ref();
    let postgres_raw = logging_raw.and_then(|cfg| cfg.postgres.as_ref());
    let cleanup_raw = postgres_raw.and_then(|cfg| cfg.cleanup.as_ref());
    let sinks_raw = logging_raw.and_then(|cfg| cfg.sinks.as_ref());
    let stderr_sink_raw = sinks_raw.and_then(|cfg| cfg.stderr.as_ref());
    let file_sink_raw = sinks_raw.and_then(|cfg| cfg.file.as_ref());
    let logging = LoggingConfig {
        level: logging_raw
            .and_then(|cfg| cfg.level)
            .unwrap_or(DEFAULT_LOGGING_LEVEL),
        capture_subprocess_output: logging_raw
            .and_then(|cfg| cfg.capture_subprocess_output)
            .unwrap_or(DEFAULT_LOGGING_CAPTURE_SUBPROCESS_OUTPUT),
        postgres: PostgresLoggingConfig {
            enabled: postgres_raw
                .and_then(|cfg| cfg.enabled)
                .unwrap_or(DEFAULT_LOGGING_POSTGRES_ENABLED),
            pg_ctl_log_file: postgres_raw.and_then(|cfg| cfg.pg_ctl_log_file.clone()),
            log_dir: postgres_raw.and_then(|cfg| cfg.log_dir.clone()),
            archive_command_log_file: postgres_raw.and_then(|cfg| cfg.archive_command_log_file.clone()),
            poll_interval_ms: postgres_raw
                .and_then(|cfg| cfg.poll_interval_ms)
                .unwrap_or(DEFAULT_LOGGING_POSTGRES_POLL_INTERVAL_MS),
            cleanup: LogCleanupConfig {
                enabled: cleanup_raw
                    .and_then(|cfg| cfg.enabled)
                    .unwrap_or(DEFAULT_LOGGING_CLEANUP_ENABLED),
                max_files: cleanup_raw
                    .and_then(|cfg| cfg.max_files)
                    .unwrap_or(DEFAULT_LOGGING_CLEANUP_MAX_FILES),
                max_age_seconds: cleanup_raw
                    .and_then(|cfg| cfg.max_age_seconds)
                    .unwrap_or(DEFAULT_LOGGING_CLEANUP_MAX_AGE_SECONDS),
            },
        },
        sinks: LoggingSinksConfig {
            stderr: StderrSinkConfig {
                enabled: stderr_sink_raw
                    .and_then(|cfg| cfg.enabled)
                    .unwrap_or(DEFAULT_LOGGING_SINK_STDERR_ENABLED),
            },
            file: FileSinkConfig {
                enabled: file_sink_raw
                    .and_then(|cfg| cfg.enabled)
                    .unwrap_or(DEFAULT_LOGGING_SINK_FILE_ENABLED),
                path: file_sink_raw.and_then(|cfg| cfg.path.clone()),
                mode: file_sink_raw
                    .and_then(|cfg| cfg.mode)
                    .unwrap_or(DEFAULT_LOGGING_SINK_FILE_MODE),
            },
        },
    };

    let api_raw = raw.api.as_ref();
    let security_raw = raw.security.as_ref();
    let api_read = api_raw.and_then(|cfg| cfg.read_auth_token.clone());
    let api_admin = api_raw.and_then(|cfg| cfg.admin_auth_token.clone());
    let legacy_token = security_raw.and_then(|cfg| cfg.auth_token.clone());
    let tls_enabled = security_raw
        .and_then(|cfg| cfg.tls_enabled)
        .unwrap_or(false);

    let auth = if api_read.is_some() || api_admin.is_some() {
        ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
            read_token: api_read,
            admin_token: api_admin,
        })
    } else if legacy_token.is_some() {
        ApiAuthConfig::RoleTokens(ApiRoleTokensConfig {
            read_token: legacy_token.clone(),
            admin_token: legacy_token,
        })
    } else {
        ApiAuthConfig::Disabled
    };

    let api = ApiConfig {
        listen_addr: api_raw
            .and_then(|cfg| cfg.listen_addr.clone())
            .unwrap_or_else(|| DEFAULT_API_LISTEN_ADDR.to_string()),
        security: ApiSecurityConfig {
            tls: TlsServerConfig {
                mode: if tls_enabled {
                    ApiTlsMode::Required
                } else {
                    ApiTlsMode::Disabled
                },
                identity: None,
                client_auth: None,
            },
            auth,
        },
    };

    let debug = DebugConfig {
        enabled: raw
            .debug
            .and_then(|cfg| cfg.enabled)
            .unwrap_or(DEFAULT_DEBUG_ENABLED),
    };

    RuntimeConfig {
        cluster: raw.cluster,
        postgres,
        dcs: raw.dcs,
        ha: raw.ha,
        process,
        logging,
        api,
        debug,
    }
}

#[cfg(test)]
    mod tests {
        use std::path::PathBuf;

        use super::*;
        use crate::pginfo::conninfo::PgSslMode;
        use crate::config::schema::{
            BinaryPaths, ClusterConfig, DcsConfig, HaConfig, PartialApiConfig, PartialDebugConfig,
            PartialFileSinkConfig, PartialLoggingConfig, PartialLoggingSinksConfig,
            PartialPostgresConfig, PartialProcessConfig, PartialSecurityConfig, PartialStderrSinkConfig,
        };

    fn base_partial() -> PartialRuntimeConfig {
        PartialRuntimeConfig {
            cluster: ClusterConfig {
                name: "cluster-a".to_string(),
                member_id: "member-a".to_string(),
            },
            postgres: PartialPostgresConfig {
                data_dir: PathBuf::from("/var/lib/postgresql/data"),
                connect_timeout_s: None,
                listen_host: None,
                listen_port: None,
                socket_dir: None,
                log_file: None,
                rewind_source_host: None,
                rewind_source_port: None,
            },
            dcs: DcsConfig {
                endpoints: vec!["http://127.0.0.1:2379".to_string()],
                scope: "demo".to_string(),
                init: None,
            },
            ha: HaConfig {
                loop_interval_ms: 1_000,
                lease_ttl_ms: 10_000,
            },
            process: PartialProcessConfig {
                pg_rewind_timeout_ms: None,
                bootstrap_timeout_ms: None,
                fencing_timeout_ms: None,
                binaries: BinaryPaths {
                    postgres: PathBuf::from("/usr/bin/postgres"),
                    pg_ctl: PathBuf::from("/usr/bin/pg_ctl"),
                    pg_rewind: PathBuf::from("/usr/bin/pg_rewind"),
                    initdb: PathBuf::from("/usr/bin/initdb"),
                    pg_basebackup: PathBuf::from("/usr/bin/pg_basebackup"),
                    psql: PathBuf::from("/usr/bin/psql"),
                },
            },
            logging: None,
            api: None,
            debug: None,
            security: None,
        }
    }

    #[test]
    fn apply_defaults_fills_optional_fields() {
        let cfg = apply_defaults(base_partial());

        assert_eq!(cfg.postgres.connect_timeout_s, DEFAULT_PG_CONNECT_TIMEOUT_S);
        assert_eq!(cfg.postgres.listen_host, DEFAULT_PG_LISTEN_HOST);
        assert_eq!(cfg.postgres.listen_port, DEFAULT_PG_LISTEN_PORT);
        assert_eq!(
            cfg.postgres.socket_dir,
            PathBuf::from(DEFAULT_PG_SOCKET_DIR)
        );
        assert_eq!(cfg.postgres.log_file, PathBuf::from(DEFAULT_PG_LOG_FILE));
        assert_eq!(
            cfg.postgres.rewind_source_host,
            DEFAULT_PG_REWIND_SOURCE_HOST
        );
        assert_eq!(
            cfg.postgres.rewind_source_port,
            DEFAULT_PG_REWIND_SOURCE_PORT
        );
        assert_eq!(
            cfg.process.pg_rewind_timeout_ms,
            DEFAULT_PG_REWIND_TIMEOUT_MS
        );
        assert_eq!(
            cfg.process.bootstrap_timeout_ms,
            DEFAULT_BOOTSTRAP_TIMEOUT_MS
        );
        assert_eq!(cfg.process.fencing_timeout_ms, DEFAULT_FENCING_TIMEOUT_MS);
        assert_eq!(cfg.api.listen_addr, DEFAULT_API_LISTEN_ADDR);
        assert!(matches!(cfg.api.security.auth, ApiAuthConfig::Disabled));
        assert_eq!(cfg.api.security.tls.mode, ApiTlsMode::Disabled);
        assert!(!cfg.debug.enabled);
        assert_eq!(cfg.postgres.local_conn_identity.user, DEFAULT_PG_LOCAL_USER);
        assert_eq!(cfg.postgres.local_conn_identity.dbname, DEFAULT_PG_LOCAL_DBNAME);
        assert_eq!(cfg.postgres.local_conn_identity.ssl_mode, PgSslMode::Prefer);
        assert_eq!(cfg.postgres.rewind_conn_identity.user, DEFAULT_PG_LOCAL_USER);
        assert_eq!(cfg.postgres.rewind_conn_identity.dbname, DEFAULT_PG_LOCAL_DBNAME);
        assert_eq!(cfg.postgres.rewind_conn_identity.ssl_mode, PgSslMode::Prefer);
        assert_eq!(cfg.postgres.roles.superuser.username, DEFAULT_PG_LOCAL_USER);
        assert_eq!(cfg.postgres.roles.replicator.username, "replicator");
        assert_eq!(cfg.postgres.roles.rewinder.username, "rewinder");
        assert_eq!(cfg.logging.level, DEFAULT_LOGGING_LEVEL);
        assert_eq!(
            cfg.logging.capture_subprocess_output,
            DEFAULT_LOGGING_CAPTURE_SUBPROCESS_OUTPUT
        );
        assert_eq!(cfg.logging.postgres.enabled, DEFAULT_LOGGING_POSTGRES_ENABLED);
        assert_eq!(
            cfg.logging.postgres.poll_interval_ms,
            DEFAULT_LOGGING_POSTGRES_POLL_INTERVAL_MS
        );
        assert_eq!(
            cfg.logging.postgres.cleanup.enabled,
            DEFAULT_LOGGING_CLEANUP_ENABLED
        );
        assert_eq!(
            cfg.logging.postgres.cleanup.max_files,
            DEFAULT_LOGGING_CLEANUP_MAX_FILES
        );
        assert_eq!(
            cfg.logging.postgres.cleanup.max_age_seconds,
            DEFAULT_LOGGING_CLEANUP_MAX_AGE_SECONDS
        );
        assert!(cfg.logging.sinks.stderr.enabled);
        assert!(!cfg.logging.sinks.file.enabled);
        assert_eq!(cfg.logging.sinks.file.path, None);
        assert_eq!(cfg.logging.sinks.file.mode, FileSinkMode::Append);
    }

    #[test]
    fn apply_defaults_preserves_caller_values() -> Result<(), String> {
        let mut raw = base_partial();
        raw.postgres.connect_timeout_s = Some(42);
        raw.postgres.listen_host = Some("0.0.0.0".to_string());
        raw.postgres.listen_port = Some(6000);
        raw.postgres.socket_dir = Some(PathBuf::from("/tmp/custom-sock"));
        raw.postgres.log_file = Some(PathBuf::from("/tmp/custom.log"));
        raw.postgres.rewind_source_host = Some("10.0.0.10".to_string());
        raw.postgres.rewind_source_port = Some(7000);
        raw.process.pg_rewind_timeout_ms = Some(2_000);
        raw.process.bootstrap_timeout_ms = Some(3_000);
        raw.process.fencing_timeout_ms = Some(4_000);
        raw.api = Some(PartialApiConfig {
            listen_addr: Some("0.0.0.0:9999".to_string()),
            read_auth_token: Some("reader".to_string()),
            admin_auth_token: Some("admin".to_string()),
        });
        raw.debug = Some(PartialDebugConfig {
            enabled: Some(true),
        });
        raw.security = Some(PartialSecurityConfig {
            tls_enabled: Some(true),
            auth_token: Some("token-123".to_string()),
        });
        raw.logging = Some(PartialLoggingConfig {
            level: Some(LogLevel::Debug),
            capture_subprocess_output: Some(false),
            postgres: None,
            sinks: Some(PartialLoggingSinksConfig {
                stderr: Some(PartialStderrSinkConfig {
                    enabled: Some(false),
                }),
                file: Some(PartialFileSinkConfig {
                    enabled: Some(true),
                    path: Some(PathBuf::from("/tmp/pgtuskmaster.jsonl")),
                    mode: Some(FileSinkMode::Truncate),
                }),
            }),
        });

        let cfg = apply_defaults(raw);

        assert_eq!(cfg.postgres.connect_timeout_s, 42);
        assert_eq!(cfg.postgres.listen_host, "0.0.0.0");
        assert_eq!(cfg.postgres.listen_port, 6000);
        assert_eq!(cfg.postgres.socket_dir, PathBuf::from("/tmp/custom-sock"));
        assert_eq!(cfg.postgres.log_file, PathBuf::from("/tmp/custom.log"));
        assert_eq!(cfg.postgres.rewind_source_host, "10.0.0.10");
        assert_eq!(cfg.postgres.rewind_source_port, 7000);
        assert_eq!(cfg.process.pg_rewind_timeout_ms, 2_000);
        assert_eq!(cfg.process.bootstrap_timeout_ms, 3_000);
        assert_eq!(cfg.process.fencing_timeout_ms, 4_000);
        assert_eq!(cfg.api.listen_addr, "0.0.0.0:9999");
        assert!(matches!(
            cfg.api.security.auth,
            ApiAuthConfig::RoleTokens(ApiRoleTokensConfig { .. })
        ));
        let tokens = match cfg.api.security.auth {
            ApiAuthConfig::RoleTokens(tokens) => tokens,
            ApiAuthConfig::Disabled => return Err("expected role_tokens auth config".to_string()),
        };
        assert_eq!(tokens.read_token.as_deref(), Some("reader"));
        assert_eq!(tokens.admin_token.as_deref(), Some("admin"));
        assert_eq!(cfg.api.security.tls.mode, ApiTlsMode::Required);
        assert!(cfg.debug.enabled);
        assert_eq!(cfg.logging.level, LogLevel::Debug);
        assert!(!cfg.logging.capture_subprocess_output);
        assert!(!cfg.logging.sinks.stderr.enabled);
        assert!(cfg.logging.sinks.file.enabled);
        assert_eq!(
            cfg.logging.sinks.file.path,
            Some(PathBuf::from("/tmp/pgtuskmaster.jsonl"))
        );
        assert_eq!(cfg.logging.sinks.file.mode, FileSinkMode::Truncate);
        Ok(())
    }
}
