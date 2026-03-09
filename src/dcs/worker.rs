use crate::{
    logging::{AppEvent, AppEventHeader, SeverityText, StructuredFields},
    state::WorkerError,
};

use super::{
    keys::DcsKey,
    state::{
        build_local_member_record, evaluate_trust, BuildLocalMemberRecordInput, DcsCache,
        DcsState, DcsTrust, DcsWorkerCtx, InitLockRecord, LeaderRecord, MemberRecord,
        SwitchoverRequest,
    },
    store::{refresh_from_etcd_watch, write_local_member},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DcsValue {
    Member(MemberRecord),
    Leader(LeaderRecord),
    Switchover(SwitchoverRequest),
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
                cache.members.insert(member_id, record);
            }
            (DcsKey::Leader, DcsValue::Leader(record)) => {
                cache.leader = Some(record);
            }
            (DcsKey::Switchover, DcsValue::Switchover(record)) => {
                cache.switchover = Some(record);
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
                cache.members.remove(&member_id);
            }
            DcsKey::Leader => {
                cache.leader = None;
            }
            DcsKey::Switchover => {
                cache.switchover = None;
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

    let mut store_healthy = ctx.store.healthy();
    let must_publish_local_member = store_healthy;
    let mut local_member_publish_succeeded = false;

    if must_publish_local_member {
        let local_member = build_local_member_record(BuildLocalMemberRecordInput {
            self_id: &ctx.self_id,
            postgres_host: ctx.local_postgres_host.as_str(),
            postgres_port: ctx.local_postgres_port,
            api_url: ctx.local_api_url.as_deref(),
            pg_state: &pg_snapshot.value,
            previous_record: ctx.cache.members.get(&ctx.self_id),
            now,
            pg_version: pg_snapshot.version,
        });
        match write_local_member(ctx.store.as_mut(), &ctx.scope, &local_member) {
            Ok(()) => {
                ctx.last_published_pg_version = Some(pg_snapshot.version);
                ctx.cache.members.insert(ctx.self_id.clone(), local_member);
                local_member_publish_succeeded = true;
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

    let trust = if local_member_publish_succeeded {
        evaluate_trust(store_healthy, &ctx.cache, &ctx.self_id, now)
    } else {
        DcsTrust::NotTrusted
    };
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
    use std::collections::BTreeMap;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use crate::{
        config::RuntimeConfig,
        dcs::{
            keys::DcsKey,
            state::{
                DcsCache, DcsState, DcsTrust, DcsWorkerCtx, InitLockRecord, LeaderRecord,
                MemberRecord, MemberRole, SwitchoverRequest,
            },
            store::{DcsStore, DcsStoreError, WatchEvent, WatchOp},
            worker::{apply_watch_update, DcsValue, DcsWatchUpdate},
        },
        logging::{decode_app_event, LogHandle, LogSink, SeverityText, TestSink},
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        state::{new_state_channel, MemberId, UnixMillis, Version, WorkerError, WorkerStatus},
    };

    use super::step_once;

    const TEST_DCS_POLL_INTERVAL: Duration = Duration::from_millis(5);

    #[derive(Clone, Default)]
    struct RecordingStore {
        healthy: bool,
        events: Arc<Mutex<VecDeque<WatchEvent>>>,
        writes: Arc<Mutex<Vec<(String, String)>>>,
    }

    impl RecordingStore {
        fn new(healthy: bool) -> Self {
            Self {
                healthy,
                events: Arc::new(Mutex::new(VecDeque::new())),
                writes: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn push_event(&self, event: WatchEvent) {
            if let Ok(mut guard) = self.events.lock() {
                guard.push_back(event);
            }
        }

        fn write_count(&self) -> usize {
            if let Ok(guard) = self.writes.lock() {
                guard.len()
            } else {
                0
            }
        }

        fn first_write_path(&self) -> Option<String> {
            if let Ok(guard) = self.writes.lock() {
                return guard.first().map(|(path, _)| path.clone());
            }
            None
        }

        fn first_write_value(&self) -> Option<String> {
            if let Ok(guard) = self.writes.lock() {
                return guard.first().map(|(_, value)| value.clone());
            }
            None
        }
    }

    impl DcsStore for RecordingStore {
        fn healthy(&self) -> bool {
            self.healthy
        }

        fn read_path(&mut self, _path: &str) -> Result<Option<String>, DcsStoreError> {
            Ok(None)
        }

        fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
            let mut guard = self
                .writes
                .lock()
                .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
            guard.push((path.to_string(), value));
            Ok(())
        }

        fn put_path_if_absent(&mut self, path: &str, value: String) -> Result<bool, DcsStoreError> {
            let mut guard = self
                .writes
                .lock()
                .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
            guard.push((path.to_string(), value));
            Ok(true)
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

        fn write_path(&mut self, _path: &str, _value: String) -> Result<(), DcsStoreError> {
            Err(DcsStoreError::Io("boom".to_string()))
        }

        fn put_path_if_absent(
            &mut self,
            _path: &str,
            _value: String,
        ) -> Result<bool, DcsStoreError> {
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

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::sample_runtime_config()
    }

    fn sample_pg() -> PgInfoState {
        PgInfoState::Primary {
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
            wal_lsn: crate::state::WalLsn(42),
            slots: Vec::new(),
        }
    }

    fn sample_cache(cfg: RuntimeConfig) -> DcsCache {
        DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: cfg,
            init_lock: None,
        }
    }

    #[test]
    fn apply_watch_update_handles_put_and_delete_paths() {
        let mut cache = sample_cache(sample_runtime_config());
        let member_id = MemberId("node-a".to_string());
        apply_watch_update(
            &mut cache,
            DcsWatchUpdate::Put {
                key: DcsKey::Member(member_id.clone()),
                value: Box::new(DcsValue::Member(MemberRecord {
                    member_id: member_id.clone(),
                    postgres_host: "10.0.0.10".to_string(),
                    postgres_port: 5432,
                    api_url: None,
                    role: MemberRole::Primary,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    updated_at: UnixMillis(1),
                    pg_version: Version(1),
                })),
            },
        );
        assert!(cache.members.contains_key(&member_id));

        apply_watch_update(
            &mut cache,
            DcsWatchUpdate::Put {
                key: DcsKey::Leader,
                value: Box::new(DcsValue::Leader(LeaderRecord {
                    member_id: member_id.clone(),
                })),
            },
        );
        assert!(cache.leader.is_some());

        apply_watch_update(
            &mut cache,
            DcsWatchUpdate::Put {
                key: DcsKey::Switchover,
                value: Box::new(DcsValue::Switchover(SwitchoverRequest {
                    switchover_to: None,
                })),
            },
        );
        assert!(cache.switchover.is_some());

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

        apply_watch_update(
            &mut cache,
            DcsWatchUpdate::Delete {
                key: DcsKey::Member(member_id.clone()),
            },
        );
        apply_watch_update(
            &mut cache,
            DcsWatchUpdate::Delete {
                key: DcsKey::Leader,
            },
        );
        apply_watch_update(
            &mut cache,
            DcsWatchUpdate::Delete {
                key: DcsKey::Switchover,
            },
        );
        apply_watch_update(
            &mut cache,
            DcsWatchUpdate::Delete {
                key: DcsKey::InitLock,
            },
        );

        assert!(!cache.members.contains_key(&member_id));
        assert!(cache.leader.is_none());
        assert!(cache.switchover.is_none());
        assert!(cache.init_lock.is_none());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_publishes_and_writes_only_self_member(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let initial_pg = sample_pg();
        let (pg_publisher, pg_subscriber) = new_state_channel(initial_pg, UnixMillis(1));
        let _ = pg_publisher.publish(sample_pg(), UnixMillis(2));

        let initial_dcs = DcsState {
            worker: WorkerStatus::Starting,
            trust: DcsTrust::NotTrusted,
            cache: sample_cache(sample_runtime_config()),
            last_refresh_at: None,
        };
        let (dcs_publisher, dcs_subscriber) = new_state_channel(initial_dcs, UnixMillis(1));

        let leader_json = serde_json::to_string(&LeaderRecord {
            member_id: MemberId("node-a".to_string()),
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

        let stepped = step_once(&mut ctx).await;
        assert_eq!(stepped, Ok(()));

        let latest = dcs_subscriber.latest();
        assert_eq!(latest.value.trust, DcsTrust::FullQuorum);
        assert!(latest.value.cache.leader.is_some());
        assert!(latest
            .value
            .cache
            .members
            .contains_key(&MemberId("node-a".to_string())));
        assert_eq!(store_probe.write_count(), 1);
        assert_eq!(
            store_probe.first_write_path(),
            Some("/scope-a/member/node-a".to_string())
        );
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_emits_local_member_write_failed_event_for_io_error(
    ) -> Result<(), WorkerError> {
        let initial_pg = sample_pg();
        let (_pg_publisher, pg_subscriber) = new_state_channel(initial_pg, UnixMillis(1));

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

        let failures = sink
            .take()
            .into_iter()
            .filter_map(|record| decode_app_event(&record).ok())
            .filter(|event| {
                event.header
                    == crate::logging::AppEventHeader::new(
                        "dcs.local_member.write_failed",
                        "dcs",
                        "failed",
                    )
            })
            .collect::<Vec<_>>();
        if failures.is_empty() {
            return Err(WorkerError::Message(
                "expected dcs.local_member.write_failed event".to_string(),
            ));
        }
        if !failures
            .iter()
            .any(|event| event.severity == SeverityText::Warn)
        {
            return Err(WorkerError::Message(
                "expected dcs.local_member.write_failed severity warn".to_string(),
            ));
        }
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_writes_member_on_every_tick() {
        let initial_pg = sample_pg();
        let (pg_publisher, pg_subscriber) = new_state_channel(initial_pg.clone(), UnixMillis(1));
        let initial_dcs = DcsState {
            worker: WorkerStatus::Starting,
            trust: DcsTrust::NotTrusted,
            cache: sample_cache(sample_runtime_config()),
            last_refresh_at: None,
        };
        let (dcs_publisher, _dcs_subscriber) = new_state_channel(initial_dcs, UnixMillis(1));

        let store = RecordingStore::new(true);
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

        let first = step_once(&mut ctx).await;
        assert_eq!(first, Ok(()));

        let second = step_once(&mut ctx).await;
        assert_eq!(second, Ok(()));
        assert_eq!(store_probe.write_count(), 2);

        let _ = pg_publisher.publish(initial_pg, UnixMillis(2));
        let third = step_once(&mut ctx).await;
        assert_eq!(third, Ok(()));
        assert_eq!(store_probe.write_count(), 3);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_publishes_local_endpoint_instead_of_cached_config_endpoint(
    ) -> Result<(), WorkerError> {
        let initial_pg = sample_pg();
        let (_pg_publisher, pg_subscriber) = new_state_channel(initial_pg, UnixMillis(1));
        let initial_dcs = DcsState {
            worker: WorkerStatus::Starting,
            trust: DcsTrust::NotTrusted,
            cache: sample_cache(sample_runtime_config()),
            last_refresh_at: None,
        };
        let (dcs_publisher, _dcs_subscriber) = new_state_channel(initial_dcs, UnixMillis(1));

        let store = RecordingStore::new(true);
        let store_probe = store.clone();
        let mut ctx = DcsWorkerCtx {
            self_id: MemberId("node-a".to_string()),
            scope: "scope-a".to_string(),
            poll_interval: TEST_DCS_POLL_INTERVAL,
            local_postgres_host: "127.0.0.9".to_string(),
            local_postgres_port: 6543,
            local_api_url: Some("http://127.0.0.9:6543".to_string()),
            pg_subscriber,
            publisher: dcs_publisher,
            store: Box::new(store),
            log: crate::logging::LogHandle::null(),
            cache: sample_cache(sample_runtime_config()),
            last_published_pg_version: None,
            last_emitted_store_healthy: None,
            last_emitted_trust: None,
        };

        assert_eq!(step_once(&mut ctx).await, Ok(()));
        let encoded = store_probe
            .first_write_value()
            .ok_or_else(|| WorkerError::Message("expected local member write".to_string()))?;
        let record: MemberRecord = serde_json::from_str(encoded.as_str()).map_err(|err| {
            WorkerError::Message(format!("decode written member record failed: {err}"))
        })?;
        assert_eq!(record.postgres_host, "127.0.0.9".to_string());
        assert_eq!(record.postgres_port, 6543);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_republishes_member_after_unhealthy_tick_even_without_pg_change() {
        let initial_pg = sample_pg();
        let (_pg_publisher, pg_subscriber) = new_state_channel(initial_pg.clone(), UnixMillis(1));
        let initial_dcs = DcsState {
            worker: WorkerStatus::Starting,
            trust: DcsTrust::NotTrusted,
            cache: sample_cache(sample_runtime_config()),
            last_refresh_at: None,
        };
        let (dcs_publisher, _dcs_subscriber) = new_state_channel(initial_dcs, UnixMillis(1));

        let store = RecordingStore::new(true);
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
            last_published_pg_version: Some(Version(1)),
            last_emitted_store_healthy: Some(false),
            last_emitted_trust: None,
        };

        let stepped = step_once(&mut ctx).await;
        assert_eq!(stepped, Ok(()));
        assert_eq!(store_probe.write_count(), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_marks_store_unhealthy_when_watch_decode_fails() {
        let initial_pg = sample_pg();
        let (_pg_publisher, pg_subscriber) = new_state_channel(initial_pg, UnixMillis(1));
        let initial_dcs = DcsState {
            worker: WorkerStatus::Starting,
            trust: DcsTrust::NotTrusted,
            cache: sample_cache(sample_runtime_config()),
            last_refresh_at: None,
        };
        let (dcs_publisher, dcs_subscriber) = new_state_channel(initial_dcs, UnixMillis(1));

        let store = RecordingStore::new(true);
        store.push_event(WatchEvent {
            op: WatchOp::Put,
            path: "/scope-a/leader".to_string(),
            value: Some("{invalid-json".to_string()),
            revision: 2,
        });

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

        assert_eq!(step_once(&mut ctx).await, Ok(()));
        let latest = dcs_subscriber.latest();
        assert_eq!(latest.value.trust, DcsTrust::NotTrusted);
        assert!(matches!(
            latest.value.worker,
            WorkerStatus::Faulted(WorkerError::Message(_))
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_marks_store_unhealthy_when_watch_key_is_unknown() {
        let initial_pg = sample_pg();
        let (_pg_publisher, pg_subscriber) = new_state_channel(initial_pg, UnixMillis(1));
        let initial_dcs = DcsState {
            worker: WorkerStatus::Starting,
            trust: DcsTrust::NotTrusted,
            cache: sample_cache(sample_runtime_config()),
            last_refresh_at: None,
        };
        let (dcs_publisher, dcs_subscriber) = new_state_channel(initial_dcs, UnixMillis(1));

        let store = RecordingStore::new(true);
        store.push_event(WatchEvent {
            op: WatchOp::Put,
            path: "/scope-a/not-a-real-key".to_string(),
            value: Some("{\"ignored\":true}".to_string()),
            revision: 2,
        });

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

        assert_eq!(step_once(&mut ctx).await, Ok(()));
        let latest = dcs_subscriber.latest();
        assert_eq!(latest.value.trust, DcsTrust::NotTrusted);
        assert!(matches!(
            latest.value.worker,
            WorkerStatus::Faulted(WorkerError::Message(_))
        ));
    }
}
