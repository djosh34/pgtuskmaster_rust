use crate::pginfo::conninfo::PgClientTls;
use crate::state::{ApiRoute, ClusterName, MemberId, PgRoute, ScopeName};
use reqwest::Url;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigErrorV2 {
    #[error("failed to read config file {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse config file {path}: {source}")]
    Parse {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("invalid config field `{field}`: {message}")]
    Validation {
        field: &'static str,
        message: String,
    },
}

#[derive(Clone, Debug)]
pub struct RuntimeConfigV2 {
    pub(crate) cluster_name: ClusterName,
    pub(crate) scope: ScopeName,
    pub(crate) member_id: MemberId,
    pub(crate) postgres: PostgresConfig,
    pub(crate) dcs: DcsConfig,
    pub(crate) ha: HaConfig,
    pub(crate) process: ProcessConfig,
    pub(crate) logging: LoggingConfig,
    pub(crate) api: ApiConfig,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TlsConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
    pub ca_cert: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Secret(String);

impl Secret {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

const DEFAULT_HA_LOOP_INTERVAL_MS: u64 = 1_000;
const DEFAULT_HA_LEASE_TTL_MS: u64 = 10_000;
const DEFAULT_PG_REWIND_TIMEOUT_MS: u64 = 120_000;
const DEFAULT_BOOTSTRAP_TIMEOUT_MS: u64 = 300_000;
const DEFAULT_FENCING_TIMEOUT_MS: u64 = 30_000;
pub(crate) const DEFAULT_RUNTIME_WORKING_ROOT: &str = "/tmp/pgtuskmaster";
const DEFAULT_LOGGING_CAPTURE_SUBPROCESS_OUTPUT: bool = true;
const DEFAULT_LOGGING_POSTGRES_ENABLED: bool = true;
const DEFAULT_LOGGING_POSTGRES_POLL_INTERVAL_MS: u64 = 200;
const DEFAULT_LOGGING_CLEANUP_ENABLED: bool = true;
const DEFAULT_LOGGING_CLEANUP_MAX_FILES: u64 = 50;
const DEFAULT_LOGGING_CLEANUP_MAX_AGE_SECONDS: u64 = 7 * 24 * 60 * 60;
const DEFAULT_LOGGING_CLEANUP_PROTECT_RECENT_SECONDS: u64 = 300;
const DEFAULT_LOGGING_SINK_STDERR_ENABLED: bool = true;
const DEFAULT_LOGGING_SINK_FILE_ENABLED: bool = false;

pub(crate) fn default_runtime_working_root() -> PathBuf {
    PathBuf::from(DEFAULT_RUNTIME_WORKING_ROOT)
}

fn default_ha_loop_interval() -> Duration {
    Duration::from_millis(DEFAULT_HA_LOOP_INTERVAL_MS)
}

fn default_ha_lease_ttl() -> Duration {
    Duration::from_millis(DEFAULT_HA_LEASE_TTL_MS)
}

fn default_pg_rewind_timeout() -> Duration {
    Duration::from_millis(DEFAULT_PG_REWIND_TIMEOUT_MS)
}

fn default_bootstrap_timeout() -> Duration {
    Duration::from_millis(DEFAULT_BOOTSTRAP_TIMEOUT_MS)
}

fn default_fencing_timeout() -> Duration {
    Duration::from_millis(DEFAULT_FENCING_TIMEOUT_MS)
}

fn default_logging_postgres_poll_interval() -> Duration {
    Duration::from_millis(DEFAULT_LOGGING_POSTGRES_POLL_INTERVAL_MS)
}

fn default_logging_cleanup_max_age() -> Duration {
    Duration::from_secs(DEFAULT_LOGGING_CLEANUP_MAX_AGE_SECONDS)
}

fn default_logging_cleanup_protect_recent() -> Duration {
    Duration::from_secs(DEFAULT_LOGGING_CLEANUP_PROTECT_RECENT_SECONDS)
}

fn deserialize_u64_default_if_zero<'de, D>(deserializer: D, default: u64) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    u64::deserialize(deserializer).map(|value| if value == 0 { default } else { value })
}

fn deserialize_duration_millis_default_if_zero<'de, D>(
    deserializer: D,
    default_millis: u64,
) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_u64_default_if_zero(deserializer, default_millis).map(Duration::from_millis)
}

