use std::{fs, path::PathBuf, time::Duration};

use crate::{
    config::{
        resolve_inline_or_path_bytes, resolve_inline_or_path_string, resolve_secret_string,
        ApiAuthConfig, ApiClientAuthConfig, ApiTransportConfig, DcsTlsConfig,
        FileSinkMode as LegacyFileSinkMode, InlineOrPath, PostgresBinaryName,
        PostgresClientTransportConfig, PostgresRoleConfig, RoleAuthConfig, RuntimeConfig,
        SecretSource, TlsServerConfig,
    },
    config_v2::types::{
        ApiAuth, ApiConfig, ApiTransport, BinariesConfig, ConfigErrorV2, DcsAuth, DcsConfig,
        DcsEndpoint, FileSinkMode, LogLevel, LoggingConfig, PostgresConfig, RoleConfig,
        RuntimeConfigV2, Secret, TimingConfig, TlsConfig,
    },
    pginfo::conninfo::PgClientTls,
};

pub(crate) fn from_legacy_runtime_config(cfg: RuntimeConfig) -> Result<RuntimeConfigV2, String> {
    let postgres_tls = map_postgres_tls(&cfg)?;
    Ok(RuntimeConfigV2 {
        cluster_name: crate::state::ClusterName(cfg.cluster.name.clone()),
        scope: crate::state::ScopeName(cfg.cluster.scope.clone()),
        member_id: crate::state::MemberId(cfg.cluster.member_id.clone()),
        postgres: PostgresConfig {
            data_dir: cfg.postgres.paths.data_dir.clone(),
            socket_dir: cfg.postgres_socket_dir(),
            log_file: cfg.postgres_log_file(),
            listen_host: cfg.postgres.network.listen_host.clone(),
            listen_port: cfg.postgres.network.listen_port,
            advertise_port: cfg
                .postgres
                .network
                .advertise_port
                .unwrap_or(cfg.postgres.network.listen_port),
            connect_timeout: Duration::from_secs(u64::from(cfg.postgres.connect_timeout_s)),
            local_database: cfg.postgres.local_database.clone(),
            source_client_tls: map_postgres_source_client_tls(&cfg)?,
            superuser: map_role(
                "postgres.roles.mandatory.superuser.auth.password",
                &cfg.postgres.roles.mandatory.superuser,
            )?,
            replicator: map_role(
                "postgres.roles.mandatory.replicator.auth.password",
                &cfg.postgres.roles.mandatory.replicator,
            )?,
            rewinder: map_role(
                "postgres.roles.mandatory.rewinder.auth.password",
                &cfg.postgres.roles.mandatory.rewinder,
            )?,
            pg_hba_file: cfg.postgres.paths.data_dir.join("pgtm.pg_hba.conf"),
            pg_ident_file: cfg.postgres.paths.data_dir.join("pgtm.pg_ident.conf"),
            pg_hba_contents: resolve_inline_or_path_string(
                "postgres.access.hba",
                &cfg.postgres.access.hba,
            )
            .map_err(|err| err.to_string())?,
            pg_ident_contents: resolve_inline_or_path_string(
                "postgres.access.ident",
                &cfg.postgres.access.ident,
            )
            .map_err(|err| err.to_string())?,
            extra_gucs: cfg.postgres.extra_gucs.clone(),
            tls: postgres_tls,
        },
        dcs: DcsConfig {
            endpoints: cfg
                .dcs
                .endpoints
                .iter()
                .map(|endpoint| DcsEndpoint::new(endpoint.to_string()))
                .collect(),
            auth: map_dcs_auth(&cfg)?,
            tls: map_dcs_tls(&cfg)?,
        },
        timing: TimingConfig {
            ha_loop_interval: Duration::from_millis(cfg.ha.loop_interval_ms),
            ha_lease_ttl: Duration::from_millis(cfg.ha.lease_ttl_ms),
            bootstrap_timeout: Duration::from_millis(cfg.process.timeouts.bootstrap_ms),
            pg_rewind_timeout: Duration::from_millis(cfg.process.timeouts.pg_rewind_ms),
            fencing_timeout: Duration::from_millis(cfg.process.timeouts.fencing_ms),
        },
        binaries: BinariesConfig {
            postgres: resolve_binary_path(&cfg, PostgresBinaryName::Postgres)?,
            pg_ctl: resolve_binary_path(&cfg, PostgresBinaryName::PgCtl)?,
            initdb: resolve_binary_path(&cfg, PostgresBinaryName::Initdb)?,
            pg_rewind: resolve_binary_path(&cfg, PostgresBinaryName::PgRewind)?,
            pg_basebackup: resolve_binary_path(&cfg, PostgresBinaryName::PgBasebackup)?,
            psql: resolve_binary_path(&cfg, PostgresBinaryName::Psql)?,
        },
        logging: LoggingConfig {
            level: map_log_level(cfg.logging.level),
            capture_subprocess_output: cfg.logging.capture_subprocess_output,
            stderr_enabled: cfg.logging.sinks.stderr.enabled,
            file_enabled: cfg.logging.sinks.file.enabled,
            file_path: cfg
                .logging
                .sinks
                .file
                .path
                .clone()
                .unwrap_or_else(|| cfg.process.working_root.join("runtime.jsonl")),
            file_mode: match cfg.logging.sinks.file.mode {
                LegacyFileSinkMode::Append => FileSinkMode::Append,
                LegacyFileSinkMode::Truncate => FileSinkMode::Truncate,
            },
            postgres_logs_enabled: cfg.logging.postgres.enabled,
            postgres_log_dir: cfg
                .logging
                .postgres
                .log_dir
                .clone()
                .unwrap_or_else(|| cfg.process.working_root.join("logs/postgres")),
            postgres_pg_ctl_log: cfg
                .logging
                .postgres
                .pg_ctl_log_file
                .clone()
                .unwrap_or_else(|| cfg.postgres_log_file()),
            postgres_log_poll_interval: Duration::from_millis(
                cfg.logging.postgres.poll_interval_ms,
            ),
            postgres_log_cleanup_enabled: cfg.logging.postgres.cleanup.enabled,
            postgres_log_cleanup_max_files: cfg.logging.postgres.cleanup.max_files,
            postgres_log_cleanup_max_age: Duration::from_secs(
                cfg.logging.postgres.cleanup.max_age_seconds,
            ),
            postgres_log_cleanup_protect_recent: Duration::from_secs(
                cfg.logging.postgres.cleanup.protect_recent_seconds,
            ),
        },
        api: ApiConfig {
            listen_addr: cfg.api.listen_addr,
            transport: map_api_transport(&cfg)?,
            auth: match &cfg.api.auth {
                ApiAuthConfig::Disabled => ApiAuth::Disabled,
                ApiAuthConfig::RoleTokens(tokens) => ApiAuth::Tokens {
                    read_token: Secret::new(
                        resolve_secret_string("api.auth.read_token", &tokens.read_token)
                            .map_err(|err| err.to_string())?,
                    ),
                    admin_token: Secret::new(
                        resolve_secret_string("api.auth.admin_token", &tokens.admin_token)
                            .map_err(|err| err.to_string())?,
                    ),
                },
            },
        },
    })
}

