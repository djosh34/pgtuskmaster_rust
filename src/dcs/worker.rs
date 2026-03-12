use crate::{
    logging::{AppEvent, AppEventHeader, SeverityText, StructuredFields},
    state::WorkerError,
};

use super::{
    keys::DcsKey,
    state::{
        build_local_member_slot, evaluate_trust, DcsCache, DcsState, DcsTrust, DcsWorkerCtx,
        InitLockRecord, LeaderLeaseRecord, MemberSlot, SwitchoverIntentRecord,
    },
    store::{leader_path, refresh_from_etcd_watch, write_local_member_slot},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsValue {
    Member(MemberSlot),
    Leader(LeaderLeaseRecord),
    Switchover(SwitchoverIntentRecord),
    Config(Box<crate::config::RuntimeConfig>),
    InitLock(InitLockRecord),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsWatchUpdate {
    Put { key: DcsKey, value: Box<DcsValue> },
    Delete { key: DcsKey },
}

fn dcs_append_base_fields(fields: &mut StructuredFields, ctx: &DcsWorkerCtx) {
    fields.insert("scope", ctx.scope.clone());
    fields.insert("member_id", ctx.self_id.0.clone());
}

fn dcs_event(severity: SeverityText, message: &str, name: &str, result: &str) -> AppEvent {
    AppEvent::new(severity, message, AppEventHeader::new(name, "dcs", result))
}

fn emit_dcs_event(
    ctx: &DcsWorkerCtx,
    origin: &str,
    event: AppEvent,
    error_prefix: &str,
) -> Result<(), WorkerError> {
    ctx.log
        .emit_app_event(origin, event)
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

pub(crate) async fn run(mut ctx: DcsWorkerCtx) -> Result<(), WorkerError> {
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.poll_interval).await;
    }
}

pub(crate) fn apply_watch_update(cache: &mut DcsCache, update: DcsWatchUpdate) {
    match update {
        DcsWatchUpdate::Put { key, value } => match (key, *value) {
            (DcsKey::Member(member_id), DcsValue::Member(record)) => {
                cache.member_slots.insert(member_id, record);
            }
            (DcsKey::Leader, DcsValue::Leader(record)) => {
                cache.leader_lease = Some(record);
            }
            (DcsKey::Switchover, DcsValue::Switchover(record)) => {
                cache.switchover_intent = Some(record);
            }
            (DcsKey::Config, DcsValue::Config(config)) => {
                cache.config = *config;
            }
            (DcsKey::InitLock, DcsValue::InitLock(record)) => {
                cache.init_lock = Some(record);
            }
            _ => {}
        },
        DcsWatchUpdate::Delete { key } => match key {
            DcsKey::Member(member_id) => {
                cache.member_slots.remove(&member_id);
            }
            DcsKey::Leader => {
                cache.leader_lease = None;
            }
            DcsKey::Switchover => {
                cache.switchover_intent = None;
            }
            DcsKey::Config => {}
            DcsKey::InitLock => {
                cache.init_lock = None;
            }
        },
    }
}

pub(crate) async fn step_once(ctx: &mut DcsWorkerCtx) -> Result<(), WorkerError> {
    let now = now_unix_millis()?;
    let pg_snapshot = ctx.pg_subscriber.latest();
    let member_ttl_ms = ctx.cache.config.ha.lease_ttl_ms;
    let local_member_path = format!("/{}/member/{}", ctx.scope.trim_matches('/'), ctx.self_id.0);
    let pg_snapshot_stale = now.0.saturating_sub(pg_snapshot.updated_at.0) > member_ttl_ms;

    let mut store_healthy = ctx.store.healthy();
    if store_healthy && pg_snapshot_stale {
        match ctx.store.delete_path(local_member_path.as_str()) {
            Ok(()) => {
                ctx.cache.member_slots.remove(&ctx.self_id);
            }
            Err(err) => {
                let mut event = dcs_event(
                    dcs_io_error_severity(&err),
                    "dcs local member delete failed",
                    "dcs.local_member.delete_failed",
                    "failed",
                );
                let fields = event.fields_mut();
                dcs_append_base_fields(fields, ctx);
                fields.insert("error", err.to_string());
                emit_dcs_event(
                    ctx,
                    "dcs_worker::step_once",
                    event,
                    "dcs local member delete log emit failed",
                )?;
                store_healthy = false;
            }
        }
    } else if store_healthy {
        let local_member = build_local_member_slot(
            &ctx.self_id,
            ctx.local_postgres_host.as_str(),
            ctx.local_postgres_port,
            ctx.local_api_url.as_deref(),
            member_ttl_ms,
            &pg_snapshot.value,
            pg_snapshot.version,
        );
        match write_local_member_slot(ctx.store.as_mut(), &ctx.scope, &local_member, member_ttl_ms)
        {
            Ok(()) => {
                ctx.last_published_pg_version = Some(pg_snapshot.version);
                ctx.cache.member_slots.insert(ctx.self_id.clone(), local_member);
            }
            Err(err) => {
                let mut event = dcs_event(
                    dcs_io_error_severity(&err),
                    "dcs local member write failed",
                    "dcs.local_member.write_failed",
                    "failed",
                );
                let fields = event.fields_mut();
                dcs_append_base_fields(fields, ctx);
                fields.insert("error", err.to_string());
                emit_dcs_event(
                    ctx,
                    "dcs_worker::step_once",
                    event,
                    "dcs local member write log emit failed",
                )?;
                store_healthy = false;
            }
        }
    }

    let events = match ctx.store.drain_watch_events() {
        Ok(events) => events,
        Err(err) => {
            let mut event = dcs_event(
                dcs_io_error_severity(&err),
                "dcs watch drain failed",
                "dcs.watch.drain_failed",
                "failed",
            );
            let fields = event.fields_mut();
            dcs_append_base_fields(fields, ctx);
            fields.insert("error", err.to_string());
            emit_dcs_event(
                ctx,
                "dcs_worker::step_once",
                event,
                "dcs drain log emit failed",
            )?;
            store_healthy = false;
            Vec::new()
        }
    };
    match refresh_from_etcd_watch(&ctx.scope, &mut ctx.cache, events) {
        Ok(result) => {
            if result.had_errors {
                let mut event = dcs_event(
                    SeverityText::Warn,
                    "dcs watch refresh had errors",
                    "dcs.watch.apply_had_errors",
                    "failed",
                );
                let fields = event.fields_mut();
                dcs_append_base_fields(fields, ctx);
                fields.insert("applied", result.applied);
                emit_dcs_event(
                    ctx,
                    "dcs_worker::step_once",
                    event,
                    "dcs refresh had_errors log emit failed",
                )?;
                store_healthy = false;
            }
        }
        Err(err) => {
            let mut event = dcs_event(
                dcs_refresh_error_severity(&err),
                "dcs watch refresh failed",
                "dcs.watch.refresh_failed",
                "failed",
            );
            let fields = event.fields_mut();
            dcs_append_base_fields(fields, ctx);
            fields.insert("error", err.to_string());
            emit_dcs_event(
                ctx,
                "dcs_worker::step_once",
                event,
                "dcs refresh log emit failed",
            )?;
            store_healthy = false;
        }
    }

    if store_healthy {
        let scope_prefix = format!("/{}/", ctx.scope.trim_matches('/'));
        match ctx.store.snapshot_prefix(scope_prefix.as_str()) {
            Ok(snapshot_events) => match refresh_from_etcd_watch(&ctx.scope, &mut ctx.cache, snapshot_events) {
                Ok(result) => {
                    if result.had_errors {
                        let mut event = dcs_event(
                            SeverityText::Warn,
                            "dcs snapshot refresh had errors",
                            "dcs.snapshot.apply_had_errors",
                            "failed",
                        );
                        let fields = event.fields_mut();
                        dcs_append_base_fields(fields, ctx);
                        fields.insert("applied", result.applied);
                        emit_dcs_event(
                            ctx,
                            "dcs_worker::step_once",
                            event,
                            "dcs snapshot had_errors log emit failed",
                        )?;
                        store_healthy = false;
                    }
                }
                Err(err) => {
                    let mut event = dcs_event(
                        dcs_refresh_error_severity(&err),
                        "dcs snapshot refresh failed",
                        "dcs.snapshot.refresh_failed",
                        "failed",
                    );
                    let fields = event.fields_mut();
                    dcs_append_base_fields(fields, ctx);
                    fields.insert("error", err.to_string());
                    emit_dcs_event(
                        ctx,
                        "dcs_worker::step_once",
                        event,
                        "dcs snapshot refresh log emit failed",
                    )?;
                    store_healthy = false;
                }
            },
            Err(err) => {
                let mut event = dcs_event(
                    dcs_io_error_severity(&err),
                    "dcs snapshot read failed",
                    "dcs.snapshot.read_failed",
                    "failed",
                );
                let fields = event.fields_mut();
                dcs_append_base_fields(fields, ctx);
                fields.insert("error", err.to_string());
                emit_dcs_event(
                    ctx,
                    "dcs_worker::step_once",
                    event,
                    "dcs snapshot read log emit failed",
                )?;
                store_healthy = false;
            }
        }
    }

    let stale_leader_holder = ctx.cache.leader_lease.as_ref().and_then(|leader| {
        (!ctx.cache.member_slots.contains_key(&leader.holder)).then(|| leader.holder.clone())
    });
    if let Some(member_id) = stale_leader_holder {
        let leader_key = leader_path(ctx.scope.as_str());
        match ctx.store.delete_path(leader_key.as_str()) {
            Ok(()) => {
                ctx.cache.leader_lease = None;
            }
            Err(err) => {
                let mut event = dcs_event(
                    dcs_io_error_severity(&err),
                    "dcs stale leader cleanup failed",
                    "dcs.leader.cleanup_failed",
                    "failed",
                );
                let fields = event.fields_mut();
                dcs_append_base_fields(fields, ctx);
                fields.insert("leader_member_id", member_id.0);
                fields.insert("error", err.to_string());
                emit_dcs_event(
                    ctx,
                    "dcs_worker::step_once",
                    event,
                    "dcs stale leader cleanup log emit failed",
                )?;
                store_healthy = false;
            }
        }
    }

    let trust = evaluate_trust(store_healthy, &ctx.cache, &ctx.self_id);
    let worker = if store_healthy {
        crate::state::WorkerStatus::Running
    } else {
        crate::state::WorkerStatus::Faulted(WorkerError::Message("dcs store unhealthy".to_string()))
    };

    let next = DcsState {
        worker,
        trust: if store_healthy {
            trust
        } else {
            DcsTrust::NotTrusted
        },
        cache: ctx.cache.clone(),
        last_refresh_at: Some(now),
    };
    if ctx.last_emitted_store_healthy != Some(store_healthy) {
        ctx.last_emitted_store_healthy = Some(store_healthy);
        let mut event = dcs_event(
            if store_healthy {
                SeverityText::Info
            } else {
                SeverityText::Warn
            },
            "dcs store health transition",
            "dcs.store.health_transition",
            if store_healthy { "recovered" } else { "failed" },
        );
        let fields = event.fields_mut();
        dcs_append_base_fields(fields, ctx);
        fields.insert("store_healthy", store_healthy);
        emit_dcs_event(
            ctx,
            "dcs_worker::step_once",
            event,
            "dcs health transition log emit failed",
        )?;
    }
    if ctx.last_emitted_trust.as_ref() != Some(&next.trust) {
        let prev = ctx
            .last_emitted_trust
            .as_ref()
            .map(|value| format!("{value:?}").to_lowercase())
            .unwrap_or_else(|| "unknown".to_string());
        ctx.last_emitted_trust = Some(next.trust.clone());
        let mut event = dcs_event(
            SeverityText::Info,
            "dcs trust transition",
            "dcs.trust.transition",
            "ok",
        );
        let fields = event.fields_mut();
        dcs_append_base_fields(fields, ctx);
        fields.insert("trust_prev", prev);
        fields.insert("trust_next", format!("{:?}", next.trust).to_lowercase());
        emit_dcs_event(
            ctx,
            "dcs_worker::step_once",
            event,
            "dcs trust transition log emit failed",
        )?;
    }
    ctx.publisher
        .publish(next, now)
        .map_err(|err| WorkerError::Message(format!("dcs publish failed: {err}")))?;
    Ok(())
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
        time::Duration,
    };

    use crate::{
        config::RuntimeConfig,
        dcs::{
            keys::DcsKey,
            state::{
                DcsCache, DcsState, DcsTrust, DcsWorkerCtx, InitLockRecord, LeaderLeaseRecord,
                MemberApiEndpoint, MemberEndpoint, MemberLease, MemberPostgresView, MemberRouting,
                MemberSlot, PrimaryObservation, SwitchoverIntentRecord, SwitchoverTargetRecord,
                WalVector,
            },
            store::{DcsStore, DcsStoreError, WatchEvent, WatchOp},
            worker::{apply_watch_update, DcsValue, DcsWatchUpdate},
        },
        logging::{LogHandle, LogSink, SeverityText, TestSink},
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        state::{new_state_channel, MemberId, TimelineId, UnixMillis, Version, WalLsn, WorkerError, WorkerStatus},
    };

    use super::step_once;

    const TEST_DCS_POLL_INTERVAL: Duration = Duration::from_millis(5);

    #[derive(Clone, Default)]
    struct RecordingStore {
        healthy: bool,
        events: Arc<Mutex<VecDeque<WatchEvent>>>,
        writes: Arc<Mutex<Vec<(String, String)>>>,
        deletes: Arc<Mutex<Vec<String>>>,
        kv: Arc<Mutex<std::collections::BTreeMap<String, String>>>,
    }

    impl RecordingStore {
        fn new(healthy: bool) -> Self {
            Self {
                healthy,
                events: Arc::new(Mutex::new(VecDeque::new())),
                writes: Arc::new(Mutex::new(Vec::new())),
                deletes: Arc::new(Mutex::new(Vec::new())),
                kv: Arc::new(Mutex::new(std::collections::BTreeMap::new())),
            }
        }

        fn push_event(&self, event: WatchEvent) {
            if let Ok(mut kv) = self.kv.lock() {
                match event.op {
                    WatchOp::Put => {
                        if let Some(value) = event.value.as_ref() {
                            kv.insert(event.path.clone(), value.clone());
                        }
                    }
                    WatchOp::Delete => {
                        kv.remove(event.path.as_str());
                    }
                    WatchOp::Reset => {
                        let prefix = event.path.trim_end_matches('/').to_string();
                        kv.retain(|path, _| !path.starts_with(prefix.as_str()));
                    }
                }
            }
            if let Ok(mut guard) = self.events.lock() {
                guard.push_back(event);
            }
        }

        fn write_count(&self) -> usize {
            self.writes.lock().map(|guard| guard.len()).unwrap_or(0)
        }

        fn first_write_path(&self) -> Option<String> {
            self.writes
                .lock()
                .ok()
                .and_then(|guard| guard.first().map(|(path, _)| path.clone()))
        }
    }

    impl DcsStore for RecordingStore {
        fn healthy(&self) -> bool {
            self.healthy
        }

        fn read_path(&mut self, path: &str) -> Result<Option<String>, DcsStoreError> {
            self.kv
                .lock()
                .map(|guard| guard.get(path).cloned())
                .map_err(|_| DcsStoreError::Io("kv lock poisoned".to_string()))
        }

        fn snapshot_prefix(&mut self, path_prefix: &str) -> Result<Vec<WatchEvent>, DcsStoreError> {
            let guard = self
                .kv
                .lock()
                .map_err(|_| DcsStoreError::Io("kv lock poisoned".to_string()))?;
            let mut events = vec![WatchEvent {
                op: WatchOp::Reset,
                path: path_prefix.to_string(),
                value: None,
                revision: 0,
            }];
            events.extend(
                guard
                    .iter()
                    .filter(|(path, _)| path.starts_with(path_prefix))
                    .map(|(path, value)| WatchEvent {
                        op: WatchOp::Put,
                        path: path.clone(),
                        value: Some(value.clone()),
                        revision: 0,
                    }),
            );
            Ok(events)
        }

        fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
            self.kv
                .lock()
                .map_err(|_| DcsStoreError::Io("kv lock poisoned".to_string()))?
                .insert(path.to_string(), value.clone());
            self.writes
                .lock()
                .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?
                .push((path.to_string(), value));
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

        fn put_path_if_absent(&mut self, path: &str, value: String) -> Result<bool, DcsStoreError> {
            if self
                .kv
                .lock()
                .map_err(|_| DcsStoreError::Io("kv lock poisoned".to_string()))?
                .contains_key(path)
            {
                return Ok(false);
            }
            self.write_path(path, value)?;
            Ok(true)
        }

        fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError> {
            self.deletes
                .lock()
                .map_err(|_| DcsStoreError::Io("deletes lock poisoned".to_string()))?
                .push(path.to_string());
            self.kv
                .lock()
                .map_err(|_| DcsStoreError::Io("kv lock poisoned".to_string()))?
                .remove(path);
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

    #[derive(Clone, Default)]
    struct FailingWriteStore {
        events: Arc<Mutex<VecDeque<WatchEvent>>>,
    }

    impl DcsStore for FailingWriteStore {
        fn healthy(&self) -> bool {
            true
        }

        fn read_path(&mut self, _path: &str) -> Result<Option<String>, DcsStoreError> {
            Ok(None)
        }

        fn snapshot_prefix(&mut self, _path_prefix: &str) -> Result<Vec<WatchEvent>, DcsStoreError> {
            Ok(Vec::new())
        }

        fn write_path(&mut self, _path: &str, _value: String) -> Result<(), DcsStoreError> {
            Err(DcsStoreError::Io("boom".to_string()))
        }

        fn write_path_with_lease(
            &mut self,
            _path: &str,
            _value: String,
            _lease_ttl_ms: u64,
        ) -> Result<(), DcsStoreError> {
            Err(DcsStoreError::Io("boom".to_string()))
        }

        fn put_path_if_absent(&mut self, _path: &str, _value: String) -> Result<bool, DcsStoreError> {
            Err(DcsStoreError::Io("boom".to_string()))
        }

        fn delete_path(&mut self, _path: &str) -> Result<(), DcsStoreError> {
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

    fn test_log_handle() -> (LogHandle, Arc<TestSink>) {
        let sink = Arc::new(TestSink::default());
        let sink_dyn: Arc<dyn LogSink> = sink.clone();
        (
            LogHandle::new("host-a".to_string(), sink_dyn, SeverityText::Trace),
            sink,
        )
    }

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::sample_runtime_config()
    }

    fn sample_cache(cfg: RuntimeConfig) -> DcsCache {
        DcsCache {
            member_slots: BTreeMap::new(),
            leader_lease: None,
            switchover_intent: None,
            config: cfg,
            init_lock: None,
        }
    }

    fn sample_member_slot(member_id: &str) -> MemberSlot {
        MemberSlot {
            lease: MemberLease {
                owner: MemberId(member_id.to_string()),
                ttl_ms: 10_000,
            },
            routing: MemberRouting {
                postgres: MemberEndpoint {
                    host: "127.0.0.1".to_string(),
                    port: 5432,
                },
                api: Some(MemberApiEndpoint {
                    url: format!("https://{member_id}:8443"),
                }),
            },
            postgres: MemberPostgresView::Primary(PrimaryObservation {
                readiness: Readiness::Ready,
                committed_wal: WalVector {
                    timeline: Some(TimelineId(1)),
                    lsn: WalLsn(42),
                },
                pg_version: Version(1),
            }),
        }
    }

    fn sample_pg() -> PgInfoState {
        PgInfoState::Primary {
            common: PgInfoCommon {
                worker: WorkerStatus::Running,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: Some(TimelineId(1)),
                pg_config: PgConfig {
                    port: None,
                    hot_standby: None,
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: BTreeMap::new(),
                },
                last_refresh_at: Some(fresh_unix_millis()),
            },
            wal_lsn: WalLsn(42),
            slots: Vec::new(),
        }
    }

    fn fresh_unix_millis() -> UnixMillis {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .and_then(|elapsed| u64::try_from(elapsed.as_millis()).ok())
            .map(UnixMillis)
            .unwrap_or(UnixMillis(0))
    }

    #[test]
    fn apply_watch_update_handles_put_and_delete_paths() {
        let mut cache = sample_cache(sample_runtime_config());
        let member_id = MemberId("node-a".to_string());
        apply_watch_update(
            &mut cache,
            DcsWatchUpdate::Put {
                key: DcsKey::Member(member_id.clone()),
                value: Box::new(DcsValue::Member(sample_member_slot("node-a"))),
            },
        );
        assert!(cache.member_slots.contains_key(&member_id));

        apply_watch_update(
            &mut cache,
            DcsWatchUpdate::Put {
                key: DcsKey::Leader,
                value: Box::new(DcsValue::Leader(LeaderLeaseRecord {
                    holder: member_id.clone(),
                    generation: 1,
                })),
            },
        );
        assert!(cache.leader_lease.is_some());

        apply_watch_update(
            &mut cache,
            DcsWatchUpdate::Put {
                key: DcsKey::Switchover,
                value: Box::new(DcsValue::Switchover(SwitchoverIntentRecord {
                    target: SwitchoverTargetRecord::AnyHealthyReplica,
                })),
            },
        );
        assert!(cache.switchover_intent.is_some());

        apply_watch_update(
            &mut cache,
            DcsWatchUpdate::Put {
                key: DcsKey::InitLock,
                value: Box::new(DcsValue::InitLock(InitLockRecord {
                    holder: member_id.clone(),
                })),
            },
        );
        assert!(cache.init_lock.is_some());

        apply_watch_update(&mut cache, DcsWatchUpdate::Delete { key: DcsKey::Member(member_id.clone()) });
        apply_watch_update(&mut cache, DcsWatchUpdate::Delete { key: DcsKey::Leader });
        apply_watch_update(&mut cache, DcsWatchUpdate::Delete { key: DcsKey::Switchover });
        apply_watch_update(&mut cache, DcsWatchUpdate::Delete { key: DcsKey::InitLock });

        assert!(!cache.member_slots.contains_key(&member_id));
        assert!(cache.leader_lease.is_none());
        assert!(cache.switchover_intent.is_none());
        assert!(cache.init_lock.is_none());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_publishes_and_writes_only_self_member() -> Result<(), Box<dyn std::error::Error>> {
        let initial_pg = sample_pg();
        let initial_pg_updated_at = fresh_unix_millis();
        let (pg_publisher, pg_subscriber) = new_state_channel(initial_pg, initial_pg_updated_at);
        let _ = pg_publisher.publish(sample_pg(), fresh_unix_millis());

        let initial_dcs = DcsState {
            worker: WorkerStatus::Starting,
            trust: DcsTrust::NotTrusted,
            cache: sample_cache(sample_runtime_config()),
            last_refresh_at: None,
        };
        let (dcs_publisher, dcs_subscriber) = new_state_channel(initial_dcs, UnixMillis(1));

        let leader_json = serde_json::to_string(&LeaderLeaseRecord {
            holder: MemberId("node-a".to_string()),
            generation: 1,
        })?;
        let store = RecordingStore::new(true);
        store.push_event(WatchEvent {
            op: WatchOp::Put,
            path: "/scope-a/leader".to_string(),
            value: Some(leader_json),
            revision: 2,
        });
        let store_probe = store.clone();
        let mut ctx = DcsWorkerCtx {
            self_id: MemberId("node-a".to_string()),
            scope: "scope-a".to_string(),
            poll_interval: TEST_DCS_POLL_INTERVAL,
            local_postgres_host: "127.0.0.1".to_string(),
            local_postgres_port: 5432,
            local_api_url: Some("http://127.0.0.1:8080".to_string()),
            pg_subscriber,
            publisher: dcs_publisher,
            store: Box::new(store),
            log: crate::logging::LogHandle::null(),
            cache: sample_cache(sample_runtime_config()),
            last_published_pg_version: None,
            last_emitted_store_healthy: None,
            last_emitted_trust: None,
        };

        step_once(&mut ctx).await?;

        let latest = dcs_subscriber.latest();
        assert_eq!(latest.value.trust, DcsTrust::FullQuorum);
        assert!(latest.value.cache.leader_lease.is_some());
        assert!(latest
            .value
            .cache
            .member_slots
            .contains_key(&MemberId("node-a".to_string())));
        assert_eq!(store_probe.write_count(), 1);
        assert_eq!(
            store_probe.first_write_path(),
            Some("/scope-a/member/node-a".to_string())
        );
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_emits_local_member_write_failed_event_for_io_error() -> Result<(), WorkerError> {
        let initial_pg = sample_pg();
        let initial_pg_updated_at = fresh_unix_millis();
        let (pg_publisher, pg_subscriber) = new_state_channel(initial_pg, initial_pg_updated_at);
        let _ = pg_publisher.publish(sample_pg(), fresh_unix_millis());

        let initial_dcs = DcsState {
            worker: WorkerStatus::Starting,
            trust: DcsTrust::NotTrusted,
            cache: sample_cache(sample_runtime_config()),
            last_refresh_at: None,
        };
        let (dcs_publisher, dcs_subscriber) = new_state_channel(initial_dcs, UnixMillis(1));

        let (log, sink) = test_log_handle();
        let mut ctx = DcsWorkerCtx {
            self_id: MemberId("node-a".to_string()),
            scope: "scope-a".to_string(),
            poll_interval: TEST_DCS_POLL_INTERVAL,
            local_postgres_host: "127.0.0.1".to_string(),
            local_postgres_port: 5432,
            local_api_url: Some("http://127.0.0.1:8080".to_string()),
            pg_subscriber,
            publisher: dcs_publisher,
            store: Box::new(FailingWriteStore::default()),
            log,
            cache: sample_cache(sample_runtime_config()),
            last_published_pg_version: None,
            last_emitted_store_healthy: None,
            last_emitted_trust: None,
        };

        step_once(&mut ctx).await?;

        let latest = dcs_subscriber.latest();
        assert_eq!(latest.value.trust, DcsTrust::NotTrusted);

        let records = sink.take();
        let failures = records
            .iter()
            .filter(|record| {
                record
                    .attributes
                    .get("event.name")
                    .and_then(serde_json::Value::as_str)
                    == Some("dcs.local_member.write_failed")
                    && record
                        .attributes
                        .get("event.domain")
                        .and_then(serde_json::Value::as_str)
                        == Some("dcs")
                    && record
                        .attributes
                        .get("event.result")
                        .and_then(serde_json::Value::as_str)
                        == Some("failed")
            })
            .collect::<Vec<_>>();
        if failures.is_empty() {
            return Err(WorkerError::Message(format!(
                "expected dcs.local_member.write_failed event, saw records: {records:#?}"
            )));
        }
        if !failures
            .into_iter()
            .any(|record| record.severity_text == SeverityText::Warn)
        {
            return Err(WorkerError::Message(
                "expected dcs.local_member.write_failed severity warn".to_string(),
            ));
        }
        Ok(())
    }
}
