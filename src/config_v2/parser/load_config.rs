use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{
    config_v2::types::{
        ApiAuth, ApiConfig, ConfigErrorV2, DcsAuth, DcsConfig, DcsEndpoint, FileSinkConfig,
        LoggingConfig, LoggingSinksConfig, OperatorConfigV2, PgtmApiTransportExpectation,
        PostgresConfig, PostgresLoggingConfig, ProcessBinariesConfig, ProcessConfig, RoleConfig,
        RuntimeConfigV2, Secret,
    },
    pginfo::conninfo::PgClientTls,
    state::{ApiRoute, ClusterName, MemberId, PgRoute, ScopeName},
};
use reqwest::Url;

use super::private_schema as raw;

type OptionalTokens = (Option<Secret>, Option<Secret>);

#[cfg(any(test, feature = "internal-test-support"))]
const RUNTIME_TEST_BINARY_PATHS_TOML: &str = r#"process.binaries.pg_ctl = "/bin/true"
process.binaries.initdb = "/bin/true"
process.binaries.pg_rewind = "/bin/true"
process.binaries.pg_basebackup = "/bin/true""#;

#[cfg(any(test, feature = "internal-test-support"))]
fn toml_string(value: &str) -> String {
    toml::Value::String(value.to_string()).to_string()
}

#[cfg(test)]
fn toml_path_source(path: &Path) -> String {
    format!(
        "{{ path = {} }}",
        toml_string(path.display().to_string().as_str())
    )
}

#[cfg(test)]
fn toml_string_secret(value: &str) -> String {
    format!(r#"{{ type = "string", value = {} }}"#, toml_string(value))
}

#[cfg(any(test, feature = "internal-test-support"))]
fn render_runtime_test_config_toml<J, T>(
    data_dir: &Path,
    scope: &str,
    role_credentials: [(&str, &str); 3],
    hba_contents: &str,
    extra_sections: J,
) -> String
where
    J: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    let extra_sections = extra_sections
        .into_iter()
        .map(|section| section.as_ref().trim().to_string())
        .filter(|section| !section.is_empty())
        .collect::<Vec<_>>();
    let has_process_binaries_override = extra_sections.iter().any(|section| {
        section.contains("process.binaries.") || section.contains("[process.binaries]")
    });
    let [(superuser_username, superuser_password), (replicator_username, replicator_password), (rewinder_username, rewinder_password)] =
        role_credentials;
    let extra_sections = (if has_process_binaries_override {
        Vec::new()
    } else {
        vec![RUNTIME_TEST_BINARY_PATHS_TOML.to_string()]
    })
    .into_iter()
    .chain(extra_sections)
    .collect::<Vec<_>>()
    .join("\n\n");
    format!(
        r#"cluster.name = "cluster-a"
cluster.scope = {scope}
cluster.member_id = "node-a"
postgres.paths.data_dir = {data_dir}
postgres.roles.mandatory.superuser.username = {superuser_username}
postgres.roles.mandatory.superuser.auth = {{ type = "password", password = {{ type = "string", value = {superuser_password} }} }}
postgres.roles.mandatory.replicator.username = {replicator_username}
postgres.roles.mandatory.replicator.auth = {{ type = "password", password = {{ type = "string", value = {replicator_password} }} }}
postgres.roles.mandatory.rewinder.username = {rewinder_username}
postgres.roles.mandatory.rewinder.auth = {{ type = "password", password = {{ type = "string", value = {rewinder_password} }} }}
postgres.access.hba = {{ content = {hba_contents} }}
postgres.access.ident = {{ content = "" }}
dcs.endpoints = ["http://127.0.0.1:2379"]

{extra_sections}
"#,
        scope = toml_string(scope),
        data_dir = toml_string(data_dir.display().to_string().as_str()),
        superuser_username = toml_string(superuser_username),
        superuser_password = toml_string(superuser_password),
        replicator_username = toml_string(replicator_username),
        replicator_password = toml_string(replicator_password),
        rewinder_username = toml_string(rewinder_username),
        rewinder_password = toml_string(rewinder_password),
        hba_contents = toml_string(hba_contents),
    )
}

#[cfg(test)]
fn render_default_runtime_test_config_toml<J, T>(
    root: &Path,
    extra_sections: J,
) -> Result<String, ConfigErrorV2>
where
    J: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    Ok(render_runtime_test_config_toml(
        root.join("data").as_path(),
        "scope-a",
        [
            ("postgres", "postgres"),
            ("replicator", "replicator"),
            ("rewinder", "rewinder"),
        ],
        "host all all 127.0.0.1/32 trust",
        extra_sections,
    ))
}

