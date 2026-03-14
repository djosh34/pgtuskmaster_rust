use std::{collections::BTreeMap, net::SocketAddr, path::PathBuf};

use crate::{
    config::{
        ApiAuthConfig, ApiConfig, ApiSecurityConfig, ApiTransportConfig, BinaryPaths,
        ClusterConfig, DcsConfig, DcsEndpoint, DebugConfig, FileSinkConfig, FileSinkMode,
        HaConfig, InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig, LoggingSinksConfig,
        PgHbaConfig, PgIdentConfig, PostgresConfig, PostgresConnIdentityConfig,
        PostgresLoggingConfig, PostgresRoleConfig, PostgresRolesConfig, ProcessConfig,
        RoleAuthConfig, RuntimeConfig, SecretSource, StderrSinkConfig, TlsServerConfig,
    },
    pginfo::conninfo::PgSslMode,
};

const SAMPLE_PG_HBA_CONTENTS: &str = "local all all trust\n";
const SAMPLE_PG_IDENT_CONTENTS: &str = "# empty\n";
#[cfg(test)]
const SAMPLE_TLS_CERT_PEM: &str = "-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n";
#[cfg(test)]
const SAMPLE_TLS_KEY_PEM: &str = "-----BEGIN PRIVATE KEY-----\nMIIB\n-----END PRIVATE KEY-----\n";
#[cfg(test)]
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

fn sample_cluster_config() -> ClusterConfig {
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

fn sample_local_conn_identity() -> PostgresConnIdentityConfig {
    PostgresConnIdentityConfig {
        user: "postgres".to_string(),
        dbname: "postgres".to_string(),
        ssl_mode: PgSslMode::Prefer,
        ca_cert: None,
    }
}

fn sample_rewind_conn_identity() -> PostgresConnIdentityConfig {
    PostgresConnIdentityConfig {
        user: "rewinder".to_string(),
        dbname: "postgres".to_string(),
        ssl_mode: PgSslMode::Prefer,
        ca_cert: None,
    }
}

fn sample_password_secret() -> SecretSource {
    SecretSource::Inline {
        content: "secret-password".to_string(),
    }
}

fn sample_postgres_roles_config() -> PostgresRolesConfig {
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

fn sample_postgres_tls_config_disabled() -> TlsServerConfig {
    TlsServerConfig::Disabled
}

#[cfg(test)]
fn sample_postgres_tls_config_enabled() -> TlsServerConfig {
    TlsServerConfig::Enabled {
        identity: crate::config::TlsServerIdentityConfig {
            cert_chain: InlineOrPath::Inline {
                content: SAMPLE_TLS_CERT_PEM.to_string(),
            },
            private_key: InlineOrPath::Inline {
                content: SAMPLE_TLS_KEY_PEM.to_string(),
            },
        },
            client_auth: Some(crate::config::TlsClientAuthConfig {
                client_ca: InlineOrPath::Inline {
                    content: SAMPLE_TLS_CA_PEM.to_string(),
                },
                client_certificate: crate::config::ClientCertificateMode::Optional,
            }),
        }
    }

fn sample_pg_hba_config() -> PgHbaConfig {
    PgHbaConfig {
        source: InlineOrPath::Inline {
            content: SAMPLE_PG_HBA_CONTENTS.to_string(),
        },
    }
}

fn sample_pg_ident_config() -> PgIdentConfig {
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

fn sample_dcs_config() -> DcsConfig {
    DcsConfig {
        endpoints: vec![sample_dcs_endpoint()],
        scope: "scope-a".to_string(),
        init: None,
    }
}

fn sample_ha_config() -> HaConfig {
    HaConfig {
        loop_interval_ms: SAMPLE_HA_LOOP_INTERVAL_MS,
        lease_ttl_ms: SAMPLE_HA_LEASE_TTL_MS,
    }
}

fn sample_process_config() -> ProcessConfig {
    ProcessConfig {
        pg_rewind_timeout_ms: SAMPLE_PROCESS_TIMEOUT_MS,
        bootstrap_timeout_ms: SAMPLE_PROCESS_TIMEOUT_MS,
        fencing_timeout_ms: SAMPLE_PROCESS_TIMEOUT_MS,
        binaries: sample_binary_paths(),
    }
}

fn sample_api_auth_disabled() -> ApiAuthConfig {
    ApiAuthConfig::Disabled
}

fn sample_api_security_config() -> ApiSecurityConfig {
    ApiSecurityConfig {
        transport: ApiTransportConfig::Http,
        auth: sample_api_auth_disabled(),
    }
}

fn sample_api_config() -> ApiConfig {
    ApiConfig {
        listen_addr: sample_api_listen_addr(),
        security: sample_api_security_config(),
    }
}

fn sample_dcs_endpoint() -> DcsEndpoint {
    DcsEndpoint::from_socket_addr(SocketAddr::from(([127, 0, 0, 1], 2379)))
}

fn sample_api_listen_addr() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 8080))
}

