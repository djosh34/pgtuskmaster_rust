use std::{fs, path::Path};

use thiserror::Error;

use crate::{
    config::RuntimeConfig,
    dcs::state::MemberRecord,
    ha::decision::HaDecision,
    postgres_managed_conf::{managed_standby_auth_from_role_auth, ManagedPostgresStartIntent},
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
            let start_intent = start_intent_from_dcs(
                ctx,
                start_postgres_leader_member_id(ctx),
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
        HaAction::FollowLeader { .. } | HaAction::SignalFailSafe => {
            Ok(ProcessDispatchOutcome::Skipped)
        }
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

fn start_intent_from_dcs(
    ctx: &HaWorkerCtx,
    replica_leader_member_id: Option<&MemberId>,
    data_dir: &Path,
) -> Result<ManagedPostgresStartIntent, ProcessDispatchError> {
    if let Some(leader_member_id) = replica_leader_member_id {
        let leader = resolve_source_member(ctx, ActionId::StartPostgres, leader_member_id)?;
        let source = basebackup_source_from_member(&ctx.self_id, &leader, &ctx.process_defaults)
            .map_err(|err| ProcessDispatchError::SourceSelection {
                action: ActionId::StartPostgres,
                message: err.to_string(),
            })?;
        return Ok(ManagedPostgresStartIntent::replica(
            source.conninfo.clone(),
            managed_standby_auth_from_role_auth(&source.auth, data_dir),
            None,
        ));
    }

    if let Some(existing_replica) = existing_replica_start_intent(
        ctx.config_subscriber
            .latest()
            .value
            .postgres
            .data_dir
            .as_path(),
    )? {
        return Ok(existing_replica);
    }

    Ok(ManagedPostgresStartIntent::primary())
}

fn existing_replica_start_intent(
    data_dir: &Path,
) -> Result<Option<ManagedPostgresStartIntent>, ProcessDispatchError> {
    crate::postgres_managed::read_existing_replica_start_intent(data_dir).map_err(|err| {
        ProcessDispatchError::ManagedConfig {
            action: ActionId::StartPostgres,
            message: err.to_string(),
        }
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
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        config::{InlineOrPath, RoleAuthConfig, RuntimeConfig, SecretSource},
        dcs::{
            state::{DcsCache, DcsState, DcsTrust, MemberRecord, MemberRole},
            store::{DcsStore, DcsStoreError, WatchEvent},
        },
        ha::{
            actions::HaAction,
            decision::HaDecision,
            process_dispatch::{
                dispatch_process_action, ProcessDispatchError, ProcessDispatchOutcome,
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

    static TEST_DATA_DIR_SEQ: AtomicU64 = AtomicU64::new(0);

    fn sample_password_auth() -> RoleAuthConfig {
        RoleAuthConfig::Password {
            password: SecretSource(InlineOrPath::Inline {
                content: "secret-password".to_string(),
            }),
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
            dcs_publisher,
            process_rx,
        )
    }

    fn primary_member(member_id: &str, host: &str, port: u16) -> MemberRecord {
        MemberRecord {
            member_id: MemberId(member_id.to_string()),
            postgres_host: host.to_string(),
            postgres_port: port,
            role: MemberRole::Primary,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
            timeline: None,
            write_lsn: None,
            replay_lsn: None,
            updated_at: UnixMillis(1),
            pg_version: crate::state::Version(1),
        }
    }

    fn remove_dir_if_present(path: &PathBuf) -> Result<(), WorkerError> {
        if path.exists() {
            fs::remove_dir_all(path)
                .map_err(|err| WorkerError::Message(format!("remove temp dir failed: {err}")))?;
        }
        Ok(())
    }

    #[test]
    fn start_postgres_dispatch_builds_request_with_managed_settings() -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("start");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, _dcs_publisher, mut process_rx) = build_context(runtime_config.clone());

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
    fn start_postgres_dispatch_preserves_replica_follow_target() -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("start-replica");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, dcs_publisher, mut process_rx) = build_context(runtime_config.clone());
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
        let (mut ctx, dcs_publisher, mut process_rx) = build_context(runtime_config.clone());
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
    fn start_postgres_dispatch_preserves_existing_replica_state_without_dcs_leader(
    ) -> Result<(), WorkerError> {
        let data_dir = unique_test_data_dir("start-existing-replica");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let (mut ctx, _dcs_publisher, mut process_rx) = build_context(runtime_config.clone());
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
        if !rendered.contains("primary_conninfo") || !rendered.contains("primary_slot_name") {
            return Err(WorkerError::Message(format!(
                "expected preserved replica managed config, got:\n{rendered}"
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
        let (mut ctx, _dcs_publisher, _process_rx) = build_context(runtime_config.clone());
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
        let (mut ctx, dcs_publisher, mut process_rx) = build_context(runtime_config.clone());
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
        let (mut ctx, dcs_publisher, mut process_rx) = build_context(runtime_config.clone());
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
        let (mut ctx, _dcs_publisher, _process_rx) = build_context(runtime_config.clone());

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
