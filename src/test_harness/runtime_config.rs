use std::{collections::BTreeMap, net::SocketAddr, path::PathBuf};

use crate::{
    config::{
        ApiAuthConfig, ApiConfig, ApiSecurityConfig, ApiTlsMode, BinaryPaths, ClusterConfig,
        DcsConfig, DcsEndpoint, DcsInitConfig, DebugConfig, FileSinkConfig, FileSinkMode,
        HaConfig, InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig, LoggingSinksConfig,
        PgHbaConfig, PgIdentConfig, PostgresConfig, PostgresConnIdentityConfig,
        PostgresLoggingConfig, PostgresRoleConfig, PostgresRolesConfig, ProcessConfig,
        RoleAuthConfig, RuntimeConfig, SecretSource, StderrSinkConfig, TlsClientAuthConfig,
        TlsServerConfig, TlsServerIdentityConfig,
    },
    pginfo::conninfo::PgSslMode,
};

const SAMPLE_PG_HBA_CONTENTS: &str = "local all all trust\n";
const SAMPLE_PG_IDENT_CONTENTS: &str = "# empty\n";
const SAMPLE_TLS_CERT_PEM: &str = "-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n";
const SAMPLE_TLS_KEY_PEM: &str = "-----BEGIN PRIVATE KEY-----\nMIIB\n-----END PRIVATE KEY-----\n";
const SAMPLE_TLS_CA_PEM: &str = "-----BEGIN CERTIFICATE-----\nMIIBCA\n-----END CERTIFICATE-----\n";
const SAMPLE_HA_LOOP_INTERVAL_MS: u64 = 1000;
const SAMPLE_HA_LEASE_TTL_MS: u64 = 10_000;
const SAMPLE_PROCESS_TIMEOUT_MS: u64 = 1000;
const SAMPLE_LOGGING_POSTGRES_POLL_INTERVAL_MS: u64 = 200;
const SAMPLE_LOGGING_CLEANUP_MAX_FILES: u64 = 10;
const SAMPLE_LOGGING_CLEANUP_MAX_AGE_SECONDS: u64 = 60;
const SAMPLE_LOGGING_CLEANUP_PROTECT_RECENT_SECONDS: u64 = 300;
const SAMPLE_POSTGRES_CONNECT_TIMEOUT_S: u32 = 5;
const SAMPLE_POSTGRES_LISTEN_HOST: &str = "127.0.0.1";
const SAMPLE_POSTGRES_LISTEN_PORT: u16 = 5432;
#[cfg(test)]
const SAMPLE_DCS_INIT_PAYLOAD_JSON: &str = "{\"ttl\":30}";

pub fn sample_cluster_config() -> ClusterConfig {
    ClusterConfig {
        name: "cluster-a".to_string(),
        member_id: "node-a".to_string(),
    }
}

pub fn sample_binary_paths() -> BinaryPaths {
    BinaryPaths {
        postgres: "/usr/bin/postgres".into(),
        pg_ctl: "/usr/bin/pg_ctl".into(),
        pg_rewind: "/usr/bin/pg_rewind".into(),
        initdb: "/usr/bin/initdb".into(),
        pg_basebackup: "/usr/bin/pg_basebackup".into(),
        psql: "/usr/bin/psql".into(),
    }
}

pub fn sample_local_conn_identity() -> PostgresConnIdentityConfig {
    PostgresConnIdentityConfig {
        user: "postgres".to_string(),
        dbname: "postgres".to_string(),
        ssl_mode: PgSslMode::Prefer,
    }
}

pub fn sample_rewind_conn_identity() -> PostgresConnIdentityConfig {
    PostgresConnIdentityConfig {
        user: "rewinder".to_string(),
        dbname: "postgres".to_string(),
        ssl_mode: PgSslMode::Prefer,
    }
}

pub fn sample_password_secret() -> SecretSource {
    SecretSource(InlineOrPath::Inline {
        content: "secret-password".to_string(),
    })
}