#[cfg(any(test, feature = "internal-test-support"))]
pub fn load_runtime_test_config_with_hba_and_sections<J, T>(
    data_dir: &Path,
    scope: &str,
    hba_contents: &str,
    extra_sections: J,
) -> Result<RuntimeConfigV2, ConfigErrorV2>
where
    J: IntoIterator<Item = T>,
    T: AsRef<str>,
{
    let contents = render_runtime_test_config_toml(
        data_dir,
        scope,
        [
            ("postgres", "secret-password"),
            ("replicator", "secret-password"),
            ("rewinder", "secret-password"),
        ],
        hba_contents,
        extra_sections,
    );
    load_runtime_config_contents(contents.as_str())
}

pub fn load_runtime_config(path: &Path) -> Result<RuntimeConfigV2, ConfigErrorV2> {
    let contents = read_config_file(path)?;
    load_runtime_config_contents_at(contents.as_str(), path)
}

pub fn load_operator_config(path: &Path) -> Result<OperatorConfigV2, ConfigErrorV2> {
    let contents = read_config_file(path)?;
    load_operator_config_contents_at(contents.as_str(), path)
}

#[cfg(any(test, feature = "internal-test-support"))]
pub fn load_runtime_config_contents(contents: &str) -> Result<RuntimeConfigV2, ConfigErrorV2> {
    load_runtime_config_contents_at(contents, Path::new("<runtime-config>"))
}

#[cfg(any(test, feature = "internal-test-support"))]
pub fn load_operator_config_contents(contents: &str) -> Result<OperatorConfigV2, ConfigErrorV2> {
    load_operator_config_contents_at(contents, Path::new("<operator-config>"))
}

fn load_runtime_config_contents_at(
    contents: &str,
    path: &Path,
) -> Result<RuntimeConfigV2, ConfigErrorV2> {
    parse_runtime_document(contents, path)?.into_runtime_config(path)
}

fn load_operator_config_contents_at(
    contents: &str,
    path: &Path,
) -> Result<OperatorConfigV2, ConfigErrorV2> {
    let document: toml::Value =
        toml::from_str(contents).map_err(|source| parse_error(path, source))?;
    if looks_like_runtime_operator_source(&document) {
        let runtime_document = document
            .try_into::<raw::RuntimeDocument>()
            .map_err(|source| parse_error(path, source))?;
        return runtime_document
            .pgtm
            .ok_or_else(|| {
                raw::validation_error("pgtm", "missing operator config block in runtime document")
            })
            .and_then(|pgtm| pgtm.into_operator_config(path, true));
    }
    toml::from_str::<raw::OperatorDocument>(contents)
        .map_err(|source| parse_error(path, source))?
        .into_operator_config(path, true)
}

fn parse_runtime_document(
    contents: &str,
    path: &Path,
) -> Result<raw::RuntimeDocument, ConfigErrorV2> {
    toml::from_str(contents).map_err(|source| parse_error(path, source))
}

impl raw::RuntimeDocument {
    fn into_runtime_config(self, path: &Path) -> Result<RuntimeConfigV2, ConfigErrorV2> {
        #[rustfmt::skip]
        let raw::RuntimeDocument { cluster, postgres, dcs, ha, process, logging, api, pgtm, debug: raw::DebugConfig { enabled: _debug_enabled } } = self;
        let config_dir = resolve_config_dir(path)?;
        let config_dir = config_dir.as_path();
        let process = process.finalize(config_dir)?;
        let working_root = process.working_root.clone();
        let operator_advertise = pgtm
            .map(|pgtm| pgtm.into_operator_config(path, false))
            .transpose()?
            .and_then(|config| config.advertised_url);

        let raw::ClusterConfig {
            name: cluster_name,
            scope,
            member_id,
        } = cluster;
        validate_non_empty("cluster.name", cluster_name.as_str())?;
        validate_non_empty("cluster.scope", scope.as_str())?;
        validate_non_empty("cluster.member_id", member_id.as_str())?;
        Ok(RuntimeConfigV2 {
            cluster_name: ClusterName(cluster_name),
            scope: ScopeName(scope),
            member_id: MemberId(member_id),
            postgres: postgres.into_runtime_config(working_root.as_path(), config_dir)?,
            dcs: dcs.into_runtime_config(config_dir)?,
            ha,
            process,
            logging: logging.finalize(working_root.as_path(), config_dir)?,
            api: ApiConfig {
                listen_addr: api.listen_addr,
                transport: api.transport.into_runtime_transport(config_dir)?,
                auth: api.auth.into_runtime_api_auth(config_dir)?,
                advertise: operator_advertise,
            },
        })
    }
}

