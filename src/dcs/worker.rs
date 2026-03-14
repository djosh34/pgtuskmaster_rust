use crate::{
    config::{DcsClientConfig, DcsEndpoint},
    logging::{DcsCommandName, DcsEvent, DcsEventIdentity, LogEvent, SeverityText},
    state::WorkerError,
};

use super::{
    command::{dcs_command_channel, DcsCommand, DcsCommandError, DcsHandle},
    etcd_store::EtcdDcsStore,
    keys::DcsKey,
    state::{
        build_dcs_view, build_local_member_record, evaluate_trust, DcsCache, DcsCadence,
        DcsControlPlane, DcsLocalMemberAdvertisement, DcsNodeIdentity, DcsObservedState,
        DcsRuntime, DcsStateChannel, DcsTrust, DcsWorkerCtx, InitLockRecord, LeaderLeaseRecord,
        MemberRecord, SwitchoverRecord, SwitchoverTargetRecord,
    },
    store::{refresh_from_etcd_watch, write_local_member_record, DcsStoreError},
};

fn advertised_api_url(advertisement: &super::state::DcsLocalMemberAdvertisement) -> Option<&str> {
    match &advertisement.api {
        super::state::DcsApiAdvertisement::NotAdvertised => None,
        super::state::DcsApiAdvertisement::Advertised(api) => Some(api.url.as_str()),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsValue {
    Member(MemberRecord),
    Leader(LeaderLeaseRecord),
    Switchover(SwitchoverRecord),
    Config(Box<crate::config::RuntimeConfig>),
    InitLock(InitLockRecord),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsWatchUpdate {
    Put { key: DcsKey, value: Box<DcsValue> },
    Delete { key: DcsKey },
}

fn dcs_event_identity(ctx: &DcsWorkerCtx) -> DcsEventIdentity {
    DcsEventIdentity {
        scope: ctx.identity.scope.clone(),
        member_id: ctx.identity.self_id.0.clone(),
    }
}

fn emit_dcs_event(
    ctx: &DcsWorkerCtx,
    origin: &str,
    event: DcsEvent,
    severity: SeverityText,
    error_prefix: &str,
) -> Result<(), WorkerError> {
    ctx.runtime
        .log
        .emit(origin, LogEvent::Dcs(crate::logging::InternalEvent::new(severity, event)))
        .map_err(|err| WorkerError::Message(format!("{error_prefix}: {err}")))
}

fn dcs_io_error_severity(err: &crate::dcs::store::DcsStoreError) -> SeverityText {
    match err {
        crate::dcs::store::DcsStoreError::Io(_) => SeverityText::Warn,
        _ => SeverityText::Error,
    }
}

fn dcs_refresh_error_severity(err: &crate::dcs::store::DcsStoreError) -> SeverityText {
    match err {
        crate::dcs::store::DcsStoreError::Io(_)
        | crate::dcs::store::DcsStoreError::InvalidKey(_)
        | crate::dcs::store::DcsStoreError::MissingValue(_) => SeverityText::Warn,
        _ => SeverityText::Error,
    }
}

pub(crate) struct DcsStoreBootstrap {
    pub(crate) endpoints: Vec<DcsEndpoint>,
    pub(crate) client: DcsClientConfig,
}

pub(crate) struct DcsWorkerBootstrap {
    pub(crate) identity: DcsNodeIdentity,
    pub(crate) store: DcsStoreBootstrap,
    pub(crate) cadence: DcsCadence,
    pub(crate) advertisement: DcsLocalMemberAdvertisement,
    pub(crate) observed: DcsObservedState,
    pub(crate) state_channel: DcsStateChannel,
    pub(crate) runtime: DcsRuntime,
}

pub(crate) fn build_worker_ctx(
    bootstrap: DcsWorkerBootstrap,
) -> Result<(DcsWorkerCtx, DcsHandle), DcsStoreError> {
    let DcsWorkerBootstrap {
        identity,
        store,
        cadence,
        advertisement,
        observed,
        state_channel,
        runtime,
    } = bootstrap;
    let DcsStoreBootstrap { endpoints, client } = store;
    let store = EtcdDcsStore::connect_with_leader_lease(
        endpoints,
        client,
        identity.scope.as_str(),
        cadence.member_ttl_ms,
    )?;
    let (handle, command_inbox) = dcs_command_channel();
    let ctx = DcsWorkerCtx {
        identity,
        cadence,
        advertisement,
        observed,
        state_channel,
        control: DcsControlPlane {
            command_inbox,
            store: Box::new(store),
        },
        runtime,
    };
    Ok((ctx, handle))
}

pub(crate) async fn run(mut ctx: DcsWorkerCtx) -> Result<(), WorkerError> {
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.cadence.poll_interval).await;
    }
}

pub(crate) fn apply_watch_update(cache: &mut DcsCache, update: DcsWatchUpdate) {
    match update {
        DcsWatchUpdate::Put { key, value } => match (key, *value) {
            (DcsKey::Member(member_id), DcsValue::Member(record)) => {
                cache.member_records.insert(member_id, record);
            }
            (DcsKey::Leader, DcsValue::Leader(record)) => {
                cache.leader_record = Some(record);
            }
            (DcsKey::Switchover, DcsValue::Switchover(record)) => {
                cache.switchover_record = Some(record);
            }
            (DcsKey::Config, DcsValue::Config(_config)) => {}
            (DcsKey::InitLock, DcsValue::InitLock(record)) => {
                cache.init_lock = Some(record);
            }
            _ => {}
        },
        DcsWatchUpdate::Delete { key } => match key {
            DcsKey::Member(member_id) => {
                cache.member_records.remove(&member_id);
            }
            DcsKey::Leader => {
                cache.leader_record = None;
            }
            DcsKey::Switchover => {
                cache.switchover_record = None;
            }
            DcsKey::Config => {}
            DcsKey::InitLock => {
                cache.init_lock = None;
            }
        },
    }
}

pub(crate) async fn step_once(ctx: &mut DcsWorkerCtx) -> Result<(), WorkerError> {
    drain_command_inbox(ctx)?;

    let now = now_unix_millis()?;
    let pg_snapshot = ctx.observed.pg.latest();
    let member_ttl_ms = ctx.cadence.member_ttl_ms;
    let local_member_path = format!(
        "/{}/member/{}",
        ctx.identity.scope.trim_matches('/'),
        ctx.identity.self_id.0
    );
    let pg_snapshot_stale = pg_snapshot
        .last_refresh_at()
        .is_none_or(|last_refresh_at| now.0.saturating_sub(last_refresh_at.0) > member_ttl_ms);

    let mut store_healthy = ctx.control.store.healthy();
    if store_healthy && pg_snapshot_stale {
        match ctx.control.store.delete_path(local_member_path.as_str()) {
            Ok(()) => {
                ctx.state_channel
                    .cache
                    .member_records
                    .remove(&ctx.identity.self_id);
                release_local_leader_lease(ctx, &mut store_healthy)?;
            }
            Err(err) => {
                let severity = dcs_io_error_severity(&err);
                emit_dcs_event(
                    ctx,
                    "dcs_worker::step_once",
                    DcsEvent::LocalMemberDeleteFailed {
                        identity: dcs_event_identity(ctx),
                        error: err.to_string(),
                    },
                    severity,
                    "dcs local member delete log emit failed",
                )?;
                store_healthy = false;
            }
        }
    } else if store_healthy {
        let local_member = build_local_member_record(
            &ctx.identity.self_id,
            ctx.advertisement.postgres.host.as_str(),
            ctx.advertisement.postgres.port,
            advertised_api_url(&ctx.advertisement),
            member_ttl_ms,
            &pg_snapshot,
        );
        match write_local_member_record(
            ctx.control.store.as_mut(),
            &ctx.identity.scope,
            &local_member,
            member_ttl_ms,
        ) {
            Ok(()) => {
                ctx.state_channel
                    .cache
                    .member_records
                    .insert(ctx.identity.self_id.clone(), local_member);
            }
            Err(err) => {
                let severity = dcs_io_error_severity(&err);
                emit_dcs_event(
                    ctx,
                    "dcs_worker::step_once",
                    DcsEvent::LocalMemberWriteFailed {
                        identity: dcs_event_identity(ctx),
                        error: err.to_string(),
                    },
                    severity,
                    "dcs local member write log emit failed",
                )?;
                store_healthy = false;
            }
        }
    }

    let events = match ctx.control.store.drain_watch_events() {
        Ok(events) => events,
        Err(err) => {
            let severity = dcs_io_error_severity(&err);
            emit_dcs_event(
                ctx,
                "dcs_worker::step_once",
                DcsEvent::WatchDrainFailed {
                    identity: dcs_event_identity(ctx),
                    error: err.to_string(),
                },
                severity,
                "dcs drain log emit failed",
            )?;
            store_healthy = false;
            Vec::new()
        }
    };
    match refresh_from_etcd_watch(&ctx.identity.scope, &mut ctx.state_channel.cache, events) {
        Ok(result) => {
            if result.had_errors {
                emit_dcs_event(
                    ctx,
                    "dcs_worker::step_once",
                    DcsEvent::WatchApplyHadErrors {
                        identity: dcs_event_identity(ctx),
                        applied: result.applied,
                    },
                    SeverityText::Warn,
                    "dcs refresh had_errors log emit failed",
                )?;
                store_healthy = false;
            }
        }
        Err(err) => {
            let severity = dcs_refresh_error_severity(&err);
            emit_dcs_event(
                ctx,
                "dcs_worker::step_once",
                DcsEvent::WatchRefreshFailed {
                    identity: dcs_event_identity(ctx),
                    error: err.to_string(),
                },
                severity,
                "dcs refresh log emit failed",
            )?;
            store_healthy = false;
        }
    }

    if store_healthy {
        let scope_prefix = format!("/{}/", ctx.identity.scope.trim_matches('/'));
        match ctx.control.store.snapshot_prefix(scope_prefix.as_str()) {
            Ok(snapshot_events) => {
                match refresh_from_etcd_watch(
                    &ctx.identity.scope,
                    &mut ctx.state_channel.cache,
                    snapshot_events,
                ) {
                    Ok(result) => {
                        if result.had_errors {
                            emit_dcs_event(
                                ctx,
                                "dcs_worker::step_once",
                                DcsEvent::SnapshotApplyHadErrors {
                                    identity: dcs_event_identity(ctx),
                                    applied: result.applied,
                                },
                                SeverityText::Warn,
                                "dcs snapshot had_errors log emit failed",
                            )?;
                            store_healthy = false;
                        }
                    }
                    Err(err) => {
                        let severity = dcs_refresh_error_severity(&err);
                        emit_dcs_event(
                            ctx,
                            "dcs_worker::step_once",
                            DcsEvent::SnapshotRefreshFailed {
                                identity: dcs_event_identity(ctx),
                                error: err.to_string(),
                            },
                            severity,
                            "dcs snapshot refresh log emit failed",
                        )?;
                        store_healthy = false;
                    }
                }
            }
            Err(err) => {
                let severity = dcs_io_error_severity(&err);
                emit_dcs_event(
                    ctx,
                    "dcs_worker::step_once",
                    DcsEvent::SnapshotReadFailed {
                        identity: dcs_event_identity(ctx),
                        error: err.to_string(),
                    },
                    severity,
                    "dcs snapshot read log emit failed",
                )?;
                store_healthy = false;
            }
        }
    }

    let trust = evaluate_trust(
        store_healthy,
        &ctx.state_channel.cache,
        &ctx.identity.self_id,
    );
    let worker = if store_healthy {
        crate::state::WorkerStatus::Running
    } else {
        crate::state::WorkerStatus::Faulted(WorkerError::Message("dcs store unhealthy".to_string()))
    };

    let next = build_dcs_view(
        worker,
        if store_healthy {
            trust
        } else {
            DcsTrust::NotTrusted
        },
        &ctx.state_channel.cache,
        Some(now),
    );
    if ctx.runtime.last_emitted_store_healthy != Some(store_healthy) {
        ctx.runtime.last_emitted_store_healthy = Some(store_healthy);
        let severity = if store_healthy {
            SeverityText::Info
        } else {
            SeverityText::Warn
        };
        emit_dcs_event(
            ctx,
            "dcs_worker::step_once",
            DcsEvent::StoreHealthTransition {
                identity: dcs_event_identity(ctx),
                store_healthy,
            },
            severity,
            "dcs health transition log emit failed",
        )?;
    }
    if ctx.runtime.last_emitted_trust.as_ref() != Some(&next.trust) {
        let previous = ctx.runtime.last_emitted_trust.clone();
        ctx.runtime.last_emitted_trust = Some(next.trust.clone());
        emit_dcs_event(
            ctx,
            "dcs_worker::step_once",
            DcsEvent::TrustTransition {
                identity: dcs_event_identity(ctx),
                previous,
                next: next.trust.clone(),
            },
            SeverityText::Info,
            "dcs trust transition log emit failed",
        )?;
    }
    ctx.state_channel
        .publisher
        .publish(next)
        .map_err(|err| WorkerError::Message(format!("dcs publish failed: {err}")))?;
    Ok(())
}

fn release_local_leader_lease(
    ctx: &mut DcsWorkerCtx,
    store_healthy: &mut bool,
) -> Result<(), WorkerError> {
    match ctx
        .control
        .store
        .release_leader_lease(&ctx.identity.scope, &ctx.identity.self_id)
    {
        Ok(()) => Ok(()),
        Err(err) => {
            let severity = dcs_io_error_severity(&err);
            emit_dcs_event(
                ctx,
                "dcs_worker::step_once",
                DcsEvent::LocalLeaderReleaseFailed {
                    identity: dcs_event_identity(ctx),
                    error: err.to_string(),
                },
                severity,
                "dcs local leader release log emit failed",
            )?;
            *store_healthy = false;
            Ok(())
        }
    }
}

fn drain_command_inbox(ctx: &mut DcsWorkerCtx) -> Result<(), WorkerError> {
    loop {
        match ctx.control.command_inbox.try_recv() {
            Ok(request) => handle_command_request(ctx, request)?,
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => return Ok(()),
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                return Err(WorkerError::Message(
                    "dcs command channel disconnected".to_string(),
                ));
            }
        }
    }
}

