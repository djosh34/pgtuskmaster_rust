use std::{
    collections::BTreeMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{
    config_v2::types::{
        ApiAuth, ApiConfig, ApiTransport, BinariesConfig, DcsConfig, DcsEndpoint, FileSinkMode,
        LogLevel, LoggingConfig, PostgresConfig, RoleConfig, RuntimeConfigV2, Secret, TimingConfig,
    },
    pginfo::conninfo::{PgClientTls, PgSslMode},
    state::{ClusterName, MemberId, PgRoute, ScopeName},
};

use super::HarnessError;

const SAMPLE_PG_HBA_CONTENTS: &str = "local all all trust\n";
const SAMPLE_PG_IDENT_CONTENTS: &str = "# empty\n";
const SAMPLE_POSTGRES_LISTEN_HOST: &str = "127.0.0.1";
const SAMPLE_POSTGRES_LISTEN_PORT: u16 = 5432;
const SAMPLE_RUNTIME_WORKING_ROOT: &str = "/tmp/pgtuskmaster";
const SAMPLE_RUNTIME_DATA_DIR: &str = "/tmp/pgdata";

fn sample_working_root() -> PathBuf {
    PathBuf::from(SAMPLE_RUNTIME_WORKING_ROOT)
}

fn sample_data_dir() -> PathBuf {
    PathBuf::from(SAMPLE_RUNTIME_DATA_DIR)
}

fn sample_secret(value: &str) -> Secret {
    Secret::new(value.to_string())
}

fn sample_role(username: &str) -> RoleConfig {
    RoleConfig {
        username: username.to_string(),
        password: sample_secret("secret-password"),
    }
}

pub(crate) fn sample_binary_paths() -> BinariesConfig {
    BinariesConfig {
        postgres: "/usr/bin/postgres".into(),
        pg_ctl: "/usr/bin/pg_ctl".into(),
        initdb: "/usr/bin/initdb".into(),
        pg_rewind: "/usr/bin/pg_rewind".into(),
        pg_basebackup: "/usr/bin/pg_basebackup".into(),
        psql: "/usr/bin/psql".into(),
    }
}

pub(crate) fn sample_logging_config() -> LoggingConfig {
    let working_root = sample_working_root();
    LoggingConfig {
        level: LogLevel::Info,
        capture_subprocess_output: true,
        stderr_enabled: true,
        file_enabled: false,
        file_path: working_root.join("runtime.jsonl"),
        file_mode: FileSinkMode::Append,
        postgres_logs_enabled: true,
        postgres_log_dir: working_root.join("logs/postgres"),
        postgres_pg_ctl_log: working_root.join("postgres.log"),
        postgres_log_poll_interval: Duration::from_millis(200),
        postgres_log_cleanup_enabled: true,
        postgres_log_cleanup_max_files: 10,
        postgres_log_cleanup_max_age: Duration::from_secs(60),
        postgres_log_cleanup_protect_recent: Duration::from_secs(300),
    }
}

fn sample_timing_config() -> TimingConfig {
    TimingConfig {
        ha_loop_interval: Duration::from_millis(1_000),
        ha_lease_ttl: Duration::from_millis(10_000),
        bootstrap_timeout: Duration::from_millis(1_000),
        pg_rewind_timeout: Duration::from_millis(1_000),
        fencing_timeout: Duration::from_millis(1_000),
    }
}

fn sample_api_config() -> ApiConfig {
    ApiConfig {
        listen_addr: SocketAddr::from(([127, 0, 0, 1], 8080)),
        transport: ApiTransport::Http,
        auth: ApiAuth::Disabled,
    }
}

fn sample_dcs_config() -> DcsConfig {
    DcsConfig {
        endpoints: vec![DcsEndpoint::new("http://127.0.0.1:2379".to_string())],
        auth: None,
        tls: None,
    }
}

fn sample_postgres_config() -> PostgresConfig {
    let working_root = sample_working_root();
    let data_dir = sample_data_dir();
    PostgresConfig {
        data_dir: data_dir.clone(),
        socket_dir: working_root.join("socket"),
        log_file: working_root.join("postgres.log"),
        listen_host: SAMPLE_POSTGRES_LISTEN_HOST.to_string(),
        listen_port: SAMPLE_POSTGRES_LISTEN_PORT,
        cluster_advertise: PgRoute::new(
            crate::state::PgEndpoint::Tcp {
                host: SAMPLE_POSTGRES_LISTEN_HOST.to_string(),
                port: SAMPLE_POSTGRES_LISTEN_PORT,
            },
            None,
        ),
        operator_advertise: None,
        connect_timeout: Duration::from_secs(5),
        local_database: "postgres".to_string(),
        source_client_tls: PgClientTls {
            mode: PgSslMode::Prefer,
            root_cert: None,
            client_cert: None,
            client_key: None,
        },
        superuser: sample_role("postgres"),
        replicator: sample_role("replicator"),
        rewinder: sample_role("rewinder"),
        pg_hba_file: data_dir.join("pgtm.pg_hba.conf"),
        pg_ident_file: data_dir.join("pgtm.pg_ident.conf"),
        pg_hba_contents: SAMPLE_PG_HBA_CONTENTS.to_string(),
        pg_ident_contents: SAMPLE_PG_IDENT_CONTENTS.to_string(),
        extra_gucs: BTreeMap::new(),
        tls: None,
    }
}

pub(crate) fn api_auth_from_optional_tokens(
    read_token: Option<&str>,
    admin_token: Option<&str>,
) -> Result<ApiAuth, String> {
    match (read_token, admin_token) {
        (None, None) => Ok(ApiAuth::Disabled),
        (Some(read_token), Some(admin_token)) => {
            let read_token = read_token.trim();
            let admin_token = admin_token.trim();
            if read_token.is_empty() {
                return Err("read token must not be empty".to_string());
            }
            if admin_token.is_empty() {
                return Err("admin token must not be empty".to_string());
            }
            Ok(ApiAuth::Tokens {
                read_token: sample_secret(read_token),
                admin_token: sample_secret(admin_token),
            })
        }
        _ => Err("read and admin tokens must either both be set or both be absent".to_string()),
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RuntimeConfigBuilder {
    config: RuntimeConfigV2,
}

impl Default for RuntimeConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeConfigBuilder {
    pub(crate) fn new() -> Self {
        Self {
            config: RuntimeConfigV2 {
                cluster_name: ClusterName("cluster-a".to_string()),
                scope: ScopeName("scope-a".to_string()),
                member_id: MemberId("node-a".to_string()),
                postgres: sample_postgres_config(),
                dcs: sample_dcs_config(),
                timing: sample_timing_config(),
                binaries: sample_binary_paths(),
                logging: sample_logging_config(),
                api: sample_api_config(),
            },
        }
    }

    pub(crate) fn build(self) -> RuntimeConfigV2 {
        self.config
    }

    #[cfg(test)]
    pub(crate) fn with_dcs_scope(self, scope: impl Into<String>) -> Self {
        let scope = scope.into();
        self.transform(|cfg| RuntimeConfigV2 {
            scope: ScopeName(scope),
            ..cfg
        })
    }

    #[cfg(test)]
    pub(crate) fn with_api_listen_addr(self, listen_addr: SocketAddr) -> Self {
        self.transform_api(move |api| ApiConfig { listen_addr, ..api })
    }

    pub(crate) fn with_api_auth(self, auth: ApiAuth) -> Self {
        self.transform_api(move |api| ApiConfig { auth, ..api })
    }

    #[cfg(test)]
    pub(crate) fn with_postgres_data_dir(self, data_dir: impl Into<PathBuf>) -> Self {
        let data_dir = data_dir.into();
        self.transform_postgres(move |postgres| PostgresConfig {
            pg_hba_file: data_dir.join("pgtm.pg_hba.conf"),
            pg_ident_file: data_dir.join("pgtm.pg_ident.conf"),
            data_dir,
            ..postgres
        })
    }

    #[cfg(test)]
    pub(crate) fn with_postgres_listen_port(self, listen_port: u16) -> Self {
        self.transform_postgres(move |postgres| PostgresConfig {
            listen_port,
            ..postgres
        })
    }

    #[cfg(test)]
    pub(crate) fn with_pg_hba_contents(self, hba: impl Into<String>) -> Self {
        let hba = hba.into();
        self.transform_postgres(move |postgres| PostgresConfig {
            pg_hba_contents: hba,
            ..postgres
        })
    }

    #[cfg(test)]
    pub(crate) fn with_logging(self, logging: LoggingConfig) -> Self {
        self.transform_logging(move |_| logging)
    }

    #[cfg(test)]
    pub(crate) fn with_timing(self, timing: TimingConfig) -> Self {
        self.transform_timing(move |_| timing)
    }

    #[cfg(test)]
    pub(crate) fn with_binaries(self, binaries: BinariesConfig) -> Self {
        self.transform(|cfg| RuntimeConfigV2 { binaries, ..cfg })
    }

    pub(crate) fn transform<F>(self, transform: F) -> Self
    where
        F: FnOnce(RuntimeConfigV2) -> RuntimeConfigV2,
    {
        let Self { config } = self;
        Self {
            config: transform(config),
        }
    }

    #[cfg(test)]
    pub(crate) fn transform_postgres<F>(self, transform: F) -> Self
    where
        F: FnOnce(PostgresConfig) -> PostgresConfig,
    {
        self.transform(|cfg| RuntimeConfigV2 {
            postgres: transform(cfg.postgres),
            ..cfg
        })
    }

    #[cfg(test)]
    pub(crate) fn transform_dcs<F>(self, transform: F) -> Self
    where
        F: FnOnce(DcsConfig) -> DcsConfig,
    {
        self.transform(|cfg| RuntimeConfigV2 {
            dcs: transform(cfg.dcs),
            ..cfg
        })
    }

    #[cfg(test)]
    pub(crate) fn transform_timing<F>(self, transform: F) -> Self
    where
        F: FnOnce(TimingConfig) -> TimingConfig,
    {
        self.transform(|cfg| RuntimeConfigV2 {
            timing: transform(cfg.timing),
            ..cfg
        })
    }

    #[cfg(test)]
    pub(crate) fn transform_logging<F>(self, transform: F) -> Self
    where
        F: FnOnce(LoggingConfig) -> LoggingConfig,
    {
        self.transform(|cfg| RuntimeConfigV2 {
            logging: transform(cfg.logging),
            ..cfg
        })
    }

    pub(crate) fn transform_api<F>(self, transform: F) -> Self
    where
        F: FnOnce(ApiConfig) -> ApiConfig,
    {
        self.transform(|cfg| RuntimeConfigV2 {
            api: transform(cfg.api),
            ..cfg
        })
    }
}

pub fn validate_runtime_config_path(path: &Path) -> Result<(), HarnessError> {
    crate::config_v2::load_runtime_config(path)
        .map(|_| ())
        .map_err(|err| HarnessError::InvalidInput(err.to_string()))
}

pub fn validate_runtime_config_contents(contents: &str) -> Result<(), HarnessError> {
    validate_with_temp_toml("runtime-parse", contents, |path| {
        crate::config_v2::validate_runtime_document(path).map(|_| ())
    })
}

pub fn validate_operator_config_contents(contents: &str) -> Result<(), HarnessError> {
    validate_with_temp_toml("operator", contents, |path| {
        crate::config_v2::load_operator_config(path).map(|_| ())
    })
}

pub fn runtime_timing_values(
    path: &Path,
) -> Result<(Duration, Duration, Duration, Duration), HarnessError> {
    crate::config_v2::load_runtime_timing_values(path)
        .map_err(|err| HarnessError::InvalidInput(err.to_string()))
}

fn validate_with_temp_toml<F, T>(label: &str, contents: &str, loader: F) -> Result<T, HarnessError>
where
    F: FnOnce(&Path) -> Result<T, crate::config_v2::ConfigErrorV2>,
{
    let path = unique_temp_toml_path(label);
    std::fs::write(&path, contents).map_err(HarnessError::Io)?;

    let load_result =
        loader(path.as_path()).map_err(|err| HarnessError::InvalidInput(err.to_string()));
    let cleanup_result = std::fs::remove_file(&path).map_err(HarnessError::Io);

    match (load_result, cleanup_result) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(err), Ok(())) => Err(err),
        (Ok(_), Err(cleanup)) => Err(cleanup),
        (Err(err), Err(cleanup)) => Err(HarnessError::InvalidInput(format!(
            "{err}; cleanup failed: {cleanup}"
        ))),
    }
}

fn unique_temp_toml_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "pgtm-{label}-{}-{}.toml",
        std::process::id(),
        crate::logging::system_now_unix_millis()
    ))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf, time::Duration};

    use crate::{
        config_v2::types::{ApiAuth, FileSinkMode},
        dev_support::test_fs::{remove_dir_if_exists, unique_test_dir},
        postgres_managed::materialize_managed_postgres_config,
        postgres_managed_conf::{ManagedPostgresStartIntent, MANAGED_POSTGRESQL_CONF_NAME},
    };

    use super::{api_auth_from_optional_tokens, sample_logging_config, RuntimeConfigBuilder};

    #[test]
    fn targeted_override_preserves_required_fields() {
        let baseline = RuntimeConfigBuilder::new().build();
        let updated = RuntimeConfigBuilder::new()
            .with_postgres_data_dir("/tmp/override-data-dir")
            .build();

        assert_eq!(
            updated.postgres.data_dir,
            PathBuf::from("/tmp/override-data-dir")
        );
        assert_eq!(
            updated.postgres.local_database,
            baseline.postgres.local_database
        );
        assert_eq!(
            updated.postgres.superuser.username,
            baseline.postgres.superuser.username
        );
        assert!(updated.postgres.tls.is_none());
        assert!(matches!(
            baseline.api.transport,
            crate::config_v2::types::ApiTransport::Http
        ));
        assert!(matches!(
            updated.api.transport,
            crate::config_v2::types::ApiTransport::Http
        ));
        assert!(matches!(
            baseline.api.auth,
            crate::config_v2::types::ApiAuth::Disabled
        ));
        assert!(matches!(
            updated.api.auth,
            crate::config_v2::types::ApiAuth::Disabled
        ));
    }

    #[test]
    fn timing_and_logging_overrides_are_localized() {
        let baseline = RuntimeConfigBuilder::new().build();
        let updated = RuntimeConfigBuilder::new()
            .transform_timing(|_| crate::config_v2::types::TimingConfig {
                ha_loop_interval: Duration::from_millis(500),
                ha_lease_ttl: Duration::from_secs(5),
                bootstrap_timeout: Duration::from_secs(30),
                pg_rewind_timeout: Duration::from_secs(30),
                fencing_timeout: Duration::from_secs(10),
            })
            .with_logging(crate::config_v2::types::LoggingConfig {
                file_enabled: true,
                file_mode: FileSinkMode::Truncate,
                ..sample_logging_config()
            })
            .build();

        assert_eq!(updated.cluster_name, baseline.cluster_name);
        assert_eq!(updated.postgres.listen_host, baseline.postgres.listen_host);
        assert!(updated.logging.file_enabled);
        assert!(matches!(updated.logging.file_mode, FileSinkMode::Truncate));
        assert_eq!(updated.timing.ha_loop_interval, Duration::from_millis(500));
    }

    #[test]
    fn builder_works_with_managed_postgres_materialization() -> Result<(), String> {
        let data_dir = unique_test_dir("runtime-config", "materialize")?;
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
    fn api_auth_requires_both_tokens_or_none() -> Result<(), String> {
        assert!(matches!(
            api_auth_from_optional_tokens(None, None)?,
            ApiAuth::Disabled
        ));
        assert!(matches!(
            api_auth_from_optional_tokens(Some("reader"), Some("admin"))?,
            ApiAuth::Tokens { .. }
        ));
        assert!(api_auth_from_optional_tokens(Some("reader"), None).is_err());
        Ok(())
    }
}
