use std::{collections::BTreeMap, net::SocketAddr, path::PathBuf};

use serde::Deserialize;

use crate::pginfo::conninfo::PgSslMode;

const DEFAULT_POSTGRES_DATABASE: &str = "postgres";
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

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum PathOrInline {
    Path(PathBuf),
    PathConfig { path: PathBuf },
    Inline { content: String },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum PathSource {
    Path(PathBuf),
    PathConfig { path: PathBuf },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum SecretSource {
    Tagged(TaggedSecretSource),
    PathConfig { path: PathBuf },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub(super) enum TaggedSecretSource {
    None,
    Env { env: String },
    File { path: PathBuf },
    String { value: String },
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ClientCertificateMode {
    Optional,
    Required,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TlsServerIdentityConfig {
    pub cert_chain: PathSource,
    pub private_key: PathSource,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TlsClientIdentityConfig {
    pub cert: PathSource,
    pub key: PathSource,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TlsClientAuthConfig {
    pub client_ca: PathSource,
    pub client_certificate: ClientCertificateMode,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub(super) enum TlsServerConfig {
    #[default]
    Disabled,
    Enabled {
        identity: TlsServerIdentityConfig,
        client_auth: Option<TlsClientAuthConfig>,
    },
}

#[derive(Clone, Debug, Default, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ApiTlsConfig {
    pub identity: TlsServerIdentityConfig,
    #[serde(default)]
    pub client_auth: ApiClientAuthConfig,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(tag = "transport", rename_all = "snake_case")]
pub(super) enum ApiTransportConfig {
    #[default]
    Http,
    Https {
        tls: ApiTlsConfig,
    },
}

#[derive(Clone, Debug, Deserialize)]
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
    pub pgtm: Option<OperatorDocument>,
    #[serde(default)]
    pub debug: DebugConfig,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum OperatorConfigDocument {
    Operator(Box<OperatorDocument>),
    Runtime(Box<RuntimeDocument>),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ClusterConfig {
    pub name: String,
    pub scope: String,
    pub member_id: String,
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresPathsConfig {
    pub data_dir: PathBuf,
    pub socket_dir: Option<PathBuf>,
    pub log_file: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresNetworkConfig {
    pub listen_host: String,
    pub listen_port: u16,
    pub advertise_port: Option<u16>,
}

impl Default for PostgresNetworkConfig {
    fn default() -> Self {
        Self {
            listen_host: DEFAULT_POSTGRES_LISTEN_HOST.to_string(),
            listen_port: DEFAULT_POSTGRES_LISTEN_PORT,
            advertise_port: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum RoleAuthConfig {
    Password { password: SecretSource },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresRoleConfig {
    pub username: String,
    pub auth: RoleAuthConfig,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MandatoryPostgresRolesConfig {
    pub superuser: PostgresRoleConfig,
    pub replicator: PostgresRoleConfig,
    pub rewinder: PostgresRoleConfig,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresRolesConfig {
    pub mandatory: MandatoryPostgresRolesConfig,
    #[serde(default)]
    pub extra: BTreeMap<String, toml::Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresAccessConfig {
    pub hba: PathOrInline,
    pub ident: PathOrInline,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DcsConfig {
    pub endpoints: Vec<String>,
    #[serde(default)]
    pub client: DcsClientConfig,
    pub init: Option<toml::Value>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DcsClientConfig {
    #[serde(default)]
    pub auth: DcsAuthConfig,
    #[serde(default)]
    pub tls: DcsTlsConfig,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum DcsAuthConfig {
    #[default]
    Disabled,
    Basic {
        username: String,
        password: SecretSource,
    },
}

#[derive(Clone, Debug, Default, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BinaryResolutionConfig {
    #[serde(default)]
    pub overrides: BinaryPathOverrides,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct BinaryPathOverrides {
    pub postgres: Option<PathBuf>,
    pub pg_ctl: Option<PathBuf>,
    pub pg_rewind: Option<PathBuf>,
    pub initdb: Option<PathBuf>,
    pub pg_basebackup: Option<PathBuf>,
    pub psql: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LoggingSinksConfig {
    #[serde(default)]
    pub stderr: StderrSinkConfig,
    #[serde(default)]
    pub file: FileSinkConfig,
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum FileSinkMode {
    #[default]
    Append,
    Truncate,
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TokenAuthConfig {
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub read_token: Option<SecretSource>,
    pub admin_token: Option<SecretSource>,
    pub tokens: Option<RoleTokens>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RoleTokens {
    pub read_token: Option<SecretSource>,
    pub admin_token: Option<SecretSource>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum PgtmApiTransportExpectation {
    Http,
    Https,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OperatorDocument {
    #[serde(default)]
    pub api: OperatorApiConfig,
    #[serde(default)]
    pub postgres: OperatorPostgresConfig,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OperatorApiConfig {
    pub base_url: Option<String>,
    pub advertised_url: Option<String>,
    pub expected_transport: Option<PgtmApiTransportExpectation>,
    pub resolve_to: Option<SocketAddr>,
    #[serde(default)]
    pub auth: TokenAuthConfig,
    #[serde(default)]
    pub tls: OperatorClientTlsConfig,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OperatorPostgresConfig {
    #[serde(default)]
    pub tls: OperatorClientTlsConfig,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OperatorClientTlsConfig {
    pub ca_cert: Option<PathSource>,
    pub identity: Option<TlsClientIdentityConfig>,
}

#[derive(Clone, Debug, Deserialize)]
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
