pub(crate) mod defaults;
pub(crate) mod endpoint;
pub(crate) mod materialize;
pub(crate) mod parser;
pub(crate) mod schema;

pub use endpoint::{DcsEndpoint, DcsEndpointError, DcsEndpointScheme};
pub use materialize::{
    resolve_inline_or_path_bytes, resolve_inline_or_path_string, resolve_secret_string,
    ConfigMaterializeError,
};
pub use parser::{
    load_operator_config, load_runtime_config, validate_operator_config, validate_runtime_config,
    ConfigError,
};
pub use schema::{
    ApiAuthConfig, ApiClientAuthConfig, ApiConfig, ApiRoleTokensConfig, ApiTlsConfig,
    ApiTransportConfig, BinaryPathOverrides, BinaryResolutionConfig, ClientCertificateMode,
    ClientCommonName, ClusterConfig, DcsAuthConfig, DcsClientConfig, DcsConfig, DcsInitConfig,
    DcsTlsConfig, DebugConfig, ExtraManagedPostgresRoleConfig, FileSinkConfig, FileSinkMode,
    HaConfig, InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig, LoggingSinksConfig,
    ManagedPostgresRoleKey, MandatoryPostgresRolesConfig, PgtmApiAuthConfig, PgtmApiConfig,
    PgtmApiTransportExpectation, PgtmClientTlsConfig, PgtmConfig, PgtmPostgresConfig,
    PgtmPrimaryTargetConfig, PostgresAccessConfig, PostgresBinaryName,
    PostgresClientTransportConfig, PostgresConfig, PostgresLoggingConfig, PostgresNetworkConfig,
    PostgresPathsConfig, PostgresRewindConfig, PostgresRoleConfig, PostgresRoleName,
    PostgresRolePrivilege, PostgresRolesConfig, ProcessConfig, ProcessTimeoutsConfig,
    RoleAuthConfig, RuntimeConfig, SecretSource, StderrSinkConfig, TlsClientAuthConfig,
    TlsClientIdentityConfig, TlsServerConfig, TlsServerIdentityConfig,
};
