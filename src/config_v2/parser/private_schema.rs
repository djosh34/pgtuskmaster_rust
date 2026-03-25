use std::{
    collections::BTreeMap,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Deserializer};

use crate::{
    config_v2::types::{
        ApiTransport, ConfigErrorV2, HaConfig, LoggingConfig, PgtmApiTransportExpectation,
        ProcessConfig, Secret, TlsConfig,
    },
    pginfo::conninfo::{PgClientTls, PgSslMode},
};

const DEFAULT_POSTGRES_DATABASE: &str = "postgres";
const DEFAULT_POSTGRES_CONNECT_TIMEOUT_S: u32 = 5;
const DEFAULT_POSTGRES_LISTEN_HOST: &str = "127.0.0.1";
const DEFAULT_POSTGRES_LISTEN_PORT: u16 = 5432;
const DEFAULT_DEBUG_ENABLED: bool = false;

fn default_api_listen_addr() -> SocketAddr {
    SocketAddr::from((std::net::Ipv4Addr::new(127, 0, 0, 1), 8080))
}

fn default_postgres_database() -> String {
    DEFAULT_POSTGRES_DATABASE.to_string()
}

fn default_postgres_connect_timeout_s() -> u32 {
    DEFAULT_POSTGRES_CONNECT_TIMEOUT_S
}

fn default_pg_ssl_mode() -> PgSslMode {
    PgSslMode::Prefer
}

macro_rules! deserialize_default_if_zero {
    ($name:ident, $ty:ty, $default:expr) => {
        fn $name<'de, D>(deserializer: D) -> Result<$ty, D::Error>
        where
            D: Deserializer<'de>,
        {
            <$ty>::deserialize(deserializer).map(|value| if value == 0 { $default } else { value })
        }
    };
}

deserialize_default_if_zero!(
    deserialize_postgres_listen_port,
    u16,
    DEFAULT_POSTGRES_LISTEN_PORT
);
deserialize_default_if_zero!(
    deserialize_postgres_connect_timeout_s,
    u32,
    DEFAULT_POSTGRES_CONNECT_TIMEOUT_S
);