pub fn sample_postgres_roles_config() -> PostgresRolesConfig {
    PostgresRolesConfig {
        superuser: PostgresRoleConfig {
            username: "postgres".to_string(),
            auth: RoleAuthConfig::Password {
                password: sample_password_secret(),
            },
        },
        replicator: PostgresRoleConfig {
            username: "replicator".to_string(),
            auth: RoleAuthConfig::Password {
                password: sample_password_secret(),
            },
        },
        rewinder: PostgresRoleConfig {
            username: "rewinder".to_string(),
            auth: RoleAuthConfig::Password {
                password: sample_password_secret(),
            },
        },
    }
}

pub fn sample_postgres_tls_config_disabled() -> TlsServerConfig {
    TlsServerConfig {
        mode: ApiTlsMode::Disabled,
        identity: None,
        client_auth: None,
    }
}

pub fn sample_postgres_tls_config_enabled(mode: ApiTlsMode) -> TlsServerConfig {
    TlsServerConfig {
        mode,
        identity: Some(TlsServerIdentityConfig {
            cert_chain: InlineOrPath::Inline {
                content: SAMPLE_TLS_CERT_PEM.to_string(),
            },
            private_key: InlineOrPath::Inline {
                content: SAMPLE_TLS_KEY_PEM.to_string(),
            },
        }),
        client_auth: Some(TlsClientAuthConfig {
            client_ca: InlineOrPath::Inline {
                content: SAMPLE_TLS_CA_PEM.to_string(),
            },
            require_client_cert: false,
        }),
    }
}

pub fn sample_pg_hba_config() -> PgHbaConfig {
    PgHbaConfig {
        source: InlineOrPath::Inline {
            content: SAMPLE_PG_HBA_CONTENTS.to_string(),
        },
    }
}

pub fn sample_pg_ident_config() -> PgIdentConfig {
    PgIdentConfig {
        source: InlineOrPath::Inline {
            content: SAMPLE_PG_IDENT_CONTENTS.to_string(),
        },
    }
}

pub fn sample_postgres_logging_config() -> PostgresLoggingConfig {
    PostgresLoggingConfig {
        enabled: true,
        pg_ctl_log_file: None,
        log_dir: None,
        poll_interval_ms: SAMPLE_LOGGING_POSTGRES_POLL_INTERVAL_MS,
        cleanup: LogCleanupConfig {
            enabled: true,
            max_files: SAMPLE_LOGGING_CLEANUP_MAX_FILES,
            max_age_seconds: SAMPLE_LOGGING_CLEANUP_MAX_AGE_SECONDS,
            protect_recent_seconds: SAMPLE_LOGGING_CLEANUP_PROTECT_RECENT_SECONDS,
        },
    }
}

pub fn sample_logging_config() -> LoggingConfig {
    LoggingConfig {
        level: LogLevel::Info,
        capture_subprocess_output: true,
        postgres: sample_postgres_logging_config(),
        sinks: LoggingSinksConfig {
            stderr: StderrSinkConfig { enabled: true },
            file: FileSinkConfig {
                enabled: false,
                path: None,
                mode: FileSinkMode::Append,
            },
        },
    }
}

pub fn sample_dcs_config() -> DcsConfig {
    DcsConfig {
        endpoints: vec![sample_dcs_endpoint()],
        scope: "scope-a".to_string(),
        init: None,
    }
}

pub fn sample_ha_config() -> HaConfig {
    HaConfig {
        loop_interval_ms: SAMPLE_HA_LOOP_INTERVAL_MS,
        lease_ttl_ms: SAMPLE_HA_LEASE_TTL_MS,
    }
}

pub fn sample_process_config() -> ProcessConfig {
    ProcessConfig {
        pg_rewind_timeout_ms: SAMPLE_PROCESS_TIMEOUT_MS,
        bootstrap_timeout_ms: SAMPLE_PROCESS_TIMEOUT_MS,
        fencing_timeout_ms: SAMPLE_PROCESS_TIMEOUT_MS,
        binaries: sample_binary_paths(),
    }
}

pub fn sample_api_auth_disabled() -> ApiAuthConfig {
    ApiAuthConfig::Disabled
}

