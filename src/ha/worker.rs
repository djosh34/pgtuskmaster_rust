use crate::{
    logging::{EventMeta, SeverityText},
    state::{WorkerError, WorkerStatus},
};

use super::{
    apply::{apply_effect_plan, format_dispatch_errors},
    decide::decide,
    events::{
        emit_ha_decision_selected, emit_ha_effect_plan_selected, ha_base_attrs, ha_role_label,
        serialize_attr_value,
    },
    state::{DecideInput, HaWorkerCtx, WorldSnapshot},
};

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
    let prev_phase = ctx.state.phase.clone();
    let world = world_snapshot(ctx);
    let output = decide(DecideInput {
        current: ctx.state.clone(),
        world,
    });
    let plan = output.outcome.decision.lower();

    emit_ha_decision_selected(ctx, output.next.tick, &output.outcome.decision, &plan)?;
    emit_ha_effect_plan_selected(ctx, output.next.tick, &plan)?;
    let dispatch_errors = apply_effect_plan(ctx, output.next.tick, &plan)?;
    let now = (ctx.now)()?;

    let worker = if dispatch_errors.is_empty() {
        WorkerStatus::Running
    } else {
        WorkerStatus::Faulted(WorkerError::Message(format_dispatch_errors(
            &dispatch_errors,
        )))
    };
    let next = crate::ha::state::HaState {
        worker,
        ..output.next
    };

    ctx.publisher
        .publish(next.clone(), now)
        .map_err(|err| WorkerError::Message(format!("ha publish failed: {err}")))?;

    if prev_phase != next.phase {
        let mut attrs = ha_base_attrs(ctx, next.tick);
        attrs.insert("phase_prev".to_string(), serialize_attr_value(&prev_phase)?);
        attrs.insert("phase_next".to_string(), serialize_attr_value(&next.phase)?);
        ctx.log
            .emit_event(
                SeverityText::Info,
                "ha phase transition",
                "ha_worker::step_once",
                EventMeta::new("ha.phase.transition", "ha", "ok"),
                attrs,
            )
            .map_err(|err| WorkerError::Message(format!("ha phase log emit failed: {err}")))?;
    }

    let prev_role = ha_role_label(&prev_phase);
    let next_role = ha_role_label(&next.phase);
    if prev_role != next_role {
        let mut attrs = ha_base_attrs(ctx, next.tick);
        attrs.insert(
            "role_prev".to_string(),
            serde_json::Value::String(prev_role.to_string()),
        );
        attrs.insert(
            "role_next".to_string(),
            serde_json::Value::String(next_role.to_string()),
        );
        ctx.log
            .emit_event(
                SeverityText::Info,
                "ha role transition",
                "ha_worker::step_once",
                EventMeta::new("ha.role.transition", "ha", "ok"),
                attrs,
            )
            .map_err(|err| WorkerError::Message(format!("ha role log emit failed: {err}")))?;
    }

    ctx.state = next;
    Ok(())
}