fn deserialize_postgres_listen_host<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer).map(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            DEFAULT_POSTGRES_LISTEN_HOST.to_string()
        } else {
            trimmed.to_string()
        }
    })
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum PathOrInline {
    Path(PathBuf),
    PathConfig { path: PathBuf },
    Inline { content: String },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub(super) enum PathSource {
    Path(PathBuf),
    PathConfig { path: PathBuf },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum SecretSource {
    Tagged(TaggedSecretSource),
    PathConfig { path: PathBuf },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub(super) enum TaggedSecretSource {
    None,
    Env { env: String },
    File { path: PathBuf },
    String { value: String },
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ClientCertificateMode {
    Optional,
    Required,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TlsServerIdentityConfig {
    pub cert_chain: PathSource,
    pub private_key: PathSource,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct TlsClientIdentityConfig {
    pub cert: PathSource,
    pub key: PathSource,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TlsClientAuthConfig {
    pub client_ca: PathSource,
    pub client_certificate: ClientCertificateMode,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub(super) enum TlsServerConfig {
    #[default]
    Disabled,
    Enabled {
        identity: TlsServerIdentityConfig,
        client_auth: Option<TlsClientAuthConfig>,
    },
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(tag = "client_certificate", rename_all = "snake_case")]
pub(super) enum ApiClientAuthConfig {
    #[default]
    Disabled,
    Optional {
        client_ca: PathSource,
    },
    Required {
        client_ca: PathSource,
        #[serde(default)]
        allowed_common_names: Vec<String>,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ApiTlsConfig {
    pub identity: TlsServerIdentityConfig,
    #[serde(default)]
    pub client_auth: ApiClientAuthConfig,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(tag = "transport", rename_all = "snake_case")]
pub(super) enum ApiTransportConfig {
    #[default]
    Http,
    Https {
        tls: ApiTlsConfig,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RuntimeDocument {
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
    #[serde(default)]
    pub pgtm: Option<OperatorDocument>,
    #[serde(default)]
    pub debug: DebugConfig,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ClusterConfig {
    pub name: String,
    pub scope: String,
    pub member_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresConfig {
    pub paths: PostgresPathsConfig,
    #[serde(default)]
    pub network: PostgresNetworkConfig,
    #[serde(
        default = "default_postgres_connect_timeout_s",
        deserialize_with = "deserialize_postgres_connect_timeout_s"
    )]
    pub connect_timeout_s: u32,
    #[serde(default = "default_postgres_database")]
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

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresPathsConfig {
    pub data_dir: PathBuf,
    pub socket_dir: Option<PathBuf>,
    pub log_file: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresNetworkConfig {
    #[serde(deserialize_with = "deserialize_postgres_listen_host")]
    pub listen_host: String,
    #[serde(deserialize_with = "deserialize_postgres_listen_port")]
    pub listen_port: u16,
    pub cluster_advertise: Option<PostgresAdvertiseConfig>,
    pub operator_advertise: Option<PostgresAdvertiseConfig>,
}

impl Default for PostgresNetworkConfig {
    fn default() -> Self {
        Self {
            listen_host: DEFAULT_POSTGRES_LISTEN_HOST.to_string(),
            listen_port: DEFAULT_POSTGRES_LISTEN_PORT,
            cluster_advertise: None,
            operator_advertise: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresAdvertiseConfig {
    pub host: String,
    pub port: u16,
    pub hostaddr: Option<IpAddr>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresRewindConfig {
    #[serde(default = "default_postgres_database")]
    pub database: String,
    #[serde(default)]
    pub transport: PostgresClientTransportConfig,
}

impl Default for PostgresRewindConfig {
    fn default() -> Self {
        Self {
            database: DEFAULT_POSTGRES_DATABASE.to_string(),
            transport: PostgresClientTransportConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresClientTransportConfig {
    #[serde(default = "default_pg_ssl_mode")]
    pub ssl_mode: PgSslMode,
    pub ca_cert: Option<PathSource>,
}

impl Default for PostgresClientTransportConfig {
    fn default() -> Self {
        Self {
            ssl_mode: PgSslMode::Prefer,
            ca_cert: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum RoleAuthConfig {
    Password { password: SecretSource },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresRoleConfig {
    pub username: String,
    pub auth: RoleAuthConfig,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MandatoryPostgresRolesConfig {
    pub superuser: PostgresRoleConfig,
    pub replicator: PostgresRoleConfig,
    pub rewinder: PostgresRoleConfig,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresRolesConfig {
    pub mandatory: MandatoryPostgresRolesConfig,
    #[serde(default)]
    pub extra: BTreeMap<String, toml::Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PostgresAccessConfig {
    pub hba: PathOrInline,
    pub ident: PathOrInline,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DcsConfig {
    pub endpoints: Vec<String>,
    #[serde(default)]
    pub client: DcsClientConfig,
    pub init: Option<toml::Value>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DcsClientConfig {
    #[serde(default)]
    pub auth: DcsAuthConfig,
    #[serde(default)]
    pub tls: DcsTlsConfig,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(super) enum DcsAuthConfig {
    #[default]
    Disabled,
    Basic {
        username: String,
        password: SecretSource,
    },
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub(super) enum DcsTlsConfig {
    #[default]
    Disabled,
    Enabled {
        #[serde(flatten)]
        tls: ClientTlsInput,
        server_name: Option<String>,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ApiConfig {
    #[serde(default = "default_api_listen_addr")]
    pub listen_addr: SocketAddr,
    #[serde(default)]
    pub transport: ApiTransportConfig,
    #[serde(default)]
    pub auth: TokenAuthConfig,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            listen_addr: default_api_listen_addr(),
            transport: ApiTransportConfig::default(),
            auth: TokenAuthConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TokenAuthConfig {
    #[serde(rename = "type")]
    pub(super) kind: Option<String>,
    pub(super) read_token: Option<SecretSource>,
    pub(super) admin_token: Option<SecretSource>,
    pub(super) tokens: Option<RoleTokens>,
}

impl TokenAuthConfig {
    pub(super) fn is_disabled(&self) -> bool {
        match self.kind.as_deref() {
            Some("disabled") => true,
            Some(_) => false,
            None => {
                self.read_token.is_none() && self.admin_token.is_none() && self.tokens.is_none()
            }
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RoleTokens {
    pub read_token: Option<SecretSource>,
    pub admin_token: Option<SecretSource>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct ClientTlsInput {
    pub(super) ca_cert: Option<PathSource>,
    pub(super) identity: Option<TlsClientIdentityConfig>,
}

impl PathSource {
    pub(super) fn resolve(
        self,
        field: &'static str,
        config_dir: &Path,
    ) -> Result<PathBuf, ConfigErrorV2> {
        match self {
            Self::Path(path) | Self::PathConfig { path } => {
                normalize_config_path(field, path, config_dir)
            }
        }
    }

    pub(super) fn resolve_optional(
        field: &'static str,
        source: Option<Self>,
        config_dir: &Path,
    ) -> Result<Option<PathBuf>, ConfigErrorV2> {
        source
            .map(|source| source.resolve(field, config_dir))
            .transpose()
    }

    fn resolve_pair(
        cert_field: &'static str,
        cert: Self,
        key_field: &'static str,
        key: Self,
        config_dir: &Path,
    ) -> Result<(PathBuf, PathBuf), ConfigErrorV2> {
        Ok((
            cert.resolve(cert_field, config_dir)?,
            key.resolve(key_field, config_dir)?,
        ))
    }
}

pub(super) fn validation_error(field: &'static str, message: impl Into<String>) -> ConfigErrorV2 {
    ConfigErrorV2::Validation {
        field,
        message: message.into(),
    }
}

pub(super) fn normalize_config_path(
    field: &'static str,
    path: PathBuf,
    config_dir: &Path,
) -> Result<PathBuf, ConfigErrorV2> {
    if path.as_os_str().is_empty() {
        return Err(validation_error(field, "must not be empty"));
    }
    Ok(if path.is_absolute() {
        path
    } else {
        config_dir.join(path)
    })
}

fn read_config_string(
    field: &'static str,
    path: PathBuf,
    config_dir: &Path,
) -> Result<String, ConfigErrorV2> {
    let path = normalize_config_path(field, path, config_dir)?;
    std::fs::read_to_string(&path).map_err(|source| ConfigErrorV2::Io {
        path: path.display().to_string(),
        source,
    })
}

impl PathOrInline {
    pub(super) fn resolve_contents(
        self,
        field: &'static str,
        config_dir: &Path,
    ) -> Result<String, ConfigErrorV2> {
        match self {
            Self::Path(path) | Self::PathConfig { path } => {
                read_config_string(field, path, config_dir)
            }
            Self::Inline { content } => Ok(content),
        }
    }
}

impl SecretSource {
    pub(super) fn resolve_optional(
        field: &'static str,
        source: Option<Self>,
        config_dir: &Path,
    ) -> Result<Option<Secret>, ConfigErrorV2> {
        source
            .map(|source| source.resolve_required(field, config_dir))
            .transpose()
    }

    pub(super) fn resolve_required(
        self,
        field: &'static str,
        config_dir: &Path,
    ) -> Result<Secret, ConfigErrorV2> {
        let value = match self {
            Self::PathConfig { path } => read_config_string(field, path, config_dir)?,
            Self::Tagged(TaggedSecretSource::None) => String::new(),
            Self::Tagged(TaggedSecretSource::Env { env }) => {
                std::env::var(&env).map_err(|err| {
                    validation_error(
                        field,
                        match err {
                            std::env::VarError::NotPresent => {
                                format!("environment variable `{env}` is not set")
                            }
                            std::env::VarError::NotUnicode(_) => {
                                format!("environment variable `{env}` is not valid unicode")
                            }
                        },
                    )
                })?
            }
            Self::Tagged(TaggedSecretSource::File { path }) => {
                read_config_string(field, path, config_dir)?
            }
            Self::Tagged(TaggedSecretSource::String { value }) => value,
        };

        let value = value.trim_end_matches(['\n', '\r']).to_string();
        if value.trim().is_empty() {
            return Err(validation_error(field, "must not be empty"));
        }
        Ok(Secret::new(value))
    }
}

impl TlsServerIdentityConfig {
    pub(super) fn resolve(
        self,
        cert_field: &'static str,
        key_field: &'static str,
        config_dir: &Path,
    ) -> Result<(PathBuf, PathBuf), ConfigErrorV2> {
        PathSource::resolve_pair(
            cert_field,
            self.cert_chain,
            key_field,
            self.private_key,
            config_dir,
        )
    }

    fn into_runtime_tls(
        self,
        cert_field: &'static str,
        key_field: &'static str,
        ca_cert: Option<PathBuf>,
        config_dir: &Path,
    ) -> Result<TlsConfig, ConfigErrorV2> {
        let (cert, key) = self.resolve(cert_field, key_field, config_dir)?;
        Ok(TlsConfig { cert, key, ca_cert })
    }
}

impl TlsClientIdentityConfig {
    pub(super) fn resolve(
        self,
        cert_field: &'static str,
        key_field: &'static str,
        config_dir: &Path,
    ) -> Result<(PathBuf, PathBuf), ConfigErrorV2> {
        PathSource::resolve_pair(cert_field, self.cert, key_field, self.key, config_dir)
    }
}

impl TlsServerConfig {
    pub(super) fn into_runtime_tls(
        self,
        config_dir: &Path,
    ) -> Result<Option<TlsConfig>, ConfigErrorV2> {
        match self {
            Self::Disabled => Ok(None),
            Self::Enabled {
                identity,
                client_auth,
            } => Ok(Some(identity.into_runtime_tls(
                "postgres.tls.identity.cert_chain",
                "postgres.tls.identity.private_key",
                PathSource::resolve_optional(
                    "postgres.tls.client_auth.client_ca",
                    client_auth.map(|client_auth| {
                        let _client_certificate_mode = client_auth.client_certificate;
                        client_auth.client_ca
                    }),
                    config_dir,
                )?,
                config_dir,
            )?)),
        }
    }
}

impl ClientTlsInput {
    pub(super) fn into_runtime_dcs_tls(
        self,
        config_dir: &Path,
    ) -> Result<TlsConfig, ConfigErrorV2> {
        let Some(identity) = self.identity else {
            return Err(validation_error(
                "dcs.client.tls.identity",
                "enabled DCS TLS currently requires a client identity",
            ));
        };
        let (cert, key) = identity.resolve(
            "dcs.client.tls.identity.cert",
            "dcs.client.tls.identity.key",
            config_dir,
        )?;
        Ok(TlsConfig {
            cert,
            key,
            ca_cert: PathSource::resolve_optional(
                "dcs.client.tls.ca_cert",
                self.ca_cert,
                config_dir,
            )?,
        })
    }

    pub(super) fn into_pg_client_tls(
        self,
        ca_field: &'static str,
        cert_field: &'static str,
        key_field: &'static str,
        config_dir: &Path,
    ) -> Result<Option<PgClientTls>, ConfigErrorV2> {
        let root_cert = PathSource::resolve_optional(ca_field, self.ca_cert, config_dir)?;
        let (client_cert, client_key) = self
            .identity
            .map(|identity| identity.resolve(cert_field, key_field, config_dir))
            .transpose()?
            .unzip();

        Ok(
            (root_cert.is_some() || client_cert.is_some() || client_key.is_some()).then_some(
                PgClientTls {
                    mode: PgSslMode::VerifyFull,
                    root_cert,
                    client_cert,
                    client_key,
                },
            ),
        )
    }
}

impl DcsTlsConfig {
    pub(super) fn into_runtime_tls(
        self,
        config_dir: &Path,
    ) -> Result<Option<TlsConfig>, ConfigErrorV2> {
        match self {
            Self::Disabled => Ok(None),
            Self::Enabled { tls, server_name } => {
                if server_name.is_some() {
                    return Err(validation_error(
                        "dcs.client.tls.server_name",
                        "is not supported by config_v2",
                    ));
                }
                tls.into_runtime_dcs_tls(config_dir).map(Some)
            }
        }
    }
}

impl ApiClientAuthConfig {
    fn into_runtime_client_auth(
        self,
        config_dir: &Path,
    ) -> Result<(Option<PathBuf>, bool, Vec<String>), ConfigErrorV2> {
        let Some((client_ca, client_cert_required, allowed_client_common_names)) = (match self {
            Self::Disabled => None,
            Self::Optional { client_ca } => Some((client_ca, false, Vec::new())),
            Self::Required {
                client_ca,
                allowed_common_names,
            } => Some((client_ca, true, allowed_common_names)),
        }) else {
            return Ok((None, false, Vec::new()));
        };
        Ok((
            Some(client_ca.resolve("api.transport.tls.client_auth.client_ca", config_dir)?),
            client_cert_required,
            allowed_client_common_names,
        ))
    }
}

impl ApiTransportConfig {
    pub(super) fn into_runtime_transport(
        self,
        config_dir: &Path,
    ) -> Result<ApiTransport, ConfigErrorV2> {
        match self {
            Self::Http => Ok(ApiTransport::Http),
            Self::Https { tls } => {
                let (client_ca, client_cert_required, allowed_client_common_names) =
                    tls.client_auth.into_runtime_client_auth(config_dir)?;
                Ok(ApiTransport::Https {
                    tls: tls.identity.into_runtime_tls(
                        "api.transport.tls.identity.cert_chain",
                        "api.transport.tls.identity.private_key",
                        None,
                        config_dir,
                    )?,
                    client_ca,
                    client_cert_required,
                    allowed_client_common_names,
                })
            }
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OperatorDocument {
    #[serde(default)]
    pub api: OperatorApiConfig,
    #[serde(default)]
    pub postgres: OperatorPostgresConfig,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OperatorApiConfig {
    pub base_url: Option<String>,
    pub advertised_url: Option<String>,
    pub expected_transport: Option<PgtmApiTransportExpectation>,
    pub resolve_to: Option<SocketAddr>,
    #[serde(default)]
    pub auth: TokenAuthConfig,
    #[serde(default)]
    pub tls: ClientTlsInput,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OperatorPostgresConfig {
    #[serde(default)]
    pub tls: ClientTlsInput,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DebugConfig {
    pub enabled: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_DEBUG_ENABLED,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PathOrInline, PathSource, SecretSource, TaggedSecretSource, TlsServerIdentityConfig,
    };
    use crate::{config_v2::ConfigErrorV2, dev_support::test_fs::unique_test_dir};
    use std::{fs, path::PathBuf};

    #[test]
    fn raw_source_owners_resolve_paths_contents_and_tls_pairs() -> Result<(), String> {
        let root = unique_test_dir("private-schema", "source-owner-resolution")?;
        let config_dir = root.join("config");
        fs::create_dir_all(config_dir.as_path()).map_err(|err| err.to_string())?;
        fs::write(
            config_dir.join("pg_hba.conf"),
            "host all all 0.0.0.0/0 md5\n",
        )
        .map_err(|err| err.to_string())?;

        assert_eq!(
            PathSource::PathConfig {
                path: PathBuf::from("tls/server.crt"),
            }
            .resolve("tls.cert", config_dir.as_path())
            .map_err(|err| err.to_string())?,
            config_dir.join("tls/server.crt")
        );
        assert_eq!(
            PathOrInline::PathConfig {
                path: PathBuf::from("pg_hba.conf"),
            }
            .resolve_contents("postgres.access.hba", config_dir.as_path())
            .map_err(|err| err.to_string())?,
            "host all all 0.0.0.0/0 md5\n"
        );
        assert_eq!(
            PathOrInline::Inline {
                content: "local all all trust".to_string(),
            }
            .resolve_contents("postgres.access.ident", config_dir.as_path())
            .map_err(|err| err.to_string())?,
            "local all all trust"
        );
        assert_eq!(
            TlsServerIdentityConfig {
                cert_chain: PathSource::PathConfig {
                    path: PathBuf::from("server.crt"),
                },
                private_key: PathSource::PathConfig {
                    path: PathBuf::from("server.key"),
                },
            }
            .resolve(
                "postgres.tls.identity.cert_chain",
                "postgres.tls.identity.private_key",
                config_dir.as_path(),
            )
            .map_err(|err| err.to_string())?,
            (config_dir.join("server.crt"), config_dir.join("server.key"))
        );
        Ok(())
    }

    #[test]
    fn secret_source_owners_trim_files_and_report_missing_env() -> Result<(), String> {
        let root = unique_test_dir("private-schema", "secret-source-resolution")?;
        let config_dir = root.join("config");
        fs::create_dir_all(config_dir.as_path()).map_err(|err| err.to_string())?;
        fs::write(config_dir.join("secret.txt"), "from-file\r\n").map_err(|err| err.to_string())?;

        assert_eq!(
            SecretSource::Tagged(TaggedSecretSource::File {
                path: PathBuf::from("secret.txt"),
            })
            .resolve_required("api.auth.read_token", config_dir.as_path())
            .map_err(|err| err.to_string())?
            .as_str(),
            "from-file"
        );
        assert_eq!(
            SecretSource::Tagged(TaggedSecretSource::String {
                value: "inline-token".to_string(),
            })
            .resolve_required("api.auth.admin_token", config_dir.as_path())
            .map_err(|err| err.to_string())?
            .as_str(),
            "inline-token"
        );
        assert_eq!(
            SecretSource::resolve_optional("api.auth.optional", None, config_dir.as_path())
                .map_err(|err| err.to_string())?,
            None
        );
        match SecretSource::Tagged(TaggedSecretSource::Env {
            env: "PGTM_PRIVATE_SCHEMA_MISSING_ENV".to_string(),
        })
        .resolve_required("api.auth.read_token", config_dir.as_path())
        {
            Ok(secret) => {
                return Err(format!(
                    "expected missing env error, got `{}`",
                    secret.as_str()
                ));
            }
            Err(ConfigErrorV2::Validation { field, message }) => {
                assert_eq!(field, "api.auth.read_token");
                assert_eq!(
                    message,
                    "environment variable `PGTM_PRIVATE_SCHEMA_MISSING_ENV` is not set"
                );
            }
            Err(other) => return Err(format!("expected validation error, got {other}")),
        }
        Ok(())
    }
}
