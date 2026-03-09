use std::{fs, path::Path};

use thiserror::Error;

use crate::{
    config::RuntimeConfig,
    dcs::state::MemberRecord,
    ha::decision::HaDecision,
    local_physical::{inspect_local_physical_state, DataDirKind, SignalFileState},
    pginfo::state::PgInfoState,
    postgres_managed_conf::{
        managed_standby_auth_from_role_auth, render_managed_primary_conninfo,
        ManagedPostgresStartIntent, ManagedStandbyAuth, MANAGED_POSTGRESQL_CONF_NAME,
    },
    process::{
        jobs::{
            BaseBackupSpec, BootstrapSpec, DemoteSpec, FencingSpec, PgRewindSpec, PromoteSpec,
            ShutdownMode, StartPostgresSpec,
        },
        state::{ProcessJobKind, ProcessJobRequest},
    },
    state::{JobId, MemberId},
};

use super::{
    actions::{ActionId, HaAction},
    source_conn::{basebackup_source_from_member, rewind_source_from_member},
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
    #[error("remote source selection failed for action `{action:?}`: {message}")]
    SourceSelection { action: ActionId, message: String },
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
            let leader_member_id = start_postgres_leader_member_id(ctx);
            if postgres_data_dir_requires_basebackup(
                &ctx.process_defaults.postgres_binary,
                runtime_config.postgres.data_dir.as_path(),
            )? {
                if let Some(leader_member_id) = leader_member_id {
                    let source = validate_basebackup_source(ctx, action.id(), leader_member_id)?;
                    let request = ProcessJobRequest {
                        id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                        kind: ProcessJobKind::BaseBackup(BaseBackupSpec {
                            data_dir: runtime_config.postgres.data_dir.clone(),
                            source,
                            timeout_ms: Some(runtime_config.process.bootstrap_timeout_ms),
                        }),
                    };
                    send_process_request(ctx, action.id(), request)?;
                    return Ok(ProcessDispatchOutcome::Applied);
                }
            }
            let start_intent = managed_start_intent_from_dcs(
                ctx,
                action.id(),
                leader_member_id,
                runtime_config.postgres.data_dir.as_path(),
            )?;
            let managed = crate::postgres_managed::materialize_managed_postgres_config(
                runtime_config,
                &start_intent,
            )
            .map_err(|err| ProcessDispatchError::ManagedConfig {
                action: action.id(),
                message: err.to_string(),
            })?;
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::StartPostgres(StartPostgresSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    config_file: managed.postgresql_conf_path,
                    log_file: ctx.process_defaults.log_file.clone(),
                    wait_seconds: None,
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::FollowLeader { leader_member_id } => {
            let leader_member_id = MemberId(leader_member_id.clone());
            let start_intent = managed_start_intent_from_dcs(
                ctx,
                action.id(),
                Some(&leader_member_id),
                runtime_config.postgres.data_dir.as_path(),
            )?;
            if follow_leader_is_already_current_or_pending(
                ctx,
                action.id(),
                runtime_config.postgres.data_dir.as_path(),
                &start_intent,
            )? {
                return Ok(ProcessDispatchOutcome::Skipped);
            }
            let _managed = crate::postgres_managed::materialize_managed_postgres_config(
                runtime_config,
                &start_intent,
            )
            .map_err(|err| ProcessDispatchError::ManagedConfig {
                action: action.id(),
                message: err.to_string(),
            })?;
            let demote_request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Demote(DemoteSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    timeout_ms: None,
                    mode: ctx.process_defaults.shutdown_mode.clone(),
                }),
            };
            send_process_request(ctx, action.id(), demote_request)?;
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
        HaAction::StartRewind { leader_member_id } => {
            let source = validate_rewind_source(ctx, action.id(), leader_member_id)?;
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::PgRewind(PgRewindSpec {
                    target_data_dir: runtime_config.postgres.data_dir.clone(),
                    source,
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::StartBaseBackup { leader_member_id } => {
            let source = validate_basebackup_source(ctx, action.id(), leader_member_id)?;
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::BaseBackup(BaseBackupSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    source,
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
        HaAction::SignalFailSafe => Ok(ProcessDispatchOutcome::Skipped),
    }
}

pub(crate) fn validate_rewind_source(
    ctx: &HaWorkerCtx,
    action: ActionId,
    leader_member_id: &crate::state::MemberId,
) -> Result<crate::process::jobs::RewinderSourceConn, ProcessDispatchError> {
    let member = resolve_source_member(ctx, action.clone(), leader_member_id)?;
    rewind_source_from_member(&ctx.self_id, &member, &ctx.process_defaults).map_err(|err| {
        ProcessDispatchError::SourceSelection {
            action,
            message: err.to_string(),
        }
    })
}

pub(crate) fn validate_basebackup_source(
    ctx: &HaWorkerCtx,
    action: ActionId,
    leader_member_id: &crate::state::MemberId,
) -> Result<crate::process::jobs::ReplicatorSourceConn, ProcessDispatchError> {
    let member = resolve_source_member(ctx, action.clone(), leader_member_id)?;
    basebackup_source_from_member(&ctx.self_id, &member, &ctx.process_defaults).map_err(|err| {
        ProcessDispatchError::SourceSelection {
            action,
            message: err.to_string(),
        }
    })
}

fn resolve_source_member(
    ctx: &HaWorkerCtx,
    action: ActionId,
    leader_member_id: &crate::state::MemberId,
) -> Result<MemberRecord, ProcessDispatchError> {
    let dcs = ctx.dcs_subscriber.latest();
    dcs.value
        .cache
        .members
        .get(leader_member_id)
        .cloned()
        .ok_or_else(|| ProcessDispatchError::SourceSelection {
            action,
            message: format!(
                "target member `{}` not present in DCS cache",
                leader_member_id.0
            ),
        })
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

fn start_postgres_leader_member_id(ctx: &HaWorkerCtx) -> Option<&MemberId> {
    match &ctx.state.decision {
        HaDecision::WaitForPostgres {
            leader_member_id, ..
        } => leader_member_id.as_ref(),
        _ => None,
    }
}

fn managed_start_intent_from_dcs(
    ctx: &HaWorkerCtx,
    action: ActionId,
    replica_leader_member_id: Option<&MemberId>,
    data_dir: &Path,
) -> Result<ManagedPostgresStartIntent, ProcessDispatchError> {
    if let Some(leader_member_id) = replica_leader_member_id {
        let leader = resolve_source_member(ctx, action.clone(), leader_member_id)?;
        let source = basebackup_source_from_member(&ctx.self_id, &leader, &ctx.process_defaults)
            .map_err(|err| ProcessDispatchError::SourceSelection {
                action: action.clone(),
                message: err.to_string(),
            })?;
        return Ok(ManagedPostgresStartIntent::replica(
            source.conninfo.clone(),
            managed_standby_auth_from_role_auth(&source.auth, data_dir),
            None,
        ));
    }

    let inspected = inspect_local_physical_state(data_dir, &ctx.process_defaults.postgres_binary)
        .map_err(|err| ProcessDispatchError::ManagedConfig {
        action: action.clone(),
        message: err.to_string(),
    })?;

    if inspected.signal_file_state != SignalFileState::None {
        return Err(ProcessDispatchError::ManagedConfig {
            action,
            message:
                "existing postgres data dir contains managed replica recovery state but no leader-derived source is available to rebuild authoritative managed config"
                    .to_string(),
        });
    }

    Ok(ManagedPostgresStartIntent::primary())
}

fn postgres_data_dir_requires_basebackup(
    postgres_binary: &Path,
    data_dir: &Path,
) -> Result<bool, ProcessDispatchError> {
    let inspected = inspect_local_physical_state(data_dir, postgres_binary).map_err(|err| {
        ProcessDispatchError::Filesystem {
            action: ActionId::StartPostgres,
            message: err.to_string(),
        }
    })?;
    Ok(!matches!(inspected.data_dir_kind, DataDirKind::Initialized))
}

fn follow_leader_is_already_current_or_pending(
    ctx: &HaWorkerCtx,
    action: ActionId,
    data_dir: &Path,
    start_intent: &ManagedPostgresStartIntent,
) -> Result<bool, ProcessDispatchError> {
    let Some((expected_primary_conninfo, _)) = standby_start_details(start_intent) else {
        return Ok(true);
    };

    let pg = ctx.pg_subscriber.latest();
    let Some(current_primary_conninfo) = current_primary_conninfo(&pg.value) else {
        return Ok(true);
    };
    if current_primary_conninfo.host == expected_primary_conninfo.host
        && current_primary_conninfo.port == expected_primary_conninfo.port
    {
        return Ok(true);
    }
    if pginfo_common(&pg.value).sql == crate::pginfo::state::SqlStatus::Healthy {
        return Ok(false);
    }

    managed_config_already_targets_start_intent(action, data_dir, start_intent)
}

fn pginfo_common(state: &PgInfoState) -> &crate::pginfo::state::PgInfoCommon {
    match state {
        PgInfoState::Unknown { common }
        | PgInfoState::Primary { common, .. }
        | PgInfoState::Replica { common, .. } => common,
    }
}

fn standby_start_details(
    start_intent: &ManagedPostgresStartIntent,
) -> Option<(&crate::pginfo::state::PgConnInfo, &ManagedStandbyAuth)> {
    match start_intent {
        ManagedPostgresStartIntent::Replica {
            primary_conninfo,
            standby_auth,
            ..
        }
        | ManagedPostgresStartIntent::Recovery {
            primary_conninfo,
            standby_auth,
            ..
        } => Some((primary_conninfo, standby_auth)),
        ManagedPostgresStartIntent::Primary => None,
    }
}

fn managed_config_already_targets_start_intent(
    action: ActionId,
    data_dir: &Path,
    start_intent: &ManagedPostgresStartIntent,
) -> Result<bool, ProcessDispatchError> {
    let Some((expected_primary_conninfo, standby_auth)) = standby_start_details(start_intent)
    else {
        return Ok(false);
    };
    let managed_conf_path = data_dir.join(MANAGED_POSTGRESQL_CONF_NAME);
    let rendered = match fs::read_to_string(&managed_conf_path) {
        Ok(rendered) => rendered,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(err) => {
            return Err(ProcessDispatchError::ManagedConfig {
                action,
                message: format!(
                    "read managed postgres config failed at {}: {err}",
                    managed_conf_path.display()
                ),
            });
        }
    };
    let expected_recovery_state = start_intent.recovery_signal();
    let actual_recovery_state = crate::postgres_managed::inspect_managed_recovery_state(data_dir)
        .map_err(|err| ProcessDispatchError::ManagedConfig {
        action: action.clone(),
        message: err.to_string(),
    })?;
    if actual_recovery_state != expected_recovery_state {
        return Ok(false);
    }

    Ok(rendered.contains(
        render_managed_primary_conninfo(expected_primary_conninfo, standby_auth).as_str(),
    ))
}

fn current_primary_conninfo(state: &PgInfoState) -> Option<&crate::pginfo::state::PgConnInfo> {
    match state {
        PgInfoState::Unknown { common }
        | PgInfoState::Primary { common, .. }
        | PgInfoState::Replica { common, .. } => common.pg_config.primary_conninfo.as_ref(),
    }
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
    set_postgres_data_dir_permissions(data_dir)?;
    Ok(())
}

fn set_postgres_data_dir_permissions(data_dir: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(data_dir, fs::Permissions::from_mode(0o700))
            .map_err(|err| format!("wipe_data_dir set_permissions failed: {err}"))?;
    }

    #[cfg(not(unix))]
    {
        let _ = data_dir;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        config::{RoleAuthConfig, RuntimeConfig, SecretSource},
        dcs::{
            state::{DcsView, DcsState, DcsTrust, MemberRecord, MemberRole},
            store::{DcsLeaderStore, DcsStore, DcsStoreError, WatchEvent},
        },
        ha::{
            actions::HaAction,
            decision::HaDecision,
            process_dispatch::{
                dispatch_process_action, managed_start_intent_from_dcs, ProcessDispatchError,
                ProcessDispatchOutcome,
            },
            state::{HaState, HaWorkerContractStubInputs, HaWorkerCtx},
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        postgres_managed_conf::managed_standby_auth_from_role_auth,
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

    impl DcsLeaderStore for NoopStore {
        fn acquire_leader_lease(
            &mut self,
            _scope: &str,
            _member_id: &MemberId,
        ) -> Result<(), DcsStoreError> {
            Ok(())
        }

        fn release_leader_lease(
            &mut self,
            _scope: &str,
            _member_id: &MemberId,
        ) -> Result<(), DcsStoreError> {
            Ok(())
        }

        fn clear_switchover(&mut self, _scope: &str) -> Result<(), DcsStoreError> {
            Ok(())
        }
    }

    static TEST_DATA_DIR_SEQ: AtomicU64 = AtomicU64::new(0);

    fn sample_password_auth() -> RoleAuthConfig {
        RoleAuthConfig::Password {
            password: SecretSource::Inline {
                content: "secret-password".to_string(),
            },
        }
    }

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
        crate::test_harness::runtime_config::RuntimeConfigBuilder::new()
            .with_postgres_data_dir(data_dir)
            .build()
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
            trust: DcsTrust::FreshQuorum,
            cache: DcsView {
                members: BTreeMap::new(),
                leader: None,
                switchover: None,
                config,
                cluster_initialized: None,
                cluster_identity: None,
                bootstrap_lock: None,
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
        crate::state::StatePublisher<PgInfoState>,
        crate::state::StatePublisher<DcsState>,
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
            pg_publisher,
            dcs_publisher,
            process_rx,
        )
    }

    fn primary_member(member_id: &str, host: &str, port: u16) -> MemberRecord {
        MemberRecord {
            member_id: MemberId(member_id.to_string()),
            postgres_host: host.to_string(),
            postgres_port: port,
            api_url: None,
            role: MemberRole::Primary,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
            timeline: None,
            write_lsn: None,
            replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
            updated_at: UnixMillis(1),
            pg_version: crate::state::Version(1),
        }
    }

    fn replica_pg_state(primary_host: &str, primary_port: u16) -> PgInfoState {
        replica_pg_state_with_sql(primary_host, primary_port, SqlStatus::Healthy)
    }

    fn replica_pg_state_with_sql(
        primary_host: &str,
        primary_port: u16,
        sql: SqlStatus,
    ) -> PgInfoState {
        PgInfoState::Replica {
            common: PgInfoCommon {
                worker: WorkerStatus::Running,
                sql: sql.clone(),
                readiness: match sql {
                    SqlStatus::Healthy => Readiness::Ready,
                    SqlStatus::Unknown => Readiness::Unknown,
                    SqlStatus::Unreachable => Readiness::NotReady,
                },
                timeline: None,
                pg_config: PgConfig {
                    port: Some(5432),
                    hot_standby: Some(true),
                    primary_conninfo: Some(crate::pginfo::state::PgConnInfo {
                        host: primary_host.to_string(),
                        port: primary_port,
                        user: "replicator".to_string(),
                        dbname: "postgres".to_string(),
                        application_name: None,
                        connect_timeout_s: Some(5),
                        ssl_mode: crate::pginfo::state::PgSslMode::Prefer,
                        options: None,
                    }),
                    primary_slot_name: None,
                    extra: BTreeMap::new(),
                },
                last_refresh_at: Some(UnixMillis(1)),
            },
            replay_lsn: crate::state::WalLsn(10),
            follow_lsn: Some(crate::state::WalLsn(11)),
            upstream: None,
        }
    }

    fn remove_dir_if_present(path: &PathBuf) -> Result<(), WorkerError> {
        if path.exists() {
            fs::remove_dir_all(path)
                .map_err(|err| WorkerError::Message(format!("remove temp dir failed: {err}")))?;
        }
        Ok(())
    }

    fn initdb_data_dir(runtime_config: &RuntimeConfig, data_dir: &Path) -> Result<(), WorkerError> {
        let output = std::process::Command::new(&runtime_config.process.binaries.initdb)
            .arg("-D")
            .arg(data_dir)
            .arg("-U")
            .arg("postgres")
            .arg("--auth=trust")
            .arg("--no-sync")
            .env("LC_ALL", "C")
            .output()
            .map_err(|err| WorkerError::Message(format!("initdb fixture failed: {err}")))?;
        if !output.status.success() {
            return Err(WorkerError::Message(format!(
                "initdb fixture failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
        Ok(())
    }

    #[test]
    fn start_postgres_dispatch_builds_request_with_managed_settings() -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("start");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, _pg_publisher, _dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());

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
            assert_eq!(
                spec.config_file,
                runtime_config
                    .postgres
                    .data_dir
                    .join("pgtm.postgresql.conf")
            );
        } else {
            return Err(WorkerError::Message(
                "expected start postgres request".to_string(),
            ));
        }

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn start_postgres_dispatch_uses_basebackup_when_data_dir_is_missing_and_leader_is_known(
    ) -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("start-missing-data-dir");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, _pg_publisher, dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        let mut dcs = sample_dcs_state(runtime_config.clone());
        dcs.cache.members.insert(
            MemberId("node-b".to_string()),
            primary_member("node-b", "10.0.0.20", 5440),
        );
        dcs_publisher
            .publish(dcs, UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish dcs fixture failed: {err}")))?;
        ctx.state.decision = HaDecision::WaitForPostgres {
            start_requested: true,
            leader_member_id: Some(MemberId("node-b".to_string())),
        };

        let outcome =
            dispatch_process_action(&mut ctx, 7, 3, &HaAction::StartPostgres, &runtime_config);
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Applied));

        let request = process_rx
            .try_recv()
            .map_err(|err| WorkerError::Message(format!("process request missing: {err}")))?;
        if let ProcessJobKind::BaseBackup(spec) = request.kind {
            assert_eq!(spec.data_dir, runtime_config.postgres.data_dir);
            assert_eq!(spec.source.conninfo.host, "10.0.0.20".to_string());
            assert_eq!(spec.source.conninfo.port, 5440);
        } else {
            return Err(WorkerError::Message(
                "expected basebackup request when data dir is missing".to_string(),
            ));
        }

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn start_postgres_dispatch_preserves_replica_follow_target() -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("start-replica");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, _pg_publisher, dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        initdb_data_dir(&runtime_config, &data_dir)?;
        let mut dcs = sample_dcs_state(runtime_config.clone());
        dcs.cache.members.insert(
            MemberId("node-b".to_string()),
            primary_member("node-b", "10.0.0.20", 5432),
        );
        dcs_publisher
            .publish(dcs, UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish dcs fixture failed: {err}")))?;
        ctx.state.decision = HaDecision::WaitForPostgres {
            start_requested: true,
            leader_member_id: Some(MemberId("node-b".to_string())),
        };

        let outcome =
            dispatch_process_action(&mut ctx, 7, 3, &HaAction::StartPostgres, &runtime_config);
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Applied));

        let request = process_rx
            .try_recv()
            .map_err(|err| WorkerError::Message(format!("process request missing: {err}")))?;
        if !matches!(request.kind, ProcessJobKind::StartPostgres(_)) {
            return Err(WorkerError::Message(
                "expected start postgres request".to_string(),
            ));
        }

        let managed_conf_path = runtime_config
            .postgres
            .data_dir
            .join("pgtm.postgresql.conf");
        let rendered = fs::read_to_string(&managed_conf_path).map_err(|err| {
            WorkerError::Message(format!(
                "read managed postgres conf failed at {}: {err}",
                managed_conf_path.display()
            ))
        })?;
        if !rendered.contains("primary_conninfo") {
            return Err(WorkerError::Message(format!(
                "expected replica managed config to include primary_conninfo, got:\n{rendered}"
            )));
        }
        if !rendered.contains("passfile=") {
            return Err(WorkerError::Message(format!(
                "expected replica managed config to include managed passfile, got:\n{rendered}"
            )));
        }
        let standby_signal = runtime_config.postgres.data_dir.join("standby.signal");
        if !standby_signal.exists() {
            return Err(WorkerError::Message(format!(
                "expected standby.signal to exist at {}",
                standby_signal.display()
            )));
        }

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn start_postgres_dispatch_without_replica_target_starts_primary() -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("start-primary");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, _pg_publisher, dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        let mut dcs = sample_dcs_state(runtime_config.clone());
        dcs.cache.members.insert(
            MemberId("node-b".to_string()),
            primary_member("node-b", "10.0.0.20", 5432),
        );
        dcs_publisher
            .publish(dcs, UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish dcs fixture failed: {err}")))?;
        ctx.state.decision = HaDecision::WaitForPostgres {
            start_requested: true,
            leader_member_id: None,
        };

        let outcome =
            dispatch_process_action(&mut ctx, 7, 3, &HaAction::StartPostgres, &runtime_config);
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Applied));

        let request = process_rx
            .try_recv()
            .map_err(|err| WorkerError::Message(format!("process request missing: {err}")))?;
        if !matches!(request.kind, ProcessJobKind::StartPostgres(_)) {
            return Err(WorkerError::Message(
                "expected start postgres request".to_string(),
            ));
        }

        let managed_conf_path = runtime_config
            .postgres
            .data_dir
            .join("pgtm.postgresql.conf");
        let rendered = fs::read_to_string(&managed_conf_path).map_err(|err| {
            WorkerError::Message(format!(
                "read managed postgres conf failed at {}: {err}",
                managed_conf_path.display()
            ))
        })?;
        if rendered.contains("primary_conninfo") {
            return Err(WorkerError::Message(format!(
                "expected primary managed config without primary_conninfo, got:\n{rendered}"
            )));
        }
        let standby_signal = runtime_config.postgres.data_dir.join("standby.signal");
        if standby_signal.exists() {
            return Err(WorkerError::Message(format!(
                "expected standby.signal to be absent at {}",
                standby_signal.display()
            )));
        }

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn start_postgres_dispatch_rejects_existing_replica_state_without_dcs_leader(
    ) -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("start-existing-replica-without-leader");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, _pg_publisher, _dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        let existing_conninfo = crate::pginfo::state::PgConnInfo {
            host: "10.0.0.20".to_string(),
            port: 5432,
            user: "replicator".to_string(),
            dbname: "postgres".to_string(),
            application_name: None,
            connect_timeout_s: Some(2),
            ssl_mode: crate::pginfo::state::PgSslMode::Prefer,
            options: Some("-c wal_receiver_status_interval=5s".to_string()),
        };
        let _ = crate::postgres_managed::materialize_managed_postgres_config(
            &runtime_config,
            &crate::postgres_managed_conf::ManagedPostgresStartIntent::replica(
                existing_conninfo.clone(),
                managed_standby_auth_from_role_auth(
                    &runtime_config.postgres.roles.replicator.auth,
                    runtime_config.postgres.data_dir.as_path(),
                ),
                Some("slot_a".to_string()),
            ),
        )
        .map_err(|err| {
            WorkerError::Message(format!("seed managed replica config failed: {err}"))
        })?;
        ctx.state.decision = HaDecision::WaitForPostgres {
            start_requested: true,
            leader_member_id: None,
        };

        let outcome =
            dispatch_process_action(&mut ctx, 7, 3, &HaAction::StartPostgres, &runtime_config);
        assert!(matches!(
            outcome,
            Err(ProcessDispatchError::ManagedConfig { .. })
        ));
        assert!(process_rx.try_recv().is_err());

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn follow_leader_dispatch_rewrites_managed_config_and_demotes_when_upstream_changes(
    ) -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("follow-leader-reload");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, pg_publisher, dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        let mut dcs = sample_dcs_state(runtime_config.clone());
        dcs.cache.members.insert(
            MemberId("node-b".to_string()),
            primary_member("node-b", "10.0.0.21", 5440),
        );
        dcs_publisher
            .publish(dcs, UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish dcs fixture failed: {err}")))?;
        pg_publisher
            .publish(replica_pg_state("10.0.0.20", 5432), UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish pg fixture failed: {err}")))?;

        let outcome = dispatch_process_action(
            &mut ctx,
            7,
            3,
            &HaAction::FollowLeader {
                leader_member_id: "node-b".to_string(),
            },
            &runtime_config,
        );
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Applied));

        let request = process_rx
            .try_recv()
            .map_err(|err| WorkerError::Message(format!("process request missing: {err}")))?;
        assert_eq!(
            request.id.0,
            "ha-scope-a-node-a-7-3-follow_leader_node-b".to_string()
        );
        if let ProcessJobKind::Demote(spec) = request.kind {
            assert_eq!(spec.data_dir, runtime_config.postgres.data_dir);
            assert_eq!(spec.mode, ctx.process_defaults.shutdown_mode);
        } else {
            return Err(WorkerError::Message("expected demote request".to_string()));
        }
        assert!(process_rx.try_recv().is_err());

        let managed_conf_path = runtime_config
            .postgres
            .data_dir
            .join("pgtm.postgresql.conf");
        let rendered = fs::read_to_string(&managed_conf_path).map_err(|err| {
            WorkerError::Message(format!(
                "read managed postgres conf failed at {}: {err}",
                managed_conf_path.display()
            ))
        })?;
        if !rendered.contains("10.0.0.21") || !rendered.contains("5440") {
            return Err(WorkerError::Message(format!(
                "expected managed config to retarget node-b, got:\n{rendered}"
            )));
        }

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn follow_leader_dispatch_skips_when_upstream_already_matches_authoritative_leader(
    ) -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("follow-leader-steady-state");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, pg_publisher, dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        let mut dcs = sample_dcs_state(runtime_config.clone());
        dcs.cache.members.insert(
            MemberId("node-b".to_string()),
            primary_member("node-b", "10.0.0.21", 5440),
        );
        dcs_publisher
            .publish(dcs, UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish dcs fixture failed: {err}")))?;
        pg_publisher
            .publish(replica_pg_state("10.0.0.21", 5440), UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish pg fixture failed: {err}")))?;

        let outcome = dispatch_process_action(
            &mut ctx,
            7,
            3,
            &HaAction::FollowLeader {
                leader_member_id: "node-b".to_string(),
            },
            &runtime_config,
        );
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Skipped));
        assert!(process_rx.try_recv().is_err());

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn follow_leader_dispatch_applies_when_live_upstream_mismatches_even_if_managed_config_already_targets_authoritative_leader(
    ) -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("follow-leader-pending-retarget");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, pg_publisher, dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        let leader_member_id = MemberId("node-b".to_string());
        let mut dcs = sample_dcs_state(runtime_config.clone());
        dcs.cache.members.insert(
            leader_member_id.clone(),
            primary_member("node-b", "10.0.0.21", 5440),
        );
        dcs_publisher
            .publish(dcs, UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish dcs fixture failed: {err}")))?;
        let _ = crate::postgres_managed::materialize_managed_postgres_config(
            &runtime_config,
            &crate::postgres_managed_conf::ManagedPostgresStartIntent::replica(
                crate::pginfo::state::PgConnInfo {
                    host: "10.0.0.21".to_string(),
                    port: 5440,
                    user: "replicator".to_string(),
                    dbname: "postgres".to_string(),
                    application_name: None,
                    connect_timeout_s: Some(2),
                    ssl_mode: crate::pginfo::state::PgSslMode::Prefer,
                    options: Some("-c wal_receiver_status_interval=5s".to_string()),
                },
                managed_standby_auth_from_role_auth(
                    &runtime_config.postgres.roles.replicator.auth,
                    runtime_config.postgres.data_dir.as_path(),
                ),
                None,
            ),
        )
        .map_err(|err| {
            WorkerError::Message(format!("seed managed replica config failed: {err}"))
        })?;
        let follow_action = HaAction::FollowLeader {
            leader_member_id: "node-b".to_string(),
        };
        let retargeted_start_intent = managed_start_intent_from_dcs(
            &ctx,
            follow_action.id(),
            Some(&leader_member_id),
            runtime_config.postgres.data_dir.as_path(),
        )
        .map_err(|err| {
            WorkerError::Message(format!(
                "derive leader-backed start intent for seeded managed config failed: {err}"
            ))
        })?;
        let _ = crate::postgres_managed::materialize_managed_postgres_config(
            &runtime_config,
            &retargeted_start_intent,
        )
        .map_err(|err| {
            WorkerError::Message(format!(
                "rewrite managed replica config to authoritative state failed: {err}"
            ))
        })?;
        pg_publisher
            .publish(replica_pg_state("10.0.0.20", 5432), UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish pg fixture failed: {err}")))?;

        let outcome = dispatch_process_action(&mut ctx, 7, 3, &follow_action, &runtime_config);
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Applied));
        let request = process_rx
            .try_recv()
            .map_err(|err| WorkerError::Message(format!("process request missing: {err}")))?;
        assert_eq!(
            request.id.0,
            "ha-scope-a-node-a-7-3-follow_leader_node-b".to_string()
        );
        if let ProcessJobKind::Demote(spec) = request.kind {
            assert_eq!(spec.data_dir, runtime_config.postgres.data_dir);
            assert_eq!(spec.mode, ctx.process_defaults.shutdown_mode);
        } else {
            return Err(WorkerError::Message("expected demote request".to_string()));
        }
        assert!(process_rx.try_recv().is_err());

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn follow_leader_dispatch_skips_when_managed_config_already_targets_authoritative_leader_and_pginfo_is_not_healthy(
    ) -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("follow-leader-pending-restart");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, pg_publisher, dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        let leader_member_id = MemberId("node-b".to_string());
        let mut dcs = sample_dcs_state(runtime_config.clone());
        dcs.cache.members.insert(
            leader_member_id.clone(),
            primary_member("node-b", "10.0.0.21", 5440),
        );
        dcs_publisher
            .publish(dcs, UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish dcs fixture failed: {err}")))?;
        let follow_action = HaAction::FollowLeader {
            leader_member_id: "node-b".to_string(),
        };
        let retargeted_start_intent = managed_start_intent_from_dcs(
            &ctx,
            follow_action.id(),
            Some(&leader_member_id),
            runtime_config.postgres.data_dir.as_path(),
        )
        .map_err(|err| {
            WorkerError::Message(format!(
                "derive leader-backed start intent for seeded managed config failed: {err}"
            ))
        })?;
        let _ = crate::postgres_managed::materialize_managed_postgres_config(
            &runtime_config,
            &retargeted_start_intent,
        )
        .map_err(|err| {
            WorkerError::Message(format!(
                "rewrite managed replica config to authoritative state failed: {err}"
            ))
        })?;
        pg_publisher
            .publish(
                replica_pg_state_with_sql(
                    "10.0.0.20",
                    5432,
                    crate::pginfo::state::SqlStatus::Unreachable,
                ),
                UnixMillis(2),
            )
            .map_err(|err| WorkerError::Message(format!("publish pg fixture failed: {err}")))?;

        let outcome = dispatch_process_action(&mut ctx, 7, 3, &follow_action, &runtime_config);
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Skipped));
        assert!(process_rx.try_recv().is_err());

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn follow_leader_dispatch_skips_until_pginfo_reports_primary_conninfo(
    ) -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("follow-leader-no-primary-conninfo-yet");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, _pg_publisher, dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        let mut dcs = sample_dcs_state(runtime_config.clone());
        dcs.cache.members.insert(
            MemberId("node-b".to_string()),
            primary_member("node-b", "10.0.0.21", 5440),
        );
        dcs_publisher
            .publish(dcs, UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish dcs fixture failed: {err}")))?;

        let outcome = dispatch_process_action(
            &mut ctx,
            7,
            3,
            &HaAction::FollowLeader {
                leader_member_id: "node-b".to_string(),
            },
            &runtime_config,
        );
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Skipped));
        assert!(process_rx.try_recv().is_err());

        remove_dir_if_present(&data_dir)?;
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
        let (mut ctx, _pg_publisher, _dcs_publisher, _process_rx) =
            build_context(runtime_config.clone());
        let outcome =
            dispatch_process_action(&mut ctx, 2, 0, &HaAction::WipeDataDir, &runtime_config);
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Applied));
        assert!(base_dir.exists());
        assert!(!nested_file.exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mode = fs::metadata(&base_dir)
                .map_err(|err| {
                    WorkerError::Message(format!("read recreated data dir metadata failed: {err}"))
                })?
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o700);
        }

        fs::remove_dir_all(&base_dir)
            .map_err(|err| WorkerError::Message(format!("remove temp dir failed: {err}")))?;
        Ok(())
    }

    #[test]
    fn start_basebackup_dispatch_uses_target_member_endpoint_and_replicator_role(
    ) -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("basebackup");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, _pg_publisher, dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        let leader_member_id = MemberId("node-b".to_string());
        let mut dcs_state = sample_dcs_state(runtime_config.clone());
        dcs_state.cache.members.insert(
            leader_member_id.clone(),
            primary_member("node-b", "10.0.0.20", 5440),
        );
        let _ = dcs_publisher
            .publish(dcs_state, UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish dcs state failed: {err}")))?;

        let outcome = dispatch_process_action(
            &mut ctx,
            9,
            0,
            &HaAction::StartBaseBackup {
                leader_member_id: leader_member_id.clone(),
            },
            &runtime_config,
        );
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Applied));

        let request = process_rx
            .try_recv()
            .map_err(|err| WorkerError::Message(format!("process request missing: {err}")))?;
        if let ProcessJobKind::BaseBackup(spec) = request.kind {
            assert_eq!(spec.source.conninfo.host, "10.0.0.20".to_string());
            assert_eq!(spec.source.conninfo.port, 5440);
            assert_eq!(spec.source.conninfo.user, "replicator".to_string());
            assert_eq!(spec.source.auth, sample_password_auth());
        } else {
            return Err(WorkerError::Message(
                "expected basebackup request".to_string(),
            ));
        }

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn start_rewind_dispatch_uses_target_member_and_ignores_unrelated_leader_key(
    ) -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("rewind");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, _pg_publisher, dcs_publisher, mut process_rx) =
            build_context(runtime_config.clone());
        let leader_member_id = MemberId("node-b".to_string());
        let unrelated_leader_id = MemberId("node-c".to_string());
        let mut dcs_state = sample_dcs_state(runtime_config.clone());
        dcs_state.cache.leader = Some(crate::dcs::state::LeaderRecord {
            member_id: unrelated_leader_id.clone(),
        });
        dcs_state.cache.members.insert(
            leader_member_id.clone(),
            primary_member("node-b", "10.0.0.21", 5441),
        );
        dcs_state.cache.members.insert(
            unrelated_leader_id.clone(),
            primary_member("node-c", "10.0.0.99", 5999),
        );
        let _ = dcs_publisher
            .publish(dcs_state, UnixMillis(2))
            .map_err(|err| WorkerError::Message(format!("publish dcs state failed: {err}")))?;

        let outcome = dispatch_process_action(
            &mut ctx,
            10,
            0,
            &HaAction::StartRewind {
                leader_member_id: leader_member_id.clone(),
            },
            &runtime_config,
        );
        assert_eq!(outcome, Ok(ProcessDispatchOutcome::Applied));

        let request = process_rx
            .try_recv()
            .map_err(|err| WorkerError::Message(format!("process request missing: {err}")))?;
        if let ProcessJobKind::PgRewind(spec) = request.kind {
            assert_eq!(spec.source.conninfo.host, "10.0.0.21".to_string());
            assert_eq!(spec.source.conninfo.port, 5441);
            assert_eq!(spec.source.conninfo.user, "rewinder".to_string());
            assert_eq!(spec.source.auth, sample_password_auth());
        } else {
            return Err(WorkerError::Message("expected rewind request".to_string()));
        }

        remove_dir_if_present(&data_dir)?;
        Ok(())
    }

    #[test]
    fn start_basebackup_dispatch_rejects_missing_target_member() {
        let data_dir = unique_test_data_dir("missing-member");
        let runtime_config = sample_runtime_config(data_dir);
        let (mut ctx, _pg_publisher, _dcs_publisher, _process_rx) =
            build_context(runtime_config.clone());

        let outcome = dispatch_process_action(
            &mut ctx,
            11,
            0,
            &HaAction::StartBaseBackup {
                leader_member_id: MemberId("node-missing".to_string()),
            },
            &runtime_config,
        );

        assert!(matches!(
            outcome,
            Err(ProcessDispatchError::SourceSelection { .. })
        ));
    }
}
