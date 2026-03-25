use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{
    config_v2::types::{
        ApiAuth, ApiConfig, ApiTransport, BinariesConfig, ConfigErrorV2, DcsAuth, DcsConfig,
        DcsEndpoint, LoggingConfig, OperatorConfigV2, PgtmApiTransportExpectation, PostgresConfig,
        RoleConfig, RuntimeConfigV2, Secret, TimingConfig, TlsConfig,
    },
    pginfo::conninfo::{PgClientTls, PgSslMode},
    state::{ApiRoute, ClusterName, MemberId, PgRoute, ScopeName},
};
use reqwest::Url;

use super::private_schema as raw;

type ResolvedOptionalTokens = (Option<Secret>, Option<Secret>);

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
    let document = toml::from_str::<raw::RuntimeDocument>(contents)
        .map_err(|source| parse_error(path, source))?;
    document.into_runtime_config(path)
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
                validation_error("pgtm", "missing operator config block in runtime document")
            })
            .and_then(|pgtm| parse_operator_config_value_at(pgtm, path, true));
    }
    parse_operator_config_value_at(document, path, true)
}

impl raw::RuntimeDocument {
    fn into_runtime_config(self, path: &Path) -> Result<RuntimeConfigV2, ConfigErrorV2> {
        #[rustfmt::skip]
        let raw::RuntimeDocument { cluster, postgres, dcs, ha, process, logging, api, pgtm, debug: raw::DebugConfig { enabled: _debug_enabled } } = self;
        let config_dir = resolve_config_dir(path)?;
        let config_dir = config_dir.as_path();
        let (working_root, timeouts, binaries) = process.into_runtime_parts(config_dir)?;
        let operator_advertise = pgtm
            .map(|pgtm| parse_operator_config_value_at(pgtm, path, false))
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
            timing: TimingConfig {
                ha_loop_interval: Duration::from_millis(ha.loop_interval_ms),
                ha_lease_ttl: Duration::from_millis(ha.lease_ttl_ms),
                bootstrap_timeout: Duration::from_millis(timeouts.bootstrap_ms),
                pg_rewind_timeout: Duration::from_millis(timeouts.pg_rewind_ms),
                fencing_timeout: Duration::from_millis(timeouts.fencing_ms),
            },
            binaries,
            logging: logging.into_runtime_config(working_root.as_path(), config_dir)?,
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
pub fn runtime_test_config() -> Result<RuntimeConfigV2, ConfigErrorV2> {
    load_runtime_test_config_from_paths(PathBuf::from("/tmp/pgdata"), "scope-a")
}

#[cfg(any(test, feature = "internal-test-support"))]
pub fn runtime_test_config_with_data_dir(
    data_dir: impl Into<PathBuf>,
) -> Result<RuntimeConfigV2, ConfigErrorV2> {
    load_runtime_test_config_from_paths(data_dir.into(), "scope-a")
}

#[cfg(any(test, feature = "internal-test-support"))]
pub fn managed_postgres_test_config(
    data_dir: impl Into<PathBuf>,
) -> Result<RuntimeConfigV2, ConfigErrorV2> {
    let config = load_runtime_test_config_from_paths(data_dir.into(), "cluster-a")?;
    Ok(RuntimeConfigV2 {
        timing: TimingConfig {
            ha_loop_interval: Duration::from_millis(500),
            ha_lease_ttl: Duration::from_secs(5),
            bootstrap_timeout: Duration::from_secs(30),
            pg_rewind_timeout: Duration::from_secs(30),
            fencing_timeout: Duration::from_secs(10),
        },
        ..config
    })
}

#[cfg(any(test, feature = "internal-test-support"))]
pub fn trace_logging_test_config() -> Result<RuntimeConfigV2, ConfigErrorV2> {
    let config = runtime_test_config()?;
    Ok(RuntimeConfigV2 {
        postgres: PostgresConfig {
            pg_hba_contents: concat!("local all all trust\n", "host all all 127.0.0.1/32 trust\n")
                .to_string(),
            ..config.postgres
        },
        logging: LoggingConfig {
            level: crate::config_v2::types::LogLevel::Trace,
            postgres_log_poll_interval: Duration::from_millis(50),
            postgres_log_cleanup_enabled: false,
            ..config.logging
        },
        ..config
    })
}

#[cfg(any(test, feature = "internal-test-support"))]
pub fn load_runtime_timing_values(
    path: &Path,
) -> Result<(Duration, Duration, Duration, Duration), ConfigErrorV2> {
    let contents = read_config_file(path)?;
    let document = toml::from_str::<raw::RuntimeDocument>(&contents)
        .map_err(|source| parse_error(path, source))?;
    Ok((
        Duration::from_millis(document.ha.loop_interval_ms),
        Duration::from_millis(document.ha.lease_ttl_ms),
        Duration::from_millis(document.process.timeouts.bootstrap_ms),
        Duration::from_millis(document.process.timeouts.pg_rewind_ms),
    ))
}

#[cfg(any(test, feature = "internal-test-support"))]
fn load_runtime_test_config_from_paths(
    data_dir: PathBuf,
    scope: &str,
) -> Result<RuntimeConfigV2, ConfigErrorV2> {
    let working_root = PathBuf::from("/tmp/pgtuskmaster");
    let postgres_log_dir = working_root.join("logs/postgres");
    let password = Secret::new("secret-password".to_string());
    let listen_host = "127.0.0.1".to_string();
    let listen_port = 5432;
    let cluster_advertise = PgRoute::tcp(listen_host.clone(), listen_port)
        .map_err(|message| validation_error("runtime_test_config", message))?;
    let config_dir = Path::new(".");

    #[rustfmt::skip]
    let config = RuntimeConfigV2 {
        cluster_name: ClusterName("cluster-a".to_string()),
        scope: ScopeName(scope.to_string()),
        member_id: MemberId("node-a".to_string()),
        postgres: PostgresConfig {
            data_dir: data_dir.clone(),
            socket_dir: working_root.join("socket"),
            log_file: working_root.join("logs/postgres.log"),
            listen_host,
            listen_port,
            cluster_advertise,
            operator_advertise: None,
            connect_timeout: Duration::from_secs(5),
            local_database: "postgres".to_string(),
            source_client_tls: PgClientTls {
                mode: PgSslMode::Prefer,
                root_cert: None,
                client_cert: None,
                client_key: None,
            },
            superuser: RoleConfig {
                username: "postgres".to_string(),
                password: password.clone(),
            },
            replicator: RoleConfig {
                username: "replicator".to_string(),
                password: password.clone(),
            },
            rewinder: RoleConfig {
                username: "rewinder".to_string(),
                password,
            },
            pg_hba_file: data_dir.join("pgtm.pg_hba.conf"),
            pg_ident_file: data_dir.join("pgtm.pg_ident.conf"),
            pg_hba_contents: "host all all 127.0.0.1/32 trust".to_string(),
            pg_ident_contents: String::new(),
            extra_gucs: Default::default(),
            tls: None,
        },
        dcs: DcsConfig {
            endpoints: vec![DcsEndpoint::new("http://127.0.0.1:2379".to_string())],
            auth: None,
            tls: None,
        },
        timing: TimingConfig {
            ha_loop_interval: Duration::from_millis(1_000),
            ha_lease_ttl: Duration::from_millis(10_000),
            bootstrap_timeout: Duration::from_millis(300_000),
            pg_rewind_timeout: Duration::from_millis(120_000),
            fencing_timeout: Duration::from_millis(30_000),
        },
        binaries: BinariesConfig {
            pg_ctl: resolve_binary_path("process.binaries.overrides.pg_ctl", "pg_ctl", None, config_dir)?,
            initdb: resolve_binary_path("process.binaries.overrides.initdb", "initdb", None, config_dir)?,
            pg_rewind: resolve_binary_path("process.binaries.overrides.pg_rewind", "pg_rewind", None, config_dir)?,
            pg_basebackup: resolve_binary_path("process.binaries.overrides.pg_basebackup", "pg_basebackup", None, config_dir)?,
        },
        logging: LoggingConfig {
            level: crate::config_v2::types::LogLevel::Info,
            capture_subprocess_output: true,
            stderr_enabled: true,
            file_enabled: false,
            file_path: working_root.join("runtime.jsonl"),
            file_mode: crate::config_v2::types::FileSinkMode::Append,
            postgres_logs_enabled: true,
            postgres_log_dir: postgres_log_dir.clone(),
            postgres_log_poll_interval: Duration::from_millis(200),
            postgres_log_cleanup_enabled: true,
            postgres_log_cleanup_max_files: 50,
            postgres_log_cleanup_max_age: Duration::from_secs(7 * 24 * 60 * 60),
            postgres_log_cleanup_protect_recent: Duration::from_secs(300),
        },
        api: ApiConfig {
            listen_addr: std::net::SocketAddr::from((std::net::Ipv4Addr::new(127, 0, 0, 1), 8080)),
            transport: ApiTransport::Http,
            auth: ApiAuth::Disabled,
            advertise: None,
        },
    };
    Ok(config)
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

pub(super) fn validation_error(field: &'static str, message: impl Into<String>) -> ConfigErrorV2 {
    ConfigErrorV2::Validation {
        field,
        message: message.into(),
    }
}

pub(super) fn validate_non_empty(field: &'static str, value: &str) -> Result<(), ConfigErrorV2> {
    if value.trim().is_empty() {
        return Err(validation_error(field, "must not be empty"));
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

pub(super) fn resolve_secret_optional(
    field: &'static str,
    source: Option<raw::SecretSource>,
    config_dir: &Path,
) -> Result<Option<Secret>, ConfigErrorV2> {
    source
        .map(|source| resolve_secret_required(field, source, config_dir))
        .transpose()
}

pub(super) fn resolve_secret_required(
    field: &'static str,
    source: raw::SecretSource,
    config_dir: &Path,
) -> Result<Secret, ConfigErrorV2> {
    let value = match source {
        raw::SecretSource::PathConfig { path } => {
            let path = normalize_config_path(field, path, config_dir)?;
            std::fs::read_to_string(&path).map_err(|source| ConfigErrorV2::Io {
                path: path.display().to_string(),
                source,
            })?
        }
        raw::SecretSource::Tagged(raw::TaggedSecretSource::None) => String::new(),
        raw::SecretSource::Tagged(raw::TaggedSecretSource::Env { env }) => std::env::var(&env)
            .map_err(|err| {
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
            })?,
        raw::SecretSource::Tagged(raw::TaggedSecretSource::File { path }) => {
            let path = normalize_config_path(field, path, config_dir)?;
            std::fs::read_to_string(&path).map_err(|source| ConfigErrorV2::Io {
                path: path.display().to_string(),
                source,
            })?
        }
        raw::SecretSource::Tagged(raw::TaggedSecretSource::String { value }) => value,
    };

    let value = value.trim_end_matches(['\n', '\r']).to_string();
    if value.trim().is_empty() {
        return Err(validation_error(field, "must not be empty"));
    }
    Ok(Secret::new(value))
}

pub(super) fn resolve_path_only(
    field: &'static str,
    source: raw::PathSource,
    config_dir: &Path,
) -> Result<PathBuf, ConfigErrorV2> {
    match source {
        raw::PathSource::Path(path) | raw::PathSource::PathConfig { path } => {
            normalize_config_path(field, path, config_dir)
        }
    }
}

fn resolve_inline_or_path_string(
    field: &'static str,
    source: raw::PathOrInline,
    config_dir: &Path,
) -> Result<String, ConfigErrorV2> {
    match source {
        raw::PathOrInline::Path(path) | raw::PathOrInline::PathConfig { path } => {
            let path = normalize_config_path(field, path, config_dir)?;
            std::fs::read_to_string(&path).map_err(|source| ConfigErrorV2::Io {
                path: path.display().to_string(),
                source,
            })
        }
        raw::PathOrInline::Inline { content } => Ok(content),
    }
}

fn resolve_optional_path(
    field: &'static str,
    source: Option<raw::PathSource>,
    config_dir: &Path,
) -> Result<Option<PathBuf>, ConfigErrorV2> {
    source
        .map(|source| resolve_path_only(field, source, config_dir))
        .transpose()
}

fn map_postgres_role(
    field_prefix: &'static str,
    role: raw::PostgresRoleConfig,
    config_dir: &Path,
) -> Result<RoleConfig, ConfigErrorV2> {
    let (username_field, password_field) = role_fields(field_prefix);
    validate_non_empty(username_field, role.username.as_str())?;
    let raw::RoleAuthConfig::Password { password } = role.auth;
    Ok(RoleConfig {
        username: role.username,
        password: resolve_secret_required(password_field, password, config_dir)?,
    })
}

fn role_fields(field_prefix: &'static str) -> (&'static str, &'static str) {
    match field_prefix {
        "postgres.roles.mandatory.superuser" => (
            "postgres.roles.mandatory.superuser.username",
            "postgres.roles.mandatory.superuser.auth.password",
        ),
        "postgres.roles.mandatory.replicator" => (
            "postgres.roles.mandatory.replicator.username",
            "postgres.roles.mandatory.replicator.auth.password",
        ),
        "postgres.roles.mandatory.rewinder" => (
            "postgres.roles.mandatory.rewinder.username",
            "postgres.roles.mandatory.rewinder.auth.password",
        ),
        _ => (field_prefix, field_prefix),
    }
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
                password: resolve_secret_required(
                    "dcs.client.auth.password",
                    password,
                    config_dir,
                )?,
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
        return Err(validation_error(field, "port must not be zero"));
    }
    PgRoute::tcp_hostaddr(host, advertise.port, advertise.hostaddr)
        .map_err(|message| validation_error(field, message))
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

fn parse_operator_config_value_at(
    value: toml::Value,
    path: &Path,
    resolve_auth_tokens: bool,
) -> Result<OperatorConfigV2, ConfigErrorV2> {
    let document: raw::OperatorDocument = value
        .try_into()
        .map_err(|source| parse_error(path, source))?;
    document.into_operator_config(path, resolve_auth_tokens)
}

fn parse_operator_url(
    field: &'static str,
    value: Option<String>,
    expected_transport: Option<PgtmApiTransportExpectation>,
) -> Result<Option<Url>, ConfigErrorV2> {
    normalize_optional_string(value)
        .map(|value| {
            let url = Url::parse(value.as_str())
                .map_err(|err| validation_error(field, format!("must be a valid URL: {err}")))?;
            validate_expected_transport(field, &url, expected_transport)?;
            Ok(url)
        })
        .transpose()
}

fn validate_expected_transport(
    field: &'static str,
    url: &Url,
    expected_transport: Option<PgtmApiTransportExpectation>,
) -> Result<(), ConfigErrorV2> {
    let Some(expected_transport) = expected_transport else {
        return Ok(());
    };

    if expected_transport.matches_url(url) {
        return Ok(());
    }

    Err(validation_error(
        field,
        format!(
            "operator config expects `{}` API transport, but resolved base URL uses `{}`",
            expected_transport.scheme(),
            url.scheme()
        ),
    ))
}

fn merge_matching<T: PartialEq>(
    left_field: &'static str,
    left: Option<T>,
    right_field: &'static str,
    right: Option<T>,
    merged_field: &'static str,
) -> Result<Option<T>, ConfigErrorV2> {
    match (left, right) {
        (Some(left), Some(right)) if left != right => Err(validation_error(
            merged_field,
            format!("`{left_field}` and `{right_field}` must match when both are configured"),
        )),
        (Some(value), Some(_)) | (Some(value), None) | (None, Some(value)) => Ok(Some(value)),
        (None, None) => Ok(None),
    }
}

fn merge_operator_client_tls(
    left: Option<PgClientTls>,
    right: Option<PgClientTls>,
) -> Result<Option<PgClientTls>, ConfigErrorV2> {
    match (left, right) {
        (Some(left), Some(right)) => {
            let merged_identity = merge_matching(
                "pgtm.api.tls.identity",
                left.client_cert.zip(left.client_key),
                "pgtm.postgres.tls.identity",
                right.client_cert.zip(right.client_key),
                "pgtm.client_tls.identity",
            )?;
            let (client_cert, client_key) =
                merged_identity.map_or((None, None), |(cert, key)| (Some(cert), Some(key)));

            Ok(Some(PgClientTls {
                mode: PgSslMode::VerifyFull,
                root_cert: merge_matching(
                    "pgtm.api.tls.ca_cert",
                    left.root_cert,
                    "pgtm.postgres.tls.ca_cert",
                    right.root_cert,
                    "pgtm.client_tls.ca_cert",
                )?,
                client_cert,
                client_key,
            }))
        }
        (Some(tls), None) | (None, Some(tls)) => Ok(Some(tls)),
        (None, None) => Ok(None),
    }
}

fn resolve_binary_path(
    field: &'static str,
    executable: &str,
    override_path: Option<PathBuf>,
    config_dir: &Path,
) -> Result<PathBuf, ConfigErrorV2> {
    if let Some(path) = override_path {
        let path = normalize_config_path(field, path, config_dir)?;
        if !path.is_file() {
            return Err(validation_error(
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

    Err(validation_error(
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
            return Err(validation_error(
                "postgres.roles.extra",
                "managed extra roles are not supported by config_v2",
            ));
        }

        let raw::PostgresPathsConfig {
            data_dir,
            socket_dir,
            log_file,
        } = paths;
        let data_dir = normalize_config_path("postgres.paths.data_dir", data_dir, config_dir)?;
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
                root_cert: resolve_optional_path(
                    "postgres.rewind.transport.ca_cert",
                    transport.ca_cert,
                    config_dir,
                )?,
                client_cert: None,
                client_key: None,
            },
            superuser: map_postgres_role(
                "postgres.roles.mandatory.superuser",
                superuser,
                config_dir,
            )?,
            replicator: map_postgres_role(
                "postgres.roles.mandatory.replicator",
                replicator,
                config_dir,
            )?,
            rewinder: map_postgres_role("postgres.roles.mandatory.rewinder", rewinder, config_dir)?,
            pg_hba_file: data_dir.join("pgtm.pg_hba.conf"),
            pg_ident_file: data_dir.join("pgtm.pg_ident.conf"),
            pg_hba_contents: resolve_inline_or_path_string(
                "postgres.access.hba",
                access.hba,
                config_dir,
            )?,
            pg_ident_contents: resolve_inline_or_path_string(
                "postgres.access.ident",
                access.ident,
                config_dir,
            )?,
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
            return Err(validation_error(
                "dcs.endpoints",
                "at least one endpoint is required",
            ));
        }
        if init.is_some() {
            return Err(validation_error(
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
            return Err(validation_error(
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

impl raw::ProcessConfig {
    fn into_runtime_parts(
        self,
        config_dir: &Path,
    ) -> Result<(PathBuf, raw::ProcessTimeoutsConfig, BinariesConfig), ConfigErrorV2> {
        let raw::ProcessConfig {
            timeouts,
            working_root,
            binaries,
        } = self;
        let working_root = normalize_config_path(
            "process.working_root",
            if working_root.as_os_str().is_empty() {
                PathBuf::from("/tmp/pgtuskmaster")
            } else {
                working_root
            },
            config_dir,
        )?;
        Ok((
            working_root,
            timeouts,
            binaries.into_runtime_config(config_dir)?,
        ))
    }
}

impl raw::BinaryResolutionConfig {
    fn into_runtime_config(self, config_dir: &Path) -> Result<BinariesConfig, ConfigErrorV2> {
        let raw::BinaryResolutionConfig {
            overrides:
                raw::BinaryPathOverrides {
                    pg_ctl,
                    pg_rewind,
                    initdb,
                    pg_basebackup,
                },
        } = self;
        Ok(BinariesConfig {
            pg_ctl: resolve_binary_path(
                "process.binaries.overrides.pg_ctl",
                "pg_ctl",
                pg_ctl,
                config_dir,
            )?,
            initdb: resolve_binary_path(
                "process.binaries.overrides.initdb",
                "initdb",
                initdb,
                config_dir,
            )?,
            pg_rewind: resolve_binary_path(
                "process.binaries.overrides.pg_rewind",
                "pg_rewind",
                pg_rewind,
                config_dir,
            )?,
            pg_basebackup: resolve_binary_path(
                "process.binaries.overrides.pg_basebackup",
                "pg_basebackup",
                pg_basebackup,
                config_dir,
            )?,
        })
    }
}

impl raw::LoggingConfig {
    fn into_runtime_config(
        self,
        working_root: &Path,
        config_dir: &Path,
    ) -> Result<LoggingConfig, ConfigErrorV2> {
        let raw::LoggingConfig {
            level,
            capture_subprocess_output,
            postgres:
                raw::PostgresLoggingConfig {
                    enabled: postgres_logs_enabled,
                    log_dir,
                    poll_interval_ms,
                    cleanup:
                        raw::LogCleanupConfig {
                            enabled: postgres_log_cleanup_enabled,
                            max_files: postgres_log_cleanup_max_files,
                            max_age_seconds: postgres_log_cleanup_max_age_seconds,
                            protect_recent_seconds: postgres_log_cleanup_protect_recent_seconds,
                        },
                },
            sinks:
                raw::LoggingSinksConfig {
                    stderr:
                        raw::StderrSinkConfig {
                            enabled: stderr_enabled,
                        },
                    file:
                        raw::FileSinkConfig {
                            enabled: file_enabled,
                            path: file_path,
                            mode: file_mode,
                        },
                },
        } = self;
        let postgres_log_dir = normalize_path_or_default(
            "logging.postgres.log_dir",
            log_dir,
            || working_root.join("logs/postgres"),
            config_dir,
        )?;
        Ok(LoggingConfig {
            level,
            capture_subprocess_output,
            stderr_enabled,
            file_enabled,
            file_path: normalize_path_or_default(
                "logging.sinks.file.path",
                file_path,
                || working_root.join("runtime.jsonl"),
                config_dir,
            )?,
            file_mode,
            postgres_logs_enabled,
            postgres_log_dir,
            postgres_log_poll_interval: Duration::from_millis(poll_interval_ms),
            postgres_log_cleanup_enabled,
            postgres_log_cleanup_max_files,
            postgres_log_cleanup_max_age: Duration::from_secs(postgres_log_cleanup_max_age_seconds),
            postgres_log_cleanup_protect_recent: Duration::from_secs(
                postgres_log_cleanup_protect_recent_seconds,
            ),
        })
    }
}

impl raw::TlsServerConfig {
    fn into_runtime_tls(self, config_dir: &Path) -> Result<Option<TlsConfig>, ConfigErrorV2> {
        match self {
            raw::TlsServerConfig::Disabled => Ok(None),
            raw::TlsServerConfig::Enabled {
                identity,
                client_auth,
            } => {
                let (cert, key) = identity.resolve(
                    "postgres.tls.identity.cert_chain",
                    "postgres.tls.identity.private_key",
                    config_dir,
                )?;
                let ca_cert = resolve_optional_path(
                    "postgres.tls.client_auth.client_ca",
                    client_auth.map(|client_auth| {
                        let _client_certificate_mode = client_auth.client_certificate;
                        client_auth.client_ca
                    }),
                    config_dir,
                )?;
                Ok(Some(TlsConfig { cert, key, ca_cert }))
            }
        }
    }
}

impl raw::DcsTlsConfig {
    fn into_runtime_tls(self, config_dir: &Path) -> Result<Option<TlsConfig>, ConfigErrorV2> {
        match self {
            raw::DcsTlsConfig::Disabled => Ok(None),
            raw::DcsTlsConfig::Enabled { tls, server_name } => {
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

impl raw::ApiClientAuthConfig {
    fn into_runtime_client_auth(
        self,
        config_dir: &Path,
    ) -> Result<(Option<PathBuf>, bool, Vec<String>), ConfigErrorV2> {
        match self {
            raw::ApiClientAuthConfig::Disabled => Ok((None, false, Vec::new())),
            raw::ApiClientAuthConfig::Optional { client_ca } => Ok((
                Some(resolve_path_only(
                    "api.transport.tls.client_auth.client_ca",
                    client_ca,
                    config_dir,
                )?),
                false,
                Vec::new(),
            )),
            raw::ApiClientAuthConfig::Required {
                client_ca,
                allowed_common_names,
            } => Ok((
                Some(resolve_path_only(
                    "api.transport.tls.client_auth.client_ca",
                    client_ca,
                    config_dir,
                )?),
                true,
                allowed_common_names,
            )),
        }
    }
}

impl raw::ApiTransportConfig {
    fn into_runtime_transport(self, config_dir: &Path) -> Result<ApiTransport, ConfigErrorV2> {
        match self {
            raw::ApiTransportConfig::Http => Ok(ApiTransport::Http),
            raw::ApiTransportConfig::Https { tls } => {
                let (client_ca, client_cert_required, allowed_client_common_names) =
                    tls.client_auth.into_runtime_client_auth(config_dir)?;
                let (cert, key) = tls.identity.resolve(
                    "api.transport.tls.identity.cert_chain",
                    "api.transport.tls.identity.private_key",
                    config_dir,
                )?;

                Ok(ApiTransport::Https {
                    tls: TlsConfig {
                        cert,
                        key,
                        ca_cert: None,
                    },
                    client_ca,
                    client_cert_required,
                    allowed_client_common_names,
                })
            }
        }
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
                validation_error("api.auth.read_token", "is required when auth is enabled")
            })?,
            admin_token: admin_token.ok_or_else(|| {
                validation_error("api.auth.admin_token", "is required when auth is enabled")
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
        match tokens {
            Some(tokens) => (
                read_token.or(tokens.read_token),
                admin_token.or(tokens.admin_token),
            ),
            None => (read_token, admin_token),
        }
    }

    fn resolve_tokens(
        self,
        read_field: &'static str,
        admin_field: &'static str,
        config_dir: &Path,
    ) -> Result<Option<ResolvedOptionalTokens>, ConfigErrorV2> {
        if self.is_disabled() {
            return Ok(None);
        }
        let (read_token, admin_token) = self.into_token_sources();
        Ok(Some((
            resolve_secret_optional(read_field, read_token, config_dir)?,
            resolve_secret_optional(admin_field, admin_token, config_dir)?,
        )))
    }
}

impl raw::ClientTlsInput {
    fn into_runtime_dcs_tls(self, config_dir: &Path) -> Result<TlsConfig, ConfigErrorV2> {
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
            ca_cert: resolve_optional_path("dcs.client.tls.ca_cert", self.ca_cert, config_dir)?,
        })
    }

    fn into_pg_client_tls(
        self,
        ca_field: &'static str,
        cert_field: &'static str,
        key_field: &'static str,
        config_dir: &Path,
    ) -> Result<Option<PgClientTls>, ConfigErrorV2> {
        let root_cert = resolve_optional_path(ca_field, self.ca_cert, config_dir)?;
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

fn resolve_path_pair(
    cert_field: &'static str,
    cert: raw::PathSource,
    key_field: &'static str,
    key: raw::PathSource,
    config_dir: &Path,
) -> Result<(PathBuf, PathBuf), ConfigErrorV2> {
    Ok((
        resolve_path_only(cert_field, cert, config_dir)?,
        resolve_path_only(key_field, key, config_dir)?,
    ))
}

impl raw::TlsServerIdentityConfig {
    fn resolve(
        self,
        cert_field: &'static str,
        key_field: &'static str,
        config_dir: &Path,
    ) -> Result<(PathBuf, PathBuf), ConfigErrorV2> {
        resolve_path_pair(
            cert_field,
            self.cert_chain,
            key_field,
            self.private_key,
            config_dir,
        )
    }
}

impl raw::TlsClientIdentityConfig {
    fn resolve(
        self,
        cert_field: &'static str,
        key_field: &'static str,
        config_dir: &Path,
    ) -> Result<(PathBuf, PathBuf), ConfigErrorV2> {
        resolve_path_pair(cert_field, self.cert, key_field, self.key, config_dir)
    }
}

impl raw::OperatorDocument {
    fn into_operator_config(
        self,
        path: &Path,
        resolve_auth_tokens: bool,
    ) -> Result<OperatorConfigV2, ConfigErrorV2> {
        #[rustfmt::skip]
        let raw::OperatorDocument { api, postgres: raw::OperatorPostgresConfig { tls: postgres_tls_input } } = self;
        #[rustfmt::skip]
        let raw::OperatorApiConfig { base_url, advertised_url, expected_transport, resolve_to, auth, tls: api_tls_input } = api;
        let config_dir = resolve_config_dir(path)?;
        let (read_token, admin_token) =
            auth.into_operator_tokens(resolve_auth_tokens, config_dir.as_path())?;
        let api_tls = api_tls_input.into_pg_client_tls(
            "pgtm.api.tls.ca_cert",
            "pgtm.api.tls.identity.cert",
            "pgtm.api.tls.identity.key",
            config_dir.as_path(),
        )?;
        let postgres_tls = postgres_tls_input.into_pg_client_tls(
            "pgtm.postgres.tls.ca_cert",
            "pgtm.postgres.tls.identity.cert",
            "pgtm.postgres.tls.identity.key",
            config_dir.as_path(),
        )?;

        #[rustfmt::skip]
        let operator_config = OperatorConfigV2 { base_url: parse_operator_url("pgtm.api.base_url", base_url, expected_transport)?, advertised_url: parse_operator_url("pgtm.api.advertised_url", advertised_url, expected_transport)?.map(|url| ApiRoute::from_url(url).map_err(|err| validation_error("pgtm.api.advertised_url", err))).transpose()?, expected_transport, resolve_to, client_tls: merge_operator_client_tls(api_tls, postgres_tls)?, read_token, admin_token };
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

fn normalize_config_path(
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

fn normalize_path_or_default<F>(
    field: &'static str,
    path: Option<PathBuf>,
    default: F,
    config_dir: &Path,
) -> Result<PathBuf, ConfigErrorV2>
where
    F: FnOnce() -> PathBuf,
{
    path.map(|path| normalize_config_path(field, path, config_dir))
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
    };
    use crate::{
        config_v2::{ConfigErrorV2, PgtmApiTransportExpectation},
        dev_support::test_fs::unique_test_dir,
        pginfo::conninfo::PgSslMode,
    };
    use std::{fs, net::SocketAddr, os::unix::fs::PermissionsExt, path::Path, time::Duration};

    fn join_rendered_sections<J, T>(base: String, extra_sections: J) -> String
    where
        J: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        std::iter::once(base)
            .chain(extra_sections.into_iter().filter_map(|section| {
                let section = section.as_ref().trim();
                (!section.is_empty()).then_some(section.to_string())
            }))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn toml_string(value: &str) -> String {
        toml::Value::String(value.to_string()).to_string()
    }

    fn toml_path_source(path: &Path) -> String {
        format!(
            "{{ path = {} }}",
            toml_string(path.display().to_string().as_str())
        )
    }

    fn toml_string_secret(value: &str) -> String {
        format!(r#"{{ type = "string", value = {} }}"#, toml_string(value))
    }

    fn render_runtime_test_config_toml<I, S, J, T>(
        cluster_name: &str,
        scope: &str,
        member_id: &str,
        paths: (&Path, &Path, &Path),
        dcs_endpoints: I,
        extra_sections: J,
    ) -> String
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
        J: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let (data_dir, socket_dir, log_file) = paths;
        let endpoints = dcs_endpoints
            .into_iter()
            .map(|endpoint| format!("  {}", toml_string(endpoint.as_ref())))
            .collect::<Vec<_>>()
            .join(",\n");

        join_rendered_sections(
            format!(
                r#"cluster.name = {cluster_name}
cluster.scope = {scope}
cluster.member_id = {member_id}
postgres.paths.data_dir = {data_dir}
postgres.paths.socket_dir = {socket_dir}
postgres.paths.log_file = {log_file}
postgres.roles.mandatory.superuser.username = "postgres"
postgres.roles.mandatory.superuser.auth.type = "password"
postgres.roles.mandatory.superuser.auth.password = {{ type = "string", value = "postgres" }}
postgres.roles.mandatory.replicator.username = "replicator"
postgres.roles.mandatory.replicator.auth.type = "password"
postgres.roles.mandatory.replicator.auth.password = {{ type = "string", value = "replicator" }}
postgres.roles.mandatory.rewinder.username = "rewinder"
postgres.roles.mandatory.rewinder.auth.type = "password"
postgres.roles.mandatory.rewinder.auth.password = {{ type = "string", value = "rewinder" }}
postgres.access.hba = {{ content = "host all all 127.0.0.1/32 trust" }}
postgres.access.ident = {{ content = "" }}
dcs.endpoints = [
{endpoints}
]"#,
                cluster_name = toml_string(cluster_name),
                scope = toml_string(scope),
                member_id = toml_string(member_id),
                data_dir = toml_string(data_dir.display().to_string().as_str()),
                socket_dir = toml_string(socket_dir.display().to_string().as_str()),
                log_file = toml_string(log_file.display().to_string().as_str()),
                endpoints = endpoints,
            ),
            extra_sections,
        )
    }

    fn render_operator_test_config_toml<J, T>(
        base_url: Option<&str>,
        advertised_url: Option<&str>,
        expected_transport: Option<&str>,
        resolve_to: Option<SocketAddr>,
        extra_sections: J,
    ) -> String
    where
        J: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        join_rendered_sections(
            format!(
                r#"[api]
{}{}{}{}
"#,
                base_url
                    .map(|value| format!("base_url = {}\n", toml_string(value)))
                    .unwrap_or_default(),
                advertised_url
                    .map(|value| format!("advertised_url = {}\n", toml_string(value)))
                    .unwrap_or_default(),
                expected_transport
                    .map(|value| format!("expected_transport = {}\n", toml_string(value)))
                    .unwrap_or_default(),
                resolve_to
                    .map(|value| format!(
                        "resolve_to = {}\n",
                        toml_string(value.to_string().as_str())
                    ))
                    .unwrap_or_default(),
            )
            .trim_end()
            .to_string(),
            extra_sections,
        )
    }

    fn runtime_config_contents_with_zero_runtime_defaults(root: &Path) -> String {
        render_default_runtime_test_config_toml(
            root,
            [r#"postgres.connect_timeout_s = 0
postgres.network.listen_host = "   "
postgres.network.listen_port = 0
ha.loop_interval_ms = 0
ha.lease_ttl_ms = 0
process.timeouts.pg_rewind_ms = 0
process.timeouts.bootstrap_ms = 0
process.timeouts.fencing_ms = 0
process.binaries.overrides.pg_ctl = "/bin/true"
process.binaries.overrides.initdb = "/bin/true"
process.binaries.overrides.pg_rewind = "/bin/true"
process.binaries.overrides.pg_basebackup = "/bin/true"
logging.capture_subprocess_output = true
logging.postgres.enabled = true
logging.postgres.poll_interval_ms = 0
logging.postgres.cleanup.enabled = true
logging.postgres.cleanup.max_files = 0
logging.postgres.cleanup.max_age_seconds = 0
logging.postgres.cleanup.protect_recent_seconds = 0"#],
        )
    }

    fn render_default_runtime_test_config_toml<J, T>(root: &Path, extra_sections: J) -> String
    where
        J: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let data_dir = root.join("data");
        render_runtime_test_config_toml(
            "cluster-a",
            "scope-a",
            "node-a",
            (
                data_dir.as_path(),
                Path::new("/tmp/pgtm-socket"),
                Path::new("/tmp/pgtm.log"),
            ),
            ["http://127.0.0.1:2379"],
            extra_sections,
        )
    }

    #[test]
    fn load_runtime_config_preserves_runtime_tls_and_operator_api_fields() -> Result<(), String> {
        let root = unique_test_dir("load-config", "runtime-config-v2-runtime-fields")?;
        let ca_cert = root.join("source-ca.crt");
        let config = load_runtime_config_contents(
            render_default_runtime_test_config_toml(
                root.as_path(),
                [format!(
                    r#"[postgres.rewind.transport]
ssl_mode = "verify-full"
ca_cert = {}

[process.binaries.overrides]
pg_ctl = "/bin/true"
initdb = "/bin/true"
pg_rewind = "/bin/true"
pg_basebackup = "/bin/true"

[pgtm.api]
advertised_url = "https://127.0.0.1:18081"
expected_transport = "{}""#,
                    toml_path_source(ca_cert.as_path()),
                    PgtmApiTransportExpectation::Https.scheme(),
                )],
            )
            .as_str(),
        )
        .map_err(|err| err.to_string())?;

        assert_eq!(
            (
                config.postgres.source_client_tls.mode,
                config.postgres.source_client_tls.root_cert,
            ),
            (PgSslMode::VerifyFull, Some(ca_cert.clone()))
        );
        assert_eq!(config.postgres.replicator.username, "replicator");
        assert_eq!(config.postgres.rewinder.username, "rewinder");
        if config
            .api
            .advertise
            .as_ref()
            .map(crate::state::ApiRoute::as_str)
            != Some("https://127.0.0.1:18081/")
        {
            return Err(format!(
                "unexpected runtime advertised API route: {:?}",
                config.api.advertise
            ));
        }

        Ok(())
    }

    #[test]
    fn load_runtime_config_and_timing_values_normalize_zero_runtime_fields() -> Result<(), String> {
        let root = unique_test_dir("load-config", "runtime-config-v2-zero-defaults")?;
        let contents = runtime_config_contents_with_zero_runtime_defaults(root.as_path());
        let config =
            load_runtime_config_contents(contents.as_str()).map_err(|err| err.to_string())?;

        assert_eq!(
            (
                config.postgres.listen_host.as_str(),
                config.postgres.listen_port,
                config.postgres.cluster_advertise.host(),
                config.postgres.cluster_advertise.port(),
                config.postgres.connect_timeout,
                config.timing.bootstrap_timeout,
                config.logging.postgres_log_poll_interval,
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
        assert_eq!(config.logging.postgres_log_cleanup_max_files, 50);
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
            render_default_runtime_test_config_toml(
                root.as_path(),
                [r#"[postgres.tls]
mode = "enabled"
identity = { cert_chain = { content = "CERT" }, private_key = { content = "KEY" } }"#],
            )
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
                [r#"[api.tls]
identity = { cert = { path = "/tmp/client.crt" }, key = { type = "env", env = "CLIENT_KEY" } }"#],
            )
            .as_str(),
        ) {
            Err(ConfigErrorV2::Parse { .. }) => Ok(()),
            Err(err) => Err(format!("expected parse error, got {err}")),
            Ok(_) => Err("expected non-path TLS identity rejection".to_string()),
        }
    }

    #[test]
    fn load_operator_config_preserves_api_routing_fields_for_operator_and_runtime_documents(
    ) -> Result<(), String> {
        let operator = load_operator_config_contents(
            render_operator_test_config_toml(
                Some("https://node-b:8443"),
                Some("https://127.0.0.1:18081"),
                Some("https"),
                Some(SocketAddr::from(([127, 0, 0, 1], 18443))),
                std::iter::empty::<String>(),
            )
            .as_str(),
        )
        .map_err(|err| err.to_string())?;

        assert_eq!(
            (
                operator.base_url.as_ref().map(reqwest::Url::as_str),
                operator.expected_transport,
                operator.resolve_to,
                operator
                    .advertised_url
                    .as_ref()
                    .map(crate::state::ApiRoute::as_str),
            ),
            (
                Some("https://node-b:8443/"),
                Some(PgtmApiTransportExpectation::Https),
                Some(SocketAddr::from(([127, 0, 0, 1], 18443))),
                Some("https://127.0.0.1:18081/"),
            )
        );

        let runtime_document = render_runtime_test_config_toml(
            "cluster-a",
            "scope-a",
            "node-a",
            (
                Path::new("/tmp/data"),
                Path::new("/tmp/pgtm-socket"),
                Path::new("/tmp/pgtm.log"),
            ),
            ["http://127.0.0.1:2379"],
            [format!(
                r#"[pgtm.api]
base_url = "https://127.0.0.1:8443"
expected_transport = "{}""#,
                PgtmApiTransportExpectation::Https.scheme(),
            )],
        );

        let runtime = load_runtime_config_contents(runtime_document.as_str())
            .map_err(|err| err.to_string())?;
        assert_eq!(
            runtime
                .api
                .advertise
                .as_ref()
                .map(crate::state::ApiRoute::as_str),
            None
        );

        let operator = load_operator_config_contents(runtime_document.as_str())
            .map_err(|err| err.to_string())?;
        assert_eq!(
            operator.expected_transport,
            Some(PgtmApiTransportExpectation::Https)
        );
        Ok(())
    }

    #[test]
    fn load_operator_config_flattens_tokens_and_merges_client_tls() -> Result<(), String> {
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

[api.tls]
ca_cert = {}
identity = {{ cert = {}, key = {} }}

[postgres.tls]
ca_cert = {}
identity = {{ cert = {}, key = {} }}"#,
                    toml_string_secret("read-token"),
                    toml_string_secret("admin-token"),
                    toml_path_source(api_ca_path.as_path()),
                    toml_path_source(identity_cert_path.as_path()),
                    toml_path_source(identity_key_path.as_path()),
                    toml_path_source(api_ca_path.as_path()),
                    toml_path_source(identity_cert_path.as_path()),
                    toml_path_source(identity_key_path.as_path()),
                )],
            )
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