#[cfg(any(test, feature = "internal-test-support"))]
pub fn runtime_test_config_with_data_dir(
    data_dir: impl Into<PathBuf>,
) -> Result<RuntimeConfigV2, ConfigErrorV2> {
    let data_dir = data_dir.into();
    load_runtime_test_config_with_hba_and_sections(
        data_dir.as_path(),
        "scope-a",
        "host all all 127.0.0.1/32 trust",
        std::iter::empty::<&str>(),
    )
}

#[cfg(any(test, feature = "internal-test-support"))]
pub fn load_runtime_timing_values(
    path: &Path,
) -> Result<(Duration, Duration, Duration, Duration), ConfigErrorV2> {
    let contents = read_config_file(path)?;
    let document = parse_runtime_document(contents.as_str(), path)?;
    Ok((
        document.ha.loop_interval,
        document.ha.lease_ttl,
        document.process.timeouts.bootstrap,
        document.process.timeouts.pg_rewind,
    ))
}

pub(super) fn read_config_file(path: &Path) -> Result<String, ConfigErrorV2> {
    std::fs::read_to_string(path).map_err(|source| ConfigErrorV2::Io {
        path: path.display().to_string(),
        source,
    })
}

pub(super) fn parse_error(path: &Path, source: toml::de::Error) -> ConfigErrorV2 {
    ConfigErrorV2::Parse {
        path: path.display().to_string(),
        source,
    }
}

pub(super) fn validate_non_empty(field: &'static str, value: &str) -> Result<(), ConfigErrorV2> {
    if value.trim().is_empty() {
        return Err(raw::validation_error(field, "must not be empty"));
    }
    Ok(())
}

fn non_empty_owned(field: &'static str, value: String) -> Result<String, ConfigErrorV2> {
    validate_non_empty(field, value.as_str())?;
    Ok(value)
}

pub(super) fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn path_override(path: PathBuf) -> Option<PathBuf> {
    if path.as_os_str().is_empty() {
        None
    } else {
        Some(path)
    }
}

fn map_postgres_role(
    username_field: &'static str,
    password_field: &'static str,
    role: raw::PostgresRoleConfig,
    config_dir: &Path,
) -> Result<RoleConfig, ConfigErrorV2> {
    validate_non_empty(username_field, role.username.as_str())?;
    let raw::RoleAuthConfig::Password { password } = role.auth;
    Ok(RoleConfig {
        username: role.username,
        password: password.resolve_required(password_field, config_dir)?,
    })
}

fn map_dcs_auth(
    auth: raw::DcsAuthConfig,
    config_dir: &Path,
) -> Result<Option<DcsAuth>, ConfigErrorV2> {
    match auth {
        raw::DcsAuthConfig::Disabled => Ok(None),
        raw::DcsAuthConfig::Basic { username, password } => {
            validate_non_empty("dcs.client.auth.username", username.as_str())?;
            Ok(Some(DcsAuth {
                username,
                password: password.resolve_required("dcs.client.auth.password", config_dir)?,
            }))
        }
    }
}

fn map_postgres_advertise(
    field: &'static str,
    advertise: raw::PostgresAdvertiseConfig,
) -> Result<PgRoute, ConfigErrorV2> {
    let host = non_empty_owned(field, advertise.host)?;
    if advertise.port == 0 {
        return Err(raw::validation_error(field, "port must not be zero"));
    }
    PgRoute::tcp_hostaddr(host, advertise.port, advertise.hostaddr)
        .map_err(|message| raw::validation_error(field, message))
}

fn looks_like_runtime_operator_source(document: &toml::Value) -> bool {
    document.as_table().is_some_and(|table| {
        [
            "cluster", "dcs", "ha", "process", "logging", "debug", "pgtm",
        ]
        .into_iter()
        .any(|field| table.contains_key(field))
    })
}

fn parse_operator_url(
    field: &'static str,
    value: Option<String>,
    expected_transport: Option<PgtmApiTransportExpectation>,
) -> Result<Option<Url>, ConfigErrorV2> {
    normalize_optional_string(value)
        .map(|value| {
            let url = Url::parse(value.as_str())
                .map_err(|err| {
                    raw::validation_error(field, format!("must be a valid URL: {err}"))
                })?;
            if let Some(expected_transport) = expected_transport.filter(|transport| {
                !transport.matches_url(&url)
            }) {
                return Err(raw::validation_error(
                    field,
                    format!(
                        "operator config expects `{}` API transport, but resolved base URL uses `{}`",
                        expected_transport.scheme(),
                        url.scheme()
                    ),
                ));
            }
            Ok(url)
        })
        .transpose()
}

