use std::path::PathBuf;

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct RuntimeConfig {
    pub cluster: ClusterConfig,
    pub postgres: PostgresConfig,
    pub dcs: DcsConfig,
    pub ha: HaConfig,
    pub process: ProcessConfig,
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
pub struct BinaryPaths {
    pub postgres: PathBuf,
    pub pg_ctl: PathBuf,
    pub pg_rewind: PathBuf,
    pub initdb: PathBuf,
    pub psql: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiConfig {
    pub listen_addr: String,
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
    pub api: Option<PartialApiConfig>,
    pub debug: Option<PartialDebugConfig>,
    pub security: Option<PartialSecurityConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialPostgresConfig {
    pub data_dir: PathBuf,
    pub connect_timeout_s: Option<u32>,
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
pub struct PartialApiConfig {
    pub listen_addr: Option<String>,
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
