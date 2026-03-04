use std::path::PathBuf;

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct RuntimeConfig {
    pub cluster: ClusterConfig,
    pub postgres: PostgresConfig,
    pub dcs: DcsConfig,
    pub ha: HaConfig,
    pub process: ProcessConfig,
    pub logging: LoggingConfig,
    pub api: ApiConfig,
    pub debug: DebugConfig,
    pub security: SecurityConfig,
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
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DcsConfig {
    pub endpoints: Vec<String>,
    pub scope: String,
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
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiConfig {
    pub listen_addr: String,
    pub read_auth_token: Option<String>,
    pub admin_auth_token: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebugConfig {
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityConfig {
    pub tls_enabled: bool,
    pub auth_token: Option<String>,
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