fn resolve_binary_path(
    field: &'static str,
    executable: &str,
    override_path: Option<PathBuf>,
    config_dir: &Path,
) -> Result<PathBuf, ConfigErrorV2> {
    if let Some(path) = override_path {
        let path = raw::normalize_config_path(field, path, config_dir)?;
        if !path.is_file() {
            return Err(raw::validation_error(
                "process.binaries",
                format!(
                    "`{field}` points to a missing executable: {}",
                    path.display()
                ),
            ));
        }
        return Ok(path);
    }

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

    Err(raw::validation_error(
        "process.binaries",
        format!(
            "unable to resolve `{executable}` via PATH or conventional PostgreSQL install locations; {detail}; set `{field}` explicitly if autodiscovery fails"
        ),
    ))
}

impl raw::PostgresConfig {
    fn into_runtime_config(
        self,
        working_root: &Path,
        config_dir: &Path,
    ) -> Result<PostgresConfig, ConfigErrorV2> {
        let raw::PostgresConfig {
            paths,
            network,
            connect_timeout_s,
            local_database,
            rewind:
                raw::PostgresRewindConfig {
                    database: _rewind_database,
                    transport,
                },
            tls,
            roles,
            access,
            extra_gucs,
        } = self;
        let raw::PostgresRolesConfig {
            mandatory:
                raw::MandatoryPostgresRolesConfig {
                    superuser,
                    replicator,
                    rewinder,
                },
            extra,
        } = roles;
        if !extra.is_empty() {
            return Err(raw::validation_error(
                "postgres.roles.extra",
                "managed extra roles are not supported by config_v2",
            ));
        }

        let raw::PostgresPathsConfig {
            data_dir,
            socket_dir,
            log_file,
        } = paths;
        let data_dir = raw::normalize_config_path("postgres.paths.data_dir", data_dir, config_dir)?;
        let socket_dir = normalize_path_or_default(
            "postgres.paths.socket_dir",
            socket_dir,
            || working_root.join("socket"),
            config_dir,
        )?;
        let log_file = normalize_path_or_default(
            "postgres.paths.log_file",
            log_file,
            || working_root.join("logs/postgres.log"),
            config_dir,
        )?;

        let raw::PostgresNetworkConfig {
            listen_host,
            listen_port,
            cluster_advertise,
            operator_advertise,
        } = network;
        let cluster_advertise = map_postgres_advertise(
            "postgres.network.cluster_advertise",
            cluster_advertise.unwrap_or(raw::PostgresAdvertiseConfig {
                host: listen_host.clone(),
                port: listen_port,
                hostaddr: None,
            }),
        )?;
        let operator_advertise = operator_advertise
            .map(|advertise| {
                map_postgres_advertise("postgres.network.operator_advertise", advertise)
            })
            .transpose()?;

        Ok(PostgresConfig {
            data_dir: data_dir.clone(),
            socket_dir,
            log_file,
            listen_host,
            listen_port,
            cluster_advertise,
            operator_advertise,
            connect_timeout: Duration::from_secs(u64::from(connect_timeout_s)),
            local_database: non_empty_owned("postgres.local_database", local_database)?,
            source_client_tls: PgClientTls {
                mode: transport.ssl_mode,
                root_cert: raw::PathSource::resolve_optional(
                    "postgres.rewind.transport.ca_cert",
                    transport.ca_cert,
                    config_dir,
                )?,
                client_cert: None,
                client_key: None,
            },
            superuser: map_postgres_role(
                "postgres.roles.mandatory.superuser.username",
                "postgres.roles.mandatory.superuser.auth.password",
                superuser,
                config_dir,
            )?,
            replicator: map_postgres_role(
                "postgres.roles.mandatory.replicator.username",
                "postgres.roles.mandatory.replicator.auth.password",
                replicator,
                config_dir,
            )?,
            rewinder: map_postgres_role(
                "postgres.roles.mandatory.rewinder.username",
                "postgres.roles.mandatory.rewinder.auth.password",
                rewinder,
                config_dir,
            )?,
            pg_hba_file: data_dir.join("pgtm.pg_hba.conf"),
            pg_ident_file: data_dir.join("pgtm.pg_ident.conf"),
            pg_hba_contents: access
                .hba
                .resolve_contents("postgres.access.hba", config_dir)?,
            pg_ident_contents: access
                .ident
                .resolve_contents("postgres.access.ident", config_dir)?,
            extra_gucs,
            tls: tls.into_runtime_tls(config_dir)?,
        })
    }
}

