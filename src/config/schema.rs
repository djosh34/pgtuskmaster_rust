use std::{collections::BTreeMap, fmt, net::SocketAddr, path::PathBuf};

use serde::Deserialize;

use super::endpoint::DcsEndpoint;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum InlineOrPath {
    Path(PathBuf),
    PathConfig { path: PathBuf },
    Inline { content: String },
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum SecretSource {
    Path(PathBuf),
    PathConfig { path: PathBuf },
    Inline { content: String },
    Env { env: String },
}

impl fmt::Debug for SecretSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(path) => f
                .debug_tuple("SecretSource")
                .field(&format_args!("path({})", path.display()))
                .finish(),
            Self::PathConfig { path } => f
                .debug_tuple("SecretSource")
                .field(&format_args!("path({})", path.display()))
                .finish(),
            Self::Inline { .. } => f
                .debug_tuple("SecretSource")
                .field(&"<inline redacted>")
                .finish(),
            Self::Env { env } => f
                .debug_tuple("SecretSource")
                .field(&format_args!("env({env})"))
                .finish(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiTlsMode {
    Disabled,
    Optional,
    Required,
}

pub type TlsMode = ApiTlsMode;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsServerIdentityConfig {
    pub cert_chain: InlineOrPath,
    pub private_key: InlineOrPath,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsClientAuthConfig {
    pub client_ca: InlineOrPath,
    pub require_client_cert: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsServerConfig {
    pub mode: TlsMode,
    pub identity: Option<TlsServerIdentityConfig>,
    pub client_auth: Option<TlsClientAuthConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct RuntimeConfig {
    pub cluster: ClusterConfig,
    pub postgres: PostgresConfig,
    pub dcs: DcsConfig,
    pub ha: HaConfig,
    pub process: ProcessConfig,
    pub logging: LoggingConfig,
    pub api: ApiConfig,
    pub pgtm: Option<PgtmConfig>,
    pub debug: DebugConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClusterConfig {
    pub name: String,
    pub member_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConfig {
    pub data_dir: PathBuf,
    pub connect_timeout_s: u32,
    pub listen_host: String,
    pub listen_port: u16,
    pub advertise_port: Option<u16>,
    pub socket_dir: PathBuf,
    pub log_file: PathBuf,
    pub local_conn_identity: PostgresConnIdentityConfig,
    pub rewind_conn_identity: PostgresConnIdentityConfig,
    pub tls: TlsServerConfig,
    pub roles: PostgresRolesConfig,
    pub pg_hba: PgHbaConfig,
    pub pg_ident: PgIdentConfig,
    pub extra_gucs: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConnIdentityConfig {
    pub user: String,
    pub dbname: String,
    pub ssl_mode: crate::pginfo::conninfo::PgSslMode,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RoleAuthConfig {
    Tls,
    Password { password: SecretSource },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRoleConfig {
    pub username: String,
    pub auth: RoleAuthConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRolesConfig {
    pub superuser: PostgresRoleConfig,
    pub replicator: PostgresRoleConfig,
    pub rewinder: PostgresRoleConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgHbaConfig {
    pub source: InlineOrPath,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgIdentConfig {
    pub source: InlineOrPath,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DcsConfig {
    pub endpoints: Vec<DcsEndpoint>,
    pub scope: String,
    pub init: Option<DcsInitConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DcsConfigInput {
    pub endpoints: Vec<String>,
    pub scope: String,
    pub init: Option<DcsInitConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DcsInitConfig {
    pub payload_json: String,
    pub write_on_bootstrap: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HaConfig {
    pub loop_interval_ms: u64,
    pub lease_ttl_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessConfig {
    pub pg_rewind_timeout_ms: u64,
    pub bootstrap_timeout_ms: u64,
    pub fencing_timeout_ms: u64,
    pub binaries: BinaryPaths,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub capture_subprocess_output: bool,
    pub postgres: PostgresLoggingConfig,
    pub sinks: LoggingSinksConfig,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresLoggingConfig {
    pub enabled: bool,
    pub pg_ctl_log_file: Option<PathBuf>,
    pub log_dir: Option<PathBuf>,
    pub poll_interval_ms: u64,
    pub cleanup: LogCleanupConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoggingSinksConfig {
    pub stderr: StderrSinkConfig,
    pub file: FileSinkConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StderrSinkConfig {
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileSinkConfig {
    pub enabled: bool,
    pub path: Option<PathBuf>,
    pub mode: FileSinkMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileSinkMode {
    Append,
    Truncate,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogCleanupConfig {
    pub enabled: bool,
    pub max_files: u64,
    pub max_age_seconds: u64,
    #[serde(default = "default_log_cleanup_protect_recent_seconds")]
    pub protect_recent_seconds: u64,
}

fn default_log_cleanup_protect_recent_seconds() -> u64 {
    300
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BinaryPaths {
    pub postgres: PathBuf,
    pub pg_ctl: PathBuf,
    pub pg_rewind: PathBuf,
    pub initdb: PathBuf,
    pub pg_basebackup: PathBuf,
    pub psql: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiConfig {
    pub listen_addr: SocketAddr,
    pub security: ApiSecurityConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiSecurityConfig {
    pub tls: TlsServerConfig,
    pub auth: ApiAuthConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ApiAuthConfig {
    Disabled,
    RoleTokens(ApiRoleTokensConfig),
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiRoleTokensConfig {
    pub read_token: Option<SecretSource>,
    pub admin_token: Option<SecretSource>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgtmConfig {
    pub api_url: Option<String>,
    pub api_client: Option<PgtmApiClientConfig>,
    pub postgres_client: Option<PgtmPostgresClientConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgtmApiClientConfig {
    pub ca_cert: Option<InlineOrPath>,
    pub client_cert: Option<InlineOrPath>,
    pub client_key: Option<SecretSource>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgtmPostgresClientConfig {
    pub ca_cert: Option<InlineOrPath>,
    pub client_cert: Option<InlineOrPath>,
    pub client_key: Option<SecretSource>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebugConfig {
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfigInput {
    pub cluster: ClusterConfig,
    pub postgres: PostgresConfigInput,
    pub dcs: DcsConfigInput,
    pub ha: HaConfig,
    pub process: ProcessConfigInput,
    pub logging: Option<LoggingConfig>,
    pub api: ApiConfigInput,
    pub pgtm: Option<PgtmConfigInput>,
    pub debug: Option<DebugConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessConfigInput {
    pub pg_rewind_timeout_ms: Option<u64>,
    pub bootstrap_timeout_ms: Option<u64>,
    pub fencing_timeout_ms: Option<u64>,
    pub binaries: Option<BinaryPathsInput>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BinaryPathsInput {
    pub postgres: Option<PathBuf>,
    pub pg_ctl: Option<PathBuf>,
    pub pg_rewind: Option<PathBuf>,
    pub initdb: Option<PathBuf>,
    pub pg_basebackup: Option<PathBuf>,
    pub psql: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiConfigInput {
    pub listen_addr: Option<String>,
    pub security: Option<ApiSecurityConfigInput>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiSecurityConfigInput {
    pub tls: Option<TlsServerConfigInput>,
    pub auth: Option<ApiAuthConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgtmConfigInput {
    pub api_url: Option<String>,
    pub api_client: Option<PgtmApiClientConfigInput>,
    pub postgres_client: Option<PgtmPostgresClientConfigInput>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgtmApiClientConfigInput {
    pub ca_cert: Option<InlineOrPath>,
    pub client_cert: Option<InlineOrPath>,
    pub client_key: Option<SecretSource>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgtmPostgresClientConfigInput {
    pub ca_cert: Option<InlineOrPath>,
    pub client_cert: Option<InlineOrPath>,
    pub client_key: Option<SecretSource>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConfigInput {
    pub data_dir: PathBuf,
    pub connect_timeout_s: Option<u32>,
    pub listen_host: String,
    pub listen_port: u16,
    pub advertise_port: Option<u16>,
    pub socket_dir: PathBuf,
    pub log_file: PathBuf,
    pub local_conn_identity: Option<PostgresConnIdentityConfigInput>,
    pub rewind_conn_identity: Option<PostgresConnIdentityConfigInput>,
    pub tls: Option<TlsServerConfigInput>,
    pub roles: Option<PostgresRolesConfigInput>,
    pub pg_hba: Option<PgHbaConfigInput>,
    pub pg_ident: Option<PgIdentConfigInput>,
    pub extra_gucs: Option<BTreeMap<String, String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConnIdentityConfigInput {
    pub user: Option<String>,
    pub dbname: Option<String>,
    pub ssl_mode: Option<crate::pginfo::conninfo::PgSslMode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRoleConfigInput {
    pub username: Option<String>,
    pub auth: Option<RoleAuthConfigInput>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRolesConfigInput {
    pub superuser: Option<PostgresRoleConfigInput>,
    pub replicator: Option<PostgresRoleConfigInput>,
    pub rewinder: Option<PostgresRoleConfigInput>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgHbaConfigInput {
    pub source: Option<InlineOrPath>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgIdentConfigInput {
    pub source: Option<InlineOrPath>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsServerIdentityConfigInput {
    pub cert_chain: Option<InlineOrPath>,
    pub private_key: Option<InlineOrPath>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RoleAuthConfigInput {
    Tls,
    Password { password: Option<SecretSource> },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsServerConfigInput {
    pub mode: Option<TlsMode>,
    pub identity: Option<TlsServerIdentityConfigInput>,
    pub client_auth: Option<TlsClientAuthConfig>,
}