pub fn sample_api_security_config() -> ApiSecurityConfig {
    ApiSecurityConfig {
        tls: sample_postgres_tls_config_disabled(),
        auth: sample_api_auth_disabled(),
    }
}

pub fn sample_api_config() -> ApiConfig {
    ApiConfig {
        listen_addr: sample_api_listen_addr(),
        security: sample_api_security_config(),
    }
}

pub fn sample_dcs_endpoint() -> DcsEndpoint {
    DcsEndpoint::from_socket_addr(SocketAddr::from(([127, 0, 0, 1], 2379)))
}

pub fn sample_api_listen_addr() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 8080))
}

pub fn sample_debug_config() -> DebugConfig {
    DebugConfig { enabled: true }
}

pub fn sample_postgres_config() -> PostgresConfig {
    PostgresConfig {
        data_dir: "/tmp/pgdata".into(),
        connect_timeout_s: SAMPLE_POSTGRES_CONNECT_TIMEOUT_S,
        listen_host: SAMPLE_POSTGRES_LISTEN_HOST.to_string(),
        listen_port: SAMPLE_POSTGRES_LISTEN_PORT,
        socket_dir: "/tmp/pgtuskmaster/socket".into(),
        log_file: "/tmp/pgtuskmaster/postgres.log".into(),
        local_conn_identity: sample_local_conn_identity(),
        rewind_conn_identity: sample_rewind_conn_identity(),
        tls: sample_postgres_tls_config_disabled(),
        roles: sample_postgres_roles_config(),
        pg_hba: sample_pg_hba_config(),
        pg_ident: sample_pg_ident_config(),
        extra_gucs: BTreeMap::new(),
    }
}