impl raw::DcsConfig {
    fn into_runtime_config(self, config_dir: &Path) -> Result<DcsConfig, ConfigErrorV2> {
        let raw::DcsConfig {
            endpoints,
            client: raw::DcsClientConfig { auth, tls },
            init,
        } = self;
        if endpoints.is_empty() {
            return Err(raw::validation_error(
                "dcs.endpoints",
                "at least one endpoint is required",
            ));
        }
        if init.is_some() {
            return Err(raw::validation_error(
                "dcs.init",
                "is not supported by config_v2",
            ));
        }

        let tls = tls.into_runtime_tls(config_dir)?;
        if endpoints
            .iter()
            .any(|endpoint| endpoint.trim_start().starts_with("https://"))
            && tls.is_none()
        {
            return Err(raw::validation_error(
                "dcs.client.tls",
                "https DCS endpoints require `dcs.client.tls` to be configured",
            ));
        }

        Ok(DcsConfig {
            endpoints: endpoints
                .into_iter()
                .map(|endpoint| DcsEndpoint::new(endpoint.trim().to_string()))
                .collect(),
            auth: map_dcs_auth(auth, config_dir)?,
            tls,
        })
    }
}

impl ProcessConfig {
    fn finalize(self, config_dir: &Path) -> Result<Self, ConfigErrorV2> {
        let Self {
            timeouts,
            working_root,
            binaries,
        } = self;
        let working_root = raw::normalize_config_path(
            "process.working_root",
            if working_root.as_os_str().is_empty() {
                PathBuf::from("/tmp/pgtuskmaster")
            } else {
                working_root
            },
            config_dir,
        )?;
        Ok(Self {
            timeouts,
            working_root,
            binaries: binaries.finalize(config_dir)?,
        })
    }
}

impl ProcessBinariesConfig {
    fn finalize(self, config_dir: &Path) -> Result<Self, ConfigErrorV2> {
        let Self {
            pg_ctl,
            pg_rewind,
            initdb,
            pg_basebackup,
        } = self;
        Ok(Self {
            pg_ctl: resolve_binary_path(
                "process.binaries.pg_ctl",
                "pg_ctl",
                path_override(pg_ctl),
                config_dir,
            )?,
            initdb: resolve_binary_path(
                "process.binaries.initdb",
                "initdb",
                path_override(initdb),
                config_dir,
            )?,
            pg_rewind: resolve_binary_path(
                "process.binaries.pg_rewind",
                "pg_rewind",
                path_override(pg_rewind),
                config_dir,
            )?,
            pg_basebackup: resolve_binary_path(
                "process.binaries.pg_basebackup",
                "pg_basebackup",
                path_override(pg_basebackup),
                config_dir,
            )?,
        })
    }
}

impl LoggingConfig {
    fn finalize(self, working_root: &Path, config_dir: &Path) -> Result<Self, ConfigErrorV2> {
        let Self {
            level,
            capture_subprocess_output,
            postgres,
            sinks,
        } = self;
        let postgres_log_dir = normalize_path_or_default(
            "logging.postgres.log_dir",
            path_override(postgres.log_dir),
            || working_root.join("logs/postgres"),
            config_dir,
        )?;
        let file_path = normalize_path_or_default(
            "logging.sinks.file.path",
            path_override(sinks.file.path),
            || working_root.join("runtime.jsonl"),
            config_dir,
        )?;
        Ok(Self {
            level,
            capture_subprocess_output,
            postgres: PostgresLoggingConfig {
                log_dir: postgres_log_dir,
                ..postgres
            },
            sinks: LoggingSinksConfig {
                file: FileSinkConfig {
                    path: file_path,
                    ..sinks.file
                },
                ..sinks
            },
        })
    }
}

impl raw::TokenAuthConfig {
    fn into_runtime_api_auth(self, config_dir: &Path) -> Result<ApiAuth, ConfigErrorV2> {
        let Some((read_token, admin_token)) =
            self.resolve_tokens("api.auth.read_token", "api.auth.admin_token", config_dir)?
        else {
            return Ok(ApiAuth::Disabled);
        };
        Ok(ApiAuth::Tokens {
            read_token: read_token.ok_or_else(|| {
                raw::validation_error("api.auth.read_token", "is required when auth is enabled")
            })?,
            admin_token: admin_token.ok_or_else(|| {
                raw::validation_error("api.auth.admin_token", "is required when auth is enabled")
            })?,
        })
    }

    fn into_operator_tokens(
        self,
        resolve_auth_tokens: bool,
        config_dir: &Path,
    ) -> Result<(Option<Secret>, Option<Secret>), ConfigErrorV2> {
        if !resolve_auth_tokens {
            return Ok((None, None));
        }
        Ok(self
            .resolve_tokens(
                "pgtm.api.auth.read_token",
                "pgtm.api.auth.admin_token",
                config_dir,
            )?
            .unwrap_or((None, None)))
    }

