use thiserror::Error;

use super::{
    keys::{key_from_path, DcsKey, DcsKeyParseError},
    state::{DcsCache, InitLockRecord, LeaderRecord, MemberRecord, SwitchoverRequest},
    worker::{apply_watch_update, DcsWatchUpdate},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WatchOp {
    Put,
    Delete,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WatchEvent {
    pub op: WatchOp,
    pub path: String,
    pub value: Option<String>,
    pub revision: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RefreshResult {
    pub(crate) applied: usize,
    pub(crate) had_errors: bool,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum DcsStoreError {
    #[error("watch value missing for put event at `{0}`")]
    MissingValue(String),
    #[error("invalid key path: {0}")]
    InvalidKey(#[from] DcsKeyParseError),
    #[error("decode failed for key `{key}`: {message}")]
    Decode { key: String, message: String },
    #[error("store I/O error: {0}")]
    Io(String),
}

pub trait DcsStore: Send {
    fn healthy(&self) -> bool;
    fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError>;
    fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError>;
    fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError>;
}

pub(crate) fn write_local_member(
    store: &mut dyn DcsStore,
    scope: &str,
    member: &MemberRecord,
) -> Result<(), DcsStoreError> {
    let path = format!("/{}/member/{}", scope.trim_matches('/'), member.member_id.0);
    let encoded = serde_json::to_string(member).map_err(|err| DcsStoreError::Decode {
        key: path.clone(),
        message: err.to_string(),
    })?;
    store.write_path(&path, encoded)?;
    Ok(())
}

pub(crate) fn refresh_from_etcd_watch(
    scope: &str,
    cache: &mut DcsCache,
    events: Vec<WatchEvent>,
) -> Result<RefreshResult, DcsStoreError> {
    let mut applied = 0usize;
    let mut had_errors = false;

    for event in events {
        let key = match key_from_path(scope, &event.path) {
            Ok(parsed) => parsed,
            Err(err) => match err {
                DcsKeyParseError::UnknownKey(_) => {
                    had_errors = true;
                    continue;
                }
                other => return Err(DcsStoreError::InvalidKey(other)),
            },
        };

        let update = match event.op {
            WatchOp::Delete => DcsWatchUpdate::Delete { key },
            WatchOp::Put => {
                let raw_value = match event.value {
                    Some(value) => value,
                    None => return Err(DcsStoreError::MissingValue(event.path)),
                };
                let value = decode_watch_value(&key, &raw_value, &event.path)?;
                DcsWatchUpdate::Put {
                    key,
                    value: Box::new(value),
                }
            }
        };

        apply_watch_update(cache, update);
        applied = applied.saturating_add(1);
    }

    Ok(RefreshResult {
        applied,
        had_errors,
    })
}

fn decode_watch_value(
    key: &DcsKey,
    raw: &str,
    path: &str,
) -> Result<super::worker::DcsValue, DcsStoreError> {
    match key {
        DcsKey::Member(_) => serde_json::from_str::<MemberRecord>(raw)
            .map(super::worker::DcsValue::Member)
            .map_err(|err| DcsStoreError::Decode {
                key: path.to_string(),
                message: err.to_string(),
            }),
        DcsKey::Leader => serde_json::from_str::<LeaderRecord>(raw)
            .map(super::worker::DcsValue::Leader)
            .map_err(|err| DcsStoreError::Decode {
                key: path.to_string(),
                message: err.to_string(),
            }),
        DcsKey::Switchover => serde_json::from_str::<SwitchoverRequest>(raw)
            .map(super::worker::DcsValue::Switchover)
            .map_err(|err| DcsStoreError::Decode {
                key: path.to_string(),
                message: err.to_string(),
            }),
        DcsKey::Config => serde_json::from_str::<crate::config::RuntimeConfig>(raw)
            .map(|cfg| super::worker::DcsValue::Config(Box::new(cfg)))
            .map_err(|err| DcsStoreError::Decode {
                key: path.to_string(),
                message: err.to_string(),
            }),
        DcsKey::InitLock => serde_json::from_str::<InitLockRecord>(raw)
            .map(super::worker::DcsValue::InitLock)
            .map_err(|err| DcsStoreError::Decode {
                key: path.to_string(),
                message: err.to_string(),
            }),
    }
}

#[cfg(test)]
use std::collections::VecDeque;

#[cfg(test)]
#[derive(Default)]
pub(crate) struct TestDcsStore {
    healthy: bool,
    events: VecDeque<WatchEvent>,
    writes: Vec<(String, String)>,
    deletes: Vec<String>,
}

#[cfg(test)]
impl TestDcsStore {
    pub(crate) fn new(healthy: bool) -> Self {
        Self {
            healthy,
            events: VecDeque::new(),
            writes: Vec::new(),
            deletes: Vec::new(),
        }
    }

    pub(crate) fn push_event(&mut self, event: WatchEvent) {
        self.events.push_back(event);
    }

    pub(crate) fn writes(&self) -> &[(String, String)] {
        &self.writes
    }

    pub(crate) fn deletes(&self) -> &[String] {
        &self.deletes
    }
}

#[cfg(test)]
impl DcsStore for TestDcsStore {
    fn healthy(&self) -> bool {
        self.healthy
    }

    fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError> {
        self.writes.push((path.to_string(), value));
        Ok(())
    }

    fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError> {
        self.deletes.push(path.to_string());
        Ok(())
    }

    fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
        Ok(self.events.drain(..).collect())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        config::{
            schema::{
                ApiConfig, ClusterConfig, DcsConfig, DebugConfig, HaConfig, PostgresConfig,
                SecurityConfig,
            },
            BinaryPaths, ProcessConfig, RuntimeConfig,
        },
        dcs::{
            state::{DcsCache, MemberRecord, MemberRole},
            worker::DcsValue,
        },
        pginfo::state::{Readiness, SqlStatus},
        state::{MemberId, UnixMillis, Version},
    };

    use super::{
        refresh_from_etcd_watch, write_local_member, DcsStore, DcsStoreError, TestDcsStore,
        WatchEvent, WatchOp,
    };

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

    fn sample_cache() -> DcsCache {
        DcsCache {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: sample_runtime_config(),
            init_lock: None,
        }
    }

    #[test]
    fn write_local_member_writes_only_member_path() {
        let mut store = TestDcsStore::new(true);
        let member = MemberRecord {
            member_id: MemberId("node-a".to_string()),
            role: MemberRole::Primary,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
            timeline: None,
            write_lsn: None,
            replay_lsn: None,
            updated_at: UnixMillis(10),
            pg_version: Version(7),
        };
        let wrote = write_local_member(&mut store, "scope-a", &member);
        assert_eq!(wrote, Ok(()));
        assert_eq!(store.writes().len(), 1);
        assert_eq!(store.writes()[0].0, "/scope-a/member/node-a");
        assert!(store.writes()[0].1.contains("\"member_id\""));
    }

    #[test]
    fn refresh_applies_member_put_and_delete() {
        let mut cache = sample_cache();
        let mut store = TestDcsStore::new(true);
        let value = serde_json::to_string(&MemberRecord {
            member_id: MemberId("node-a".to_string()),
            role: MemberRole::Replica,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
            timeline: None,
            write_lsn: None,
            replay_lsn: None,
            updated_at: UnixMillis(10),
            pg_version: Version(1),
        });
        let encoded = match value {
            Ok(v) => v,
            Err(err) => panic!("member encoding failed: {err}"),
        };
        store.push_event(WatchEvent {
            op: WatchOp::Put,
            path: "/scope-a/member/node-a".to_string(),
            value: Some(encoded),
            revision: 1,
        });
        store.push_event(WatchEvent {
            op: WatchOp::Delete,
            path: "/scope-a/member/node-a".to_string(),
            value: None,
            revision: 2,
        });

        let events = match store.drain_watch_events() {
            Ok(v) => v,
            Err(err) => panic!("drain watch events failed: {err}"),
        };
        let refreshed = refresh_from_etcd_watch("scope-a", &mut cache, events);
        assert!(refreshed.is_ok());
        assert!(cache.members.is_empty());
    }

    #[test]
    fn refresh_reports_decode_error() {
        let mut cache = sample_cache();
        let result = refresh_from_etcd_watch(
            "scope-a",
            &mut cache,
            vec![WatchEvent {
                op: WatchOp::Put,
                path: "/scope-a/member/node-a".to_string(),
                value: Some("{\"bad\":1}".to_string()),
                revision: 1,
            }],
        );
        match result {
            Err(DcsStoreError::Decode { .. }) => {}
            other => panic!("expected decode error, got: {other:?}"),
        }
    }

    #[test]
    fn dcs_value_type_is_exercised_to_keep_contracts_live() {
        let _value = DcsValue::Leader(crate::dcs::state::LeaderRecord {
            member_id: MemberId("node-a".to_string()),
        });
    }
}
