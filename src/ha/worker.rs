use thiserror::Error;

use crate::{
    dcs::store::DcsStoreError,
    process::{
        jobs::{
            BootstrapSpec, DemoteSpec, FencingSpec, PgRewindSpec, PromoteSpec, StartPostgresSpec,
        },
        state::{ProcessJobKind, ProcessJobRequest},
    },
    state::{JobId, WorkerError, WorkerStatus},
};

use super::{
    actions::{ActionId, HaAction},
    decide::decide,
    state::{DecideInput, HaWorkerCtx, WorldSnapshot},
};

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ActionDispatchError {
    #[error("process send failed for action `{action:?}`: {message}")]
    ProcessSend { action: ActionId, message: String },
    #[error("dcs write failed for action `{action:?}` at `{path}`: {message}")]
    DcsWrite {
        action: ActionId,
        path: String,
        message: String,
    },
    #[error("dcs delete failed for action `{action:?}` at `{path}`: {message}")]
    DcsDelete {
        action: ActionId,
        path: String,
        message: String,
    },
    #[error("clock failed for action `{action:?}`: {message}")]
    Clock { action: ActionId, message: String },
}

pub(crate) async fn run(mut ctx: HaWorkerCtx) -> Result<(), WorkerError> {
    let mut interval = tokio::time::interval(ctx.poll_interval);
    loop {
        tokio::select! {
            changed = ctx.pg_subscriber.changed() => {
                changed.map_err(|err| {
                    WorkerError::Message(format!("ha pg subscriber closed: {err}"))
                })?;
            }
            changed = ctx.dcs_subscriber.changed() => {
                changed.map_err(|err| {
                    WorkerError::Message(format!("ha dcs subscriber closed: {err}"))
                })?;
            }
            changed = ctx.process_subscriber.changed() => {
                changed.map_err(|err| {
                    WorkerError::Message(format!("ha process subscriber closed: {err}"))
                })?;
            }
            changed = ctx.config_subscriber.changed() => {
                changed.map_err(|err| {
                    WorkerError::Message(format!("ha config subscriber closed: {err}"))
                })?;
            }
            _ = interval.tick() => {}
        }
        step_once(&mut ctx).await?;
    }
}

pub(crate) async fn step_once(ctx: &mut HaWorkerCtx) -> Result<(), WorkerError> {
    let world = world_snapshot(ctx);
    let output = decide(DecideInput {
        current: ctx.state.clone(),
        world,
    })
    .map_err(|err| WorkerError::Message(format!("ha decide failed: {err}")))?;

    let dispatch_errors = dispatch_actions(ctx, &output.actions);
    let now = (ctx.now)()?;

    let mut next = output.next;
    next.worker = if dispatch_errors.is_empty() {
        WorkerStatus::Running
    } else {
        WorkerStatus::Faulted(WorkerError::Message(format_dispatch_errors(
            &dispatch_errors,
        )))
    };

    ctx.publisher
        .publish(next.clone(), now)
        .map_err(|err| WorkerError::Message(format!("ha publish failed: {err}")))?;
    ctx.state = next;
    Ok(())
}