    fn into_token_sources(self) -> (Option<raw::SecretSource>, Option<raw::SecretSource>) {
        let raw::TokenAuthConfig {
            kind: _kind,
            read_token,
            admin_token,
            tokens,
        } = self;
        let raw::RoleTokens {
            read_token: nested_read_token,
            admin_token: nested_admin_token,
        } = tokens.unwrap_or_default();
        (
            read_token.or(nested_read_token),
            admin_token.or(nested_admin_token),
        )
    }

    fn resolve_tokens(
        self,
        read_field: &'static str,
        admin_field: &'static str,
        config_dir: &Path,
    ) -> Result<Option<OptionalTokens>, ConfigErrorV2> {
        if self.is_disabled() {
            return Ok(None);
        }
        let (read_token, admin_token) = self.into_token_sources();
        Ok(Some((
            raw::SecretSource::resolve_optional(read_field, read_token, config_dir)?,
            raw::SecretSource::resolve_optional(admin_field, admin_token, config_dir)?,
        )))
    }
}

impl raw::OperatorDocument {
    fn into_operator_config(
        self,
        path: &Path,
        resolve_auth_tokens: bool,
    ) -> Result<OperatorConfigV2, ConfigErrorV2> {
        #[rustfmt::skip]
        let raw::OperatorDocument { api, client_tls } = self;
        #[rustfmt::skip]
        let raw::OperatorApiConfig { base_url, advertised_url, expected_transport, resolve_to, auth } = api;
        let config_dir = resolve_config_dir(path)?;
        let (read_token, admin_token) =
            auth.into_operator_tokens(resolve_auth_tokens, config_dir.as_path())?;
        let client_tls = client_tls.into_pg_client_tls(
            "pgtm.client_tls.ca_cert",
            "pgtm.client_tls.identity.cert",
            "pgtm.client_tls.identity.key",
            config_dir.as_path(),
        )?;

        #[rustfmt::skip]
        let operator_config = OperatorConfigV2 { base_url: parse_operator_url("pgtm.api.base_url", base_url, expected_transport)?, advertised_url: parse_operator_url("pgtm.api.advertised_url", advertised_url, expected_transport)?.map(|url| ApiRoute::from_url(url).map_err(|err| raw::validation_error("pgtm.api.advertised_url", err))).transpose()?, expected_transport, resolve_to, client_tls, read_token, admin_token };
        Ok(operator_config)
    }
}

fn resolve_config_dir(path: &Path) -> Result<PathBuf, ConfigErrorV2> {
    let relative_parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    if relative_parent.is_absolute() {
        return Ok(relative_parent);
    }
    std::env::current_dir()
        .map(|cwd| cwd.join(relative_parent))
        .map_err(|source| ConfigErrorV2::Io {
            path: ".".to_string(),
            source,
        })
}

fn normalize_path_or_default<F>(
    field: &'static str,
    path: Option<PathBuf>,
    default: F,
    config_dir: &Path,
) -> Result<PathBuf, ConfigErrorV2>
where
    F: FnOnce() -> PathBuf,
{
    path.map(|path| raw::normalize_config_path(field, path, config_dir))
        .transpose()?
        .map_or_else(|| Ok(default()), Ok)
}

fn conventional_postgres_bin_dirs() -> Vec<PathBuf> {
    let mut directories = Vec::new();
    directories.extend(child_bin_dirs_matching(
        Path::new("/usr/lib/postgresql"),
        |_| true,
    ));
    directories.extend(child_bin_dirs_matching(Path::new("/usr"), |name| {
        name.starts_with("pgsql-")
    }));
    directories.extend(child_bin_dirs_matching(
        Path::new("/opt/homebrew/opt"),
        |name| name.starts_with("postgresql@"),
    ));
    directories.extend(child_bin_dirs_matching(
        Path::new("/usr/local/opt"),
        |name| name.starts_with("postgresql@"),
    ));
    directories.push(PathBuf::from("/opt/homebrew/opt/libpq/bin"));
    directories.push(PathBuf::from("/usr/local/opt/libpq/bin"));
    directories
}

fn child_bin_dirs_matching<F>(root: &Path, predicate: F) -> Vec<PathBuf>
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
            predicate(name).then(|| entry.path().join("bin"))
        })
        .collect::<Vec<_>>();
    directories.sort();
    directories
}

