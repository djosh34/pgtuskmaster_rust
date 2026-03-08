pub(crate) mod defaults;
pub(crate) mod endpoint;
pub(crate) mod parser;
pub(crate) mod schema;

pub use endpoint::{DcsEndpoint, DcsEndpointError};
pub use parser::{load_runtime_config, validate_runtime_config, ConfigError};
pub use schema::{
    ApiAuthConfig, ApiConfig, ApiRoleTokensConfig, ApiSecurityConfig, ApiTlsMode, BinaryPaths,
    ClusterConfig, DcsConfig, DcsConfigInput, DcsInitConfig, DebugConfig, FileSinkConfig,
    FileSinkMode, HaConfig, InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig,
    LoggingSinksConfig, PgHbaConfig, PgIdentConfig, PostgresConfig,
    PostgresConnIdentityConfig, PostgresLoggingConfig, PostgresRoleConfig, PostgresRolesConfig,
    ProcessConfig, RoleAuthConfig, RuntimeConfig, RuntimeConfigInput, SecretSource,
    StderrSinkConfig, TlsClientAuthConfig, TlsServerConfig, TlsServerIdentityConfig,
};