fn deserialize_duration_secs_default_if_zero<'de, D>(
    deserializer: D,
    default_seconds: u64,
) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_u64_default_if_zero(deserializer, default_seconds).map(Duration::from_secs)
}

pub(crate) fn deserialize_ha_loop_interval<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_duration_millis_default_if_zero(deserializer, DEFAULT_HA_LOOP_INTERVAL_MS)
}

pub(crate) fn deserialize_ha_lease_ttl<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_duration_millis_default_if_zero(deserializer, DEFAULT_HA_LEASE_TTL_MS)
}

pub(crate) fn deserialize_pg_rewind_timeout<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_duration_millis_default_if_zero(deserializer, DEFAULT_PG_REWIND_TIMEOUT_MS)
}

pub(crate) fn deserialize_bootstrap_timeout<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_duration_millis_default_if_zero(deserializer, DEFAULT_BOOTSTRAP_TIMEOUT_MS)
}

pub(crate) fn deserialize_fencing_timeout<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_duration_millis_default_if_zero(deserializer, DEFAULT_FENCING_TIMEOUT_MS)
}

pub(crate) fn deserialize_logging_postgres_poll_interval<'de, D>(
    deserializer: D,
) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_duration_millis_default_if_zero(
        deserializer,
        DEFAULT_LOGGING_POSTGRES_POLL_INTERVAL_MS,
    )
}

pub(crate) fn deserialize_logging_cleanup_max_files<'de, D>(
    deserializer: D,
) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_u64_default_if_zero(deserializer, DEFAULT_LOGGING_CLEANUP_MAX_FILES)
}

pub(crate) fn deserialize_logging_cleanup_max_age<'de, D>(
    deserializer: D,
) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_duration_secs_default_if_zero(deserializer, DEFAULT_LOGGING_CLEANUP_MAX_AGE_SECONDS)
}

pub(crate) fn deserialize_logging_cleanup_protect_recent<'de, D>(
    deserializer: D,
) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_duration_secs_default_if_zero(
        deserializer,
        DEFAULT_LOGGING_CLEANUP_PROTECT_RECENT_SECONDS,
    )
}

// === POSTGRES CONFIG ===

#[derive(Clone, Debug)]
pub(crate) struct PostgresConfig {
    pub data_dir: PathBuf,
    pub socket_dir: PathBuf,
    pub log_file: PathBuf,
    pub listen_host: String,
    pub listen_port: u16,
    pub cluster_advertise: PgRoute,
    pub operator_advertise: Option<PgRoute>,
    pub connect_timeout: Duration,
    pub local_database: String,
    pub source_client_tls: PgClientTls,
    pub superuser: RoleConfig,
    pub replicator: RoleConfig,
    pub rewinder: RoleConfig,
    pub pg_hba_file: PathBuf,
    pub pg_ident_file: PathBuf,
    pub pg_hba_contents: String,
    pub pg_ident_contents: String,
    pub extra_gucs: BTreeMap<String, String>,
    pub tls: Option<TlsConfig>,
}

#[derive(Clone, Debug)]
pub(crate) struct RoleConfig {
    pub username: String,
    pub password: Secret,
}

// === DCS CONFIG ===

#[derive(Clone, Debug)]
pub(crate) struct DcsConfig {
    pub endpoints: Vec<DcsEndpoint>,
    pub auth: Option<DcsAuth>,
    pub tls: Option<TlsConfig>,
}

#[derive(Clone, Debug)]
pub(crate) struct DcsEndpoint(String);

impl DcsEndpoint {
    pub fn new(url: String) -> Self {
        Self(url)
    }
}

impl std::fmt::Display for DcsEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DcsAuth {
    pub username: String,
    pub password: Secret,
}

