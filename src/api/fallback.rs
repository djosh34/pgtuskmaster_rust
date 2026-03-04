use serde::{Deserialize, Serialize};

use crate::{
    api::{AcceptedResponse, ApiError, ApiResult},
    config::RuntimeConfig,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct FallbackClusterView {
    pub(crate) name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FallbackHeartbeatInput {
    pub(crate) source: String,
}

pub(crate) fn get_fallback_cluster(cfg: &RuntimeConfig) -> FallbackClusterView {
    FallbackClusterView {
        name: cfg.cluster.name.clone(),
    }
}

pub(crate) fn post_fallback_heartbeat(
    input: FallbackHeartbeatInput,
) -> ApiResult<AcceptedResponse> {
    if input.source.trim().is_empty() {
        return Err(ApiError::bad_request("source must be non-empty"));
    }
    Ok(AcceptedResponse { accepted: true })
}

#[cfg(test)]
mod tests {
    use crate::{
        api::fallback::{get_fallback_cluster, post_fallback_heartbeat, FallbackHeartbeatInput},
        config::{
            schema::ClusterConfig, LogCleanupConfig, LogLevel, LoggingConfig, PostgresLoggingConfig,
            RuntimeConfig,
        },
    };

    fn sample_runtime_config() -> RuntimeConfig {
        RuntimeConfig {
            cluster: ClusterConfig {
                name: "cluster-a".to_string(),
                member_id: "node-a".to_string(),
            },
            postgres: crate::config::schema::PostgresConfig {
                data_dir: "/tmp/pgdata".into(),
                connect_timeout_s: 5,
                listen_host: "127.0.0.1".to_string(),
                listen_port: 5432,
                socket_dir: "/tmp/pgtuskmaster/socket".into(),
                log_file: "/tmp/pgtuskmaster/postgres.log".into(),
                rewind_source_host: "127.0.0.1".to_string(),
                rewind_source_port: 5432,
            },
            dcs: crate::config::schema::DcsConfig {
                endpoints: vec!["http://127.0.0.1:2379".to_string()],
                scope: "scope-a".to_string(),
            },
            ha: crate::config::schema::HaConfig {
                loop_interval_ms: 1000,
                lease_ttl_ms: 10_000,
            },
            process: crate::config::ProcessConfig {
                pg_rewind_timeout_ms: 1000,
                bootstrap_timeout_ms: 1000,
                fencing_timeout_ms: 1000,
                binaries: crate::config::BinaryPaths {
                    postgres: "/usr/bin/postgres".into(),
                    pg_ctl: "/usr/bin/pg_ctl".into(),
                    pg_rewind: "/usr/bin/pg_rewind".into(),
                    initdb: "/usr/bin/initdb".into(),
                    pg_basebackup: "/usr/bin/pg_basebackup".into(),
                    psql: "/usr/bin/psql".into(),
                },
            },
            logging: LoggingConfig {
                level: LogLevel::Info,
                capture_subprocess_output: true,
                postgres: PostgresLoggingConfig {
                    enabled: true,
                    pg_ctl_log_file: None,
                    log_dir: None,
                    archive_command_log_file: None,
                    poll_interval_ms: 200,
                    cleanup: LogCleanupConfig {
                        enabled: true,
                        max_files: 10,
                        max_age_seconds: 60,
                    },
                },
            },
            api: crate::config::schema::ApiConfig {
                listen_addr: "127.0.0.1:8080".to_string(),
                read_auth_token: None,
                admin_auth_token: None,
            },
            debug: crate::config::schema::DebugConfig { enabled: true },
            security: crate::config::schema::SecurityConfig {
                tls_enabled: false,
                auth_token: None,
            },
        }
    }

    #[test]
    fn heartbeat_denies_unknown_fields() {
        let raw = r#"{"source":"x","extra":1}"#;
        let parsed = serde_json::from_str::<FallbackHeartbeatInput>(raw);
        assert!(parsed.is_err());
    }

    #[test]
    fn get_cluster_returns_config_name() {
        let cfg = sample_runtime_config();
        let view = get_fallback_cluster(&cfg);
        assert_eq!(view.name, "cluster-a");
    }

    #[test]
    fn heartbeat_rejects_empty_source() {
        let result = post_fallback_heartbeat(FallbackHeartbeatInput {
            source: "   ".to_string(),
        });
        assert!(matches!(result, Err(crate::api::ApiError::BadRequest(_))));
    }
}
