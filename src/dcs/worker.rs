use crate::state::WorkerError;

use super::{
    keys::DcsKey,
    state::{
        build_local_member_record, evaluate_trust, DcsCache, DcsState, DcsTrust, DcsWorkerCtx,
        InitLockRecord, LeaderRecord, MemberRecord, SwitchoverRequest,
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

    if ctx.last_published_pg_version != Some(pg_snapshot.version) {
        let local_member =
            build_local_member_record(&ctx.self_id, &pg_snapshot.value, now, pg_snapshot.version);
        if write_local_member(ctx.store.as_mut(), &ctx.scope, &local_member).is_ok() {
            ctx.last_published_pg_version = Some(pg_snapshot.version);
            ctx.cache.members.insert(ctx.self_id.clone(), local_member);
        } else {
            store_healthy = false;
        }
    }

    let events = match ctx.store.drain_watch_events() {
        Ok(events) => events,
        Err(_) => {
            store_healthy = false;
            Vec::new()
        }
    };
    match refresh_from_etcd_watch(&ctx.scope, &mut ctx.cache, events) {
        Ok(result) => {
            if result.had_errors {
                store_healthy = false;
            }
        }
        Err(_) => {
            store_healthy = false;
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
        config::{
            schema::{
                ApiConfig, ClusterConfig, DcsConfig, DebugConfig, HaConfig, PostgresConfig,
                SecurityConfig,
            },
            BinaryPaths, ProcessConfig, RuntimeConfig,
        },
        dcs::{
            keys::DcsKey,
            state::{
                DcsCache, DcsState, DcsTrust, DcsWorkerCtx, InitLockRecord, LeaderRecord,
                MemberRecord, MemberRole, SwitchoverRequest,
            },
            store::{DcsStore, DcsStoreError, WatchEvent, WatchOp},
            worker::{apply_watch_update, DcsValue, DcsWatchUpdate},
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        state::{new_state_channel, MemberId, UnixMillis, Version, WorkerError, WorkerStatus},
    };

    use super::step_once;

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
    }

    impl DcsStore for RecordingStore {
        fn healthy(&self) -> bool {
            self.healthy
        }

        fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
            let mut guard = self
                .writes
                .lock()
                .map_err(|_| DcsStoreError::Io("writes lock poisoned".to_string()))?;
            guard.push((path.to_string(), value));
            Ok(())
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
                    pg_basebackup: "/usr/bin/pg_basebackup".into(),
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
                    requested_by: member_id.clone(),
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
            poll_interval: Duration::from_millis(5),
            pg_subscriber,
            publisher: dcs_publisher,
            store: Box::new(store),
            cache: sample_cache(sample_runtime_config()),
            last_published_pg_version: None,
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
    async fn step_once_writes_member_only_when_pg_version_changes() {
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
            poll_interval: Duration::from_millis(5),
            pg_subscriber,
            publisher: dcs_publisher,
            store: Box::new(store),
            cache: sample_cache(sample_runtime_config()),
            last_published_pg_version: None,
        };

        let first = step_once(&mut ctx).await;
        assert_eq!(first, Ok(()));

        let second = step_once(&mut ctx).await;
        assert_eq!(second, Ok(()));
        assert_eq!(store_probe.write_count(), 1);

        let _ = pg_publisher.publish(initial_pg, UnixMillis(2));
        let third = step_once(&mut ctx).await;
        assert_eq!(third, Ok(()));
        assert_eq!(store_probe.write_count(), 2);
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
            poll_interval: Duration::from_millis(5),
            pg_subscriber,
            publisher: dcs_publisher,
            store: Box::new(store),
            cache: sample_cache(sample_runtime_config()),
            last_published_pg_version: None,
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
            poll_interval: Duration::from_millis(5),
            pg_subscriber,
            publisher: dcs_publisher,
            store: Box::new(store),
            cache: sample_cache(sample_runtime_config()),
            last_published_pg_version: None,
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
