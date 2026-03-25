use std::{
    collections::BTreeMap,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::{
    config_v2::types::{FileSinkMode, LogLevel, PgtmApiTransportExpectation},
    pginfo::conninfo::PgSslMode,
};

const DEFAULT_POSTGRES_DATABASE: &str = "postgres";
const DEFAULT_POSTGRES_CONNECT_TIMEOUT_S: u32 = 5;
const DEFAULT_POSTGRES_LISTEN_HOST: &str = "127.0.0.1";
const DEFAULT_POSTGRES_LISTEN_PORT: u16 = 5432;
const DEFAULT_HA_LOOP_INTERVAL_MS: u64 = 1_000;
const DEFAULT_HA_LEASE_TTL_MS: u64 = 10_000;
const DEFAULT_PG_REWIND_TIMEOUT_MS: u64 = 120_000;
const DEFAULT_BOOTSTRAP_TIMEOUT_MS: u64 = 300_000;
const DEFAULT_FENCING_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_RUNTIME_WORKING_ROOT: &str = "/tmp/pgtuskmaster";
const DEFAULT_LOGGING_CAPTURE_SUBPROCESS_OUTPUT: bool = true;
const DEFAULT_LOGGING_POSTGRES_ENABLED: bool = true;
const DEFAULT_LOGGING_POSTGRES_POLL_INTERVAL_MS: u64 = 200;
const DEFAULT_LOGGING_CLEANUP_ENABLED: bool = true;
const DEFAULT_LOGGING_CLEANUP_MAX_FILES: u64 = 50;
const DEFAULT_LOGGING_CLEANUP_MAX_AGE_SECONDS: u64 = 7 * 24 * 60 * 60;
const DEFAULT_LOGGING_CLEANUP_PROTECT_RECENT_SECONDS: u64 = 300;
const DEFAULT_LOGGING_SINK_STDERR_ENABLED: bool = true;
const DEFAULT_LOGGING_SINK_FILE_ENABLED: bool = false;
const DEFAULT_DEBUG_ENABLED: bool = false;

fn default_runtime_working_root() -> PathBuf {
    PathBuf::from(DEFAULT_RUNTIME_WORKING_ROOT)
}

fn default_api_listen_addr() -> SocketAddr {
    SocketAddr::from((std::net::Ipv4Addr::new(127, 0, 0, 1), 8080))
}

fn default_postgres_database() -> String {
    DEFAULT_POSTGRES_DATABASE.to_string()
}

fn default_pg_ssl_mode() -> PgSslMode {
    PgSslMode::Prefer
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(super) enum PathOrInline {
    Path(PathBuf),
    PathConfig { path: PathBuf },
    Inline { content: String },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(super) enum PathSource {
    Path(PathBuf),
    PathConfig { path: PathBuf },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub(super) enum SecretSource {
    Tagged(TaggedSecretSource),
    PathConfig { path: PathBuf },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub(super) enum TaggedSecretSource {
    None,
    Env { env: String },
    File { path: PathBuf },
    String { value: String },
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ClientCertificateMode {
    Optional,
    Required,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TlsServerIdentityConfig {
    pub cert_chain: PathSource,
    pub private_key: PathSource,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TlsClientIdentityConfig {
    pub cert: PathSource,
    pub key: PathSource,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TlsClientAuthConfig {
    pub client_ca: PathSource,
    pub client_certificate: ClientCertificateMode,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub(super) enum TlsServerConfig {
    #[default]
    Disabled,
    Enabled {
        identity: TlsServerIdentityConfig,
        client_auth: Option<TlsClientAuthConfig>,
    },
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(tag = "client_certificate", rename_all = "snake_case")]
pub(super) enum ApiClientAuthConfig {
    #[default]
    Disabled,
    Optional {
        client_ca: PathSource,
    },
    Required {
        client_ca: PathSource,
        #[serde(default)]
        allowed_common_names: Vec<String>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ApiTlsConfig {
    pub identity: TlsServerIdentityConfig,
    #[serde(default)]
    pub client_auth: ApiClientAuthConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(tag = "transport", rename_all = "snake_case")]
pub(super) enum ApiTransportConfig {
    #[default]
    Http,
    Https {
        tls: ApiTlsConfig,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RuntimeDocument {
    pub cluster: ClusterConfig,
    pub postgres: PostgresConfig,
    pub dcs: DcsConfig,
    #[serde(default)]
    pub ha: HaConfig,
    #[serde(default)]
    pub process: ProcessConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub pgtm: Option<toml::Value>,
    #[serde(default)]
    pub debug: DebugConfig,
}

impl RuntimeDocument {
    pub(super) fn normalize(mut self) -> Self {
        self.postgres.network.listen_host = {
            let listen_host = self.postgres.network.listen_host.trim();
            if listen_host.is_empty() {
                DEFAULT_POSTGRES_LISTEN_HOST.to_string()
            } else {
                listen_host.to_string()
            }
        };
        self.postgres.network.listen_port = default_if_zero(
            self.postgres.network.listen_port,
            DEFAULT_POSTGRES_LISTEN_PORT,
        );
        self.postgres.connect_timeout_s = default_if_zero(
            self.postgres.connect_timeout_s,
            DEFAULT_POSTGRES_CONNECT_TIMEOUT_S,
        );
        for (value, default) in [
            (&mut self.ha.loop_interval_ms, DEFAULT_HA_LOOP_INTERVAL_MS),
            (&mut self.ha.lease_ttl_ms, DEFAULT_HA_LEASE_TTL_MS),
            (
                &mut self.process.timeouts.pg_rewind_ms,
                DEFAULT_PG_REWIND_TIMEOUT_MS,
            ),
            (
                &mut self.process.timeouts.bootstrap_ms,
                DEFAULT_BOOTSTRAP_TIMEOUT_MS,
            ),
            (
                &mut self.process.timeouts.fencing_ms,
                DEFAULT_FENCING_TIMEOUT_MS,
            ),
            (
                &mut self.logging.postgres.poll_interval_ms,
                DEFAULT_LOGGING_POSTGRES_POLL_INTERVAL_MS,
            ),
            (
                &mut self.logging.postgres.cleanup.max_files,
                DEFAULT_LOGGING_CLEANUP_MAX_FILES,
            ),
            (
                &mut self.logging.postgres.cleanup.max_age_seconds,
                DEFAULT_LOGGING_CLEANUP_MAX_AGE_SECONDS,
            ),
            (
                &mut self.logging.postgres.cleanup.protect_recent_seconds,
                DEFAULT_LOGGING_CLEANUP_PROTECT_RECENT_SECONDS,
            ),
        ] {
            *value = default_if_zero(*value, default);
        }
        self
    }
}

fn default_if_zero<T>(value: T, default: T) -> T
where
    T: Default + PartialEq,
{
    if value == T::default() {
        default
    } else {
        value
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ClusterConfig {
    pub name: String,
    pub scope: String,
    pub member_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresConfig {
    pub paths: PostgresPathsConfig,
    #[serde(default)]
    pub network: PostgresNetworkConfig,
    #[serde(default)]
    pub connect_timeout_s: u32,
    #[serde(default = "default_postgres_database")]
    pub local_database: String,
    #[serde(default)]
    pub rewind: PostgresRewindConfig,
    #[serde(default)]
    pub tls: TlsServerConfig,
    pub roles: PostgresRolesConfig,
    pub access: PostgresAccessConfig,
    #[serde(default)]
    pub extra_gucs: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresPathsConfig {
    pub data_dir: PathBuf,
    pub socket_dir: Option<PathBuf>,
    pub log_file: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresNetworkConfig {
    pub listen_host: String,
    pub listen_port: u16,
    pub cluster_advertise: Option<PostgresAdvertiseConfig>,
    pub operator_advertise: Option<PostgresAdvertiseConfig>,
}

impl Default for PostgresNetworkConfig {
    fn default() -> Self {
        Self {
            listen_host: DEFAULT_POSTGRES_LISTEN_HOST.to_string(),
            listen_port: DEFAULT_POSTGRES_LISTEN_PORT,
            cluster_advertise: None,
            operator_advertise: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresAdvertiseConfig {
    pub host: String,
    pub port: u16,
    pub hostaddr: Option<IpAddr>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresRewindConfig {
    #[serde(default = "default_postgres_database")]
    pub database: String,
    #[serde(default)]
    pub transport: PostgresClientTransportConfig,
}

impl Default for PostgresRewindConfig {
    fn default() -> Self {
        Self {
            database: DEFAULT_POSTGRES_DATABASE.to_string(),
            transport: PostgresClientTransportConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresClientTransportConfig {
    #[serde(default = "default_pg_ssl_mode")]
    pub ssl_mode: PgSslMode,
    pub ca_cert: Option<PathSource>,
}

impl Default for PostgresClientTransportConfig {
    fn default() -> Self {
        Self {
            ssl_mode: PgSslMode::Prefer,
            ca_cert: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum RoleAuthConfig {
    Password { password: SecretSource },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresRoleConfig {
    pub username: String,
    pub auth: RoleAuthConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MandatoryPostgresRolesConfig {
    pub superuser: PostgresRoleConfig,
    pub replicator: PostgresRoleConfig,
    pub rewinder: PostgresRoleConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresRolesConfig {
    pub mandatory: MandatoryPostgresRolesConfig,
    #[serde(default)]
    pub extra: BTreeMap<String, toml::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresAccessConfig {
    pub hba: PathOrInline,
    pub ident: PathOrInline,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DcsConfig {
    pub endpoints: Vec<String>,
    #[serde(default)]
    pub client: DcsClientConfig,
    pub init: Option<toml::Value>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DcsClientConfig {
    #[serde(default)]
    pub auth: DcsAuthConfig,
    #[serde(default)]
    pub tls: DcsTlsConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum DcsAuthConfig {
    #[default]
    Disabled,
    Basic {
        username: String,
        password: SecretSource,
    },
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub(super) enum DcsTlsConfig {
    #[default]
    Disabled,
    Enabled {
        ca_cert: Option<PathSource>,
        identity: Option<TlsClientIdentityConfig>,
        server_name: Option<String>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HaConfig {
    pub loop_interval_ms: u64,
    pub lease_ttl_ms: u64,
}

impl Default for HaConfig {
    fn default() -> Self {
        Self {
            loop_interval_ms: DEFAULT_HA_LOOP_INTERVAL_MS,
            lease_ttl_ms: DEFAULT_HA_LEASE_TTL_MS,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProcessConfig {
    #[serde(default)]
    pub timeouts: ProcessTimeoutsConfig,
    #[serde(default = "default_runtime_working_root")]
    pub working_root: PathBuf,
    #[serde(default)]
    pub binaries: BinaryResolutionConfig,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            timeouts: ProcessTimeoutsConfig::default(),
            working_root: PathBuf::from(DEFAULT_RUNTIME_WORKING_ROOT),
            binaries: BinaryResolutionConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProcessTimeoutsConfig {
    pub pg_rewind_ms: u64,
    pub bootstrap_ms: u64,
    pub fencing_ms: u64,
}

impl Default for ProcessTimeoutsConfig {
    fn default() -> Self {
        Self {
            pg_rewind_ms: DEFAULT_PG_REWIND_TIMEOUT_MS,
            bootstrap_ms: DEFAULT_BOOTSTRAP_TIMEOUT_MS,
            fencing_ms: DEFAULT_FENCING_TIMEOUT_MS,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BinaryResolutionConfig {
    #[serde(default)]
    pub overrides: BinaryPathOverrides,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BinaryPathOverrides {
    pub pg_ctl: Option<PathBuf>,
    pub pg_rewind: Option<PathBuf>,
    pub initdb: Option<PathBuf>,
    pub pg_basebackup: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LoggingConfig {
    #[serde(default)]
    pub level: LogLevel,
    pub capture_subprocess_output: bool,
    #[serde(default)]
    pub postgres: PostgresLoggingConfig,
    #[serde(default)]
    pub sinks: LoggingSinksConfig,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            capture_subprocess_output: DEFAULT_LOGGING_CAPTURE_SUBPROCESS_OUTPUT,
            postgres: PostgresLoggingConfig::default(),
            sinks: LoggingSinksConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresLoggingConfig {
    pub enabled: bool,
    pub pg_ctl_log_file: Option<PathBuf>,
    pub log_dir: Option<PathBuf>,
    pub poll_interval_ms: u64,
    #[serde(default)]
    pub cleanup: LogCleanupConfig,
}

impl Default for PostgresLoggingConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_LOGGING_POSTGRES_ENABLED,
            pg_ctl_log_file: None,
            log_dir: None,
            poll_interval_ms: DEFAULT_LOGGING_POSTGRES_POLL_INTERVAL_MS,
            cleanup: LogCleanupConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LoggingSinksConfig {
    #[serde(default)]
    pub stderr: StderrSinkConfig,
    #[serde(default)]
    pub file: FileSinkConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StderrSinkConfig {
    pub enabled: bool,
}

impl Default for StderrSinkConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_LOGGING_SINK_STDERR_ENABLED,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FileSinkConfig {
    pub enabled: bool,
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub mode: FileSinkMode,
}

impl Default for FileSinkConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_LOGGING_SINK_FILE_ENABLED,
            path: None,
            mode: FileSinkMode::Append,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LogCleanupConfig {
    pub enabled: bool,
    pub max_files: u64,
    pub max_age_seconds: u64,
    pub protect_recent_seconds: u64,
}

impl Default for LogCleanupConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_LOGGING_CLEANUP_ENABLED,
            max_files: DEFAULT_LOGGING_CLEANUP_MAX_FILES,
            max_age_seconds: DEFAULT_LOGGING_CLEANUP_MAX_AGE_SECONDS,
            protect_recent_seconds: DEFAULT_LOGGING_CLEANUP_PROTECT_RECENT_SECONDS,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ApiConfig {
    #[serde(default = "default_api_listen_addr")]
    pub listen_addr: SocketAddr,
    #[serde(default)]
    pub transport: ApiTransportConfig,
    #[serde(default)]
    pub auth: TokenAuthConfig,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            listen_addr: default_api_listen_addr(),
            transport: ApiTransportConfig::default(),
            auth: TokenAuthConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TokenAuthConfig {
    #[serde(rename = "type")]
    pub(super) kind: Option<String>,
    pub(super) read_token: Option<SecretSource>,
    pub(super) admin_token: Option<SecretSource>,
    pub(super) tokens: Option<RoleTokens>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RoleTokens {
    pub read_token: Option<SecretSource>,
    pub admin_token: Option<SecretSource>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OperatorClientTlsInput {
    pub(super) ca_cert: Option<PathSource>,
    pub(super) identity: Option<TlsClientIdentityConfig>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OperatorDocument {
    #[serde(default)]
    pub api: OperatorApiConfig,
    #[serde(default)]
    #[serde(skip_serializing_if = "operator_postgres_config_is_empty")]
    pub postgres: OperatorPostgresConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OperatorApiConfig {
    pub base_url: Option<String>,
    pub advertised_url: Option<String>,
    pub expected_transport: Option<PgtmApiTransportExpectation>,
    pub resolve_to: Option<SocketAddr>,
    #[serde(default, skip_serializing_if = "token_auth_config_is_disabled")]
    pub auth: TokenAuthConfig,
    #[serde(default, skip_serializing_if = "operator_client_tls_input_is_empty")]
    pub tls: OperatorClientTlsInput,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OperatorPostgresConfig {
    #[serde(default, skip_serializing_if = "operator_client_tls_input_is_empty")]
    pub tls: OperatorClientTlsInput,
}

fn token_auth_config_is_disabled(auth: &TokenAuthConfig) -> bool {
    auth.kind.is_none()
        && auth.read_token.is_none()
        && auth.admin_token.is_none()
        && auth.tokens.is_none()
}

fn operator_client_tls_input_is_empty(tls: &OperatorClientTlsInput) -> bool {
    tls.ca_cert.is_none() && tls.identity.is_none()
}

fn operator_postgres_config_is_empty(postgres: &OperatorPostgresConfig) -> bool {
    operator_client_tls_input_is_empty(&postgres.tls)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DebugConfig {
    pub enabled: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_DEBUG_ENABLED,
        }
    }
}

#[cfg(any(test, feature = "internal-test-support"))]
fn string_secret(value: &str) -> SecretSource {
    SecretSource::Tagged(TaggedSecretSource::String {
        value: value.to_string(),
    })
}

#[cfg(any(test, feature = "internal-test-support"))]
fn toml_value<T: Serialize>(label: &str, value: T) -> Result<toml::Value, String> {
    toml::Value::try_from(value)
        .map_err(|error| format!("{label} serialization to toml::Value failed: {error}"))
}

#[cfg(any(test, feature = "internal-test-support"))]
fn render_toml_value(label: &str, value: &toml::Value) -> Result<String, String> {
    toml::to_string(value).map_err(|error| format!("{label} serialization failed: {error}"))
}

#[cfg(any(test, feature = "internal-test-support"))]
fn trim_runtime_test_document(value: &mut toml::Value) -> Result<(), String> {
    let root = value
        .as_table_mut()
        .ok_or_else(|| "runtime test document should serialize as a TOML table".to_string())?;
    for key in ["ha", "process", "logging", "api", "pgtm", "debug"] {
        let _ = root.remove(key);
    }

    if let Some(postgres) = root.get_mut("postgres").and_then(toml::Value::as_table_mut) {
        let _ = postgres.remove("connect_timeout_s");
        let _ = postgres.remove("rewind");
        let _ = postgres.remove("tls");
        let _ = postgres.remove("extra_gucs");
        if let Some(roles) = postgres
            .get_mut("roles")
            .and_then(toml::Value::as_table_mut)
        {
            let _ = roles.remove("extra");
        }
    }

    if let Some(dcs) = root.get_mut("dcs").and_then(toml::Value::as_table_mut) {
        let _ = dcs.remove("client");
        let _ = dcs.remove("init");
    }

    Ok(())
}

#[cfg(any(test, feature = "internal-test-support"))]
fn join_rendered_sections<J, T>(base: String, extra_sections: J) -> String
where
    J: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    std::iter::once(base)
        .chain(extra_sections.into_iter().filter_map(|section| {
            let section = section.as_ref().trim();
            (!section.is_empty()).then_some(section.to_string())
        }))
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(any(test, feature = "internal-test-support"))]
fn build_runtime_test_document<I, S>(
    cluster_name: &str,
    scope: &str,
    member_id: &str,
    paths: (&std::path::Path, &std::path::Path, &std::path::Path),
    dcs_endpoints: I,
) -> RuntimeDocument
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let (data_dir, socket_dir, log_file) = paths;
    RuntimeDocument {
        cluster: ClusterConfig {
            name: cluster_name.to_string(),
            scope: scope.to_string(),
            member_id: member_id.to_string(),
        },
        postgres: PostgresConfig {
            paths: PostgresPathsConfig {
                data_dir: data_dir.to_path_buf(),
                socket_dir: Some(socket_dir.to_path_buf()),
                log_file: Some(log_file.to_path_buf()),
            },
            network: PostgresNetworkConfig::default(),
            connect_timeout_s: DEFAULT_POSTGRES_CONNECT_TIMEOUT_S,
            local_database: DEFAULT_POSTGRES_DATABASE.to_string(),
            rewind: PostgresRewindConfig::default(),
            tls: TlsServerConfig::default(),
            roles: PostgresRolesConfig {
                mandatory: MandatoryPostgresRolesConfig {
                    superuser: PostgresRoleConfig {
                        username: "postgres".to_string(),
                        auth: RoleAuthConfig::Password {
                            password: string_secret("postgres"),
                        },
                    },
                    replicator: PostgresRoleConfig {
                        username: "replicator".to_string(),
                        auth: RoleAuthConfig::Password {
                            password: string_secret("replicator"),
                        },
                    },
                    rewinder: PostgresRoleConfig {
                        username: "rewinder".to_string(),
                        auth: RoleAuthConfig::Password {
                            password: string_secret("rewinder"),
                        },
                    },
                },
                extra: BTreeMap::new(),
            },
            access: PostgresAccessConfig {
                hba: PathOrInline::Inline {
                    content: "host all all 127.0.0.1/32 trust".to_string(),
                },
                ident: PathOrInline::Inline {
                    content: String::new(),
                },
            },
            extra_gucs: BTreeMap::new(),
        },
        dcs: DcsConfig {
            endpoints: dcs_endpoints
                .into_iter()
                .map(|endpoint| endpoint.as_ref().to_string())
                .collect(),
            client: DcsClientConfig::default(),
            init: None,
        },
        ha: HaConfig::default(),
        process: ProcessConfig::default(),
        logging: LoggingConfig::default(),
        api: ApiConfig::default(),
        pgtm: None,
        debug: DebugConfig::default(),
    }
}

#[cfg(any(test, feature = "internal-test-support"))]
pub(crate) fn render_runtime_test_config_toml<I, S, J, T>(
    cluster_name: &str,
    scope: &str,
    member_id: &str,
    paths: (&std::path::Path, &std::path::Path, &std::path::Path),
    dcs_endpoints: I,
    extra_sections: J,
) -> Result<String, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
    J: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    let mut value = toml_value(
        "runtime test document",
        build_runtime_test_document(cluster_name, scope, member_id, paths, dcs_endpoints),
    )?;
    trim_runtime_test_document(&mut value)?;
    Ok(join_rendered_sections(
        render_toml_value("runtime test config", &value)?,
        extra_sections,
    ))
}