fn map_role(field: &str, role: &PostgresRoleConfig) -> Result<RoleConfig, String> {
    let RoleAuthConfig::Password { password } = &role.auth;
    Ok(RoleConfig {
        username: role.username.as_str().to_string(),
        password: Secret::new(
            resolve_secret_string(field, password).map_err(|err| err.to_string())?,
        ),
    })
}

fn map_postgres_source_client_tls(cfg: &RuntimeConfig) -> Result<PgClientTls, String> {
    map_legacy_postgres_client_transport(
        "postgres.rewind.transport.ca_cert",
        cfg.postgres.rewind.transport.clone(),
        cfg.postgres.paths.data_dir.join("pgtm.test.source-ca.crt"),
    )
}

fn map_postgres_tls(cfg: &RuntimeConfig) -> Result<Option<TlsConfig>, String> {
    match &cfg.postgres.tls {
        TlsServerConfig::Disabled => Ok(None),
        TlsServerConfig::Enabled {
            identity,
            client_auth,
        } => Ok(Some(TlsConfig {
            cert: materialize_path_or_inline(
                "postgres.tls.identity.cert_chain",
                &identity.cert_chain,
                cfg.postgres
                    .paths
                    .data_dir
                    .join("pgtm.test.source.server.crt"),
            )?,
            key: materialize_path_or_inline(
                "postgres.tls.identity.private_key",
                &identity.private_key,
                cfg.postgres
                    .paths
                    .data_dir
                    .join("pgtm.test.source.server.key"),
            )?,
            ca_cert: client_auth
                .as_ref()
                .map(|client_auth| {
                    materialize_path_or_inline(
                        "postgres.tls.client_auth.client_ca",
                        &client_auth.client_ca,
                        cfg.postgres
                            .paths
                            .data_dir
                            .join("pgtm.test.source.client-ca.crt"),
                    )
                })
                .transpose()?,
        })),
    }
}