pub fn sample_runtime_config() -> RuntimeConfig {
    RuntimeConfigBuilder::new().build()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeConfigBuilder {
    config: RuntimeConfig,
}

impl Default for RuntimeConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: RuntimeConfig {
                cluster: sample_cluster_config(),
                postgres: sample_postgres_config(),
                dcs: sample_dcs_config(),
                ha: sample_ha_config(),
                process: sample_process_config(),
                logging: sample_logging_config(),
                api: sample_api_config(),
                debug: sample_debug_config(),
            },
        }
    }

    pub fn build(self) -> RuntimeConfig {
        self.config
    }

    pub fn with_cluster_name(self, name: impl Into<String>) -> Self {
        let name = name.into();
        self.transform_cluster(move |cluster| ClusterConfig { name, ..cluster })
    }

    pub fn with_member_id(self, member_id: impl Into<String>) -> Self {
        let member_id = member_id.into();
        self.transform_cluster(move |cluster| ClusterConfig {
            member_id,
            ..cluster
        })
    }

    pub fn with_dcs_scope(self, scope: impl Into<String>) -> Self {
        let scope = scope.into();
        self.transform_dcs(move |dcs| DcsConfig { scope, ..dcs })
    }

    pub fn with_dcs_endpoints(self, endpoints: Vec<DcsEndpoint>) -> Self {
        self.transform_dcs(move |dcs| DcsConfig { endpoints, ..dcs })
    }

    pub fn with_dcs_init(self, init: Option<DcsInitConfig>) -> Self {
        self.transform_dcs(move |dcs| DcsConfig { init, ..dcs })
    }

    pub fn with_api_listen_addr(self, listen_addr: SocketAddr) -> Self {
        self.transform_api(move |api| ApiConfig { listen_addr, ..api })
    }

    pub fn with_api_auth(self, auth: ApiAuthConfig) -> Self {
        self.transform_api(|api| ApiConfig {
            security: ApiSecurityConfig {
                auth,
                ..api.security
            },
            ..api
        })
    }

    pub fn with_api_security(self, security: ApiSecurityConfig) -> Self {
        self.transform_api(move |api| ApiConfig { security, ..api })
    }

    pub fn with_postgres_data_dir(self, data_dir: impl Into<PathBuf>) -> Self {
        let data_dir = data_dir.into();
        self.transform_postgres(move |postgres| PostgresConfig {
            data_dir,
            ..postgres
        })
    }

    pub fn with_postgres_connect_timeout_s(self, connect_timeout_s: u32) -> Self {
        self.transform_postgres(move |postgres| PostgresConfig {
            connect_timeout_s,
            ..postgres
        })
    }

    pub fn with_postgres_listen_host(self, listen_host: impl Into<String>) -> Self {
        let listen_host = listen_host.into();
        self.transform_postgres(move |postgres| PostgresConfig {
            listen_host,
            ..postgres
        })
    }

    pub fn with_postgres_listen_port(self, listen_port: u16) -> Self {
        self.transform_postgres(move |postgres| PostgresConfig {
            listen_port,
            ..postgres
        })
    }

    pub fn with_postgres_socket_dir(self, socket_dir: impl Into<PathBuf>) -> Self {
        let socket_dir = socket_dir.into();
        self.transform_postgres(move |postgres| PostgresConfig {
            socket_dir,
            ..postgres
        })
    }

    pub fn with_postgres_log_file(self, log_file: impl Into<PathBuf>) -> Self {
        let log_file = log_file.into();
        self.transform_postgres(move |postgres| PostgresConfig {
            log_file,
            ..postgres
        })
    }

    pub fn with_postgres_tls(self, tls: TlsServerConfig) -> Self {
        self.transform_postgres(move |postgres| PostgresConfig { tls, ..postgres })
    }

    pub fn with_postgres_extra_gucs(self, extra_gucs: BTreeMap<String, String>) -> Self {
        self.transform_postgres(move |postgres| PostgresConfig {
            extra_gucs,
            ..postgres
        })
    }

    pub fn with_pg_hba(self, pg_hba: PgHbaConfig) -> Self {
        self.transform_postgres(move |postgres| PostgresConfig { pg_hba, ..postgres })
    }

    pub fn with_pg_ident(self, pg_ident: PgIdentConfig) -> Self {
        self.transform_postgres(move |postgres| PostgresConfig {
            pg_ident,
            ..postgres
        })
    }

    pub fn with_logging(self, logging: LoggingConfig) -> Self {
        self.transform_logging(move |_| logging)
    }

    pub fn with_process(self, process: ProcessConfig) -> Self {
        self.transform_process(move |_| process)
    }

    pub fn with_cluster(self, cluster: ClusterConfig) -> Self {
        self.transform_cluster(move |_| cluster)
    }

    pub fn with_postgres(self, postgres: PostgresConfig) -> Self {
        self.transform_postgres(move |_| postgres)
    }

    pub fn with_dcs(self, dcs: DcsConfig) -> Self {
        self.transform_dcs(move |_| dcs)
    }

    pub fn with_ha(self, ha: HaConfig) -> Self {
        self.transform_ha(move |_| ha)
    }

    pub fn with_api(self, api: ApiConfig) -> Self {
        self.transform_api(move |_| api)
    }

    pub fn with_debug(self, debug: DebugConfig) -> Self {
        self.transform_debug(move |_| debug)
    }

    pub fn transform_cluster<F>(self, transform: F) -> Self
    where
        F: FnOnce(ClusterConfig) -> ClusterConfig,
    {
        let RuntimeConfigBuilder { config } = self;
        Self {
            config: RuntimeConfig {
                cluster: transform(config.cluster),
                ..config
            },
        }
    }

    pub fn transform_postgres<F>(self, transform: F) -> Self
    where
        F: FnOnce(PostgresConfig) -> PostgresConfig,
    {
        let RuntimeConfigBuilder { config } = self;
        Self {
            config: RuntimeConfig {
                postgres: transform(config.postgres),
                ..config
            },
        }
    }

    pub fn transform_dcs<F>(self, transform: F) -> Self
    where
        F: FnOnce(DcsConfig) -> DcsConfig,
    {
        let RuntimeConfigBuilder { config } = self;
        Self {
            config: RuntimeConfig {
                dcs: transform(config.dcs),
                ..config
            },
        }
    }

    pub fn transform_ha<F>(self, transform: F) -> Self
    where
        F: FnOnce(HaConfig) -> HaConfig,
    {
        let RuntimeConfigBuilder { config } = self;
        Self {
            config: RuntimeConfig {
                ha: transform(config.ha),
                ..config
            },
        }
    }

    pub fn transform_process<F>(self, transform: F) -> Self
    where
        F: FnOnce(ProcessConfig) -> ProcessConfig,
    {
        let RuntimeConfigBuilder { config } = self;
        Self {
            config: RuntimeConfig {
                process: transform(config.process),
                ..config
            },
        }
    }

    pub fn transform_logging<F>(self, transform: F) -> Self
    where
        F: FnOnce(LoggingConfig) -> LoggingConfig,
    {
        let RuntimeConfigBuilder { config } = self;
        Self {
            config: RuntimeConfig {
                logging: transform(config.logging),
                ..config
            },
        }
    }

    pub fn transform_api<F>(self, transform: F) -> Self
    where
        F: FnOnce(ApiConfig) -> ApiConfig,
    {
        let RuntimeConfigBuilder { config } = self;
        Self {
            config: RuntimeConfig {
                api: transform(config.api),
                ..config
            },
        }
    }

    pub fn transform_debug<F>(self, transform: F) -> Self
    where
        F: FnOnce(DebugConfig) -> DebugConfig,
    {
        let RuntimeConfigBuilder { config } = self;
        Self {
            config: RuntimeConfig {
                debug: transform(config.debug),
                ..config
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::config::validate_runtime_config;
    use crate::postgres_managed::materialize_managed_postgres_config;
    use crate::postgres_managed_conf::{ManagedPostgresStartIntent, MANAGED_POSTGRESQL_CONF_NAME};
    use crate::test_harness::runtime_config::{
        sample_postgres_tls_config_enabled, sample_runtime_config, RuntimeConfigBuilder,
    };

    fn sample_override_api_listen_addr() -> std::net::SocketAddr {
        std::net::SocketAddr::from(([127, 0, 0, 1], 18080))
    }

    fn unique_temp_dir(label: &str) -> Result<std::path::PathBuf, String> {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("clock error: {err}"))?
            .as_millis();
        Ok(std::env::temp_dir().join(format!(
            "pgtm-runtime-config-{label}-{}-{millis}",
            std::process::id()
        )))
    }

    fn remove_dir_if_exists(path: &Path) -> Result<(), String> {
        match fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(format!("remove {} failed: {err}", path.display())),
        }
    }

    #[test]
    fn baseline_builder_output_passes_runtime_validation() {
        let cfg = sample_runtime_config();
        assert!(validate_runtime_config(&cfg).is_ok());
    }

    #[test]
    fn targeted_override_preserves_required_secure_fields() {
        let baseline = sample_runtime_config();
        let updated = RuntimeConfigBuilder::new()
            .with_postgres_data_dir("/tmp/override-data-dir")
            .build();

        assert_eq!(
            updated.postgres.data_dir,
            PathBuf::from("/tmp/override-data-dir")
        );
        assert_eq!(
            updated.postgres.local_conn_identity,
            baseline.postgres.local_conn_identity
        );
        assert_eq!(
            updated.postgres.rewind_conn_identity,
            baseline.postgres.rewind_conn_identity
        );
        assert_eq!(updated.postgres.roles, baseline.postgres.roles);
        assert_eq!(updated.postgres.tls, baseline.postgres.tls);
        assert_eq!(updated.api.security, baseline.api.security);
    }

    #[test]
    fn leaf_overrides_only_touch_the_intended_fields() {
        let baseline = sample_runtime_config();
        let updated = RuntimeConfigBuilder::new()
            .with_postgres_listen_port(6543)
            .with_api_listen_addr(sample_override_api_listen_addr())
            .with_dcs_scope("scope-b")
            .build();

        assert_eq!(updated.postgres.listen_port, 6543);
        assert_eq!(updated.api.listen_addr, sample_override_api_listen_addr());
        assert_eq!(updated.dcs.scope, "scope-b");
        assert_eq!(updated.cluster, baseline.cluster);
        assert_eq!(updated.postgres.listen_host, baseline.postgres.listen_host);
        assert_eq!(updated.postgres.pg_hba, baseline.postgres.pg_hba);
        assert_eq!(updated.logging, baseline.logging);
    }

    #[test]
    fn section_transform_methods_preserve_unmodified_siblings() {
        let baseline = sample_runtime_config();
        let updated = RuntimeConfigBuilder::new()
            .transform_postgres(|postgres| crate::config::PostgresConfig {
                listen_port: 5544,
                ..postgres
            })
            .transform_api(|api| crate::config::ApiConfig {
                security: crate::config::ApiSecurityConfig {
                    tls: sample_postgres_tls_config_enabled(crate::config::ApiTlsMode::Required),
                    ..api.security
                },
                ..api
            })
            .build();

        assert_eq!(updated.postgres.listen_port, 5544);
        assert_eq!(updated.postgres.data_dir, baseline.postgres.data_dir);
        assert_eq!(updated.postgres.roles, baseline.postgres.roles);
        assert_eq!(updated.api.listen_addr, baseline.api.listen_addr);
        assert_eq!(updated.debug, baseline.debug);
    }

    #[test]
    fn baseline_builder_works_with_managed_postgres_materialization() -> Result<(), String> {
        let data_dir = unique_temp_dir("materialize")?;
        remove_dir_if_exists(data_dir.as_path())?;

        let cfg = RuntimeConfigBuilder::new()
            .with_postgres_data_dir(data_dir.clone())
            .build();

        materialize_managed_postgres_config(&cfg, &ManagedPostgresStartIntent::primary())
            .map_err(|err| format!("materialize managed config failed: {err}"))?;

        let managed_conf_path = data_dir.join(MANAGED_POSTGRESQL_CONF_NAME);
        let rendered = fs::read_to_string(&managed_conf_path).map_err(|err| {
            format!(
                "read managed config {} failed: {err}",
                managed_conf_path.display()
            )
        })?;
        assert!(rendered.contains("listen_addresses = '127.0.0.1'"));
        assert!(rendered.contains("hba_file = "));
        assert!(rendered.contains("ident_file = "));

        remove_dir_if_exists(data_dir.as_path())?;
        Ok(())
    }

    #[test]
    fn builder_can_override_auth_and_dcs_init() {
        let cfg = RuntimeConfigBuilder::new()
            .with_api_auth(crate::config::ApiAuthConfig::RoleTokens(
                crate::config::ApiRoleTokensConfig {
                    read_token: Some("read-token".to_string()),
                    admin_token: Some("admin-token".to_string()),
                },
            ))
            .with_dcs_init(Some(crate::config::DcsInitConfig {
                payload_json: super::SAMPLE_DCS_INIT_PAYLOAD_JSON.to_string(),
                write_on_bootstrap: true,
            }))
            .build();

        assert_eq!(
            cfg.api.security.auth,
            crate::config::ApiAuthConfig::RoleTokens(crate::config::ApiRoleTokensConfig {
                read_token: Some("read-token".to_string()),
                admin_token: Some("admin-token".to_string()),
            })
        );
        assert_eq!(
            cfg.dcs.init,
            Some(crate::config::DcsInitConfig {
                payload_json: super::SAMPLE_DCS_INIT_PAYLOAD_JSON.to_string(),
                write_on_bootstrap: true,
            })
        );
    }

    #[test]
    fn sample_helpers_expose_password_and_tls_inputs_when_needed() {
        let auth = crate::config::RoleAuthConfig::Password {
            password: super::sample_password_secret(),
        };
        let tls = super::sample_postgres_tls_config_enabled(crate::config::ApiTlsMode::Optional);

        assert!(matches!(
            auth,
            crate::config::RoleAuthConfig::Password { .. }
        ));
        assert_eq!(tls.mode, crate::config::ApiTlsMode::Optional);
        assert!(tls.identity.is_some());
        assert!(tls.client_auth.is_some());
    }
}