#[cfg(test)]
mod tests {
    use super::{
        load_operator_config_contents, load_runtime_config_contents, load_runtime_timing_values,
        render_default_runtime_test_config_toml, toml_path_source, toml_string, toml_string_secret,
    };
    use crate::{config_v2::ConfigErrorV2, dev_support::test_fs::unique_test_dir};
    use std::{fs, os::unix::fs::PermissionsExt, time::Duration};

    #[rustfmt::skip]
    const INLINE_OPERATOR_TLS_SECTION: &str = "[client_tls]\nidentity = { cert = { path = \"/tmp/client.crt\" }, key = { type = \"env\", env = \"CLIENT_KEY\" } }";
    #[rustfmt::skip]
    const INLINE_RUNTIME_TLS_SECTION: &str = "[postgres.tls]\nmode = \"enabled\"\nidentity = { cert_chain = { content = \"CERT\" }, private_key = { content = \"KEY\" } }";
    #[rustfmt::skip]
    const ZERO_RUNTIME_DEFAULT_OVERRIDES: &str = "postgres.connect_timeout_s = 0\npostgres.network.listen_host = \"   \"\npostgres.network.listen_port = 0\nha.loop_interval_ms = 0\nha.lease_ttl_ms = 0\nprocess.timeouts.pg_rewind_ms = 0\nprocess.timeouts.bootstrap_ms = 0\nprocess.timeouts.fencing_ms = 0\nprocess.binaries.pg_ctl = \"/bin/true\"\nprocess.binaries.initdb = \"/bin/true\"\nprocess.binaries.pg_rewind = \"/bin/true\"\nprocess.binaries.pg_basebackup = \"/bin/true\"\nlogging.capture_subprocess_output = true\nlogging.postgres.enabled = true\nlogging.postgres.poll_interval_ms = 0\nlogging.postgres.cleanup.enabled = true\nlogging.postgres.cleanup.max_files = 0\nlogging.postgres.cleanup.max_age_seconds = 0\nlogging.postgres.cleanup.protect_recent_seconds = 0";