fn map_dcs_auth(cfg: &RuntimeConfig) -> Result<Option<DcsAuth>, String> {
    match &cfg.dcs.client.auth {
        crate::config::DcsAuthConfig::Disabled => Ok(None),
        crate::config::DcsAuthConfig::Basic { username, password } => Ok(Some(DcsAuth {
            username: username.clone(),
            password: Secret::new(
                resolve_secret_string("dcs.client.auth.password", password)
                    .map_err(|err| err.to_string())?,
            ),
        })),
    }
}

fn map_dcs_tls(cfg: &RuntimeConfig) -> Result<Option<TlsConfig>, String> {
    match &cfg.dcs.client.tls {
        DcsTlsConfig::Disabled => Ok(None),
        DcsTlsConfig::Enabled {
            ca_cert,
            identity,
            server_name,
        } => {
            if server_name.is_some() {
                return Err(validation_error(
                    "dcs.client.tls.server_name",
                    "is not supported by config_v2",
                ));
            }

            let Some(identity) = identity else {
                return Err(validation_error(
                    "dcs.client.tls.identity",
                    "enabled DCS TLS currently requires a client identity",
                ));
            };

            Ok(Some(TlsConfig {
                cert: materialize_path_or_inline(
                    "dcs.client.tls.identity.cert",
                    &identity.cert,
                    cfg.postgres.paths.data_dir.join("pgtm.test.dcs.client.crt"),
                )?,
                key: materialize_secret_or_path(
                    "dcs.client.tls.identity.key",
                    &identity.key,
                    cfg.postgres.paths.data_dir.join("pgtm.test.dcs.client.key"),
                )?,
                ca_cert: ca_cert
                    .as_ref()
                    .map(|ca_cert| {
                        materialize_path_or_inline(
                            "dcs.client.tls.ca_cert",
                            ca_cert,
                            cfg.postgres.paths.data_dir.join("pgtm.test.dcs.ca.crt"),
                        )
                    })
                    .transpose()?,
            }))
        }
    }
}

fn map_api_transport(cfg: &RuntimeConfig) -> Result<ApiTransport, String> {
    match &cfg.api.transport {
        ApiTransportConfig::Http => Ok(ApiTransport::Http),
        ApiTransportConfig::Https { tls } => {
            let (client_ca, client_cert_required, allowed_client_common_names) =
                match &tls.client_auth {
                    ApiClientAuthConfig::Disabled => (None, false, Vec::new()),
                    ApiClientAuthConfig::Optional { client_ca } => (
                        Some(materialize_path_or_inline(
                            "api.transport.tls.client_auth.client_ca",
                            client_ca,
                            cfg.postgres
                                .paths
                                .data_dir
                                .join("pgtm.test.api.client-ca.crt"),
                        )?),
                        false,
                        Vec::new(),
                    ),
                    ApiClientAuthConfig::Required {
                        client_ca,
                        allowed_common_names,
                    } => (
                        Some(materialize_path_or_inline(
                            "api.transport.tls.client_auth.client_ca",
                            client_ca,
                            cfg.postgres
                                .paths
                                .data_dir
                                .join("pgtm.test.api.client-ca.crt"),
                        )?),
                        true,
                        allowed_common_names
                            .iter()
                            .map(|name| name.0.clone())
                            .collect(),
                    ),
                };

            Ok(ApiTransport::Https {
                tls: TlsConfig {
                    cert: materialize_path_or_inline(
                        "api.transport.tls.identity.cert_chain",
                        &tls.identity.cert_chain,
                        cfg.postgres.paths.data_dir.join("pgtm.test.api.server.crt"),
                    )?,
                    key: materialize_path_or_inline(
                        "api.transport.tls.identity.private_key",
                        &tls.identity.private_key,
                        cfg.postgres.paths.data_dir.join("pgtm.test.api.server.key"),
                    )?,
                    ca_cert: None,
                },
                client_ca,
                client_cert_required,
                allowed_client_common_names,
            })
        }
    }
}

fn resolve_binary_path(
    cfg: &RuntimeConfig,
    binary: PostgresBinaryName,
) -> Result<std::path::PathBuf, String> {
    cfg.process.binaries.resolve_binary_path(binary)
}

fn map_log_level(level: crate::config::LogLevel) -> LogLevel {
    match level {
        crate::config::LogLevel::Trace => LogLevel::Trace,
        crate::config::LogLevel::Debug => LogLevel::Debug,
        crate::config::LogLevel::Info => LogLevel::Info,
        crate::config::LogLevel::Warn => LogLevel::Warn,
        crate::config::LogLevel::Error => LogLevel::Error,
        crate::config::LogLevel::Fatal => LogLevel::Fatal,
    }
}

