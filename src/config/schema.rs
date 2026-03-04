use std::{fmt, path::PathBuf};

use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigVersion {
    V1,
    V2,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum InlineOrPath {
    Path(PathBuf),
    PathConfig { path: PathBuf },
    Inline { content: String },
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
pub struct SecretSource(pub InlineOrPath);

impl fmt::Debug for SecretSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            InlineOrPath::Path(path) => f
                .debug_tuple("SecretSource")
                .field(&format_args!("path({})", path.display()))
                .finish(),
            InlineOrPath::PathConfig { path } => f
                .debug_tuple("SecretSource")
                .field(&format_args!("path({})", path.display()))
                .finish(),
            InlineOrPath::Inline { .. } => f
                .debug_tuple("SecretSource")
                .field(&"<inline redacted>")
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
    pub backup: BackupConfig,
    pub logging: LoggingConfig,
    pub api: ApiConfig,
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
    pub socket_dir: PathBuf,
    pub log_file: PathBuf,
    pub rewind_source_host: String,
    pub rewind_source_port: u16,
    pub local_conn_identity: PostgresConnIdentityConfig,
    pub rewind_conn_identity: PostgresConnIdentityConfig,
    pub tls: TlsServerConfig,
    pub roles: PostgresRolesConfig,
    pub pg_hba: PgHbaConfig,
    pub pg_ident: PgIdentConfig,
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
    pub backup_timeout_ms: u64,
    pub binaries: BinaryPaths,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackupConfig {
    pub enabled: bool,
    pub provider: BackupProvider,
    pub bootstrap: BackupBootstrapConfig,
    pub pgbackrest: Option<PgBackRestConfig>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BackupProvider {
    #[default]
    Pgbackrest,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackupBootstrapConfig {
    pub enabled: bool,
    pub takeover_policy: BackupTakeoverPolicy,
    pub recovery_mode: BackupRecoveryMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BackupTakeoverPolicy {
    #[default]
    Quarantine,
    Delete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BackupRecoveryMode {
    #[default]
    Default,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PgBackRestConfig {
    pub stanza: Option<String>,
    pub repo: Option<String>,
    pub options: BackupOptions,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct BackupOptions {
    pub backup: Vec<String>,
    pub info: Vec<String>,
    pub check: Vec<String>,
    pub restore: Vec<String>,
    pub archive_push: Vec<String>,
    pub archive_get: Vec<String>,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: BackupProvider::default(),
            bootstrap: BackupBootstrapConfig {
                enabled: false,
                takeover_policy: BackupTakeoverPolicy::default(),
                recovery_mode: BackupRecoveryMode::default(),
            },
            pgbackrest: Some(PgBackRestConfig::default()),
        }
    }
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
    pub archive_command_log_file: Option<PathBuf>,
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
    pub pgbackrest: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiConfig {
    pub listen_addr: String,
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
    pub read_token: Option<String>,
    pub admin_token: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebugConfig {
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialRuntimeConfig {
    pub cluster: ClusterConfig,
    pub postgres: PartialPostgresConfig,
    pub dcs: DcsConfig,
    pub ha: HaConfig,
    pub process: PartialProcessConfig,
    pub logging: Option<PartialLoggingConfig>,
    pub api: Option<PartialApiConfig>,
    pub debug: Option<PartialDebugConfig>,
    pub security: Option<PartialSecurityConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialPostgresConfig {
    pub data_dir: PathBuf,
    pub connect_timeout_s: Option<u32>,
    pub listen_host: Option<String>,
    pub listen_port: Option<u16>,
    pub socket_dir: Option<PathBuf>,
    pub log_file: Option<PathBuf>,
    pub rewind_source_host: Option<String>,
    pub rewind_source_port: Option<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialProcessConfig {
    pub pg_rewind_timeout_ms: Option<u64>,
    pub bootstrap_timeout_ms: Option<u64>,
    pub fencing_timeout_ms: Option<u64>,
    pub binaries: BinaryPaths,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialLoggingConfig {
    pub level: Option<LogLevel>,
    pub capture_subprocess_output: Option<bool>,
    pub postgres: Option<PartialPostgresLoggingConfig>,
    pub sinks: Option<PartialLoggingSinksConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialPostgresLoggingConfig {
    pub enabled: Option<bool>,
    pub pg_ctl_log_file: Option<PathBuf>,
    pub log_dir: Option<PathBuf>,
    pub archive_command_log_file: Option<PathBuf>,
    pub poll_interval_ms: Option<u64>,
    pub cleanup: Option<PartialLogCleanupConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialLogCleanupConfig {
    pub enabled: Option<bool>,
    pub max_files: Option<u64>,
    pub max_age_seconds: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialLoggingSinksConfig {
    pub stderr: Option<PartialStderrSinkConfig>,
    pub file: Option<PartialFileSinkConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialStderrSinkConfig {
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialFileSinkConfig {
    pub enabled: Option<bool>,
    pub path: Option<PathBuf>,
    pub mode: Option<FileSinkMode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialApiConfig {
    pub listen_addr: Option<String>,
    pub read_auth_token: Option<String>,
    pub admin_auth_token: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialDebugConfig {
    pub enabled: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialSecurityConfig {
    pub tls_enabled: Option<bool>,
    pub auth_token: Option<String>,
}

// -------------------------------
// v2 input schema (explicit secure)
// -------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfigV2Input {
    pub config_version: ConfigVersion,
    pub cluster: ClusterConfig,
    pub postgres: PostgresConfigV2Input,
    pub dcs: DcsConfig,
    pub ha: HaConfig,
    pub process: ProcessConfigV2Input,
    pub backup: Option<BackupConfigV2Input>,
    pub logging: Option<LoggingConfig>,
    pub api: ApiConfigV2Input,
    pub debug: Option<DebugConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessConfigV2Input {
    pub pg_rewind_timeout_ms: Option<u64>,
    pub bootstrap_timeout_ms: Option<u64>,
    pub fencing_timeout_ms: Option<u64>,
    pub backup_timeout_ms: Option<u64>,
    pub binaries: Option<BinaryPathsV2Input>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BinaryPathsV2Input {
    pub postgres: Option<PathBuf>,
    pub pg_ctl: Option<PathBuf>,
    pub pg_rewind: Option<PathBuf>,
    pub initdb: Option<PathBuf>,
    pub pg_basebackup: Option<PathBuf>,
    pub psql: Option<PathBuf>,
    pub pgbackrest: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackupConfigV2Input {
    pub enabled: Option<bool>,
    pub provider: Option<BackupProvider>,
    pub bootstrap: Option<BackupBootstrapConfigV2Input>,
    pub pgbackrest: Option<PgBackRestConfigV2Input>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackupBootstrapConfigV2Input {
    pub enabled: Option<bool>,
    pub takeover_policy: Option<BackupTakeoverPolicy>,
    pub recovery_mode: Option<BackupRecoveryMode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgBackRestConfigV2Input {
    pub stanza: Option<String>,
    pub repo: Option<String>,
    pub options: Option<BackupOptionsV2Input>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackupOptionsV2Input {
    pub backup: Option<Vec<String>>,
    pub info: Option<Vec<String>>,
    pub check: Option<Vec<String>>,
    pub restore: Option<Vec<String>>,
    pub archive_push: Option<Vec<String>>,
    pub archive_get: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiConfigV2Input {
    pub listen_addr: Option<String>,
    pub security: Option<ApiSecurityConfigV2Input>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiSecurityConfigV2Input {
    pub tls: Option<TlsServerConfigV2Input>,
    pub auth: Option<ApiAuthConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConfigV2Input {
    pub data_dir: PathBuf,
    pub connect_timeout_s: Option<u32>,
    pub listen_host: String,
    pub listen_port: u16,
    pub socket_dir: PathBuf,
    pub log_file: PathBuf,
    pub rewind_source_host: String,
    pub rewind_source_port: u16,
    pub local_conn_identity: Option<PostgresConnIdentityConfigV2Input>,
    pub rewind_conn_identity: Option<PostgresConnIdentityConfigV2Input>,
    pub tls: Option<TlsServerConfigV2Input>,
    pub roles: Option<PostgresRolesConfigV2Input>,
    pub pg_hba: Option<PgHbaConfigV2Input>,
    pub pg_ident: Option<PgIdentConfigV2Input>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConnIdentityConfigV2Input {
    pub user: Option<String>,
    pub dbname: Option<String>,
    pub ssl_mode: Option<crate::pginfo::conninfo::PgSslMode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRoleConfigV2Input {
    pub username: Option<String>,
    pub auth: Option<RoleAuthConfigV2Input>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRolesConfigV2Input {
    pub superuser: Option<PostgresRoleConfigV2Input>,
    pub replicator: Option<PostgresRoleConfigV2Input>,
    pub rewinder: Option<PostgresRoleConfigV2Input>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgHbaConfigV2Input {
    pub source: Option<InlineOrPath>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgIdentConfigV2Input {
    pub source: Option<InlineOrPath>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsServerIdentityConfigV2Input {
    pub cert_chain: Option<InlineOrPath>,
    pub private_key: Option<InlineOrPath>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RoleAuthConfigV2Input {
    Tls,
    Password { password: Option<SecretSource> },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsServerConfigV2Input {
    pub mode: Option<TlsMode>,
    pub identity: Option<TlsServerIdentityConfigV2Input>,
    pub client_auth: Option<TlsClientAuthConfig>,
}
