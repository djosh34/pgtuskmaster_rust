use thiserror::Error;

use crate::{
    dcs::{state::LeaderRecord, store::DcsStoreError},
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
    let runtime_config = ctx.config_subscriber.latest().value;

    for (index, action) in actions.iter().enumerate() {
        match action {
            HaAction::AcquireLeaderLease => {
                let leader_payload = serde_json::to_string(&LeaderRecord {
                    member_id: ctx.self_id.clone(),
                });
                let encoded = match leader_payload {
                    Ok(value) => value,
                    Err(err) => {
                        errors.push(ActionDispatchError::DcsWrite {
                            action: action.id(),
                            path: leader_key.clone(),
                            message: err.to_string(),
                        });
                        continue;
                    }
                };
                if let Err(err) = ctx.dcs_store.write_path(&leader_key, encoded) {
                    errors.push(ActionDispatchError::DcsWrite {
                        action: action.id(),
                        path: leader_key.clone(),
                        message: dcs_error_message(err),
                    });
                }
            }
            HaAction::ReleaseLeaderLease => {
                if let Err(err) = ctx.dcs_store.delete_path(&leader_key) {
                    errors.push(ActionDispatchError::DcsDelete {
                        action: action.id(),
                        path: leader_key.clone(),
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
            state::{DcsCache, DcsState, DcsTrust},
            store::{DcsStore, DcsStoreError, WatchEvent},
        },
        ha::{
            actions::{ActionId, HaAction},
            state::{
                HaPhase, HaState, HaWorkerContractStubInputs, HaWorkerCtx, ProcessDispatchDefaults,
            },
            worker::{dispatch_actions, run, step_once, ActionDispatchError},
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::{
            jobs::ShutdownMode,
            state::{ProcessJobKind, ProcessJobRequest, ProcessState},
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

        fn deletes_len(&self) -> usize {
            if let Ok(guard) = self.deletes.lock() {
                return guard.len();
            }
            0
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
            rewind_source_conninfo: "host=127.0.0.1 port=5432 user=postgres dbname=postgres"
                .to_string(),
            shutdown_mode: ShutdownMode::Fast,
        }
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
        let clock = Arc::new(Mutex::new(10_u64));
        ctx.now = Box::new(move || {
            let mut guard = clock
                .lock()
                .map_err(|_| WorkerError::Message("clock lock poisoned".to_string()))?;
            let now = *guard;
            *guard = guard.saturating_add(1);
            Ok(UnixMillis(now))
        });

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
        match request {
            Ok(job) => match job.kind {
                ProcessJobKind::StartPostgres(spec) => {
                    assert_eq!(spec.data_dir, sample_runtime_config().postgres.data_dir);
                    assert_eq!(spec.host, "127.0.0.1");
                    assert_eq!(spec.port, 5432);
                }
                other => panic!("unexpected process job kind: {other:?}"),
            },
            Err(err) => panic!("expected process request, got: {err}"),
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
