use std::{
    collections::BTreeMap,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

use serde::{Deserialize, Deserializer, Serialize};

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

fn default_postgres_connect_timeout_s() -> u32 {
    DEFAULT_POSTGRES_CONNECT_TIMEOUT_S
}

fn default_pg_ssl_mode() -> PgSslMode {
    PgSslMode::Prefer
}

macro_rules! deserialize_default_if_zero {
    ($name:ident, $ty:ty, $default:expr) => {
        fn $name<'de, D>(deserializer: D) -> Result<$ty, D::Error>
        where
            D: Deserializer<'de>,
        {
            <$ty>::deserialize(deserializer).map(|value| if value == 0 { $default } else { value })
        }
    };
}

deserialize_default_if_zero!(
    deserialize_postgres_listen_port,
    u16,
    DEFAULT_POSTGRES_LISTEN_PORT
);
deserialize_default_if_zero!(
    deserialize_postgres_connect_timeout_s,
    u32,
    DEFAULT_POSTGRES_CONNECT_TIMEOUT_S
);
deserialize_default_if_zero!(
    deserialize_ha_loop_interval_ms,
    u64,
    DEFAULT_HA_LOOP_INTERVAL_MS
);
deserialize_default_if_zero!(deserialize_ha_lease_ttl_ms, u64, DEFAULT_HA_LEASE_TTL_MS);
deserialize_default_if_zero!(
    deserialize_pg_rewind_timeout_ms,
    u64,
    DEFAULT_PG_REWIND_TIMEOUT_MS
);
deserialize_default_if_zero!(
    deserialize_bootstrap_timeout_ms,
    u64,
    DEFAULT_BOOTSTRAP_TIMEOUT_MS
);
deserialize_default_if_zero!(
    deserialize_fencing_timeout_ms,
    u64,
    DEFAULT_FENCING_TIMEOUT_MS
);
deserialize_default_if_zero!(
    deserialize_logging_postgres_poll_interval_ms,
    u64,
    DEFAULT_LOGGING_POSTGRES_POLL_INTERVAL_MS
);
deserialize_default_if_zero!(
    deserialize_logging_cleanup_max_files,
    u64,
    DEFAULT_LOGGING_CLEANUP_MAX_FILES
);
deserialize_default_if_zero!(
    deserialize_logging_cleanup_max_age_seconds,
    u64,
    DEFAULT_LOGGING_CLEANUP_MAX_AGE_SECONDS
);
deserialize_default_if_zero!(
    deserialize_logging_cleanup_protect_recent_seconds,
    u64,
    DEFAULT_LOGGING_CLEANUP_PROTECT_RECENT_SECONDS
);

fn deserialize_postgres_listen_host<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer).map(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            DEFAULT_POSTGRES_LISTEN_HOST.to_string()
        } else {
            trimmed.to_string()
        }
    })
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
    #[serde(
        default = "default_postgres_connect_timeout_s",
        deserialize_with = "deserialize_postgres_connect_timeout_s"
    )]
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
    #[serde(deserialize_with = "deserialize_postgres_listen_host")]
    pub listen_host: String,
    #[serde(deserialize_with = "deserialize_postgres_listen_port")]
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
        #[serde(flatten)]
        tls: ClientTlsInput,
        server_name: Option<String>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HaConfig {
    #[serde(deserialize_with = "deserialize_ha_loop_interval_ms")]
    pub loop_interval_ms: u64,
    #[serde(deserialize_with = "deserialize_ha_lease_ttl_ms")]
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
    #[serde(deserialize_with = "deserialize_pg_rewind_timeout_ms")]
    pub pg_rewind_ms: u64,
    #[serde(deserialize_with = "deserialize_bootstrap_timeout_ms")]
    pub bootstrap_ms: u64,
    #[serde(deserialize_with = "deserialize_fencing_timeout_ms")]
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
    pub log_dir: Option<PathBuf>,
    #[serde(deserialize_with = "deserialize_logging_postgres_poll_interval_ms")]
    pub poll_interval_ms: u64,
    #[serde(default)]
    pub cleanup: LogCleanupConfig,
}

impl Default for PostgresLoggingConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_LOGGING_POSTGRES_ENABLED,
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
    #[serde(deserialize_with = "deserialize_logging_cleanup_max_files")]
    pub max_files: u64,
    #[serde(deserialize_with = "deserialize_logging_cleanup_max_age_seconds")]
    pub max_age_seconds: u64,
    #[serde(deserialize_with = "deserialize_logging_cleanup_protect_recent_seconds")]
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
pub(super) struct ClientTlsInput {
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
    #[serde(default, skip_serializing_if = "client_tls_input_is_empty")]
    pub tls: ClientTlsInput,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OperatorPostgresConfig {
    #[serde(default, skip_serializing_if = "client_tls_input_is_empty")]
    pub tls: ClientTlsInput,
}

fn token_auth_config_is_disabled(auth: &TokenAuthConfig) -> bool {
    auth.kind.is_none()
        && auth.read_token.is_none()
        && auth.admin_token.is_none()
        && auth.tokens.is_none()
}

fn client_tls_input_is_empty(tls: &ClientTlsInput) -> bool {
    tls.ca_cert.is_none() && tls.identity.is_none()
}

fn operator_postgres_config_is_empty(postgres: &OperatorPostgresConfig) -> bool {
    client_tls_input_is_empty(&postgres.tls)
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
