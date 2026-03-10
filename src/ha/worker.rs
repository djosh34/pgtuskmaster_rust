use crate::{
    process::{jobs::ActiveJobKind, state::ProcessState},
    state::{WorkerError, WorkerStatus},
};

use super::{
    apply::{apply_effect_plan, format_dispatch_errors},
    decide::decide,
    events::{
        emit_ha_decision_selected, emit_ha_effect_plan_selected, emit_ha_phase_transition,
        emit_ha_role_transition, ha_role_label,
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
    let process_state = world.process.value.clone();
    let output = decide(DecideInput {
        current: ctx.state.clone(),
        world,
    });
    let plan = output.outcome.decision.lower();
    let skip_redundant_process_dispatch =
        should_skip_redundant_process_dispatch(&ctx.state, &output.next, &process_state);

    emit_ha_decision_selected(ctx, output.next.tick, &output.outcome.decision, &plan)?;
    emit_ha_effect_plan_selected(ctx, output.next.tick, &plan)?;
    let published_next = crate::ha::state::HaState {
        worker: WorkerStatus::Running,
        ..output.next.clone()
    };
    let now = (ctx.now)()?;

    ctx.publisher
        .publish(published_next.clone(), now)
        .map_err(|err| WorkerError::Message(format!("ha publish failed: {err}")))?;

    if prev_phase != published_next.phase {
        emit_ha_phase_transition(ctx, published_next.tick, &prev_phase, &published_next.phase)?;
    }

    let prev_role = ha_role_label(&prev_phase);
    let next_role = ha_role_label(&published_next.phase);
    if prev_role != next_role {
        emit_ha_role_transition(ctx, published_next.tick, prev_role, next_role)?;
    }

    ctx.state = published_next.clone();

    let dispatch_errors = if skip_redundant_process_dispatch {
        Vec::new()
    } else {
        apply_effect_plan(ctx, published_next.tick, &plan)?
    };
    if !dispatch_errors.is_empty() {
        let faulted = crate::ha::state::HaState {
            worker: WorkerStatus::Faulted(WorkerError::Message(format_dispatch_errors(
                &dispatch_errors,
            ))),
            ..published_next
        };
        let faulted_now = (ctx.now)()?;
        ctx.publisher
            .publish(faulted.clone(), faulted_now)
            .map_err(|err| WorkerError::Message(format!("ha publish failed: {err}")))?;
        ctx.state = faulted;
    }

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

fn should_skip_redundant_process_dispatch(
    current: &crate::ha::state::HaState,
    next: &crate::ha::state::HaState,
    process_state: &ProcessState,
) -> bool {
    current.phase == next.phase
        && current.decision == next.decision
        && decision_is_already_active(&next.decision, process_state)
}

fn decision_is_already_active(
    decision: &crate::ha::decision::HaDecision,
    process_state: &ProcessState,
) -> bool {
    match decision {
        crate::ha::decision::HaDecision::WaitForPostgres {
            start_requested: true,
            ..
        } => process_state_is_running_one_of(process_state, &[ActiveJobKind::StartPostgres]),
        crate::ha::decision::HaDecision::RecoverReplica { strategy } => match strategy {
            crate::ha::decision::RecoveryStrategy::Rewind { .. } => {
                process_state_is_running_one_of(process_state, &[ActiveJobKind::PgRewind])
            }
            crate::ha::decision::RecoveryStrategy::BaseBackup { .. } => {
                process_state_is_running_one_of(process_state, &[ActiveJobKind::BaseBackup])
            }
            crate::ha::decision::RecoveryStrategy::Bootstrap => {
                process_state_is_running_one_of(process_state, &[ActiveJobKind::Bootstrap])
            }
        },
        crate::ha::decision::HaDecision::FenceNode => {
            process_state_is_running_one_of(process_state, &[ActiveJobKind::Fencing])
        }
        _ => false,
    }
}

fn process_state_is_running_one_of(
    process_state: &ProcessState,
    expected_kinds: &[ActiveJobKind],
) -> bool {
    match process_state {
        ProcessState::Running { active, .. } => expected_kinds.contains(&active.kind),
        ProcessState::Idle { .. } => false,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, VecDeque},
        path::PathBuf,
        sync::{
            atomic::{AtomicU64, Ordering},
            mpsc::{self, RecvTimeoutError},
            Arc, Mutex,
        },
        time::{Duration, SystemTime, UNIX_EPOCH},
    };
    use tokio::sync::mpsc::error::TryRecvError;

    use crate::{
        config::RuntimeConfig,
        dcs::{
            state::{DcsCache, DcsState, DcsTrust, LeaderRecord, MemberRecord, MemberRole},
            store::{DcsLeaderStore, DcsStore, DcsStoreError, WatchEvent, WatchOp},
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
                DecideInput, HaPhase, HaState, HaWorkerContractStubInputs, HaWorkerCtx,
                ProcessDispatchDefaults,
            },
            worker::{run, step_once},
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::{
            jobs::{
                ActiveJobKind, ProcessCommandRunner, ProcessCommandSpec, ProcessError, ProcessExit,
                ProcessHandle,
            },
            state::{
                JobOutcome, ProcessJobKind, ProcessJobRequest, ProcessState, ProcessWorkerCtx,
            },
        },
        state::{
            new_state_channel, JobId, MemberId, UnixMillis, Version, WorkerError, WorkerStatus,
        },
    };

    const TEST_DCS_AND_PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(5);
    const TEST_HA_VERSION_WAIT_TIMEOUT: Duration = Duration::from_millis(250);

    #[derive(Clone, Default)]
    struct RecordingStore {
        fail_write: bool,
        fail_delete: bool,
        reject_put_if_absent: bool,
        writes: Arc<Mutex<Vec<(String, String)>>>,
        deletes: Arc<Mutex<Vec<String>>>,
        events: Arc<Mutex<VecDeque<WatchEvent>>>,
        delete_block_started: Option<Arc<Mutex<Option<mpsc::Sender<()>>>>>,
        delete_block_release: Option<Arc<Mutex<mpsc::Receiver<()>>>>,
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
            if self.reject_put_if_absent {
                return Ok(false);
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
            if let Some(started) = &self.delete_block_started {
                let mut guard = started
                    .lock()
                    .map_err(|_| DcsStoreError::Io("delete start lock poisoned".to_string()))?;
                if let Some(tx) = guard.take() {
                    tx.send(())
                        .map_err(|_| DcsStoreError::Io("delete start signal failed".to_string()))?;
                }
            }
            if let Some(release) = &self.delete_block_release {
                let guard = release
                    .lock()
                    .map_err(|_| DcsStoreError::Io("delete release lock poisoned".to_string()))?;
                match guard.recv_timeout(Duration::from_secs(5)) {
                    Ok(()) => {}
                    Err(RecvTimeoutError::Timeout) => {
                        return Err(DcsStoreError::Io(
                            "delete release unblock timed out".to_string(),
                        ));
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        return Err(DcsStoreError::Io(
                            "delete release unblock disconnected".to_string(),
                        ));
                    }
                }
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

    impl DcsLeaderStore for RecordingStore {
        fn acquire_leader_lease(
            &mut self,
            scope: &str,
            member_id: &MemberId,
        ) -> Result<(), DcsStoreError> {
            let path = format!("/{}/leader", scope.trim_matches('/'));
            let encoded = serde_json::to_string(&LeaderRecord {
                member_id: member_id.clone(),
            })
            .map_err(|err| DcsStoreError::Decode {
                key: path.clone(),
                message: err.to_string(),
            })?;

            if self.put_path_if_absent(path.as_str(), encoded)? {
                Ok(())
            } else {
                Err(DcsStoreError::AlreadyExists(path))
            }
        }

        fn release_leader_lease(
            &mut self,
            scope: &str,
            _member_id: &MemberId,
        ) -> Result<(), DcsStoreError> {
            self.delete_path(format!("/{}/leader", scope.trim_matches('/')).as_str())
        }

        fn clear_switchover(&mut self, scope: &str) -> Result<(), DcsStoreError> {
            self.delete_path(format!("/{}/switchover", scope.trim_matches('/')).as_str())
        }
    }

    static TEST_DATA_DIR_SEQ: AtomicU64 = AtomicU64::new(0);

    fn test_now_unix_millis() -> UnixMillis {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| {
                u64::try_from(duration.as_millis()).map_or(u64::MAX, |value| value)
            });
        UnixMillis(millis)
    }

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
        crate::test_harness::runtime_config::RuntimeConfigBuilder::new()
            .with_postgres_data_dir(unique_test_data_dir("pgdata"))
            .build()
    }

    fn sample_pg_common(sql: SqlStatus) -> PgInfoCommon {
        PgInfoCommon {
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
        }
    }

    fn sample_pg_state(sql: SqlStatus) -> PgInfoState {
        PgInfoState::Unknown {
            common: sample_pg_common(sql),
        }
    }

    fn sample_primary_pg_state(sql: SqlStatus) -> PgInfoState {
        PgInfoState::Primary {
            common: sample_pg_common(sql),
            wal_lsn: crate::state::WalLsn(1),
            slots: Vec::new(),
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
        ProcessDispatchDefaults::contract_stub()
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

    #[derive(Clone)]
    struct HaWorkerTestBuilder {
        store: RecordingStore,
        poll_interval: Duration,
        dcs_trust: DcsTrust,
        initial_phase: HaPhase,
        initial_tick: u64,
        initial_decision: HaDecision,
        pg_state: PgInfoState,
        process_state: ProcessState,
    }

    impl HaWorkerTestBuilder {
        fn new() -> Self {
            Self {
                store: RecordingStore::default(),
                poll_interval: Duration::from_millis(100),
                dcs_trust: DcsTrust::FullQuorum,
                initial_phase: HaPhase::Init,
                initial_tick: 0,
                initial_decision: HaDecision::NoChange,
                pg_state: sample_pg_state(SqlStatus::Healthy),
                process_state: sample_process_state(),
            }
        }

        fn with_store(self, store: RecordingStore) -> Self {
            Self { store, ..self }
        }

        fn with_poll_interval(self, poll_interval: Duration) -> Self {
            Self {
                poll_interval,
                ..self
            }
        }

        fn with_dcs_trust(self, dcs_trust: DcsTrust) -> Self {
            Self { dcs_trust, ..self }
        }

        fn with_phase(self, initial_phase: HaPhase) -> Self {
            Self {
                initial_phase,
                ..self
            }
        }

        fn with_decision(self, initial_decision: HaDecision) -> Self {
            Self {
                initial_decision,
                ..self
            }
        }

        fn with_tick(self, initial_tick: u64) -> Self {
            Self {
                initial_tick,
                ..self
            }
        }

        fn with_pg_state(self, pg_state: PgInfoState) -> Self {
            Self { pg_state, ..self }
        }

        fn with_process_state(self, process_state: ProcessState) -> Self {
            Self {
                process_state,
                ..self
            }
        }

        fn build(self) -> Result<BuiltContext, WorkerError> {
            let BuiltContext {
                mut ctx,
                ha_subscriber,
                _config_publisher,
                pg_publisher,
                _dcs_publisher,
                _process_publisher,
                process_rx,
                store,
            } = build_context(self.store, self.poll_interval, self.dcs_trust);

            pg_publisher
                .publish(self.pg_state, UnixMillis(50))
                .map_err(|err| WorkerError::Message(format!("pg publish failed: {err}")))?;
            _process_publisher
                .publish(self.process_state, UnixMillis(50))
                .map_err(|err| WorkerError::Message(format!("process publish failed: {err}")))?;
            ctx.state = HaState {
                worker: WorkerStatus::Running,
                phase: self.initial_phase,
                tick: self.initial_tick,
                decision: self.initial_decision,
            };

            Ok(BuiltContext {
                ctx,
                ha_subscriber,
                _config_publisher,
                pg_publisher,
                _dcs_publisher,
                _process_publisher,
                process_rx,
                store,
            })
        }
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
                poll_interval: TEST_DCS_AND_PROCESS_POLL_INTERVAL,
                local_postgres_host: runtime_config.postgres.listen_host.clone(),
                local_postgres_port: runtime_config.postgres.listen_port,
                local_api_url: Some("http://127.0.0.1:8080".to_string()),
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
            process_ctx.poll_interval = TEST_DCS_AND_PROCESS_POLL_INTERVAL;
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
            ha_ctx.poll_interval = TEST_DCS_AND_PROCESS_POLL_INTERVAL;
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
            self.publish_pg_sql_state(sample_pg_state(status), now)
        }

        fn publish_pg_sql_state(&self, state: PgInfoState, now: u64) -> Result<(), WorkerError> {
            self.pg_publisher
                .publish(state, UnixMillis(now))
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
            postgres_host: "10.0.0.10".to_string(),
            postgres_port: 5432,
            api_url: None,
            role,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
            timeline: None,
            write_lsn: None,
            replay_lsn: None,
            updated_at: test_now_unix_millis(),
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
            tokio::time::sleep(TEST_DCS_AND_PROCESS_POLL_INTERVAL).await;
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_uses_subscribers_and_publishes_next_state() -> Result<(), WorkerError> {
        let BuiltContext {
            mut ctx,
            ha_subscriber: subscriber,
            ..
        } = HaWorkerTestBuilder::new().build()?;

        let stepped = step_once(&mut ctx).await;
        assert_eq!(stepped, Ok(()));
        assert_eq!(ctx.state.phase, HaPhase::WaitingPostgresReachable);
        assert_eq!(ctx.state.tick, 1);
        assert_eq!(ctx.state.worker, WorkerStatus::Running);

        let published = subscriber.latest();
        assert_eq!(published.version, Version(1));
        assert_eq!(published.value.phase, HaPhase::WaitingPostgresReachable);
        assert_eq!(published.value.tick, 1);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_suppresses_duplicate_start_postgres_dispatch_while_unreachable(
    ) -> Result<(), WorkerError> {
        let BuiltContext {
            mut ctx,
            ha_subscriber: _ha_subscriber,
            mut process_rx,
            ..
        } = HaWorkerTestBuilder::new()
            .with_phase(HaPhase::WaitingPostgresReachable)
            .with_pg_state(sample_pg_state(SqlStatus::Unreachable))
            .build()?;

        step_once(&mut ctx).await?;
        let first = process_rx.try_recv().map_err(|err| {
            WorkerError::Message(format!(
                "expected process job request after first tick: {err}"
            ))
        })?;
        assert!(matches!(first.kind, ProcessJobKind::StartPostgres(_)));

        step_once(&mut ctx).await?;
        assert_eq!(process_rx.try_recv(), Err(TryRecvError::Empty));
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_retries_fence_dispatch_after_demote_finishes() -> Result<(), WorkerError> {
        let BuiltContext {
            mut ctx,
            ha_subscriber: _ha_subscriber,
            mut process_rx,
            ..
        } = HaWorkerTestBuilder::new()
            .with_phase(HaPhase::Fencing)
            .with_decision(HaDecision::FenceNode)
            .with_process_state(ProcessState::Idle {
                worker: WorkerStatus::Running,
                last_outcome: Some(JobOutcome::Success {
                    id: JobId("demote-complete".to_string()),
                    job_kind: ActiveJobKind::Demote,
                    finished_at: UnixMillis(10),
                }),
            })
            .build()?;

        step_once(&mut ctx).await?;
        let request = process_rx.try_recv().map_err(|err| {
            WorkerError::Message(format!(
                "expected fence process job request after demote completion: {err}"
            ))
        })?;
        assert!(matches!(request.kind, ProcessJobKind::Fencing(_)));
        assert_eq!(process_rx.try_recv(), Err(TryRecvError::Empty));
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_matches_decide_output_for_same_snapshot() -> Result<(), WorkerError> {
        let BuiltContext {
            mut ctx,
            ha_subscriber,
            ..
        } = HaWorkerTestBuilder::new()
            .with_phase(HaPhase::WaitingDcsTrusted)
            .with_tick(7)
            .build()?;

        let expected = crate::ha::decide::decide(DecideInput {
            current: ctx.state.clone(),
            world: super::world_snapshot(&ctx),
        });

        step_once(&mut ctx).await?;

        assert_eq!(ctx.state.phase, expected.next.phase);
        assert_eq!(ctx.state.tick, expected.next.tick);
        assert_eq!(ctx.state.decision, expected.next.decision);
        assert_eq!(ha_subscriber.latest().value, ctx.state);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_primary_quorum_loss_enqueues_fencing_without_releasing_lease(
    ) -> Result<(), WorkerError> {
        let BuiltContext {
            mut ctx,
            ha_subscriber: _ha_subscriber,
            mut process_rx,
            store,
            ..
        } = HaWorkerTestBuilder::new()
            .with_phase(HaPhase::Primary)
            .with_dcs_trust(DcsTrust::NotTrusted)
            .with_pg_state(sample_primary_pg_state(SqlStatus::Healthy))
            .build()?;

        step_once(&mut ctx).await?;

        assert_eq!(ctx.state.phase, HaPhase::FailSafe);
        assert_eq!(
            ctx.state.decision,
            HaDecision::EnterFailSafe {
                release_leader_lease: false,
            }
        );
        assert!(!store.has_delete_path("/scope-a/leader"));
        let request = process_rx.try_recv().map_err(|err| {
            WorkerError::Message(format!("expected fencing dispatch during fail-safe: {err}"))
        })?;
        assert!(matches!(request.kind, ProcessJobKind::Fencing(_)));
        assert_eq!(process_rx.try_recv(), Err(TryRecvError::Empty));
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_failsafe_primary_with_restored_quorum_attempts_leadership(
    ) -> Result<(), WorkerError> {
        let BuiltContext {
            mut ctx,
            ha_subscriber,
            mut process_rx,
            store: store_handle,
            ..
        } = HaWorkerTestBuilder::new()
            .with_phase(HaPhase::FailSafe)
            .with_dcs_trust(DcsTrust::FullQuorum)
            .with_pg_state(sample_primary_pg_state(SqlStatus::Healthy))
            .build()?;

        step_once(&mut ctx).await?;

        let published = ha_subscriber.latest();
        assert_eq!(published.version, Version(1));
        assert_eq!(published.value.phase, HaPhase::Primary);
        assert_eq!(published.value.decision, HaDecision::AttemptLeadership);
        assert_eq!(published.value.worker, WorkerStatus::Running);
        assert_eq!(ctx.state.phase, HaPhase::Primary);
        assert_eq!(ctx.state.decision, HaDecision::AttemptLeadership);
        assert_eq!(
            store_handle.first_write_path().as_deref(),
            Some("/scope-a/leader")
        );
        assert!(!store_handle.has_delete_path("/scope-a/leader"));
        assert_eq!(process_rx.try_recv(), Err(TryRecvError::Empty));
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_primary_outage_enqueues_only_recovery_dispatch() -> Result<(), WorkerError> {
        let BuiltContext {
            mut ctx,
            ha_subscriber: _ha_subscriber,
            _dcs_publisher: dcs_publisher,
            mut process_rx,
            store,
            ..
        } = HaWorkerTestBuilder::new()
            .with_phase(HaPhase::Primary)
            .with_pg_state(sample_pg_state(SqlStatus::Unreachable))
            .build()?;
        let mut dcs_state = sample_dcs_state(sample_runtime_config(), DcsTrust::FullQuorum);
        let leader_member = sample_member_record("node-b", MemberRole::Primary);
        dcs_state
            .cache
            .members
            .insert(leader_member.member_id.clone(), leader_member.clone());
        dcs_state.cache.leader = Some(LeaderRecord {
            member_id: leader_member.member_id,
        });
        dcs_publisher
            .publish(dcs_state, UnixMillis(60))
            .map_err(|err| WorkerError::Message(format!("dcs publish failed: {err}")))?;

        step_once(&mut ctx).await?;

        let first = process_rx.try_recv().map_err(|err| {
            WorkerError::Message(format!("expected a recovery process request: {err}"))
        })?;
        assert!(matches!(first.kind, ProcessJobKind::PgRewind(_)));
        assert_eq!(process_rx.try_recv(), Err(TryRecvError::Empty));
        assert_eq!(store.writes_len(), 0);
        assert_eq!(store.deletes_len(), 0);
        assert_eq!(ctx.state.phase, HaPhase::Rewinding);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn apply_effect_plan_maps_dcs_and_process_requests() -> Result<(), WorkerError> {
        let BuiltContext {
            mut ctx,
            mut process_rx,
            store,
            ..
        } = HaWorkerTestBuilder::new().build()?;
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
                assert_eq!(
                    spec.config_file,
                    ctx.config_subscriber
                        .latest()
                        .value
                        .postgres
                        .data_dir
                        .join("pgtm.postgresql.conf")
                );
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
        let BuiltContext {
            mut ctx,
            process_rx,
            store: store_handle,
            ..
        } = HaWorkerTestBuilder::new().with_store(store).build()?;
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
        let BuiltContext { mut ctx, store, .. } = HaWorkerTestBuilder::new().build()?;

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
    async fn apply_effect_plan_surfaces_leader_lease_conflict() -> Result<(), WorkerError> {
        let store = RecordingStore {
            reject_put_if_absent: true,
            ..RecordingStore::default()
        };
        let BuiltContext { mut ctx, .. } = HaWorkerTestBuilder::new()
            .with_store(store.clone())
            .build()?;

        let ha_tick = ctx.state.tick;
        let plan = HaEffectPlan {
            lease: LeaseEffect::AcquireLeader,
            switchover: SwitchoverEffect::None,
            replication: ReplicationEffect::None,
            postgres: PostgresEffect::None,
            safety: SafetyEffect::None,
        };

        let errors = apply_effect_plan(&mut ctx, ha_tick, &plan)?;

        assert_eq!(store.writes_len(), 0);
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0],
            ActionDispatchError::DcsWrite {
                action: ActionId::AcquireLeaderLease,
                path: "/scope-a/leader".to_string(),
                message: "path already exists: /scope-a/leader".to_string(),
            }
        );
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

        assert_eq!(
            fixture.publish_pg_sql_state(sample_primary_pg_state(SqlStatus::Healthy), 20),
            Ok(())
        );
        assert_eq!(fixture.push_leader_event("node-z"), Ok(()));
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
    }

    #[tokio::test(flavor = "current_thread")]
    async fn integration_primary_unreachable_rewinds_then_returns_replica_on_success() {
        let mut fixture = IntegrationFixture::new(HaPhase::Primary);

        assert_eq!(
            fixture.push_member_event("node-b", MemberRole::Primary),
            Ok(())
        );
        assert_eq!(fixture.push_leader_event("node-b"), Ok(()));
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
        assert_eq!(fixture.queue_process_success(), Ok(()));
        let process_version_before = fixture.process_subscriber.latest().version;
        assert_eq!(
            crate::process::worker::step_once(&mut fixture.process_ctx).await,
            Ok(())
        );
        assert_eq!(step_once(&mut fixture.ha_ctx).await, Ok(()));
        assert_eq!(fixture.latest_ha().phase, HaPhase::Fencing);

        let process_version_mid = fixture.process_subscriber.latest().version;
        assert_eq!(
            process_version_mid.0,
            process_version_before.0.saturating_add(2)
        );

        assert_eq!(
            crate::process::worker::step_once(&mut fixture.process_ctx).await,
            Ok(())
        );
        assert_eq!(step_once(&mut fixture.ha_ctx).await, Ok(()));

        let process_version_after = fixture.process_subscriber.latest().version;
        assert!(process_version_after.0 > process_version_mid.0);
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
    async fn run_reacts_to_interval_tick_and_watcher_change() -> Result<(), WorkerError> {
        let BuiltContext {
            ctx,
            ha_subscriber: subscriber,
            _config_publisher,
            pg_publisher,
            _dcs_publisher,
            _process_publisher,
            ..
        } = HaWorkerTestBuilder::new()
            .with_poll_interval(Duration::from_millis(20))
            .build()?;

        let handle = tokio::spawn(async move { run(ctx).await });

        let first_advanced =
            wait_for_ha_version(&subscriber, 1, TEST_HA_VERSION_WAIT_TIMEOUT).await;
        assert!(first_advanced);

        let publish_result =
            pg_publisher.publish(sample_pg_state(SqlStatus::Unreachable), UnixMillis(50));
        assert!(publish_result.is_ok());
        let second_advanced =
            wait_for_ha_version(&subscriber, 2, TEST_HA_VERSION_WAIT_TIMEOUT).await;
        assert!(second_advanced);

        handle.abort();
        let _ = handle.await;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn run_initial_buffered_updates_match_explicit_buffered_prefix() -> Result<(), WorkerError>
    {
        let BuiltContext {
            ctx: mut step_ctx,
            ha_subscriber: _ha_subscriber,
            ..
        } = HaWorkerTestBuilder::new()
            .with_poll_interval(Duration::from_secs(1))
            .build()?;

        let stepped = step_once(&mut step_ctx).await;
        assert_eq!(stepped, Ok(()));
        let expected_after_first = step_ctx.state.clone();
        let stepped = step_once(&mut step_ctx).await;
        assert_eq!(stepped, Ok(()));
        let expected_after_second = step_ctx.state.clone();

        let BuiltContext {
            ctx: run_ctx,
            ha_subscriber: run_subscriber,
            _config_publisher,
            _dcs_publisher,
            _process_publisher,
            ..
        } = HaWorkerTestBuilder::new()
            .with_poll_interval(Duration::from_secs(1))
            .build()?;
        let handle = tokio::spawn(async move { run(run_ctx).await });

        let advanced = wait_for_ha_version(&run_subscriber, 1, TEST_HA_VERSION_WAIT_TIMEOUT).await;
        assert!(advanced);
        let observed = run_subscriber.latest().value;
        assert!(
            observed == expected_after_first || observed == expected_after_second,
            "observed buffered run prefix did not match either explicit prefix: observed={observed:?} first={expected_after_first:?} second={expected_after_second:?}"
        );

        handle.abort();
        let _ = handle.await;
        Ok(())
    }
}