fn sample_debug_config() -> DebugConfig {
    DebugConfig { enabled: true }
}

fn sample_postgres_config() -> PostgresConfig {
    PostgresConfig {
        data_dir: "/tmp/pgdata".into(),
        connect_timeout_s: SAMPLE_POSTGRES_CONNECT_TIMEOUT_S,
        listen_host: SAMPLE_POSTGRES_LISTEN_HOST.to_string(),
        listen_port: SAMPLE_POSTGRES_LISTEN_PORT,
        advertise_port: None,
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

#[cfg(test)]
fn sample_runtime_config() -> RuntimeConfig {
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
                pgtm: None,
                debug: sample_debug_config(),
            },
        }
    }

    pub fn build(self) -> RuntimeConfig {
        self.config
    }

    pub fn with_dcs_scope(self, scope: impl Into<String>) -> Self {
        let scope = scope.into();
        self.transform_dcs(move |dcs| DcsConfig { scope, ..dcs })
    }

    #[cfg(test)]
    fn with_dcs_init(self, init: Option<crate::config::DcsInitConfig>) -> Self {
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

    pub fn with_postgres_data_dir(self, data_dir: impl Into<PathBuf>) -> Self {
        let data_dir = data_dir.into();
        self.transform_postgres(move |postgres| PostgresConfig {
            data_dir,
            ..postgres
        })
    }

    #[cfg(test)]
    fn with_postgres_listen_port(self, listen_port: u16) -> Self {
        self.transform_postgres(move |postgres| PostgresConfig {
            listen_port,
            ..postgres
        })
    }

    pub fn with_postgres_advertise_port(self, advertise_port: Option<u16>) -> Self {
        self.transform_postgres(move |postgres| PostgresConfig {
            advertise_port,
            ..postgres
        })
    }

    pub fn with_pg_hba(self, pg_hba: PgHbaConfig) -> Self {
        self.transform_postgres(move |postgres| PostgresConfig { pg_hba, ..postgres })
    }

    pub fn with_logging(self, logging: LoggingConfig) -> Self {
        self.transform_logging(move |_| logging)
    }

    pub fn with_process(self, process: ProcessConfig) -> Self {
        self.transform_process(move |_| process)
    }

    pub fn with_ha(self, ha: HaConfig) -> Self {
        self.transform_ha(move |_| ha)
    }

    pub fn with_debug(self, debug: DebugConfig) -> Self {
        self.transform_debug(move |_| debug)
    }

    pub fn transform<F>(self, transform: F) -> Self
    where
        F: FnOnce(RuntimeConfig) -> RuntimeConfig,
    {
        let RuntimeConfigBuilder { config } = self;
        Self {
            config: transform(config),
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

    fn transform_dcs<F>(self, transform: F) -> Self
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

    fn transform_ha<F>(self, transform: F) -> Self
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

    fn transform_process<F>(self, transform: F) -> Self
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

    fn transform_logging<F>(self, transform: F) -> Self
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

    fn transform_debug<F>(self, transform: F) -> Self
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
    use crate::dev_support::runtime_config::{
        sample_runtime_config, RuntimeConfigBuilder,
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
            .with_postgres_advertise_port(Some(6544))
            .with_api_listen_addr(sample_override_api_listen_addr())
            .with_dcs_scope("scope-b")
            .build();

        assert_eq!(updated.postgres.listen_port, 6543);
        assert_eq!(updated.postgres.advertise_port, Some(6544));
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
                    transport: crate::config::ApiTransportConfig::Https {
                        tls: crate::config::ApiTlsConfig {
                            identity: crate::config::TlsServerIdentityConfig {
                                cert_chain: crate::config::InlineOrPath::Inline {
                                    content: super::SAMPLE_TLS_CERT_PEM.to_string(),
                                },
                                private_key: crate::config::InlineOrPath::Inline {
                                    content: super::SAMPLE_TLS_KEY_PEM.to_string(),
                                },
                            },
                            client_auth: crate::config::ApiClientAuthConfig::Disabled,
                        },
                    },
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
                    read_token: Some(crate::config::SecretSource::Inline {
                        content: "read-token".to_string(),
                    }),
                    admin_token: Some(crate::config::SecretSource::Inline {
                        content: "admin-token".to_string(),
                    }),
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
                read_token: Some(crate::config::SecretSource::Inline {
                    content: "read-token".to_string(),
                }),
                admin_token: Some(crate::config::SecretSource::Inline {
                    content: "admin-token".to_string(),
                }),
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
        let tls = super::sample_postgres_tls_config_enabled();

        assert!(matches!(
            auth,
            crate::config::RoleAuthConfig::Password { .. }
        ));
        assert!(matches!(tls, crate::config::TlsServerConfig::Enabled { .. }));
    }
}
