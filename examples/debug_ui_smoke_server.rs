use std::time::Duration;

use pgtuskmaster_rust::{
    api::worker::{step_once, ApiWorkerCtx},
    config::{
        ApiAuthConfig, ApiConfig, ApiSecurityConfig, ApiTlsMode, BackupConfig, BinaryPaths,
        ClusterConfig, DcsConfig, DebugConfig, HaConfig, InlineOrPath, LogCleanupConfig, LogLevel,
        LoggingConfig, PgHbaConfig, PgIdentConfig, PostgresConnIdentityConfig, PostgresConfig,
        PostgresLoggingConfig, PostgresRoleConfig, PostgresRolesConfig, ProcessConfig,
        RoleAuthConfig, RuntimeConfig, StderrSinkConfig, TlsServerConfig,
    },
    dcs::store::{DcsStore, DcsStoreError, WatchEvent},
    state::{new_state_channel, UnixMillis, WorkerError},
};
use pgtuskmaster_rust::pginfo::conninfo::PgSslMode;

struct SmokeStore;

impl DcsStore for SmokeStore {
    fn healthy(&self) -> bool {
        true
    }

    fn write_path(&mut self, _path: &str, _value: String) -> Result<(), DcsStoreError> {
        Ok(())
    }

    fn delete_path(&mut self, _path: &str) -> Result<(), DcsStoreError> {
        Ok(())
    }

    fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
        Ok(Vec::new())
    }
}

fn sample_runtime_config() -> RuntimeConfig {
    RuntimeConfig {
        cluster: ClusterConfig {
            name: "cluster-a".to_string(),
            member_id: "node-a".to_string(),
        },
        postgres: PostgresConfig {
            data_dir: "/tmp/pgdata".into(),
            connect_timeout_s: 5,
            listen_host: "127.0.0.1".to_string(),
            listen_port: 5432,
            socket_dir: "/tmp/pgtuskmaster/socket".into(),
            log_file: "/tmp/pgtuskmaster/postgres.log".into(),
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
            loop_interval_ms: 1000,
            lease_ttl_ms: 10_000,
        },
        process: ProcessConfig {
            pg_rewind_timeout_ms: 1000,
            bootstrap_timeout_ms: 1000,
            fencing_timeout_ms: 1000,
            backup_timeout_ms: 1000,
            binaries: BinaryPaths {
                postgres: "/usr/bin/postgres".into(),
                pg_ctl: "/usr/bin/pg_ctl".into(),
                pg_rewind: "/usr/bin/pg_rewind".into(),
                initdb: "/usr/bin/initdb".into(),
                pg_basebackup: "/usr/bin/pg_basebackup".into(),
                psql: "/usr/bin/psql".into(),
                pgbackrest: None,
            },
        },
        backup: BackupConfig::default(),
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
            sinks: pgtuskmaster_rust::config::LoggingSinksConfig {
                stderr: StderrSinkConfig { enabled: true },
                file: pgtuskmaster_rust::config::FileSinkConfig {
                    enabled: false,
                    path: None,
                    mode: pgtuskmaster_rust::config::FileSinkMode::Append,
                },
            },
        },
        api: ApiConfig {
            listen_addr: "127.0.0.1:18080".to_string(),
            security: ApiSecurityConfig {
                tls: TlsServerConfig {
                    mode: ApiTlsMode::Disabled,
                    identity: None,
                    client_auth: None,
                },
                auth: ApiAuthConfig::Disabled,
            },
        },
        debug: DebugConfig { enabled: true },
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), WorkerError> {
    let cfg = sample_runtime_config();
    let (_cfg_publisher, cfg_subscriber) = new_state_channel(cfg, UnixMillis(1));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:18080")
        .await
        .map_err(|err| WorkerError::Message(format!("bind failed: {err}")))?;
    let mut ctx = ApiWorkerCtx::contract_stub(listener, cfg_subscriber, Box::new(SmokeStore));

    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}