// === HA/PROCESS CONFIG ===

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HaConfig {
    #[serde(
        rename = "loop_interval_ms",
        default = "default_ha_loop_interval",
        deserialize_with = "deserialize_ha_loop_interval"
    )]
    pub loop_interval: Duration,
    #[serde(
        rename = "lease_ttl_ms",
        default = "default_ha_lease_ttl",
        deserialize_with = "deserialize_ha_lease_ttl"
    )]
    pub lease_ttl: Duration,
}

impl Default for HaConfig {
    fn default() -> Self {
        Self {
            loop_interval: default_ha_loop_interval(),
            lease_ttl: default_ha_lease_ttl(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ProcessConfig {
    pub timeouts: ProcessTimeoutsConfig,
    pub working_root: PathBuf,
    pub binaries: ProcessBinariesConfig,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            timeouts: ProcessTimeoutsConfig::default(),
            working_root: default_runtime_working_root(),
            binaries: ProcessBinariesConfig::default(),
        }
    }
}

impl<'de> Deserialize<'de> for ProcessConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Default, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct BinaryResolutionConfig {
            #[serde(default)]
            overrides: ProcessBinariesConfig,
        }

        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct ProcessConfigInput {
            #[serde(default)]
            timeouts: ProcessTimeoutsConfig,
            #[serde(default = "default_runtime_working_root")]
            working_root: PathBuf,
            #[serde(default)]
            binaries: BinaryResolutionConfig,
        }

        let ProcessConfigInput {
            timeouts,
            working_root,
            binaries,
        } = ProcessConfigInput::deserialize(deserializer)?;
        Ok(Self {
            timeouts,
            working_root,
            binaries: binaries.overrides,
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProcessTimeoutsConfig {
    #[serde(
        rename = "pg_rewind_ms",
        default = "default_pg_rewind_timeout",
        deserialize_with = "deserialize_pg_rewind_timeout"
    )]
    pub pg_rewind: Duration,
    #[serde(
        rename = "bootstrap_ms",
        default = "default_bootstrap_timeout",
        deserialize_with = "deserialize_bootstrap_timeout"
    )]
    pub bootstrap: Duration,
    #[serde(
        rename = "fencing_ms",
        default = "default_fencing_timeout",
        deserialize_with = "deserialize_fencing_timeout"
    )]
    pub fencing: Duration,
}

impl Default for ProcessTimeoutsConfig {
    fn default() -> Self {
        Self {
            pg_rewind: default_pg_rewind_timeout(),
            bootstrap: default_bootstrap_timeout(),
            fencing: default_fencing_timeout(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProcessBinariesConfig {
    #[serde(default)]
    pub pg_ctl: PathBuf,
    #[serde(default)]
    pub initdb: PathBuf,
    #[serde(default)]
    pub pg_rewind: PathBuf,
    #[serde(default)]
    pub pg_basebackup: PathBuf,
}

// === LOGGING CONFIG ===

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LoggingConfig {
    #[serde(default)]
    pub level: LogLevel,
    #[serde(default = "default_logging_capture_subprocess_output")]
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
            capture_subprocess_output: default_logging_capture_subprocess_output(),
            postgres: PostgresLoggingConfig::default(),
            sinks: LoggingSinksConfig::default(),
        }
    }
}

fn default_logging_capture_subprocess_output() -> bool {
    DEFAULT_LOGGING_CAPTURE_SUBPROCESS_OUTPUT
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PostgresLoggingConfig {
    #[serde(default = "default_logging_postgres_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub log_dir: PathBuf,
    #[serde(
        rename = "poll_interval_ms",
        default = "default_logging_postgres_poll_interval",
        deserialize_with = "deserialize_logging_postgres_poll_interval"
    )]
    pub poll_interval: Duration,
    #[serde(default)]
    pub cleanup: LogCleanupConfig,
}

impl Default for PostgresLoggingConfig {
    fn default() -> Self {
        Self {
            enabled: default_logging_postgres_enabled(),
            log_dir: PathBuf::new(),
            poll_interval: default_logging_postgres_poll_interval(),
            cleanup: LogCleanupConfig::default(),
        }
    }
}

fn default_logging_postgres_enabled() -> bool {
    DEFAULT_LOGGING_POSTGRES_ENABLED
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LoggingSinksConfig {
    #[serde(default)]
    pub stderr: StderrSinkConfig,
    #[serde(default)]
    pub file: FileSinkConfig,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StderrSinkConfig {
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
pub(crate) struct FileSinkConfig {
    pub enabled: bool,
    #[serde(default)]
    pub path: PathBuf,
    #[serde(default)]
    pub mode: FileSinkMode,
}

impl Default for FileSinkConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_LOGGING_SINK_FILE_ENABLED,
            path: PathBuf::new(),
            mode: FileSinkMode::Append,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LogCleanupConfig {
    pub enabled: bool,
    #[serde(
        default = "default_logging_cleanup_max_files",
        deserialize_with = "deserialize_logging_cleanup_max_files"
    )]
    pub max_files: u64,
    #[serde(
        rename = "max_age_seconds",
        default = "default_logging_cleanup_max_age",
        deserialize_with = "deserialize_logging_cleanup_max_age"
    )]
    pub max_age: Duration,
    #[serde(
        rename = "protect_recent_seconds",
        default = "default_logging_cleanup_protect_recent",
        deserialize_with = "deserialize_logging_cleanup_protect_recent"
    )]
    pub protect_recent: Duration,
}