fn materialize_path_or_inline(
    field: &str,
    source: &InlineOrPath,
    target: PathBuf,
) -> Result<PathBuf, String> {
    match source {
        InlineOrPath::Path(path) | InlineOrPath::PathConfig { path } => Ok(path.clone()),
        InlineOrPath::Inline { .. } => {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .map_err(|err| format!("create {} failed: {err}", parent.display()))?;
            }
            let bytes =
                resolve_inline_or_path_bytes(field, source).map_err(|err| err.to_string())?;
            fs::write(&target, bytes)
                .map_err(|err| format!("write {} failed: {err}", target.display()))?;
            Ok(target)
        }
    }
}

fn materialize_secret_or_path(
    field: &'static str,
    source: &SecretSource,
    target: PathBuf,
) -> Result<PathBuf, String> {
    match source {
        SecretSource::File { path } => Ok(path.clone()),
        SecretSource::None => Err(validation_error(field, "must not be empty")),
        SecretSource::String { .. } | SecretSource::Env { .. } => {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .map_err(|err| format!("create {} failed: {err}", parent.display()))?;
            }
            let secret = resolve_secret_string(field, source).map_err(|err| err.to_string())?;
            if secret.trim().is_empty() {
                return Err(validation_error(field, "must not be empty"));
            }
            fs::write(&target, secret)
                .map_err(|err| format!("write {} failed: {err}", target.display()))?;
            Ok(target)
        }
    }
}

fn validation_error(field: &'static str, message: impl Into<String>) -> String {
    ConfigErrorV2::Validation {
        field,
        message: message.into(),
    }
    .to_string()
}