pub(crate) fn dispatch_actions(
    ctx: &mut HaWorkerCtx,
    actions: &[HaAction],
) -> Vec<ActionDispatchError> {
    let mut errors = Vec::new();
    let leader_key = leader_path(&ctx.scope);
    let switchover_key = switchover_path(&ctx.scope);
    let runtime_config = ctx.config_subscriber.latest().value;

    for (index, action) in actions.iter().enumerate() {
        match action {
            HaAction::AcquireLeaderLease => {
                if let Err(err) = ctx.dcs_store.write_leader_lease(&ctx.scope, &ctx.self_id) {
                    errors.push(ActionDispatchError::DcsWrite {
                        action: action.id(),
                        path: leader_key.clone(),
                        message: dcs_error_message(err),
                    });
                }
            }
            HaAction::ReleaseLeaderLease => {
                if let Err(err) = ctx.dcs_store.delete_leader(&ctx.scope) {
                    errors.push(ActionDispatchError::DcsDelete {
                        action: action.id(),
                        path: leader_key.clone(),
                        message: dcs_error_message(err),
                    });
                }
            }
            HaAction::ClearSwitchover => {
                if let Err(err) = ctx.dcs_store.clear_switchover(&ctx.scope) {
                    errors.push(ActionDispatchError::DcsDelete {
                        action: action.id(),
                        path: switchover_key.clone(),
                        message: dcs_error_message(err),
                    });
                }
            }
            HaAction::StartPostgres => {
                let now = match (ctx.now)() {
                    Ok(value) => value,
                    Err(err) => {
                        errors.push(ActionDispatchError::Clock {
                            action: action.id(),
                            message: err.to_string(),
                        });
                        continue;
                    }
                };
                let request = ProcessJobRequest {
                    id: process_job_id(action, index, ctx.state.tick, now.0),
                    kind: ProcessJobKind::StartPostgres(StartPostgresSpec {
                        data_dir: runtime_config.postgres.data_dir.clone(),
                        host: ctx.process_defaults.postgres_host.clone(),
                        port: ctx.process_defaults.postgres_port,
                        socket_dir: ctx.process_defaults.socket_dir.clone(),
                        log_file: ctx.process_defaults.log_file.clone(),
                        wait_seconds: None,
                        timeout_ms: None,
                    }),
                };
                if let Err(err) = ctx.process_inbox.send(request) {
                    errors.push(ActionDispatchError::ProcessSend {
                        action: action.id(),
                        message: err.to_string(),
                    });
                }
            }
            HaAction::PromoteToPrimary => {
                let now = match (ctx.now)() {
                    Ok(value) => value,
                    Err(err) => {
                        errors.push(ActionDispatchError::Clock {
                            action: action.id(),
                            message: err.to_string(),
                        });
                        continue;
                    }
                };
                let request = ProcessJobRequest {
                    id: process_job_id(action, index, ctx.state.tick, now.0),
                    kind: ProcessJobKind::Promote(PromoteSpec {
                        data_dir: runtime_config.postgres.data_dir.clone(),
                        wait_seconds: None,
                        timeout_ms: None,
                    }),
                };
                if let Err(err) = ctx.process_inbox.send(request) {
                    errors.push(ActionDispatchError::ProcessSend {
                        action: action.id(),
                        message: err.to_string(),
                    });
                }
            }
            HaAction::DemoteToReplica => {
                let now = match (ctx.now)() {
                    Ok(value) => value,
                    Err(err) => {
                        errors.push(ActionDispatchError::Clock {
                            action: action.id(),
                            message: err.to_string(),
                        });
                        continue;
                    }
                };
                let request = ProcessJobRequest {
                    id: process_job_id(action, index, ctx.state.tick, now.0),
                    kind: ProcessJobKind::Demote(DemoteSpec {
                        data_dir: runtime_config.postgres.data_dir.clone(),
                        mode: ctx.process_defaults.shutdown_mode.clone(),
                        timeout_ms: None,
                    }),
                };
                if let Err(err) = ctx.process_inbox.send(request) {
                    errors.push(ActionDispatchError::ProcessSend {
                        action: action.id(),
                        message: err.to_string(),
                    });
                }
            }
            HaAction::StartRewind => {
                let now = match (ctx.now)() {
                    Ok(value) => value,
                    Err(err) => {
                        errors.push(ActionDispatchError::Clock {
                            action: action.id(),
                            message: err.to_string(),
                        });
                        continue;
                    }
                };
                let request = ProcessJobRequest {
                    id: process_job_id(action, index, ctx.state.tick, now.0),
                    kind: ProcessJobKind::PgRewind(PgRewindSpec {
                        target_data_dir: runtime_config.postgres.data_dir.clone(),
                        source_conninfo: ctx.process_defaults.rewind_source_conninfo.clone(),
                        timeout_ms: None,
                    }),
                };
                if let Err(err) = ctx.process_inbox.send(request) {
                    errors.push(ActionDispatchError::ProcessSend {
                        action: action.id(),
                        message: err.to_string(),
                    });
                }
            }
            HaAction::RunBootstrap => {
                let now = match (ctx.now)() {
                    Ok(value) => value,
                    Err(err) => {
                        errors.push(ActionDispatchError::Clock {
                            action: action.id(),
                            message: err.to_string(),
                        });
                        continue;
                    }
                };
                let request = ProcessJobRequest {
                    id: process_job_id(action, index, ctx.state.tick, now.0),
                    kind: ProcessJobKind::Bootstrap(BootstrapSpec {
                        data_dir: runtime_config.postgres.data_dir.clone(),
                        timeout_ms: None,
                    }),
                };
                if let Err(err) = ctx.process_inbox.send(request) {
                    errors.push(ActionDispatchError::ProcessSend {
                        action: action.id(),
                        message: err.to_string(),
                    });
                }
            }
            HaAction::FenceNode => {
                let now = match (ctx.now)() {
                    Ok(value) => value,
                    Err(err) => {
                        errors.push(ActionDispatchError::Clock {
                            action: action.id(),
                            message: err.to_string(),
                        });
                        continue;
                    }
                };
                let request = ProcessJobRequest {
                    id: process_job_id(action, index, ctx.state.tick, now.0),
                    kind: ProcessJobKind::Fencing(FencingSpec {
                        data_dir: runtime_config.postgres.data_dir.clone(),
                        mode: ctx.process_defaults.shutdown_mode.clone(),
                        timeout_ms: None,
                    }),
                };
                if let Err(err) = ctx.process_inbox.send(request) {
                    errors.push(ActionDispatchError::ProcessSend {
                        action: action.id(),
                        message: err.to_string(),
                    });
                }
            }
            HaAction::FollowLeader { .. } | HaAction::SignalFailSafe => {
                // These actions are coordination-only in this task; they intentionally do not
                // enqueue process jobs.
            }
        }
    }

    errors
}

fn dcs_error_message(error: DcsStoreError) -> String {
    error.to_string()
}

fn world_snapshot(ctx: &HaWorkerCtx) -> WorldSnapshot {
    WorldSnapshot {
        config: ctx.config_subscriber.latest(),
        pg: ctx.pg_subscriber.latest(),
        dcs: ctx.dcs_subscriber.latest(),
        process: ctx.process_subscriber.latest(),
    }
}

fn leader_path(scope: &str) -> String {
    format!("/{}/leader", scope.trim_matches('/'))
}

fn switchover_path(scope: &str) -> String {
    format!("/{}/switchover", scope.trim_matches('/'))
}

