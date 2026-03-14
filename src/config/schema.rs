use std::{
    collections::BTreeMap,
    fmt,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::state::{ClusterName, MemberId, NonEmptyStringError, ScopeName};

use super::{defaults, endpoint::DcsEndpoint};

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
#[serde(rename_all = "snake_case")]
pub enum ClientCertificateMode {
    Optional,
    Required,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(try_from = "String")]
pub struct ClientCommonName(pub(crate) String);

impl ClientCommonName {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<String> for ClientCommonName {
    type Error = NonEmptyStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        require_non_empty("client_common_name", value.as_str())?;
        Ok(Self(value))
    }
}

impl TryFrom<&str> for ClientCommonName {
    type Error = NonEmptyStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        require_non_empty("client_common_name", value)?;
        Ok(Self(value.to_string()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsServerIdentityConfig {
    pub cert_chain: InlineOrPath,
    pub private_key: InlineOrPath,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsClientIdentityConfig {
    pub cert: InlineOrPath,
    pub key: SecretSource,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsClientAuthConfig {
    pub client_ca: InlineOrPath,
    pub client_certificate: ClientCertificateMode,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum TlsServerConfig {
    #[default]
    Disabled,
    Enabled {
        identity: TlsServerIdentityConfig,
        client_auth: Option<TlsClientAuthConfig>,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(tag = "client_certificate", rename_all = "snake_case")]
pub enum ApiClientAuthConfig {
    #[default]
    Disabled,
    Optional {
        client_ca: InlineOrPath,
    },
    Required {
        client_ca: InlineOrPath,
        #[serde(default)]
        allowed_common_names: Vec<ClientCommonName>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiTlsConfig {
    pub identity: TlsServerIdentityConfig,
    #[serde(default)]
    pub client_auth: ApiClientAuthConfig,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(tag = "transport", rename_all = "snake_case")]
pub enum ApiTransportConfig {
    #[default]
    Http,
    Https {
        tls: ApiTlsConfig,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfig {
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
    pub pgtm: Option<PgtmConfig>,
    #[serde(default)]
    pub debug: DebugConfig,
}

impl RuntimeConfig {
    pub fn postgres_socket_dir(&self) -> PathBuf {
        self.postgres
            .paths
            .socket_dir
            .clone()
            .unwrap_or_else(|| self.process.working_root.join("socket"))
    }

    pub fn postgres_log_file(&self) -> PathBuf {
        self.postgres
            .paths
            .log_file
            .clone()
            .unwrap_or_else(|| self.process.working_root.join("logs/postgres.log"))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClusterConfig {
    pub name: ClusterName,
    pub scope: ScopeName,
    pub member_id: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresConfig {
    pub paths: PostgresPathsConfig,
    #[serde(default)]
    pub network: PostgresNetworkConfig,
    #[serde(default = "defaults::default_postgres_connect_timeout_s")]
    pub connect_timeout_s: u32,
    #[serde(default = "defaults::default_postgres_database")]
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

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresPathsConfig {
    pub data_dir: PathBuf,
    pub socket_dir: Option<PathBuf>,
    pub log_file: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresNetworkConfig {
    #[serde(default = "defaults::default_postgres_listen_host")]
    pub listen_host: String,
    #[serde(default = "defaults::default_postgres_listen_port")]
    pub listen_port: u16,
    pub advertise_port: Option<u16>,
}

impl Default for PostgresNetworkConfig {
    fn default() -> Self {
        Self {
            listen_host: defaults::default_postgres_listen_host(),
            listen_port: defaults::default_postgres_listen_port(),
            advertise_port: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRewindConfig {
    #[serde(default = "defaults::default_postgres_database")]
    pub database: String,
    #[serde(default)]
    pub transport: PostgresClientTransportConfig,
}

impl Default for PostgresRewindConfig {
    fn default() -> Self {
        Self {
            database: defaults::default_postgres_database(),
            transport: PostgresClientTransportConfig::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresClientTransportConfig {
    #[serde(default = "defaults::default_pg_ssl_mode")]
    pub ssl_mode: crate::pginfo::conninfo::PgSslMode,
    pub ca_cert: Option<InlineOrPath>,
}

impl Default for PostgresClientTransportConfig {
    fn default() -> Self {
        Self {
            ssl_mode: defaults::default_pg_ssl_mode(),
            ca_cert: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RoleAuthConfig {
    Password { password: SecretSource },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(try_from = "String")]
pub struct PostgresRoleName(pub(crate) String);

impl PostgresRoleName {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<String> for PostgresRoleName {
    type Error = NonEmptyStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        require_non_empty("postgres_role_name", value.as_str())?;
        Ok(Self(value))
    }
}

impl TryFrom<&str> for PostgresRoleName {
    type Error = NonEmptyStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        require_non_empty("postgres_role_name", value)?;
        Ok(Self(value.to_string()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(try_from = "String")]
pub struct ManagedPostgresRoleKey(pub(crate) String);

impl ManagedPostgresRoleKey {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<String> for ManagedPostgresRoleKey {
    type Error = NonEmptyStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        require_non_empty("managed_postgres_role_key", value.as_str())?;
        Ok(Self(value))
    }
}

impl TryFrom<&str> for ManagedPostgresRoleKey {
    type Error = NonEmptyStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        require_non_empty("managed_postgres_role_key", value)?;
        Ok(Self(value.to_string()))
    }
}

fn require_non_empty(
    label: &'static str,
    value: &str,
) -> Result<(), NonEmptyStringError> {
    if value.trim().is_empty() {
        return Err(NonEmptyStringError::Empty { label });
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostgresRolePrivilege {
    Login,
    Replication,
    Superuser,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRoleConfig {
    pub username: PostgresRoleName,
    pub auth: RoleAuthConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MandatoryPostgresRolesConfig {
    pub superuser: PostgresRoleConfig,
    pub replicator: PostgresRoleConfig,
    pub rewinder: PostgresRoleConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExtraManagedPostgresRoleConfig {
    #[serde(flatten)]
    pub role: PostgresRoleConfig,
    #[serde(default = "default_extra_managed_postgres_role_privilege")]
    pub privilege: PostgresRolePrivilege,
    #[serde(default)]
    pub member_of: Vec<PostgresRoleName>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRolesConfig {
    pub mandatory: MandatoryPostgresRolesConfig,
    #[serde(default)]
    pub extra: BTreeMap<ManagedPostgresRoleKey, ExtraManagedPostgresRoleConfig>,
}

const fn default_extra_managed_postgres_role_privilege() -> PostgresRolePrivilege {
    PostgresRolePrivilege::Login
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresAccessConfig {
    pub hba: InlineOrPath,
    pub ident: InlineOrPath,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DcsConfig {
    pub endpoints: Vec<DcsEndpoint>,
    #[serde(default)]
    pub client: DcsClientConfig,
    pub init: Option<DcsInitConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DcsClientConfig {
    #[serde(default)]
    pub auth: DcsAuthConfig,
    #[serde(default)]
    pub tls: DcsTlsConfig,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DcsAuthConfig {
    #[default]
    Disabled,
    Basic {
        username: String,
        password: SecretSource,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum DcsTlsConfig {
    #[default]
    Disabled,
    Enabled {
        ca_cert: Option<InlineOrPath>,
        identity: Option<TlsClientIdentityConfig>,
        server_name: Option<String>,
    },
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
    #[serde(default = "defaults::default_ha_loop_interval_ms")]
    pub loop_interval_ms: u64,
    #[serde(default = "defaults::default_ha_lease_ttl_ms")]
    pub lease_ttl_ms: u64,
}

impl Default for HaConfig {
    fn default() -> Self {
        Self {
            loop_interval_ms: defaults::default_ha_loop_interval_ms(),
            lease_ttl_ms: defaults::default_ha_lease_ttl_ms(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessConfig {
    #[serde(default)]
    pub timeouts: ProcessTimeoutsConfig,
    #[serde(default = "defaults::default_runtime_working_root")]
    pub working_root: PathBuf,
    #[serde(default)]
    pub binaries: BinaryResolutionConfig,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            timeouts: ProcessTimeoutsConfig::default(),
            working_root: defaults::default_runtime_working_root(),
            binaries: BinaryResolutionConfig::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessTimeoutsConfig {
    #[serde(default = "defaults::default_pg_rewind_timeout_ms")]
    pub pg_rewind_ms: u64,
    #[serde(default = "defaults::default_bootstrap_timeout_ms")]
    pub bootstrap_ms: u64,
    #[serde(default = "defaults::default_fencing_timeout_ms")]
    pub fencing_ms: u64,
}

impl Default for ProcessTimeoutsConfig {
    fn default() -> Self {
        Self {
            pg_rewind_ms: defaults::default_pg_rewind_timeout_ms(),
            bootstrap_ms: defaults::default_bootstrap_timeout_ms(),
            fencing_ms: defaults::default_fencing_timeout_ms(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct BinaryResolutionConfig {
    #[serde(default)]
    pub overrides: BinaryPathOverrides,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct BinaryPathOverrides {
    pub postgres: Option<PathBuf>,
    pub pg_ctl: Option<PathBuf>,
    pub pg_rewind: Option<PathBuf>,
    pub initdb: Option<PathBuf>,
    pub pg_basebackup: Option<PathBuf>,
    pub psql: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostgresBinaryName {
    Postgres,
    PgCtl,
    PgRewind,
    Initdb,
    PgBasebackup,
    Psql,
}

impl PostgresBinaryName {
    pub fn executable_name(self) -> &'static str {
        match self {
            Self::Postgres => "postgres",
            Self::PgCtl => "pg_ctl",
            Self::PgRewind => "pg_rewind",
            Self::Initdb => "initdb",
            Self::PgBasebackup => "pg_basebackup",
            Self::Psql => "psql",
        }
    }

    pub fn config_field(self) -> &'static str {
        match self {
            Self::Postgres => "process.binaries.overrides.postgres",
            Self::PgCtl => "process.binaries.overrides.pg_ctl",
            Self::PgRewind => "process.binaries.overrides.pg_rewind",
            Self::Initdb => "process.binaries.overrides.initdb",
            Self::PgBasebackup => "process.binaries.overrides.pg_basebackup",
            Self::Psql => "process.binaries.overrides.psql",
        }
    }

    fn override_path(self, overrides: &BinaryPathOverrides) -> Option<&PathBuf> {
        match self {
            Self::Postgres => overrides.postgres.as_ref(),
            Self::PgCtl => overrides.pg_ctl.as_ref(),
            Self::PgRewind => overrides.pg_rewind.as_ref(),
            Self::Initdb => overrides.initdb.as_ref(),
            Self::PgBasebackup => overrides.pg_basebackup.as_ref(),
            Self::Psql => overrides.psql.as_ref(),
        }
    }
}

impl BinaryResolutionConfig {
    pub fn resolve_binary_path(&self, binary: PostgresBinaryName) -> Result<PathBuf, String> {
        if let Some(path) = binary.override_path(&self.overrides) {
            if !path.is_file() {
                return Err(format!(
                    "`{}` points to a missing executable: {}",
                    binary.config_field(),
                    path.display()
                ));
            }
            return Ok(path.clone());
        }

        let executable = binary.executable_name();
        let mut searched = Vec::new();

        if let Some(path_env) = std::env::var_os("PATH") {
            for directory in std::env::split_paths(&path_env) {
                let candidate = directory.join(executable);
                if candidate.is_file() {
                    return Ok(candidate);
                }
                searched.push(candidate);
            }
        }

        for directory in conventional_postgres_bin_dirs() {
            let candidate = directory.join(executable);
            if candidate.is_file() {
                return Ok(candidate);
            }
            searched.push(candidate);
        }

        let preview = searched
            .iter()
            .take(6)
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let detail = if preview.is_empty() {
            "no candidate paths were discovered".to_string()
        } else {
            format!("searched {preview}")
        };

        Err(format!(
            "unable to resolve `{executable}` via PATH or conventional PostgreSQL install locations; {detail}; set `{}` explicitly if autodiscovery fails",
            binary.config_field()
        ))
    }
}

fn conventional_postgres_bin_dirs() -> Vec<PathBuf> {
    let mut directories = Vec::new();
    directories.extend(all_child_bin_dirs(Path::new("/usr/lib/postgresql")));
    directories.extend(prefixed_child_bin_dirs(Path::new("/usr"), "pgsql-"));
    directories.extend(prefixed_child_bin_dirs(
        Path::new("/opt/homebrew/opt"),
        "postgresql@",
    ));
    directories.extend(prefixed_child_bin_dirs(
        Path::new("/usr/local/opt"),
        "postgresql@",
    ));
    directories.push(PathBuf::from("/opt/homebrew/opt/libpq/bin"));
    directories.push(PathBuf::from("/usr/local/opt/libpq/bin"));
    directories
}

fn all_child_bin_dirs(root: &Path) -> Vec<PathBuf> {
    child_dirs_matching(root, |_| true)
        .into_iter()
        .map(|path| path.join("bin"))
        .collect()
}

fn prefixed_child_bin_dirs(root: &Path, prefix: &str) -> Vec<PathBuf> {
    child_dirs_matching(root, |name| name.starts_with(prefix))
        .into_iter()
        .map(|path| path.join("bin"))
        .collect()
}

fn child_dirs_matching<F>(root: &Path, predicate: F) -> Vec<PathBuf>
where
    F: Fn(&str) -> bool,
{
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };

    let mut directories = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let file_type = entry.file_type().ok()?;
            if !file_type.is_dir() {
                return None;
            }
            let name = entry.file_name();
            let name = name.to_str()?;
            predicate(name).then(|| entry.path())
        })
        .collect::<Vec<_>>();
    directories.sort();
    directories
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    #[serde(default)]
    pub level: LogLevel,
    #[serde(default = "defaults::default_logging_capture_subprocess_output")]
    pub capture_subprocess_output: bool,
    #[serde(default)]
    pub postgres: PostgresLoggingConfig,
    #[serde(default)]
    pub sinks: LoggingSinksConfig,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::default(),
            capture_subprocess_output: defaults::default_logging_capture_subprocess_output(),
            postgres: PostgresLoggingConfig::default(),
            sinks: LoggingSinksConfig::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresLoggingConfig {
    #[serde(default = "defaults::default_logging_postgres_enabled")]
    pub enabled: bool,
    pub pg_ctl_log_file: Option<PathBuf>,
    pub log_dir: Option<PathBuf>,
    #[serde(default = "defaults::default_logging_postgres_poll_interval_ms")]
    pub poll_interval_ms: u64,
    #[serde(default)]
    pub cleanup: LogCleanupConfig,
}

impl Default for PostgresLoggingConfig {
    fn default() -> Self {
        Self {
            enabled: defaults::default_logging_postgres_enabled(),
            pg_ctl_log_file: None,
            log_dir: None,
            poll_interval_ms: defaults::default_logging_postgres_poll_interval_ms(),
            cleanup: LogCleanupConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoggingSinksConfig {
    #[serde(default)]
    pub stderr: StderrSinkConfig,
    #[serde(default)]
    pub file: FileSinkConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StderrSinkConfig {
    #[serde(default = "defaults::default_logging_sink_stderr_enabled")]
    pub enabled: bool,
}

impl Default for StderrSinkConfig {
    fn default() -> Self {
        Self {
            enabled: defaults::default_logging_sink_stderr_enabled(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileSinkConfig {
    #[serde(default = "defaults::default_logging_sink_file_enabled")]
    pub enabled: bool,
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub mode: FileSinkMode,
}

impl Default for FileSinkConfig {
    fn default() -> Self {
        Self {
            enabled: defaults::default_logging_sink_file_enabled(),
            path: None,
            mode: FileSinkMode::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileSinkMode {
    #[default]
    Append,
    Truncate,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogCleanupConfig {
    #[serde(default = "defaults::default_logging_cleanup_enabled")]
    pub enabled: bool,
    #[serde(default = "defaults::default_logging_cleanup_max_files")]
    pub max_files: u64,
    #[serde(default = "defaults::default_logging_cleanup_max_age_seconds")]
    pub max_age_seconds: u64,
    #[serde(default = "defaults::default_logging_cleanup_protect_recent_seconds")]
    pub protect_recent_seconds: u64,
}

impl Default for LogCleanupConfig {
    fn default() -> Self {
        Self {
            enabled: defaults::default_logging_cleanup_enabled(),
            max_files: defaults::default_logging_cleanup_max_files(),
            max_age_seconds: defaults::default_logging_cleanup_max_age_seconds(),
            protect_recent_seconds: defaults::default_logging_cleanup_protect_recent_seconds(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiConfig {
    #[serde(default = "defaults::default_api_listen_addr")]
    pub listen_addr: SocketAddr,
    #[serde(default)]
    pub transport: ApiTransportConfig,
    #[serde(default)]
    pub auth: ApiAuthConfig,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            listen_addr: defaults::default_api_listen_addr(),
            transport: ApiTransportConfig::default(),
            auth: ApiAuthConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ApiAuthConfig {
    #[default]
    Disabled,
    RoleTokens(ApiRoleTokensConfig),
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ApiRoleTokensConfig {
    pub read_token: Option<SecretSource>,
    pub admin_token: Option<SecretSource>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PgtmApiTransportExpectation {
    Http,
    Https,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PgtmConfig {
    #[serde(default)]
    pub api: PgtmApiConfig,
    #[serde(default)]
    pub postgres: PgtmPostgresConfig,
    pub primary_target: Option<PgtmPrimaryTargetConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PgtmApiConfig {
    pub base_url: Option<String>,
    pub advertised_url: Option<String>,
    pub expected_transport: Option<PgtmApiTransportExpectation>,
    #[serde(default)]
    pub auth: PgtmApiAuthConfig,
    #[serde(default)]
    pub tls: PgtmClientTlsConfig,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PgtmApiAuthConfig {
    #[default]
    Disabled,
    RoleTokens {
        read_token: Option<SecretSource>,
        admin_token: Option<SecretSource>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PgtmPostgresConfig {
    #[serde(default)]
    pub tls: PgtmClientTlsConfig,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct PgtmClientTlsConfig {
    pub ca_cert: Option<InlineOrPath>,
    pub identity: Option<TlsClientIdentityConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgtmPrimaryTargetConfig {
    pub host: String,
    pub port: Option<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebugConfig {
    #[serde(default = "defaults::default_debug_enabled")]
    pub enabled: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enabled: defaults::default_debug_enabled(),
        }
    }
}