fn map_legacy_postgres_client_transport(
    ca_field: &str,
    transport: PostgresClientTransportConfig,
    inline_target: PathBuf,
) -> Result<PgClientTls, String> {
    Ok(PgClientTls {
        mode: transport.ssl_mode,
        root_cert: transport
            .ca_cert
            .as_ref()
            .map(|ca_cert| materialize_path_or_inline(ca_field, ca_cert, inline_target))
            .transpose()?,
        client_cert: None,
        client_key: None,
    })
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        config::{
            ApiClientAuthConfig, ApiConfig, ApiTlsConfig, ApiTransportConfig, ClientCommonName,
            DcsClientConfig, DcsConfig, DcsTlsConfig, InlineOrPath, RuntimeConfig, SecretSource,
            TlsClientIdentityConfig, TlsServerIdentityConfig,
        },
        config_v2::types::ApiTransport,
        dev_support::{runtime_config::RuntimeConfigBuilder, tls::build_adversarial_tls_fixture},
    };

    use super::from_legacy_runtime_config;

    fn unique_test_dir(label: &str) -> Result<PathBuf, String> {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("clock error: {err}"))?
            .as_millis();
        let dir = std::env::temp_dir().join(format!(
            "pgtm-runtime-config-v2-{label}-{}-{millis}",
            std::process::id()
        ));
        fs::create_dir_all(&dir)
            .map_err(|err| format!("create test dir {} failed: {err}", dir.display()))?;
        Ok(dir)
    }

    fn remove_dir_if_exists(path: &Path) -> Result<(), String> {
        match fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(format!("remove {} failed: {err}", path.display())),
        }
    }

    fn read_string(path: &Path) -> Result<String, String> {
        fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))
    }

    #[test]
    fn preserves_api_https_transport_and_client_auth_details() -> Result<(), String> {
        let data_dir = unique_test_dir("api-https")?;
        let fixture = build_adversarial_tls_fixture().map_err(|err| err.to_string())?;
        let cfg = RuntimeConfigBuilder::new()
            .with_postgres_data_dir(data_dir.clone())
            .transform_api(|api| ApiConfig {
                transport: ApiTransportConfig::Https {
                    tls: ApiTlsConfig {
                        identity: TlsServerIdentityConfig {
                            cert_chain: InlineOrPath::Inline {
                                content: fixture.valid_server.cert_pem.clone(),
                            },
                            private_key: InlineOrPath::Inline {
                                content: fixture.valid_server.key_pem.clone(),
                            },
                        },
                        client_auth: ApiClientAuthConfig::Required {
                            client_ca: InlineOrPath::Inline {
                                content: fixture.trusted_client_ca.cert.cert_pem.clone(),
                            },
                            allowed_common_names: vec![
                                ClientCommonName("client-a".to_string()),
                                ClientCommonName("client-b".to_string()),
                            ],
                        },
                    },
                },
                ..api
            })
            .build();

        let converted = from_legacy_runtime_config(cfg)?;
        match converted.api.transport {
            ApiTransport::Http => Err("expected HTTPS transport".to_string()),
            ApiTransport::Https {
                tls,
                client_ca,
                client_cert_required,
                allowed_client_common_names,
            } => {
                assert_eq!(read_string(&tls.cert)?, fixture.valid_server.cert_pem);
                assert_eq!(read_string(&tls.key)?, fixture.valid_server.key_pem);
                assert_eq!(
                    read_string(
                        client_ca
                            .as_ref()
                            .ok_or_else(|| "expected client CA path".to_string())?,
                    )?,
                    fixture.trusted_client_ca.cert.cert_pem
                );
                assert!(tls.ca_cert.is_none());
                assert!(client_cert_required);
                assert_eq!(
                    allowed_client_common_names,
                    vec!["client-a".to_string(), "client-b".to_string()]
                );
                remove_dir_if_exists(data_dir.as_path())?;
                Ok(())
            }
        }
    }

    #[test]
    fn preserves_representable_dcs_tls() -> Result<(), String> {
        let data_dir = unique_test_dir("dcs-tls")?;
        let fixture = build_adversarial_tls_fixture().map_err(|err| err.to_string())?;
        let cfg = RuntimeConfigBuilder::new()
            .with_postgres_data_dir(data_dir.clone())
            .transform(|runtime| RuntimeConfig {
                dcs: DcsConfig {
                    client: DcsClientConfig {
                        tls: DcsTlsConfig::Enabled {
                            ca_cert: Some(InlineOrPath::Inline {
                                content: fixture.valid_server_ca.cert.cert_pem.clone(),
                            }),
                            identity: Some(TlsClientIdentityConfig {
                                cert: InlineOrPath::Inline {
                                    content: fixture.trusted_client.cert_pem.clone(),
                                },
                                key: SecretSource::String {
                                    value: fixture.trusted_client.key_pem.clone(),
                                },
                            }),
                            server_name: None,
                        },
                        ..runtime.dcs.client
                    },
                    ..runtime.dcs
                },
                ..runtime
            })
            .build();

        let converted = from_legacy_runtime_config(cfg)?;
        let tls = converted
            .dcs
            .tls
            .ok_or_else(|| "expected DCS TLS".to_string())?;
        assert_eq!(read_string(&tls.cert)?, fixture.trusted_client.cert_pem);
        assert_eq!(
            read_string(&tls.key)?,
            fixture
                .trusted_client
                .key_pem
                .trim_end_matches(['\n', '\r'])
        );
        assert_eq!(
            read_string(
                tls.ca_cert
                    .as_ref()
                    .ok_or_else(|| "expected DCS CA cert".to_string())?,
            )?,
            fixture.valid_server_ca.cert.cert_pem
        );
        remove_dir_if_exists(data_dir.as_path())?;
        Ok(())
    }

    #[test]
    fn rejects_dcs_tls_server_name() -> Result<(), String> {
        let cfg = RuntimeConfigBuilder::new()
            .transform(|runtime| RuntimeConfig {
                dcs: DcsConfig {
                    client: DcsClientConfig {
                        tls: DcsTlsConfig::Enabled {
                            ca_cert: None,
                            identity: Some(TlsClientIdentityConfig {
                                cert: InlineOrPath::Path(PathBuf::from("/tmp/client.crt")),
                                key: SecretSource::File {
                                    path: PathBuf::from("/tmp/client.key"),
                                },
                            }),
                            server_name: Some("etcd.internal".to_string()),
                        },
                        ..runtime.dcs.client
                    },
                    ..runtime.dcs
                },
                ..runtime
            })
            .build();

        match from_legacy_runtime_config(cfg) {
            Ok(_) => Err("server_name should be rejected".to_string()),
            Err(err) => {
                assert!(err.contains("dcs.client.tls.server_name"));
                assert!(err.contains("is not supported by config_v2"));
                Ok(())
            }
        }
    }

    #[test]
    fn rejects_dcs_tls_without_identity() -> Result<(), String> {
        let cfg = RuntimeConfigBuilder::new()
            .transform(|runtime| RuntimeConfig {
                dcs: DcsConfig {
                    client: DcsClientConfig {
                        tls: DcsTlsConfig::Enabled {
                            ca_cert: None,
                            identity: None,
                            server_name: None,
                        },
                        ..runtime.dcs.client
                    },
                    ..runtime.dcs
                },
                ..runtime
            })
            .build();

        match from_legacy_runtime_config(cfg) {
            Ok(_) => Err("missing DCS identity should be rejected".to_string()),
            Err(err) => {
                assert!(err.contains("dcs.client.tls.identity"));
                assert!(err.contains("requires a client identity"));
                Ok(())
            }
        }
    }
}