    fn render_operator_test_config_toml<J, T>(
        base_url: Option<&str>,
        advertised_url: Option<&str>,
        expected_transport: Option<&str>,
        resolve_to: Option<std::net::SocketAddr>,
        extra_sections: J,
    ) -> Result<String, ConfigErrorV2>
    where
        J: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let expected_transport = expected_transport
            .map(|transport| match transport {
                "http" | "https" => Ok(format!("expected_transport = {transport:?}")),
                other => Err(ConfigErrorV2::Validation {
                    field: "test-support",
                    message: format!("unsupported test transport expectation `{other}`"),
                }),
            })
            .transpose()?;
        let api_lines = [
            base_url.map(|value| format!("base_url = {}", toml_string(value))),
            advertised_url.map(|value| format!("advertised_url = {}", toml_string(value))),
            expected_transport,
            resolve_to
                .map(|value| format!("resolve_to = {}", toml_string(value.to_string().as_str()))),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n");
        let extra_sections = extra_sections
            .into_iter()
            .map(|section| section.as_ref().trim().to_string())
            .filter(|section| !section.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");
        let extra_sections = if extra_sections.is_empty() {
            String::new()
        } else {
            format!("\n{extra_sections}")
        };
        Ok(format!("[api]\n{api_lines}{extra_sections}\n"))
    }

    #[test]
    fn load_runtime_config_and_timing_values_normalize_zero_runtime_fields() -> Result<(), String> {
        let root = unique_test_dir("load-config", "runtime-config-v2-zero-defaults")?;
        #[rustfmt::skip]
        let contents = render_default_runtime_test_config_toml(root.as_path(), [ZERO_RUNTIME_DEFAULT_OVERRIDES]).map_err(|err| err.to_string())?;
        let config =
            load_runtime_config_contents(contents.as_str()).map_err(|err| err.to_string())?;

        assert_eq!(
            (
                config.postgres.listen_host.as_str(),
                config.postgres.listen_port,
                config.postgres.cluster_advertise.host(),
                config.postgres.cluster_advertise.port(),
                config.postgres.connect_timeout,
                config.process.timeouts.bootstrap,
                config.logging.postgres.poll_interval,
            ),
            (
                "127.0.0.1",
                5432,
                "127.0.0.1",
                5432,
                Duration::from_secs(5),
                Duration::from_millis(300_000),
                Duration::from_millis(200),
            )
        );
        assert_eq!(config.logging.postgres.cleanup.max_files, 50);
        let config_path = root.join("runtime.toml");
        fs::write(config_path.as_path(), contents).map_err(|err| err.to_string())?;

        let (loop_interval, lease_ttl, bootstrap_timeout, pg_rewind_timeout) =
            load_runtime_timing_values(config_path.as_path()).map_err(|err| err.to_string())?;

        assert_eq!(
            (
                loop_interval,
                lease_ttl,
                bootstrap_timeout,
                pg_rewind_timeout
            ),
            (
                Duration::from_millis(1_000),
                Duration::from_millis(10_000),
                Duration::from_millis(300_000),
                Duration::from_millis(120_000)
            )
        );
        Ok(())
    }

    #[test]
    fn load_runtime_config_does_not_require_readable_operator_auth_tokens() -> Result<(), String> {
        let root = unique_test_dir("load-config", "runtime-config-v2-operator-auth")?;
        let unreadable_token = root.join("operator-admin-token");
        fs::write(unreadable_token.as_path(), "secret\n").map_err(|err| err.to_string())?;
        fs::set_permissions(
            unreadable_token.as_path(),
            fs::Permissions::from_mode(0o000),
        )
        .map_err(|err| err.to_string())?;
        let result = load_runtime_config_contents(
            render_default_runtime_test_config_toml(
                root.as_path(),
                [format!(
                    r#"[pgtm.api]
base_url = "https://127.0.0.1:8443"

[pgtm.api.auth]
type = "role_tokens"

[pgtm.api.auth.tokens]
admin_token = {{ type = "file", path = "{}" }}"#,
                    unreadable_token.display()
                )],
            )
            .map_err(|err| err.to_string())?
            .as_str(),
        );

        fs::set_permissions(
            unreadable_token.as_path(),
            fs::Permissions::from_mode(0o600),
        )
        .map_err(|err| err.to_string())?;
        let _ = fs::remove_dir_all(root);

        result.map(|_| ()).map_err(|err| err.to_string())
    }

    #[test]
    fn parse_boundaries_reject_non_path_tls_sources() -> Result<(), String> {
        let root = unique_test_dir("load-config", "runtime-config-v2-inline-tls")?;
        match load_runtime_config_contents(
            render_default_runtime_test_config_toml(root.as_path(), [INLINE_RUNTIME_TLS_SECTION])
                .map_err(|err| err.to_string())?
                .as_str(),
        ) {
            Err(ConfigErrorV2::Parse { .. }) => {}
            Err(err) => return Err(format!("expected parse error, got {err}")),
            Ok(_) => return Err("expected inline TLS parse rejection".to_string()),
        }

        match load_operator_config_contents(
            render_operator_test_config_toml(
                Some("https://127.0.0.1:8443"),
                None,
                None,
                None,
                [INLINE_OPERATOR_TLS_SECTION],
            )
            .map_err(|err| err.to_string())?
            .as_str(),
        ) {
            Err(ConfigErrorV2::Parse { .. }) => Ok(()),
            Err(err) => Err(format!("expected parse error, got {err}")),
            Ok(_) => Err("expected non-path TLS identity rejection".to_string()),
        }
    }

    #[test]
    fn load_operator_config_flattens_tokens_and_resolves_client_tls() -> Result<(), String> {
        let dir = unique_test_dir("load-operator-config", "merge")?;
        let api_ca_path = dir.join("api-ca.pem");
        let identity_cert_path = dir.join("client.crt");
        let identity_key_path = dir.join("client.key");
        let config = load_operator_config_contents(
            render_operator_test_config_toml(
                Some("https://127.0.0.1:8443"),
                None,
                None,
                None,
                [format!(
                    r#"[api.auth]
type = "role_tokens"
read_token = {}
admin_token = {}

[client_tls]
ca_cert = {}
identity = {{ cert = {}, key = {} }}"#,
                    toml_string_secret("read-token"),
                    toml_string_secret("admin-token"),
                    toml_path_source(api_ca_path.as_path()),
                    toml_path_source(identity_cert_path.as_path()),
                    toml_path_source(identity_key_path.as_path()),
                )],
            )
            .map_err(|err| err.to_string())?
            .as_str(),
        )
        .map_err(|err| err.to_string())?;
        let _ = std::fs::remove_dir_all(dir);

        assert_eq!(
            config
                .read_token
                .as_ref()
                .map(|token| token.as_str())
                .zip(config.admin_token.as_ref().map(|token| token.as_str())),
            Some(("read-token", "admin-token"))
        );
        assert_eq!(
            config.client_tls.as_ref().map(|tls| {
                (
                    tls.root_cert.as_ref(),
                    tls.client_cert.as_ref(),
                    tls.client_key.as_ref(),
                )
            }),
            Some((
                Some(&api_ca_path),
                Some(&identity_cert_path),
                Some(&identity_key_path),
            ))
        );
        Ok(())
    }
}
