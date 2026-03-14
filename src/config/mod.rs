pub(crate) mod defaults;
pub(crate) mod endpoint;
pub(crate) mod materialize;
pub(crate) mod parser;
pub(crate) mod schema;

pub use endpoint::{DcsEndpoint, DcsEndpointError};
pub use materialize::{
    resolve_inline_or_path_bytes, resolve_inline_or_path_string, resolve_secret_string,
    ConfigMaterializeError,
};
pub use parser::{load_runtime_config, validate_runtime_config, ConfigError};
pub use schema::{
    ApiAuthConfig, ApiClientAuthConfig, ApiClientAuthConfigInput, ApiConfig, ApiRoleTokensConfig,
    ApiSecurityConfig, ApiTlsConfig, ApiTlsConfigInput, ApiTransportConfig,
    ApiTransportConfigInput, BinaryPaths, ClientCertificateMode, ClientCommonName, ClusterConfig,
    DcsConfig, DcsConfigInput, DcsInitConfig, DebugConfig, FileSinkConfig, FileSinkMode,
    HaConfig, InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig, LoggingSinksConfig,
    PgHbaConfig, PgIdentConfig, PgtmApiClientConfig, PgtmApiClientConfigInput, PgtmConfig,
    PgtmConfigInput, PgtmPostgresClientConfig, PgtmPostgresClientConfigInput, PostgresConfig,
    PostgresConnIdentityConfig, PostgresLoggingConfig, PostgresRoleConfig, PostgresRolesConfig,
    ProcessConfig, RoleAuthConfig, RuntimeConfig, RuntimeConfigInput, SecretSource,
    ServerTlsMode, StderrSinkConfig, TlsClientAuthConfig, TlsClientAuthConfigInput,
    TlsServerConfig, TlsServerConfigInput, TlsServerIdentityConfig,
};