fn handle_command_request(
    ctx: &mut DcsWorkerCtx,
    request: super::command::DcsCommandRequest,
) -> Result<(), WorkerError> {
    let command_name = dcs_command_name(&request.command);
    let result = execute_command(ctx, request.command);
    if request.response_tx.send(result).is_err() {
        emit_dcs_event(
            ctx,
            "dcs_worker::handle_command_request",
            DcsEvent::CommandResponseDropped {
                identity: dcs_event_identity(ctx),
                command: command_name,
            },
            SeverityText::Warn,
            "dcs command response_dropped log emit failed",
        )?;
    }
    Ok(())
}

fn execute_command(ctx: &mut DcsWorkerCtx, command: DcsCommand) -> Result<(), DcsCommandError> {
    match command {
        DcsCommand::AcquireLeadership => ctx
            .control
            .store
            .acquire_leader_lease(&ctx.identity.scope, &ctx.identity.self_id)
            .or_else(handle_acquire_leadership_error)
            .map_err(dcs_command_error),
        DcsCommand::ReleaseLeadership => ctx
            .control
            .store
            .release_leader_lease(&ctx.identity.scope, &ctx.identity.self_id)
            .map_err(dcs_command_error),
        DcsCommand::PublishSwitchover { target } => {
            let path = format!("/{}/switchover", ctx.identity.scope.trim_matches('/'));
            let record = SwitchoverRecord {
                target: match target {
                    crate::dcs::DcsSwitchoverTargetView::AnyHealthyReplica => {
                        SwitchoverTargetRecord::AnyHealthyReplica
                    }
                    crate::dcs::DcsSwitchoverTargetView::Specific(member_id) => {
                        SwitchoverTargetRecord::Specific(member_id)
                    }
                },
            };
            let encoded = serde_json::to_string(&record).map_err(|err| {
                DcsCommandError::Transport(format!("switchover encode failed: {err}"))
            })?;
            ctx.control
                .store
                .write_path(path.as_str(), encoded)
                .map_err(dcs_command_error)
        }
        DcsCommand::ClearSwitchover => ctx
            .control
            .store
            .clear_switchover(&ctx.identity.scope)
            .map_err(dcs_command_error),
    }
}

