use std::{fs, path::Path};

use thiserror::Error;

use crate::{
    config::RuntimeConfig,
    process::{
        jobs::{
            BaseBackupSpec, BootstrapSpec, DemoteSpec, FencingSpec, PgRewindSpec, PromoteSpec,
            ShutdownMode, StartPostgresSpec,
        },
        state::{ProcessJobKind, ProcessJobRequest},
    },
    state::JobId,
};

use super::{
    actions::{ActionId, HaAction},
    state::HaWorkerCtx,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProcessDispatchOutcome {
    Applied,
    Skipped,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ProcessDispatchError {
    #[error("process send failed for action `{action:?}`: {message}")]
    ProcessSend { action: ActionId, message: String },
    #[error("managed config materialization failed for action `{action:?}`: {message}")]
    ManagedConfig { action: ActionId, message: String },
    #[error("filesystem operation failed for action `{action:?}`: {message}")]
    Filesystem { action: ActionId, message: String },
    #[error("process dispatch does not support action `{action:?}`")]
    UnsupportedAction { action: ActionId },
}

pub(crate) fn dispatch_process_action(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
    runtime_config: &RuntimeConfig,
) -> Result<ProcessDispatchOutcome, ProcessDispatchError> {
    match action {
        HaAction::AcquireLeaderLease | HaAction::ReleaseLeaderLease | HaAction::ClearSwitchover => {
            Err(ProcessDispatchError::UnsupportedAction {
                action: action.id(),
            })
        }
        HaAction::StartPostgres => {
            let managed =
                crate::postgres_managed::materialize_managed_postgres_config(runtime_config)
                    .map_err(|err| ProcessDispatchError::ManagedConfig {
                        action: action.id(),
                        message: err.to_string(),
                    })?;
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::StartPostgres(StartPostgresSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    host: ctx.process_defaults.postgres_host.clone(),
                    port: ctx.process_defaults.postgres_port,
                    socket_dir: ctx.process_defaults.socket_dir.clone(),
                    log_file: ctx.process_defaults.log_file.clone(),
                    extra_postgres_settings: managed.extra_settings,
                    wait_seconds: None,
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::PromoteToPrimary => {
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Promote(PromoteSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    wait_seconds: None,
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::DemoteToReplica => {
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Demote(DemoteSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    mode: ctx.process_defaults.shutdown_mode.clone(),
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::StartRewind => {
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::PgRewind(PgRewindSpec {
                    target_data_dir: runtime_config.postgres.data_dir.clone(),
                    source: ctx.process_defaults.rewind_source.clone(),
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::StartBaseBackup => {
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::BaseBackup(BaseBackupSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    source: ctx.process_defaults.basebackup_source.clone(),
                    timeout_ms: Some(runtime_config.process.bootstrap_timeout_ms),
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::RunBootstrap => {
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Bootstrap(BootstrapSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    superuser_username: runtime_config.postgres.roles.superuser.username.clone(),
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::FenceNode => {
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Fencing(FencingSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    mode: ShutdownMode::Immediate,
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::WipeDataDir => {
            wipe_data_dir(runtime_config.postgres.data_dir.as_path()).map_err(|message| {
                ProcessDispatchError::Filesystem {
                    action: action.id(),
                    message,
                }
            })?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::FollowLeader { .. } | HaAction::SignalFailSafe => {
            Ok(ProcessDispatchOutcome::Skipped)
        }
    }
}

fn send_process_request(
    ctx: &mut HaWorkerCtx,
    action: ActionId,
    request: ProcessJobRequest,
) -> Result<(), ProcessDispatchError> {
    ctx.process_inbox
        .send(request)
        .map_err(|err| ProcessDispatchError::ProcessSend {
            action,
            message: err.to_string(),
        })
}

fn process_job_id(
    scope: &str,
    self_id: &crate::state::MemberId,
    action: &HaAction,
    index: usize,
    tick: u64,
) -> JobId {
    JobId(format!(
        "ha-{}-{}-{}-{}-{}",
        scope.trim_matches('/'),
        self_id.0,
        tick,
        index,
        action.id().label(),
    ))
}

fn wipe_data_dir(data_dir: &Path) -> Result<(), String> {
    if data_dir.as_os_str().is_empty() {
        return Err("wipe_data_dir data_dir must not be empty".to_string());
    }
    if data_dir.exists() {
        fs::remove_dir_all(data_dir)
            .map_err(|err| format!("wipe_data_dir remove_dir_all failed: {err}"))?;
    }
    fs::create_dir_all(data_dir)
        .map_err(|err| format!("wipe_data_dir create_dir_all failed: {err}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        config::{
            schema::{ClusterConfig, DebugConfig, HaConfig, PostgresConfig},
            ApiAuthConfig, ApiConfig, ApiSecurityConfig, ApiTlsMode, BinaryPaths, DcsConfig,
            InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig, PgHbaConfig, PgIdentConfig,
            PostgresConnIdentityConfig, PostgresLoggingConfig, PostgresRoleConfig,
            PostgresRolesConfig, ProcessConfig, RoleAuthConfig, RuntimeConfig, StderrSinkConfig,
            TlsServerConfig,
        },
        dcs::{
            state::{DcsCache, DcsState, DcsTrust},
            store::{DcsStore, DcsStoreError, WatchEvent},
        },
        ha::{
            actions::HaAction,
            process_dispatch::{dispatch_process_action, ProcessDispatchOutcome},
            state::{HaState, HaWorkerContractStubInputs, HaWorkerCtx},
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, PgSslMode, Readiness, SqlStatus},
        process::state::{ProcessJobKind, ProcessState},
        state::{new_state_channel, MemberId, UnixMillis, WorkerError, WorkerStatus},
    };

    #[derive(Default)]
    struct NoopStore;

    impl DcsStore for NoopStore {
        fn healthy(&self) -> bool {
            true
        }

        fn read_path(&mut self, _path: &str) -> Result<Option<String>, DcsStoreError> {
            Ok(None)
        }

        fn write_path(&mut self, _path: &str, _value: String) -> Result<(), DcsStoreError> {
            Ok(())
        }

        fn put_path_if_absent(
            &mut self,
            _path: &str,
            _value: String,
        ) -> Result<bool, DcsStoreError> {
            Ok(true)
        }

        fn delete_path(&mut self, _path: &str) -> Result<(), DcsStoreError> {
            Ok(())
        }

        fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
            Ok(Vec::new())
        }
    }

    static TEST_DATA_DIR_SEQ: AtomicU64 = AtomicU64::new(0);

    fn unique_test_data_dir(label: &str) -> PathBuf {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis());
        let sequence = TEST_DATA_DIR_SEQ.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "pgtuskmaster-process-dispatch-{label}-{}-{millis}-{sequence}",
            std::process::id(),
        ))
    }

    fn sample_runtime_config(data_dir: PathBuf) -> RuntimeConfig {
        RuntimeConfig {
            cluster: ClusterConfig {
                name: "cluster-a".to_string(),
                member_id: "node-a".to_string(),
            },
            postgres: PostgresConfig {
                data_dir,
                connect_timeout_s: 5,
                listen_host: "127.0.0.1".to_string(),
                listen_port: 5432,
                socket_dir: "/tmp/pgtuskmaster/socket".into(),
                log_file: "/tmp/pgtuskmaster/postgres.log".into(),
                rewind_source_host: "127.0.0.1".to_string(),
                rewind_source_port: 5432,
                local_conn_identity: PostgresConnIdentityConfig {
                    user: "postgres".to_string(),
                    dbname: "postgres".to_string(),
                    ssl_mode: PgSslMode::Prefer,
                },
                rewind_conn_identity: PostgresConnIdentityConfig {
                    user: "rewinder".to_string(),
                    dbname: "postgres".to_string(),
                    ssl_mode: PgSslMode::Prefer,
                },
                tls: TlsServerConfig {
                    mode: ApiTlsMode::Disabled,
                    identity: None,
                    client_auth: None,
                },
                roles: PostgresRolesConfig {
                    superuser: PostgresRoleConfig {
                        username: "postgres".to_string(),
                        auth: RoleAuthConfig::Tls,
                    },
                    replicator: PostgresRoleConfig {
                        username: "replicator".to_string(),
                        auth: RoleAuthConfig::Tls,
                    },
                    rewinder: PostgresRoleConfig {
                        username: "rewinder".to_string(),
                        auth: RoleAuthConfig::Tls,
                    },
                },
                pg_hba: PgHbaConfig {
                    source: InlineOrPath::Inline {
                        content: "local all all trust\n".to_string(),
                    },
                },
                pg_ident: PgIdentConfig {
                    source: InlineOrPath::Inline {
                        content: "# empty\n".to_string(),
                    },
                },
            },
            dcs: DcsConfig {
                endpoints: vec!["http://127.0.0.1:2379".to_string()],
                scope: "scope-a".to_string(),
                init: None,
            },
            ha: HaConfig {
                loop_interval_ms: 1000,
                lease_ttl_ms: 10_000,
            },
            process: ProcessConfig {
                pg_rewind_timeout_ms: 1000,
                bootstrap_timeout_ms: 1000,
                fencing_timeout_ms: 1000,
                backup_timeout_ms: 1000,
                binaries: BinaryPaths {
                    postgres: "/usr/bin/postgres".into(),
                    pg_ctl: "/usr/bin/pg_ctl".into(),
                    pg_rewind: "/usr/bin/pg_rewind".into(),
                    initdb: "/usr/bin/initdb".into(),
                    pg_basebackup: "/usr/bin/pg_basebackup".into(),
                    psql: "/usr/bin/psql".into(),
                    pgbackrest: None,
                },
            },
            backup: crate::config::BackupConfig::default(),
            logging: LoggingConfig {
                level: LogLevel::Info,
                capture_subprocess_output: true,
                postgres: PostgresLoggingConfig {
                    enabled: true,
                    pg_ctl_log_file: None,
                    log_dir: None,
                    poll_interval_ms: 200,
                    cleanup: LogCleanupConfig {
                        enabled: true,
                        max_files: 10,
                        max_age_seconds: 60,
                        protect_recent_seconds: 300,
                    },
                },
                sinks: crate::config::LoggingSinksConfig {
                    stderr: StderrSinkConfig { enabled: true },
                    file: crate::config::FileSinkConfig {
                        enabled: false,
                        path: None,
                        mode: crate::config::FileSinkMode::Append,
                    },
                },
            },
            api: ApiConfig {
                listen_addr: "127.0.0.1:8080".to_string(),
                security: ApiSecurityConfig {
                    tls: TlsServerConfig {
                        mode: ApiTlsMode::Disabled,
                        identity: None,
                        client_auth: None,
                    },
                    auth: ApiAuthConfig::Disabled,
                },
            },
            debug: DebugConfig { enabled: true },
        }
    }

    fn sample_pg_state() -> PgInfoState {
        PgInfoState::Unknown {
            common: PgInfoCommon {
                worker: WorkerStatus::Running,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                pg_config: PgConfig {
                    port: None,
                    hot_standby: None,
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: BTreeMap::new(),
                },
                last_refresh_at: Some(UnixMillis(1)),
            },
        }
    }

    fn sample_dcs_state(config: RuntimeConfig) -> DcsState {
        DcsState {
            worker: WorkerStatus::Running,
            trust: DcsTrust::FullQuorum,
            cache: DcsCache {
                members: BTreeMap::new(),
                leader: None,
                switchover: None,
                config,
                init_lock: None,
            },
            last_refresh_at: Some(UnixMillis(1)),
        }
    }

    fn sample_process_state() -> ProcessState {
        ProcessState::Idle {
            worker: WorkerStatus::Running,
            last_outcome: None,
        }
    }

    fn build_context(
        runtime_config: RuntimeConfig,
    ) -> (
        HaWorkerCtx,
        tokio::sync::mpsc::UnboundedReceiver<crate::process::state::ProcessJobRequest>,
    ) {
        let (config_publisher, config_subscriber) =
            new_state_channel(runtime_config.clone(), UnixMillis(1));
        let (pg_publisher, pg_subscriber) = new_state_channel(sample_pg_state(), UnixMillis(1));
        let (dcs_publisher, dcs_subscriber) =
            new_state_channel(sample_dcs_state(runtime_config.clone()), UnixMillis(1));
        let (process_publisher, process_subscriber) =
            new_state_channel(sample_process_state(), UnixMillis(1));
        let (ha_publisher, _ha_subscriber) = new_state_channel(
            HaState {
                worker: WorkerStatus::Starting,
                phase: crate::ha::state::HaPhase::Init,
                tick: 0,
                decision: crate::ha::decision::HaDecision::NoChange,
            },
            UnixMillis(1),
        );
        let (process_tx, process_rx) = tokio::sync::mpsc::unbounded_channel();

        let _ = config_publisher;
        let _ = pg_publisher;
        let _ = dcs_publisher;
        let _ = process_publisher;

        (
            HaWorkerCtx::contract_stub(HaWorkerContractStubInputs {
                publisher: ha_publisher,
                config_subscriber,
                pg_subscriber,
                dcs_subscriber,
                process_subscriber,
                process_inbox: process_tx,
                dcs_store: Box::new(NoopStore),
                scope: "scope-a".to_string(),
                self_id: MemberId("node-a".to_string()),
            }),
            process_rx,
        )
    }

    #[test]
    fn start_postgres_dispatch_builds_request_with_managed_settings() -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("start");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, mut process_rx) = build_context(runtime_config.clone());

        let outcome =
            dispatch_process_action(&mut ctx, 7, 3, &HaAction::StartPostgres, &runtime_config);
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Applied));

        let request = process_rx
            .try_recv()
            .map_err(|err| WorkerError::Message(format!("process request missing: {err}")))?;
        assert_eq!(
            request.id.0,
            "ha-scope-a-node-a-7-3-start_postgres".to_string()
        );
        if let ProcessJobKind::StartPostgres(spec) = request.kind {
            assert_eq!(spec.data_dir, runtime_config.postgres.data_dir);
            assert_eq!(spec.host, "127.0.0.1".to_string());
            assert_eq!(spec.port, 5432);
        } else {
            return Err(WorkerError::Message(
                "expected start postgres request".to_string(),
            ));
        }

        fs::remove_dir_all(&data_dir)
            .map_err(|err| WorkerError::Message(format!("remove temp dir failed: {err}")))?;
        Ok(())
    }

    #[test]
    fn wipe_data_dir_dispatch_recreates_directory() -> Result<(), WorkerError> {
        let base_dir = std::env::temp_dir().join(format!(
            "pgtuskmaster-process-dispatch-{}",
            std::process::id()
        ));
        let nested_file = base_dir.join("stale.txt");
        if base_dir.exists() {
            fs::remove_dir_all(&base_dir).map_err(|err| {
                WorkerError::Message(format!("cleanup existing temp dir failed: {err}"))
            })?;
        }
        fs::create_dir_all(&base_dir)
            .and_then(|()| fs::write(&nested_file, b"stale"))
            .map_err(|err| {
                WorkerError::Message(format!("create temp dir fixture failed: {err}"))
            })?;

        let runtime_config = sample_runtime_config(base_dir.clone());
        let (mut ctx, _process_rx) = build_context(runtime_config.clone());
        let outcome =
            dispatch_process_action(&mut ctx, 2, 0, &HaAction::WipeDataDir, &runtime_config);
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Applied));
        assert!(base_dir.exists());
        assert!(!nested_file.exists());

        fs::remove_dir_all(&base_dir)
            .map_err(|err| WorkerError::Message(format!("remove temp dir failed: {err}")))?;
        Ok(())
    }
}