impl Default for LogCleanupConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_LOGGING_CLEANUP_ENABLED,
            max_files: default_logging_cleanup_max_files(),
            max_age: default_logging_cleanup_max_age(),
            protect_recent: default_logging_cleanup_protect_recent(),
        }
    }
}

fn default_logging_cleanup_max_files() -> u64 {
    DEFAULT_LOGGING_CLEANUP_MAX_FILES
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum FileSinkMode {
    #[default]
    Append,
    Truncate,
}

// === API CONFIG ===

#[derive(Clone, Debug)]
pub(crate) struct ApiConfig {
    pub listen_addr: SocketAddr,
    pub transport: ApiTransport,
    pub auth: ApiAuth,
    pub advertise: Option<ApiRoute>,
}

#[derive(Clone, Debug)]
pub(crate) enum ApiTransport {
    Http,
    Https {
        tls: TlsConfig,
        client_ca: Option<PathBuf>,
        client_cert_required: bool,
        allowed_client_common_names: Vec<String>,
    },
}

#[derive(Clone, Debug)]
pub(crate) enum ApiAuth {
    Disabled,
    Tokens {
        read_token: Secret,
        admin_token: Secret,
    },
}

// === OPERATOR CONFIG ===

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PgtmApiTransportExpectation {
    Http,
    Https,
}

impl PgtmApiTransportExpectation {
    pub fn scheme(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
        }
    }

    pub fn matches_url(self, url: &Url) -> bool {
        url.scheme() == self.scheme()
    }
}

#[derive(Clone, Debug)]
pub struct OperatorConfigV2 {
    pub(crate) base_url: Option<Url>,
    pub(crate) advertised_url: Option<ApiRoute>,
    pub(crate) expected_transport: Option<PgtmApiTransportExpectation>,
    pub(crate) resolve_to: Option<SocketAddr>,
    pub(crate) client_tls: Option<PgClientTls>,
    pub(crate) read_token: Option<Secret>,
    pub(crate) admin_token: Option<Secret>,
}

impl OperatorConfigV2 {
    pub fn api_auth_enabled(&self) -> bool {
        self.read_token.is_some() || self.admin_token.is_some()
    }
}

impl RuntimeConfigV2 {
    pub(crate) fn startup_directories(&self) -> impl Iterator<Item = (&'static str, &Path)> {
        [
            Some(("postgres data dir", self.postgres.data_dir.as_path())),
            Some(("postgres socket dir", self.postgres.socket_dir.as_path())),
            self.postgres
                .log_file
                .parent()
                .map(|path| ("postgres log dir", path)),
        ]
        .into_iter()
        .flatten()
    }
}
