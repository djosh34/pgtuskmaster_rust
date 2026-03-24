use crate::pginfo::conninfo::PgClientTls;
use crate::state::{ClusterName, MemberId, ScopeName};
use reqwest::Url;
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::PathBuf;
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
pub(crate) struct RuntimeConfigV2 {
    pub cluster_name: ClusterName,
    pub scope: ScopeName,
    pub member_id: MemberId,
    pub postgres: PostgresConfig,
    pub dcs: DcsConfig,
    pub timing: TimingConfig,
    pub binaries: BinariesConfig,
    pub logging: LoggingConfig,
    pub api: ApiConfig,
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

// === POSTGRES CONFIG ===

#[derive(Clone, Debug)]
pub(crate) struct PostgresConfig {
    pub data_dir: PathBuf,
    pub socket_dir: PathBuf,
    pub log_file: PathBuf,
    pub listen_host: String,
    pub listen_port: u16,
    pub advertise_port: u16,
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

// === TIMING CONFIG ===

#[derive(Clone, Debug)]
pub(crate) struct TimingConfig {
    pub ha_loop_interval: Duration,
    pub ha_lease_ttl: Duration,
    pub bootstrap_timeout: Duration,
    pub pg_rewind_timeout: Duration,
    pub fencing_timeout: Duration,
}

// === BINARIES CONFIG ===

#[derive(Clone, Debug)]
pub(crate) struct BinariesConfig {
    pub postgres: PathBuf,
    pub pg_ctl: PathBuf,
    pub initdb: PathBuf,
    pub pg_rewind: PathBuf,
    pub pg_basebackup: PathBuf,
    pub psql: PathBuf,
}

// === LOGGING CONFIG ===

#[derive(Clone, Debug)]
pub(crate) struct LoggingConfig {
    pub level: LogLevel,
    pub capture_subprocess_output: bool,
    pub stderr_enabled: bool,
    pub file_enabled: bool,
    pub file_path: PathBuf,
    pub file_mode: FileSinkMode,
    pub postgres_logs_enabled: bool,
    pub postgres_log_dir: PathBuf,
    pub postgres_pg_ctl_log: PathBuf,
    pub postgres_log_poll_interval: Duration,
    pub postgres_log_cleanup_enabled: bool,
    pub postgres_log_cleanup_max_files: u64,
    pub postgres_log_cleanup_max_age: Duration,
    pub postgres_log_cleanup_protect_recent: Duration,
}

#[derive(Clone, Debug)]
pub(crate) enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Clone, Debug)]
pub(crate) enum FileSinkMode {
    Append,
    Truncate,
}

// === API CONFIG ===

#[derive(Clone, Debug)]
pub(crate) struct ApiConfig {
    pub listen_addr: SocketAddr,
    pub transport: ApiTransport,
    pub auth: ApiAuth,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PgtmApiTransportExpectation {
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
pub(crate) struct OperatorConfigV2 {
    pub base_url: Option<Url>,
    pub expected_transport: Option<PgtmApiTransportExpectation>,
    pub resolve_to: Option<SocketAddr>,
    pub client_tls: OperatorClientTlsConfig,
    pub read_token: Option<Secret>,
    pub admin_token: Option<Secret>,
}

impl OperatorConfigV2 {
    pub fn api_auth_enabled(&self) -> bool {
        self.read_token.is_some() || self.admin_token.is_some()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct OperatorClientTlsConfig {
    pub ca_cert: Option<PathBuf>,
    pub identity: Option<TlsConfig>,
}