fn world_snapshot(ctx: &HaWorkerCtx) -> WorldSnapshot {
    WorldSnapshot {
        config: ctx.config_subscriber.latest(),
        pg: ctx.pg_subscriber.latest(),
        dcs: ctx.dcs_subscriber.latest(),
        process: ctx.process_subscriber.latest(),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, VecDeque},
        path::PathBuf,
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc, Mutex,
        },
        time::{Duration, SystemTime, UNIX_EPOCH},
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
            state::{DcsCache, DcsState, DcsTrust, LeaderRecord, MemberRecord, MemberRole},
            store::{DcsStore, DcsStoreError, WatchEvent, WatchOp},
        },
        ha::{
            actions::ActionId,
            apply::{apply_effect_plan, ActionDispatchError},
            decision::HaDecision,
            lower::{
                lower_decision, HaEffectPlan, LeaseEffect, PostgresEffect, ReplicationEffect,
                SafetyEffect, SwitchoverEffect,
            },
            state::{
                HaPhase, HaState, HaWorkerContractStubInputs, HaWorkerCtx, ProcessDispatchDefaults,
            },
            worker::{run, step_once},
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

        fn read_path(&mut self, _path: &str) -> Result<Option<String>, DcsStoreError> {
            Ok(None)
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

        fn put_path_if_absent(&mut self, path: &str, value: String) -> Result<bool, DcsStoreError> {
            if self.fail_write {
                return Err(DcsStoreError::Io("forced write failure".to_string()));
            }
            let mut guard = self
                .writes
                .lock()
                .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
            guard.push((path.to_string(), value));
            Ok(true)
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

    static TEST_DATA_DIR_SEQ: AtomicU64 = AtomicU64::new(0);

    fn unique_test_data_dir(label: &str) -> PathBuf {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis());
        let sequence = TEST_DATA_DIR_SEQ.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "pgtuskmaster-worker-{label}-{}-{millis}-{sequence}",
            std::process::id(),
        ))
    }

    fn sample_runtime_config() -> RuntimeConfig {
        RuntimeConfig {
            cluster: ClusterConfig {
                name: "cluster-a".to_string(),
                member_id: "node-a".to_string(),
            },
            postgres: PostgresConfig {
                data_dir: unique_test_data_dir("pgdata"),
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
            decision: HaDecision::NoChange,
        }
    }

    fn sample_process_defaults() -> ProcessDispatchDefaults {
        ProcessDispatchDefaults {
            postgres_host: "127.0.0.1".to_string(),
            postgres_port: 5432,
            socket_dir: "/tmp/pgtuskmaster/socket".into(),
            log_file: "/tmp/pgtuskmaster/postgres.log".into(),
            basebackup_source: crate::process::jobs::ReplicatorSourceConn {
                conninfo: PgConnInfo {
                    host: "127.0.0.1".to_string(),
                    port: 5432,
                    user: "replicator".to_string(),
                    dbname: "postgres".to_string(),
                    application_name: None,
                    connect_timeout_s: None,
                    ssl_mode: PgSslMode::Prefer,
                    options: None,
                },
                auth: crate::config::RoleAuthConfig::Tls,
            },
            rewind_source: crate::process::jobs::RewinderSourceConn {
                conninfo: PgConnInfo {
                    host: "127.0.0.1".to_string(),
                    port: 5432,
                    user: "rewinder".to_string(),
                    dbname: "postgres".to_string(),
                    application_name: None,
                    connect_timeout_s: None,
                    ssl_mode: PgSslMode::Prefer,
                    options: None,
                },
                auth: crate::config::RoleAuthConfig::Tls,
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

        fn drain_output<'a>(
            &'a mut self,
            _max_bytes: usize,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<Vec<crate::process::jobs::ProcessOutputLine>, ProcessError>,
                    > + Send
                    + 'a,
            >,
        > {
            Box::pin(async move { Ok(Vec::new()) })
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
                    None => Err(ProcessError::InvalidSpec(
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
                log: crate::logging::LogHandle::null(),
                cache: DcsCache {
                    members: BTreeMap::new(),
                    leader: None,
                    switchover: None,
                    config: runtime_config.clone(),
                    init_lock: None,
                },
                last_published_pg_version: None,
                last_emitted_store_healthy: None,
                last_emitted_trust: None,
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
                decision: HaDecision::NoChange,
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
    async fn step_once_dispatches_start_postgres_every_tick_while_unreachable(
    ) -> Result<(), WorkerError> {
        let built = build_context(
            RecordingStore::default(),
            Duration::from_millis(100),
            DcsTrust::FullQuorum,
        );
        let mut ctx = built.ctx;
        let mut process_rx = built.process_rx;

        built
            .pg_publisher
            .publish(sample_pg_state(SqlStatus::Unreachable), UnixMillis(50))
            .map_err(|err| WorkerError::Message(format!("pg publish failed: {err}")))?;

        ctx.state = HaState {
            worker: WorkerStatus::Running,
            phase: HaPhase::WaitingPostgresReachable,
            tick: 0,
            decision: HaDecision::NoChange,
        };

        step_once(&mut ctx).await?;
        let first = process_rx.try_recv().map_err(|err| {
            WorkerError::Message(format!(
                "expected process job request after first tick: {err}"
            ))
        })?;
        assert!(matches!(first.kind, ProcessJobKind::StartPostgres(_)));

        step_once(&mut ctx).await?;
        let second = process_rx.try_recv().map_err(|err| {
            WorkerError::Message(format!(
                "expected process job request after second tick: {err}"
            ))
        })?;
        assert!(matches!(second.kind, ProcessJobKind::StartPostgres(_)));

        assert_ne!(first.id, second.id);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn apply_effect_plan_maps_dcs_and_process_requests() -> Result<(), WorkerError> {
        let built = build_context(
            RecordingStore::default(),
            Duration::from_millis(100),
            DcsTrust::FullQuorum,
        );
        let mut ctx = built.ctx;
        let mut process_rx = built.process_rx;
        let store = built.store;
        let plan = HaEffectPlan {
            lease: LeaseEffect::ReleaseLeader,
            switchover: SwitchoverEffect::None,
            replication: ReplicationEffect::None,
            postgres: PostgresEffect::Start,
            safety: SafetyEffect::None,
        };
        let acquire_only = HaEffectPlan {
            lease: LeaseEffect::AcquireLeader,
            switchover: SwitchoverEffect::None,
            replication: ReplicationEffect::None,
            postgres: PostgresEffect::None,
            safety: SafetyEffect::None,
        };

        let ha_tick = ctx.state.tick;
        let acquire_errors = apply_effect_plan(&mut ctx, ha_tick, &acquire_only)?;
        assert!(acquire_errors.is_empty());
        let errors = apply_effect_plan(&mut ctx, ha_tick, &plan)?;
        assert!(errors.is_empty(), "dispatch errors were: {errors:?}");
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
                assert_eq!(
                    spec.data_dir,
                    ctx.config_subscriber.latest().value.postgres.data_dir
                );
                assert_eq!(spec.host, "127.0.0.1");
                assert_eq!(spec.port, 5432);
            }
        }
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn apply_effect_plan_is_best_effort_and_reports_typed_errors() -> Result<(), WorkerError>
    {
        let store = RecordingStore {
            fail_write: true,
            ..RecordingStore::default()
        };
        let built = build_context(store, Duration::from_millis(100), DcsTrust::FullQuorum);
        let mut ctx = built.ctx;
        let process_rx = built.process_rx;
        let store_handle = built.store;
        drop(process_rx);

        let plan = HaEffectPlan {
            lease: LeaseEffect::ReleaseLeader,
            switchover: SwitchoverEffect::None,
            replication: ReplicationEffect::None,
            postgres: PostgresEffect::Start,
            safety: SafetyEffect::None,
        };
        let acquire_only = HaEffectPlan {
            lease: LeaseEffect::AcquireLeader,
            switchover: SwitchoverEffect::None,
            replication: ReplicationEffect::None,
            postgres: PostgresEffect::None,
            safety: SafetyEffect::None,
        };
        let ha_tick = ctx.state.tick;
        let mut errors = apply_effect_plan(&mut ctx, ha_tick, &acquire_only)?;
        errors.extend(apply_effect_plan(&mut ctx, ha_tick, &plan)?);

        assert_eq!(store_handle.deletes_len(), 1);
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().any(|err| matches!(
            err,
            ActionDispatchError::DcsWrite {
                action: ActionId::AcquireLeaderLease,
                ..
            }
        )));
        assert!(
            errors.iter().any(|err| matches!(
                err,
                ActionDispatchError::ProcessSend {
                    action: ActionId::StartPostgres,
                    ..
                }
            )),
            "dispatch errors were: {errors:?}"
        );
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn apply_effect_plan_clears_switchover_key() -> Result<(), WorkerError> {
        let built = build_context(
            RecordingStore::default(),
            Duration::from_millis(100),
            DcsTrust::FullQuorum,
        );
        let mut ctx = built.ctx;
        let store = built.store;

        let ha_tick = ctx.state.tick;
        let plan = HaEffectPlan {
            lease: LeaseEffect::None,
            switchover: SwitchoverEffect::ClearRequest,
            replication: ReplicationEffect::None,
            postgres: PostgresEffect::None,
            safety: SafetyEffect::None,
        };
        let errors = apply_effect_plan(&mut ctx, ha_tick, &plan)?;
        assert!(errors.is_empty());
        assert!(store.has_delete_path("/scope-a/switchover"));
        Ok(())
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
            lower_decision(&replica.decision),
            HaEffectPlan {
                lease: LeaseEffect::None,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::FollowLeader {
                    leader_member_id: MemberId("node-b".to_string()),
                },
                postgres: PostgresEffect::None,
                safety: SafetyEffect::None,
            }
        );
        assert_eq!(fixture.latest_dcs().trust, DcsTrust::FullQuorum);

        assert_eq!(fixture.delete_leader_event(), Ok(()));
        assert_eq!(fixture.step_dcs_and_ha().await, Ok(()));
        let candidate = fixture.latest_ha();
        assert_eq!(candidate.phase, HaPhase::CandidateLeader);
        assert_eq!(
            lower_decision(&candidate.decision),
            HaEffectPlan {
                lease: LeaseEffect::AcquireLeader,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::None,
                safety: SafetyEffect::None,
            }
        );
        assert!(fixture.store.has_write_path("/scope-a/leader"));

        assert_eq!(fixture.push_leader_event("node-a"), Ok(()));
        assert_eq!(fixture.step_dcs_and_ha().await, Ok(()));
        let primary = fixture.latest_ha();
        assert_eq!(primary.phase, HaPhase::Primary);
        assert_eq!(
            lower_decision(&primary.decision),
            HaEffectPlan {
                lease: LeaseEffect::None,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::Promote,
                safety: SafetyEffect::None,
            }
        );

        assert_eq!(fixture.push_leader_event("node-z"), Ok(()));
        assert_eq!(fixture.step_dcs_and_ha().await, Ok(()));
        let failsafe = fixture.latest_ha();
        assert_eq!(failsafe.phase, HaPhase::FailSafe);
        assert_eq!(
            lower_decision(&failsafe.decision).safety,
            SafetyEffect::SignalFailSafe
        );
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
            lower_decision(&fencing.decision),
            HaEffectPlan {
                lease: LeaseEffect::ReleaseLeader,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::Demote,
                safety: SafetyEffect::FenceNode,
            }
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
        assert_eq!(
            lower_decision(&waiting.decision),
            HaEffectPlan {
                lease: LeaseEffect::None,
                switchover: SwitchoverEffect::None,
                replication: ReplicationEffect::None,
                postgres: PostgresEffect::Start,
                safety: SafetyEffect::None,
            }
        );

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
        assert_eq!(observed.decision, expected.decision);

        handle.abort();
        let _ = handle.await;
    }
}