fn handle_acquire_leadership_error(err: DcsStoreError) -> Result<(), DcsStoreError> {
    match err {
        DcsStoreError::AlreadyExists(_) => Ok(()),
        other => Err(other),
    }
}

fn dcs_command_error(err: DcsStoreError) -> DcsCommandError {
    DcsCommandError::Rejected(err.to_string())
}

fn dcs_command_name(command: &DcsCommand) -> DcsCommandName {
    match command {
        DcsCommand::AcquireLeadership => DcsCommandName::AcquireLeadership,
        DcsCommand::ReleaseLeadership => DcsCommandName::ReleaseLeadership,
        DcsCommand::PublishSwitchover { .. } => DcsCommandName::PublishSwitchover,
        DcsCommand::ClearSwitchover => DcsCommandName::ClearSwitchover,
    }
}

fn now_unix_millis() -> Result<crate::state::UnixMillis, WorkerError> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| WorkerError::Message(format!("system clock before unix epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| WorkerError::Message(format!("unix millis conversion failed: {err}")))?;
    Ok(crate::state::UnixMillis(millis))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, VecDeque},
        sync::{Arc, Mutex},
    };

    use super::*;
    use crate::{
        dcs::{
            state::{
                build_local_member_record, DcsLeaderStateView, DcsMemberLeaseView,
                LeaderLeaseRecord, MemberRecord,
            },
            store::{encode_leader_record, leader_path, DcsStore, WatchEvent, WatchOp},
        },
        logging::LogHandle,
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        state::{new_state_channel, MemberId, TimelineId, UnixMillis, WorkerStatus},
    };

    #[derive(Default)]
    struct FakeStoreState {
        leader_release_calls: usize,
        paths: BTreeMap<String, String>,
    }

    struct FakeDcsStore {
        state: Arc<Mutex<FakeStoreState>>,
    }

    impl FakeDcsStore {
        fn new(state: Arc<Mutex<FakeStoreState>>) -> Self {
            Self { state }
        }
    }

    impl DcsStore for FakeDcsStore {
        fn healthy(&self) -> bool {
            true
        }

        fn snapshot_prefix(&mut self, path_prefix: &str) -> Result<Vec<WatchEvent>, DcsStoreError> {
            let state = self
                .state
                .lock()
                .map_err(|err| DcsStoreError::Io(format!("fake store lock failed: {err}")))?;
            let mut events = vec![WatchEvent {
                op: WatchOp::Reset,
                path: path_prefix.to_string(),
                value: None,
                revision: 0,
            }];
            for (path, value) in &state.paths {
                if path.starts_with(path_prefix) {
                    events.push(WatchEvent {
                        op: WatchOp::Put,
                        path: path.clone(),
                        value: Some(value.clone()),
                        revision: 0,
                    });
                }
            }
            Ok(events)
        }

        fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
            let mut state = self
                .state
                .lock()
                .map_err(|err| DcsStoreError::Io(format!("fake store lock failed: {err}")))?;
            state.paths.insert(path.to_string(), value);
            Ok(())
        }

        fn write_path_with_lease(
            &mut self,
            path: &str,
            value: String,
            _lease_ttl_ms: u64,
        ) -> Result<(), DcsStoreError> {
            self.write_path(path, value)
        }

        fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError> {
            let mut state = self
                .state
                .lock()
                .map_err(|err| DcsStoreError::Io(format!("fake store lock failed: {err}")))?;
            state.paths.remove(path);
            Ok(())
        }

        fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
            Ok(VecDeque::new().into())
        }

        fn acquire_leader_lease(
            &mut self,
            _scope: &str,
            _member_id: &MemberId,
        ) -> Result<(), DcsStoreError> {
            Ok(())
        }

        fn release_leader_lease(
            &mut self,
            scope: &str,
            member_id: &MemberId,
        ) -> Result<(), DcsStoreError> {
            let mut state = self
                .state
                .lock()
                .map_err(|err| DcsStoreError::Io(format!("fake store lock failed: {err}")))?;
            state.leader_release_calls = state.leader_release_calls.saturating_add(1);
            state.paths.remove(&leader_path(scope));
            let member_path = format!("/{}/member/{}", scope.trim_matches('/'), member_id.0);
            state.paths.remove(&member_path);
            Ok(())
        }

        fn clear_switchover(&mut self, _scope: &str) -> Result<(), DcsStoreError> {
            Ok(())
        }
    }

    fn stale_pg_state() -> PgInfoState {
        PgInfoState::Unknown {
            common: PgInfoCommon {
                worker: WorkerStatus::Running,
                sql: SqlStatus::Unknown,
                readiness: Readiness::Unknown,
                timeline: Some(TimelineId(1)),
                system_identifier: None,
                pg_config: PgConfig {
                    port: None,
                    hot_standby: None,
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: BTreeMap::new(),
                },
                last_refresh_at: None,
            },
        }
    }

    fn primary_member_record(self_id: &MemberId, lease_ttl_ms: u64) -> MemberRecord {
        build_local_member_record(
            self_id,
            "127.0.0.1",
            5432,
            Some("https://127.0.0.1:8443"),
            lease_ttl_ms,
            &PgInfoState::Primary {
                common: PgInfoCommon {
                    worker: WorkerStatus::Running,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: Some(TimelineId(1)),
                    system_identifier: None,
                    pg_config: PgConfig {
                        port: None,
                        hot_standby: None,
                        primary_conninfo: None,
                        primary_slot_name: None,
                        extra: BTreeMap::new(),
                    },
                    last_refresh_at: Some(UnixMillis(1)),
                },
                wal_lsn: crate::state::WalLsn(42),
                slots: Vec::new(),
            },
        )
    }

    #[tokio::test]
    async fn stale_local_snapshot_releases_owned_leader_lease() -> Result<(), WorkerError> {
        let self_id = MemberId("node-a".to_string());
        let scope = "scope-a".to_string();
        let lease_ttl_ms = 10_000;
        let stale_pg = stale_pg_state();
        let member_record = primary_member_record(&self_id, lease_ttl_ms);
        let member_path = format!("/{}/member/{}", scope, self_id.0);
        let member_value = serde_json::to_string(&member_record)
            .map_err(|err| WorkerError::Message(format!("encode member record failed: {err}")))?;
        let (leader_path, leader_value) = encode_leader_record(&scope, &self_id, 7)
            .map_err(|err| WorkerError::Message(err.to_string()))?;

        let shared_state = Arc::new(Mutex::new(FakeStoreState {
            leader_release_calls: 0,
            paths: BTreeMap::from([
                (member_path.clone(), member_value),
                (leader_path.clone(), leader_value),
            ]),
        }));

        let (_pg_publisher, pg_subscriber) = new_state_channel(stale_pg);
        let (dcs_publisher, dcs_subscriber) =
            new_state_channel(crate::dcs::DcsView::empty(WorkerStatus::Starting));
        let (_handle, command_inbox) = crate::dcs::command::dcs_command_channel();

        let mut ctx = DcsWorkerCtx {
            identity: DcsNodeIdentity {
                self_id: self_id.clone(),
                scope: scope.clone(),
            },
            cadence: DcsCadence {
                poll_interval: std::time::Duration::from_secs(1),
                member_ttl_ms: lease_ttl_ms,
            },
            advertisement: DcsLocalMemberAdvertisement {
                postgres: crate::dcs::DcsMemberEndpointView {
                    host: "127.0.0.1".to_string(),
                    port: 5432,
                },
                api: super::super::state::DcsApiAdvertisement::Advertised(
                    crate::dcs::DcsMemberApiView {
                        url: "https://127.0.0.1:8443".to_string(),
                    },
                ),
            },
            observed: DcsObservedState { pg: pg_subscriber },
            state_channel: DcsStateChannel {
                publisher: dcs_publisher,
                cache: DcsCache {
                    member_records: BTreeMap::from([(self_id.clone(), member_record)]),
                    leader_record: Some(LeaderLeaseRecord {
                        holder: self_id.clone(),
                        generation: 7,
                    }),
                    switchover_record: None,
                    init_lock: None,
                },
            },
            control: DcsControlPlane {
                command_inbox,
                store: Box::new(FakeDcsStore::new(Arc::clone(&shared_state))),
            },
            runtime: DcsRuntime {
                log: LogHandle::disabled(),
                last_emitted_store_healthy: None,
                last_emitted_trust: None,
            },
        };

        step_once(&mut ctx).await?;

        let published = dcs_subscriber.latest();
        let state = shared_state
            .lock()
            .map_err(|err| WorkerError::Message(format!("fake store lock failed: {err}")))?;

        if state.leader_release_calls != 1 {
            return Err(WorkerError::Message(format!(
                "expected exactly one leader release, observed {}",
                state.leader_release_calls
            )));
        }
        if state.paths.contains_key(&leader_path) {
            return Err(WorkerError::Message(
                "expected leader path to be removed after stale snapshot".to_string(),
            ));
        }
        if state.paths.contains_key(&member_path) {
            return Err(WorkerError::Message(
                "expected member path to be removed after stale snapshot".to_string(),
            ));
        }
        if !matches!(published.leader, DcsLeaderStateView::Unheld) {
            return Err(WorkerError::Message(format!(
                "expected published leader to be unheld, observed {:?}",
                published.leader
            )));
        }
        if !matches!(published.trust, DcsTrust::Degraded) {
            return Err(WorkerError::Message(format!(
                "expected published trust to be degraded, observed {:?}",
                published.trust
            )));
        }
        if published.members.get(&self_id).is_some_and(|member| {
            matches!(
                member.lease,
                DcsMemberLeaseView { ttl_ms, .. } if ttl_ms == lease_ttl_ms
            )
        }) {
            return Err(WorkerError::Message(
                "expected local member to be absent from published DCS view".to_string(),
            ));
        }

        Ok(())
    }
}