fn process_job_id(action: &HaAction, index: usize, tick: u64, now_millis: u64) -> JobId {
    JobId(format!(
        "ha-{}-{}-{}-{}",
        tick,
        index,
        action_id_label(&action.id()),
        now_millis
    ))
}

fn action_id_label(id: &ActionId) -> String {
    match id {
        ActionId::AcquireLeaderLease => "acquire_leader_lease".to_string(),
        ActionId::ReleaseLeaderLease => "release_leader_lease".to_string(),
        ActionId::ClearSwitchover => "clear_switchover".to_string(),
        ActionId::FollowLeader(leader) => format!("follow_leader_{}", leader),
        ActionId::StartRewind => "start_rewind".to_string(),
        ActionId::RunBootstrap => "run_bootstrap".to_string(),
        ActionId::FenceNode => "fence_node".to_string(),
        ActionId::SignalFailSafe => "signal_failsafe".to_string(),
        ActionId::StartPostgres => "start_postgres".to_string(),
        ActionId::PromoteToPrimary => "promote_to_primary".to_string(),
        ActionId::DemoteToReplica => "demote_to_replica".to_string(),
    }
}

fn format_dispatch_errors(errors: &[ActionDispatchError]) -> String {
    let mut details = String::new();
    for (index, err) in errors.iter().enumerate() {
        if index > 0 {
            details.push_str("; ");
        }
        details.push_str(&err.to_string());
    }
    format!(
        "ha dispatch failed with {} error(s): {details}",
        errors.len()
    )
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, BTreeSet, VecDeque},
        sync::{Arc, Mutex},
        time::Duration,
    };

    use crate::{
        config::{
            schema::{
                ApiConfig, ClusterConfig, DcsConfig, DebugConfig, HaConfig, PostgresConfig,
                SecurityConfig,
            },
            BinaryPaths, ProcessConfig, RuntimeConfig,
        },
        dcs::{
            state::{DcsCache, DcsState, DcsTrust, LeaderRecord, MemberRecord, MemberRole},
            store::{DcsStore, DcsStoreError, WatchEvent, WatchOp},
        },
        ha::{
            actions::{ActionId, HaAction},
            state::{
                HaPhase, HaState, HaWorkerContractStubInputs, HaWorkerCtx, ProcessDispatchDefaults,
            },
            worker::{dispatch_actions, run, step_once, ActionDispatchError},
        },
        pginfo::state::{
            PgConfig, PgConnInfo, PgInfoCommon, PgInfoState, PgSslMode, Readiness, SqlStatus,
        },
        process::{
            jobs::{
                ProcessCommandRunner, ProcessCommandSpec, ProcessError, ProcessExit, ProcessHandle,
                ShutdownMode,
            },
            state::{
                JobOutcome, ProcessJobKind, ProcessJobRequest, ProcessState, ProcessWorkerCtx,
            },
        },
        state::{new_state_channel, MemberId, UnixMillis, Version, WorkerError, WorkerStatus},
    };

    #[derive(Clone, Default)]
    struct RecordingStore {
        fail_write: bool,
        fail_delete: bool,
        writes: Arc<Mutex<Vec<(String, String)>>>,
        deletes: Arc<Mutex<Vec<String>>>,
        events: Arc<Mutex<VecDeque<WatchEvent>>>,
    }

    impl RecordingStore {
        fn writes_len(&self) -> usize {
            if let Ok(guard) = self.writes.lock() {
                return guard.len();
            }
            0
        }

        fn has_write_path(&self, path: &str) -> bool {
            if let Ok(guard) = self.writes.lock() {
                return guard.iter().any(|(key, _)| key == path);
            }
            false
        }

        fn deletes_len(&self) -> usize {
            if let Ok(guard) = self.deletes.lock() {
                return guard.len();
            }
            0
        }

        fn has_delete_path(&self, path: &str) -> bool {
            if let Ok(guard) = self.deletes.lock() {
                return guard.iter().any(|key| key == path);
            }
            false
        }

        fn push_event(&self, event: WatchEvent) -> Result<(), WorkerError> {
            let mut guard = self
                .events
                .lock()
                .map_err(|_| WorkerError::Message("events lock poisoned".to_string()))?;
            guard.push_back(event);
            Ok(())
        }

        fn first_write_path(&self) -> Option<String> {
            if let Ok(guard) = self.writes.lock() {
                return guard.first().map(|(path, _)| path.clone());
            }
            None
        }
    }

    impl DcsStore for RecordingStore {
        fn healthy(&self) -> bool {
            true
        }

        fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
            if self.fail_write {
                return Err(DcsStoreError::Io("forced write failure".to_string()));
            }
            let mut guard = self
                .writes
                .lock()
                .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
            guard.push((path.to_string(), value));
            Ok(())
        }

        fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError> {
            if self.fail_delete {
                return Err(DcsStoreError::Io("forced delete failure".to_string()));
            }
            let mut guard = self
                .deletes
                .lock()
                .map_err(|_| DcsStoreError::Io("deletes lock poisoned".to_string()))?;
            guard.push(path.to_string());
            Ok(())
        }

        fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
            let mut guard = self
                .events
                .lock()
                .map_err(|_| DcsStoreError::Io("events lock poisoned".to_string()))?;
            Ok(guard.drain(..).collect())
        }
    }

    fn sample_runtime_config() -> RuntimeConfig {
        RuntimeConfig {
            cluster: ClusterConfig {
                name: "cluster-a".to_string(),
                member_id: "node-a".to_string(),
            },
            postgres: PostgresConfig {
                data_dir: "/tmp/pgdata".into(),
                connect_timeout_s: 5,
            },
            dcs: DcsConfig {
                endpoints: vec!["http://127.0.0.1:2379".to_string()],
                scope: "scope-a".to_string(),
            },
            ha: HaConfig {
                loop_interval_ms: 1000,
                lease_ttl_ms: 10_000,
            },
            process: ProcessConfig {
                pg_rewind_timeout_ms: 1000,
                bootstrap_timeout_ms: 1000,
                fencing_timeout_ms: 1000,
                binaries: BinaryPaths {
                    postgres: "/usr/bin/postgres".into(),
                    pg_ctl: "/usr/bin/pg_ctl".into(),
                    pg_rewind: "/usr/bin/pg_rewind".into(),
                    initdb: "/usr/bin/initdb".into(),
                    psql: "/usr/bin/psql".into(),
                },
            },
            api: ApiConfig {
                listen_addr: "127.0.0.1:8080".to_string(),
                read_auth_token: None,
                admin_auth_token: None,
            },
            debug: DebugConfig { enabled: true },
            security: SecurityConfig {
                tls_enabled: false,
                auth_token: None,
            },
        }
    }

    fn sample_pg_state(sql: SqlStatus) -> PgInfoState {
        PgInfoState::Unknown {
            common: PgInfoCommon {
                worker: WorkerStatus::Running,
                sql,
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

    fn sample_dcs_state(config: RuntimeConfig, trust: DcsTrust) -> DcsState {
        DcsState {
            worker: WorkerStatus::Running,
            trust,
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

    fn sample_ha_state() -> HaState {
        HaState {
            worker: WorkerStatus::Starting,
            phase: HaPhase::Init,
            tick: 0,
            pending: Vec::new(),
            recent_action_ids: BTreeSet::new(),
        }
    }

    fn sample_process_defaults() -> ProcessDispatchDefaults {
        ProcessDispatchDefaults {
            postgres_host: "127.0.0.1".to_string(),
            postgres_port: 5432,
            socket_dir: "/tmp/pgtuskmaster/socket".into(),
            log_file: "/tmp/pgtuskmaster/postgres.log".into(),
            rewind_source_conninfo: PgConnInfo {
                host: "127.0.0.1".to_string(),
                port: 5432,
                user: "postgres".to_string(),
                dbname: "postgres".to_string(),
                application_name: None,
                connect_timeout_s: None,
                ssl_mode: PgSslMode::Prefer,
                options: None,
            },
            shutdown_mode: ShutdownMode::Fast,
        }
    }

    fn monotonic_clock(start: u64) -> Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send> {
        let clock = Arc::new(Mutex::new(start));
        Box::new(move || {
            let mut guard = clock
                .lock()
                .map_err(|_| WorkerError::Message("clock lock poisoned".to_string()))?;
            let now = *guard;
            *guard = guard.saturating_add(1);
            Ok(UnixMillis(now))
        })
    }

    struct BuiltContext {
        ctx: HaWorkerCtx,
        ha_subscriber: crate::state::StateSubscriber<HaState>,
        _config_publisher: crate::state::StatePublisher<RuntimeConfig>,
        pg_publisher: crate::state::StatePublisher<PgInfoState>,
        _dcs_publisher: crate::state::StatePublisher<DcsState>,
        _process_publisher: crate::state::StatePublisher<ProcessState>,
        process_rx: tokio::sync::mpsc::UnboundedReceiver<ProcessJobRequest>,
        store: RecordingStore,
    }

    fn build_context(
        store: RecordingStore,
        poll_interval: Duration,
        dcs_trust: DcsTrust,
    ) -> BuiltContext {
        let runtime_config = sample_runtime_config();
        let (config_publisher, config_subscriber) =
            new_state_channel(runtime_config.clone(), UnixMillis(1));
        let (pg_publisher, pg_subscriber) =
            new_state_channel(sample_pg_state(SqlStatus::Healthy), UnixMillis(1));
        let dcs_state = sample_dcs_state(runtime_config.clone(), dcs_trust);
        let (dcs_publisher, dcs_subscriber) = new_state_channel(dcs_state, UnixMillis(1));
        let (process_publisher, process_subscriber) =
            new_state_channel(sample_process_state(), UnixMillis(1));
        let (ha_publisher, ha_subscriber) = new_state_channel(sample_ha_state(), UnixMillis(1));
        let (process_tx, process_rx) = tokio::sync::mpsc::unbounded_channel();

        let mut ctx = HaWorkerCtx::contract_stub(HaWorkerContractStubInputs {
            publisher: ha_publisher,
            config_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            process_inbox: process_tx,
            dcs_store: Box::new(store.clone()),
            scope: "scope-a".to_string(),
            self_id: MemberId("node-a".to_string()),
        });
        ctx.poll_interval = poll_interval;
        ctx.state = sample_ha_state();
        ctx.process_defaults = sample_process_defaults();
        ctx.now = monotonic_clock(10);

        BuiltContext {
            ctx,
            ha_subscriber,
            _config_publisher: config_publisher,
            pg_publisher,
            _dcs_publisher: dcs_publisher,
            _process_publisher: process_publisher,
            process_rx,
            store,
        }
    }

    #[derive(Clone)]
    struct ScriptedProcess {
        polls: VecDeque<Result<Option<ProcessExit>, ProcessError>>,
        cancel_result: Result<(), ProcessError>,
    }

    struct ScriptedHandle {
        polls: VecDeque<Result<Option<ProcessExit>, ProcessError>>,
        cancel_result: Result<(), ProcessError>,
    }

    impl ProcessHandle for ScriptedHandle {
        fn poll_exit(&mut self) -> Result<Option<ProcessExit>, ProcessError> {
            match self.polls.pop_front() {
                Some(next) => next,
                None => Ok(None),
            }
        }

        fn cancel<'a>(
            &'a mut self,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<(), ProcessError>> + Send + 'a>,
        > {
            let result = self.cancel_result.clone();
            Box::pin(async move { result })
        }
    }

    #[derive(Clone, Default)]
    struct ScriptedRunner {
        scripts: Arc<Mutex<VecDeque<Result<ScriptedProcess, ProcessError>>>>,
        spawned_specs: Arc<Mutex<Vec<ProcessCommandSpec>>>,
    }

    impl ScriptedRunner {
        fn queue_success_exit(&self) -> Result<(), WorkerError> {
            let mut scripts = self
                .scripts
                .lock()
                .map_err(|_| WorkerError::Message("scripts lock poisoned".to_string()))?;
            scripts.push_back(Ok(ScriptedProcess {
                polls: VecDeque::from(vec![Ok(Some(ProcessExit::Success))]),
                cancel_result: Ok(()),
            }));
            Ok(())
        }

        fn spawn_count(&self) -> usize {
            if let Ok(specs) = self.spawned_specs.lock() {
                return specs.len();
            }
            0
        }

        fn any_spawn_contains_arg(&self, needle: &str) -> bool {
            if let Ok(specs) = self.spawned_specs.lock() {
                return specs
                    .iter()
                    .any(|spec| spec.args.iter().any(|arg| arg == needle));
            }
            false
        }
    }

    impl ProcessCommandRunner for ScriptedRunner {
        fn spawn(
            &mut self,
            spec: ProcessCommandSpec,
        ) -> Result<Box<dyn ProcessHandle>, ProcessError> {
            {
                let mut spawned = self
                    .spawned_specs
                    .lock()
                    .map_err(|_| ProcessError::OperationFailed)?;
                spawned.push(spec);
            }
            let scripted = {
                let mut scripts = self
                    .scripts
                    .lock()
                    .map_err(|_| ProcessError::OperationFailed)?;
                match scripts.pop_front() {
                    Some(next) => next,
                    None => Err(ProcessError::UnsupportedInput(
                        "scripted runner queue exhausted".to_string(),
                    )),
                }
            }?;

            Ok(Box::new(ScriptedHandle {
                polls: scripted.polls,
                cancel_result: scripted.cancel_result,
            }))
        }
    }

    struct IntegrationFixture {
        store: RecordingStore,
        runner: ScriptedRunner,
        _config_publisher: crate::state::StatePublisher<RuntimeConfig>,
        pg_publisher: crate::state::StatePublisher<PgInfoState>,
        dcs_subscriber: crate::state::StateSubscriber<DcsState>,
        process_subscriber: crate::state::StateSubscriber<ProcessState>,
        ha_subscriber: crate::state::StateSubscriber<HaState>,
        dcs_ctx: crate::dcs::state::DcsWorkerCtx,
        process_ctx: ProcessWorkerCtx,
        ha_ctx: HaWorkerCtx,
        next_revision: i64,
    }

    impl IntegrationFixture {
        fn new(initial_phase: HaPhase) -> Self {
            let runtime_config = sample_runtime_config();
            let store = RecordingStore::default();
            let runner = ScriptedRunner::default();

            let (config_publisher, config_subscriber) =
                new_state_channel(runtime_config.clone(), UnixMillis(1));
            let (pg_publisher, pg_subscriber) =
                new_state_channel(sample_pg_state(SqlStatus::Healthy), UnixMillis(1));
            let (dcs_publisher, dcs_subscriber) = new_state_channel(
                sample_dcs_state(runtime_config.clone(), DcsTrust::NotTrusted),
                UnixMillis(1),
            );
            let (process_publisher, process_subscriber) =
                new_state_channel(sample_process_state(), UnixMillis(1));
            let (ha_publisher, ha_subscriber) = new_state_channel(sample_ha_state(), UnixMillis(1));
            let (process_tx, process_rx) = tokio::sync::mpsc::unbounded_channel();

            let dcs_ctx = crate::dcs::state::DcsWorkerCtx {
                self_id: MemberId("node-a".to_string()),
                scope: "scope-a".to_string(),
                poll_interval: Duration::from_millis(5),
                pg_subscriber: pg_subscriber.clone(),
                publisher: dcs_publisher,
                store: Box::new(store.clone()),
                cache: DcsCache {
                    members: BTreeMap::new(),
                    leader: None,
                    switchover: None,
                    config: runtime_config.clone(),
                    init_lock: None,
                },
                last_published_pg_version: None,
            };

            let mut process_ctx = ProcessWorkerCtx::contract_stub(
                runtime_config.process.clone(),
                process_publisher,
                process_rx,
            );
            process_ctx.poll_interval = Duration::from_millis(5);
            process_ctx.command_runner = Box::new(runner.clone());
            process_ctx.now = monotonic_clock(100);

            let mut ha_ctx = HaWorkerCtx::contract_stub(HaWorkerContractStubInputs {
                publisher: ha_publisher,
                config_subscriber,
                pg_subscriber,
                dcs_subscriber: dcs_subscriber.clone(),
                process_subscriber: process_subscriber.clone(),
                process_inbox: process_tx,
                dcs_store: Box::new(store.clone()),
                scope: "scope-a".to_string(),
                self_id: MemberId("node-a".to_string()),
            });
            ha_ctx.poll_interval = Duration::from_millis(5);
            ha_ctx.process_defaults = sample_process_defaults();
            ha_ctx.now = monotonic_clock(1_000);
            ha_ctx.state = HaState {
                worker: WorkerStatus::Running,
                phase: initial_phase,
                tick: 0,
                pending: Vec::new(),
                recent_action_ids: BTreeSet::new(),
            };

            Self {
                store,
                runner,
                _config_publisher: config_publisher,
                pg_publisher,
                dcs_subscriber,
                process_subscriber,
                ha_subscriber,
                dcs_ctx,
                process_ctx,
                ha_ctx,
                next_revision: 1,
            }
        }

        fn queue_process_success(&self) -> Result<(), WorkerError> {
            self.runner.queue_success_exit()
        }

        fn publish_pg_sql(&self, status: SqlStatus, now: u64) -> Result<(), WorkerError> {
            self.pg_publisher
                .publish(sample_pg_state(status), UnixMillis(now))
                .map(|_| ())
                .map_err(|err| WorkerError::Message(format!("pg publish failed: {err}")))
        }

        fn push_member_event(
            &mut self,
            member_id: &str,
            role: MemberRole,
        ) -> Result<(), WorkerError> {
            let record = sample_member_record(member_id, role);
            let value = serde_json::to_string(&record)
                .map_err(|err| WorkerError::Message(format!("member encode failed: {err}")))?;
            let event = WatchEvent {
                op: WatchOp::Put,
                path: format!("/scope-a/member/{member_id}"),
                value: Some(value),
                revision: self.take_revision(),
            };
            self.store.push_event(event)
        }

        fn push_leader_event(&mut self, member_id: &str) -> Result<(), WorkerError> {
            let record = LeaderRecord {
                member_id: MemberId(member_id.to_string()),
            };
            let value = serde_json::to_string(&record)
                .map_err(|err| WorkerError::Message(format!("leader encode failed: {err}")))?;
            let event = WatchEvent {
                op: WatchOp::Put,
                path: "/scope-a/leader".to_string(),
                value: Some(value),
                revision: self.take_revision(),
            };
            self.store.push_event(event)
        }

        fn delete_leader_event(&mut self) -> Result<(), WorkerError> {
            let event = WatchEvent {
                op: WatchOp::Delete,
                path: "/scope-a/leader".to_string(),
                value: None,
                revision: self.take_revision(),
            };
            self.store.push_event(event)
        }

        async fn step_dcs_and_ha(&mut self) -> Result<(), WorkerError> {
            crate::dcs::worker::step_once(&mut self.dcs_ctx).await?;
            step_once(&mut self.ha_ctx).await
        }

        async fn step_dcs_ha_process_ha(&mut self) -> Result<(), WorkerError> {
            crate::dcs::worker::step_once(&mut self.dcs_ctx).await?;
            step_once(&mut self.ha_ctx).await?;
            crate::process::worker::step_once(&mut self.process_ctx).await?;
            step_once(&mut self.ha_ctx).await
        }

        fn latest_ha(&self) -> HaState {
            self.ha_subscriber.latest().value
        }

        fn latest_dcs(&self) -> DcsState {
            self.dcs_subscriber.latest().value
        }

        fn latest_process(&self) -> ProcessState {
            self.process_subscriber.latest().value
        }

        fn take_revision(&mut self) -> i64 {
            let current = self.next_revision;
            self.next_revision = self.next_revision.saturating_add(1);
            current
        }
    }

    fn sample_member_record(member_id: &str, role: MemberRole) -> MemberRecord {
        MemberRecord {
            member_id: MemberId(member_id.to_string()),
            role,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
            timeline: None,
            write_lsn: None,
            replay_lsn: None,
            updated_at: UnixMillis(1),
            pg_version: Version(1),
        }
    }

    async fn wait_for_ha_version(
        subscriber: &crate::state::StateSubscriber<HaState>,
        min_version: u64,
        timeout: Duration,
    ) -> bool {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if subscriber.latest().version.0 >= min_version {
                return true;
            }
            if tokio::time::Instant::now() >= deadline {
                return false;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_uses_subscribers_and_publishes_next_state() {
        let built = build_context(
            RecordingStore::default(),
            Duration::from_millis(100),
            DcsTrust::FullQuorum,
        );
        let mut ctx = built.ctx;
        let subscriber = built.ha_subscriber;

        let stepped = step_once(&mut ctx).await;
        assert_eq!(stepped, Ok(()));
        assert_eq!(ctx.state.phase, HaPhase::WaitingPostgresReachable);
        assert_eq!(ctx.state.tick, 1);
        assert_eq!(ctx.state.worker, WorkerStatus::Running);

        let published = subscriber.latest();
        assert_eq!(published.version, Version(1));
        assert_eq!(published.value.phase, HaPhase::WaitingPostgresReachable);
        assert_eq!(published.value.tick, 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn dispatch_actions_maps_dcs_and_process_requests() {
        let built = build_context(
            RecordingStore::default(),
            Duration::from_millis(100),
            DcsTrust::FullQuorum,
        );
        let mut ctx = built.ctx;
        let mut process_rx = built.process_rx;
        let store = built.store;
        let actions = vec![
            HaAction::AcquireLeaderLease,
            HaAction::StartPostgres,
            HaAction::ReleaseLeaderLease,
        ];

        let errors = dispatch_actions(&mut ctx, &actions);
        assert!(errors.is_empty());
        assert_eq!(store.writes_len(), 1);
        assert_eq!(store.deletes_len(), 1);
        assert_eq!(
            store.first_write_path(),
            Some("/scope-a/leader".to_string())
        );

        let request = process_rx.try_recv();
        assert!(request.is_ok());
        if let Ok(job) = request {
            assert!(matches!(job.kind, ProcessJobKind::StartPostgres(_)));
            if let ProcessJobKind::StartPostgres(spec) = job.kind {
                assert_eq!(spec.data_dir, sample_runtime_config().postgres.data_dir);
                assert_eq!(spec.host, "127.0.0.1");
                assert_eq!(spec.port, 5432);
            }
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn dispatch_actions_is_best_effort_and_reports_typed_errors() {
        let store = RecordingStore {
            fail_write: true,
            ..RecordingStore::default()
        };
        let built = build_context(store, Duration::from_millis(100), DcsTrust::FullQuorum);
        let mut ctx = built.ctx;
        let process_rx = built.process_rx;
        let store_handle = built.store;
        drop(process_rx);

        let actions = vec![
            HaAction::AcquireLeaderLease,
            HaAction::StartPostgres,
            HaAction::ReleaseLeaderLease,
        ];
        let errors = dispatch_actions(&mut ctx, &actions);

        assert_eq!(store_handle.deletes_len(), 1);
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().any(|err| matches!(
            err,
            ActionDispatchError::DcsWrite {
                action: ActionId::AcquireLeaderLease,
                ..
            }
        )));
        assert!(errors.iter().any(|err| matches!(
            err,
            ActionDispatchError::ProcessSend {
                action: ActionId::StartPostgres,
                ..
            }
        )));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn dispatch_actions_clears_switchover_key() {
        let built = build_context(
            RecordingStore::default(),
            Duration::from_millis(100),
            DcsTrust::FullQuorum,
        );
        let mut ctx = built.ctx;
        let store = built.store;

        let errors = dispatch_actions(&mut ctx, &[HaAction::ClearSwitchover]);
        assert!(errors.is_empty());
        assert!(store.has_delete_path("/scope-a/switchover"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn integration_transitions_replica_candidate_primary_and_failsafe() {
        let mut fixture = IntegrationFixture::new(HaPhase::WaitingDcsTrusted);

        assert_eq!(fixture.publish_pg_sql(SqlStatus::Healthy, 10), Ok(()));
        assert_eq!(
            fixture.push_member_event("node-b", MemberRole::Primary),
            Ok(())
        );
        assert_eq!(fixture.push_leader_event("node-b"), Ok(()));
        assert_eq!(fixture.step_dcs_and_ha().await, Ok(()));

        let replica = fixture.latest_ha();
        assert_eq!(replica.phase, HaPhase::Replica);
        assert_eq!(
            replica.pending,
            vec![HaAction::FollowLeader {
                leader_member_id: "node-b".to_string(),
            }]
        );
        assert_eq!(fixture.latest_dcs().trust, DcsTrust::FullQuorum);

        assert_eq!(fixture.delete_leader_event(), Ok(()));
        assert_eq!(fixture.step_dcs_and_ha().await, Ok(()));
        let candidate = fixture.latest_ha();
        assert_eq!(candidate.phase, HaPhase::CandidateLeader);
        assert_eq!(candidate.pending, vec![HaAction::AcquireLeaderLease]);
        assert!(fixture.store.has_write_path("/scope-a/leader"));

        assert_eq!(fixture.push_leader_event("node-a"), Ok(()));
        assert_eq!(fixture.step_dcs_and_ha().await, Ok(()));
        let primary = fixture.latest_ha();
        assert_eq!(primary.phase, HaPhase::Primary);
        assert_eq!(primary.pending, vec![HaAction::PromoteToPrimary]);

        assert_eq!(fixture.push_leader_event("node-z"), Ok(()));
        assert_eq!(fixture.step_dcs_and_ha().await, Ok(()));
        let failsafe = fixture.latest_ha();
        assert_eq!(failsafe.phase, HaPhase::FailSafe);
        assert!(failsafe
            .pending
            .iter()
            .any(|action| matches!(action, HaAction::SignalFailSafe)));
        assert!(fixture.store.has_delete_path("/scope-a/leader"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn integration_primary_unreachable_rewinds_then_returns_replica_on_success() {
        let mut fixture = IntegrationFixture::new(HaPhase::Primary);

        assert_eq!(fixture.publish_pg_sql(SqlStatus::Unreachable, 20), Ok(()));
        assert_eq!(fixture.queue_process_success(), Ok(()));
        assert_eq!(fixture.step_dcs_ha_process_ha().await, Ok(()));

        let latest_ha = fixture.latest_ha();
        assert_eq!(latest_ha.phase, HaPhase::Replica);
        assert!(fixture.runner.any_spawn_contains_arg("--target-pgdata"));
        assert!(matches!(
            fixture.latest_process(),
            ProcessState::Idle {
                last_outcome: Some(JobOutcome::Success { .. }),
                ..
            }
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn integration_primary_split_brain_enters_fencing_and_process_feedback_advances() {
        let mut fixture = IntegrationFixture::new(HaPhase::Primary);

        assert_eq!(fixture.publish_pg_sql(SqlStatus::Healthy, 30), Ok(()));
        assert_eq!(
            fixture.push_member_event("node-b", MemberRole::Primary),
            Ok(())
        );
        assert_eq!(fixture.push_leader_event("node-b"), Ok(()));
        assert_eq!(fixture.step_dcs_and_ha().await, Ok(()));

        let fencing = fixture.latest_ha();
        assert_eq!(fencing.phase, HaPhase::Fencing);
        assert_eq!(
            fencing.pending,
            vec![
                HaAction::DemoteToReplica,
                HaAction::ReleaseLeaderLease,
                HaAction::FenceNode,
            ]
        );
        assert!(fixture.store.has_delete_path("/scope-a/leader"));

        assert_eq!(fixture.queue_process_success(), Ok(()));
        let process_version_before = fixture.process_subscriber.latest().version;
        assert_eq!(
            crate::process::worker::step_once(&mut fixture.process_ctx).await,
            Ok(())
        );
        assert_eq!(step_once(&mut fixture.ha_ctx).await, Ok(()));

        let process_version_after = fixture.process_subscriber.latest().version;
        assert_eq!(
            process_version_after.0,
            process_version_before.0.saturating_add(2)
        );
        assert!(matches!(
            fixture.latest_process(),
            ProcessState::Idle {
                last_outcome: Some(JobOutcome::Success { .. }),
                ..
            }
        ));
        assert_eq!(fixture.latest_ha().phase, HaPhase::WaitingDcsTrusted);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn integration_start_postgres_dispatch_updates_process_state_versions() {
        let mut fixture = IntegrationFixture::new(HaPhase::WaitingPostgresReachable);

        assert_eq!(fixture.publish_pg_sql(SqlStatus::Unreachable, 40), Ok(()));
        assert_eq!(fixture.step_dcs_and_ha().await, Ok(()));
        let waiting = fixture.latest_ha();
        assert_eq!(waiting.phase, HaPhase::WaitingPostgresReachable);
        assert_eq!(waiting.pending, vec![HaAction::StartPostgres]);

        assert_eq!(fixture.queue_process_success(), Ok(()));
        let process_version_before = fixture.process_subscriber.latest().version;
        assert_eq!(
            crate::process::worker::step_once(&mut fixture.process_ctx).await,
            Ok(())
        );
        let process_version_after = fixture.process_subscriber.latest().version;
        assert_eq!(
            process_version_after.0,
            process_version_before.0.saturating_add(2)
        );
        assert!(matches!(
            fixture.latest_process(),
            ProcessState::Idle {
                last_outcome: Some(JobOutcome::Success { .. }),
                ..
            }
        ));
        assert!(fixture.runner.any_spawn_contains_arg("start"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_reacts_to_interval_tick_and_watcher_change() {
        let built = build_context(
            RecordingStore::default(),
            Duration::from_millis(20),
            DcsTrust::FullQuorum,
        );
        let ctx = built.ctx;
        let subscriber = built.ha_subscriber;
        let pg_publisher = built.pg_publisher;

        let handle = tokio::spawn(async move { run(ctx).await });

        let first_advanced = wait_for_ha_version(&subscriber, 1, Duration::from_millis(250)).await;
        assert!(first_advanced);

        let publish_result =
            pg_publisher.publish(sample_pg_state(SqlStatus::Unreachable), UnixMillis(50));
        assert!(publish_result.is_ok());
        let second_advanced = wait_for_ha_version(&subscriber, 2, Duration::from_millis(250)).await;
        assert!(second_advanced);

        handle.abort();
        let _ = handle.await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_first_tick_matches_step_once_for_same_inputs() {
        let built_step = build_context(
            RecordingStore::default(),
            Duration::from_millis(200),
            DcsTrust::FullQuorum,
        );
        let mut step_ctx = built_step.ctx;

        let stepped = step_once(&mut step_ctx).await;
        assert_eq!(stepped, Ok(()));
        let expected = step_ctx.state.clone();

        let built_run = build_context(
            RecordingStore::default(),
            Duration::from_millis(200),
            DcsTrust::FullQuorum,
        );
        let run_ctx = built_run.ctx;
        let run_subscriber = built_run.ha_subscriber;
        let handle = tokio::spawn(async move { run(run_ctx).await });

        let advanced = wait_for_ha_version(&run_subscriber, 1, Duration::from_millis(250)).await;
        assert!(advanced);
        let observed = run_subscriber.latest().value;
        assert_eq!(observed.phase, expected.phase);
        assert_eq!(observed.tick, expected.tick);
        assert_eq!(observed.pending, expected.pending);

        handle.abort();
        let _ = handle.await;
    }
}
