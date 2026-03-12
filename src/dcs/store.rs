use thiserror::Error;

use super::{
    keys::{key_from_path, DcsKey, DcsKeyParseError},
    state::{DcsCache, InitLockRecord, LeaderRecord, MemberRecord, SwitchoverRequest},
    worker::{apply_watch_update, DcsWatchUpdate},
};
use crate::state::MemberId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WatchOp {
    Put,
    Delete,
    /// Indicates that the watch consumer should treat the following snapshot as authoritative
    /// and reset any previously cached DCS state for this scope.
    ///
    /// This is synthesized by the etcd store during reconnect/resnapshot and does not come from
    /// etcd itself.
    Reset,
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
    #[error("path already exists: {0}")]
    AlreadyExists(String),
    #[error("leader lease support is not configured for `{0}`")]
    LeaderLeaseNotConfigured(String),
    #[error("leader lease is not owned locally for `{0}`")]
    LeaderLeaseNotOwned(String),
    #[error("store I/O error: {0}")]
    Io(String),
}

pub trait DcsStore: Send {
    fn healthy(&self) -> bool;
    fn read_path(&mut self, path: &str) -> Result<Option<String>, DcsStoreError>;
    fn snapshot_prefix(&mut self, path_prefix: &str) -> Result<Vec<WatchEvent>, DcsStoreError>;
    fn write_path(&mut self, path: &str, value: String) -> Result<(), DcsStoreError>;
    fn put_path_if_absent(&mut self, path: &str, value: String) -> Result<bool, DcsStoreError>;
    fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError>;
    fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError>;
}

pub(crate) trait DcsLeaderStore: Send {
    fn acquire_leader_lease(
        &mut self,
        scope: &str,
        member_id: &MemberId,
    ) -> Result<(), DcsStoreError>;
    fn release_leader_lease(
        &mut self,
        scope: &str,
        member_id: &MemberId,
    ) -> Result<(), DcsStoreError>;
    fn clear_switchover(&mut self, scope: &str) -> Result<(), DcsStoreError>;
}

pub(crate) fn leader_path(scope: &str) -> String {
    format!("/{}/leader", scope.trim_matches('/'))
}

pub(crate) fn encode_leader_record(
    scope: &str,
    member_id: &MemberId,
    generation: u64,
) -> Result<(String, String), DcsStoreError> {
    let path = leader_path(scope);
    let encoded = serde_json::to_string(&LeaderRecord {
        member_id: member_id.clone(),
        generation,
    })
    .map_err(|err| DcsStoreError::Decode {
        key: path.clone(),
        message: err.to_string(),
    })?;
    Ok((path, encoded))
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
        if event.op == WatchOp::Reset {
            cache.members.clear();
            cache.leader = None;
            cache.switchover = None;
            cache.init_lock = None;
            applied = applied.saturating_add(1);
            continue;
        }

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
            WatchOp::Reset => {
                // Handled above, before key parsing.
                continue;
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
    kv: std::collections::BTreeMap<String, String>,
    writes: Vec<(String, String)>,
    deletes: Vec<String>,
}

#[cfg(test)]
impl TestDcsStore {
    pub(crate) fn new(healthy: bool) -> Self {
        Self {
            healthy,
            events: VecDeque::new(),
            kv: std::collections::BTreeMap::new(),
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

    fn read_path(&mut self, path: &str) -> Result<Option<String>, DcsStoreError> {
        Ok(self.kv.get(path).cloned())
    }

    fn snapshot_prefix(&mut self, path_prefix: &str) -> Result<Vec<WatchEvent>, DcsStoreError> {
        let mut events = vec![WatchEvent {
            op: WatchOp::Reset,
            path: path_prefix.to_string(),
            value: None,
            revision: 0,
        }];
        events.extend(
            self.kv
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
        self.kv.insert(path.to_string(), value.clone());
        self.writes.push((path.to_string(), value));
        Ok(())
    }

    fn put_path_if_absent(&mut self, path: &str, value: String) -> Result<bool, DcsStoreError> {
        if self.kv.contains_key(path) {
            return Ok(false);
        }
        self.kv.insert(path.to_string(), value.clone());
        self.writes.push((path.to_string(), value));
        Ok(true)
    }

    fn delete_path(&mut self, path: &str) -> Result<(), DcsStoreError> {
        self.kv.remove(path);
        self.deletes.push(path.to_string());
        Ok(())
    }

    fn drain_watch_events(&mut self) -> Result<Vec<WatchEvent>, DcsStoreError> {
        Ok(self.events.drain(..).collect())
    }
}

#[cfg(test)]
impl DcsLeaderStore for TestDcsStore {
    fn acquire_leader_lease(
        &mut self,
        scope: &str,
        member_id: &MemberId,
    ) -> Result<(), DcsStoreError> {
        let generation = self.writes.len() as u64 + 1;
        let (path, encoded) = encode_leader_record(scope, member_id, generation)?;
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
        self.delete_path(&leader_path(scope))
    }

    fn clear_switchover(&mut self, scope: &str) -> Result<(), DcsStoreError> {
        self.delete_path(format!("/{}/switchover", scope.trim_matches('/')).as_str())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        config::RuntimeConfig,
        dcs::{
            state::{DcsCache, MemberRecord, MemberRole},
            worker::DcsValue,
        },
        pginfo::state::{Readiness, SqlStatus},
        state::{MemberId, UnixMillis, Version},
    };

    use super::{
        refresh_from_etcd_watch, write_local_member, DcsLeaderStore, DcsStore, DcsStoreError,
        RefreshResult, TestDcsStore, WatchEvent, WatchOp,
    };

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::sample_runtime_config()
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
            postgres_host: "10.0.0.10".to_string(),
            postgres_port: 5432,
            api_url: None,
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
    fn refresh_applies_member_put_and_delete() -> Result<(), Box<dyn std::error::Error>> {
        let mut cache = sample_cache();
        let mut store = TestDcsStore::new(true);
        let encoded = serde_json::to_string(&MemberRecord {
            member_id: MemberId("node-a".to_string()),
            postgres_host: "10.0.0.11".to_string(),
            postgres_port: 5433,
            api_url: None,
            role: MemberRole::Replica,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
            timeline: None,
            write_lsn: None,
            replay_lsn: None,
            updated_at: UnixMillis(10),
            pg_version: Version(1),
        })?;
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

        let events = store.drain_watch_events()?;
        let refreshed = refresh_from_etcd_watch("scope-a", &mut cache, events);
        assert!(refreshed.is_ok());
        assert!(cache.members.is_empty());
        Ok(())
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
        assert!(matches!(result, Err(DcsStoreError::Decode { .. })));
    }

    #[test]
    fn refresh_sets_had_errors_for_unknown_keys_and_applies_known_updates() {
        let mut cache = sample_cache();
        let result = refresh_from_etcd_watch(
            "scope-a",
            &mut cache,
            vec![
                WatchEvent {
                    op: WatchOp::Put,
                    path: "/scope-a/not-a-real-key".to_string(),
                    value: Some("{\"ignored\":true}".to_string()),
                    revision: 1,
                },
                WatchEvent {
                    op: WatchOp::Put,
                    path: "/scope-a/leader".to_string(),
                    value: Some("{\"member_id\":\"node-a\",\"generation\":1}".to_string()),
                    revision: 2,
                },
            ],
        );

        assert!(matches!(
            result,
            Ok(RefreshResult {
                had_errors: true,
                applied: 1
            })
        ));
        assert_eq!(
            cache.leader,
            Some(crate::dcs::state::LeaderRecord {
                member_id: MemberId("node-a".to_string()),
                generation: 1,
            })
        );
    }

    #[test]
    fn refresh_reset_clears_cached_records_but_preserves_config(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cache = sample_cache();
        let preserved_config = cache.config.clone();

        cache.members.insert(
            MemberId("node-stale".to_string()),
            MemberRecord {
                member_id: MemberId("node-stale".to_string()),
                postgres_host: "10.0.0.12".to_string(),
                postgres_port: 5434,
                api_url: None,
                role: MemberRole::Replica,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
                updated_at: UnixMillis(10),
                pg_version: Version(1),
            },
        );
        cache.leader = Some(crate::dcs::state::LeaderRecord {
            member_id: MemberId("node-stale".to_string()),
            generation: 1,
        });
        cache.switchover = Some(crate::dcs::state::SwitchoverRequest {
            switchover_to: None,
        });
        cache.init_lock = Some(crate::dcs::state::InitLockRecord {
            holder: MemberId("node-stale".to_string()),
        });

        let result = refresh_from_etcd_watch(
            "scope-a",
            &mut cache,
            vec![WatchEvent {
                op: WatchOp::Reset,
                path: "/scope-a".to_string(),
                value: None,
                revision: 42,
            }],
        )?;

        assert_eq!(
            result,
            RefreshResult {
                applied: 1,
                had_errors: false
            }
        );
        assert!(cache.members.is_empty());
        assert!(cache.leader.is_none());
        assert!(cache.switchover.is_none());
        assert!(cache.init_lock.is_none());
        assert_eq!(cache.config, preserved_config);

        Ok(())
    }

    #[test]
    fn refresh_put_then_reset_then_put_keeps_only_post_reset_state(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut cache = sample_cache();

        let stale_json = serde_json::to_string(&crate::dcs::state::LeaderRecord {
            member_id: MemberId("node-stale".to_string()),
            generation: 1,
        })?;
        let fresh_json = serde_json::to_string(&crate::dcs::state::LeaderRecord {
            member_id: MemberId("node-fresh".to_string()),
            generation: 2,
        })?;

        let result = refresh_from_etcd_watch(
            "scope-a",
            &mut cache,
            vec![
                WatchEvent {
                    op: WatchOp::Put,
                    path: "/scope-a/leader".to_string(),
                    value: Some(stale_json),
                    revision: 1,
                },
                WatchEvent {
                    op: WatchOp::Reset,
                    path: "/scope-a".to_string(),
                    value: None,
                    revision: 2,
                },
                WatchEvent {
                    op: WatchOp::Put,
                    path: "/scope-a/leader".to_string(),
                    value: Some(fresh_json),
                    revision: 3,
                },
            ],
        )?;

        assert_eq!(
            result,
            RefreshResult {
                applied: 3,
                had_errors: false
            }
        );
        assert_eq!(
            cache.leader,
            Some(crate::dcs::state::LeaderRecord {
                member_id: MemberId("node-fresh".to_string()),
                generation: 2,
            })
        );

        Ok(())
    }

    #[test]
    fn dcs_value_type_is_exercised_to_keep_contracts_live() {
        let _value = DcsValue::Leader(crate::dcs::state::LeaderRecord {
            member_id: MemberId("node-a".to_string()),
            generation: 1,
        });
    }

    #[test]
    fn acquire_leader_lease_writes_leader_path_and_payload() {
        let mut store = TestDcsStore::new(true);
        let result = DcsLeaderStore::acquire_leader_lease(
            &mut store,
            "scope-a",
            &MemberId("node-a".to_string()),
        );
        assert_eq!(result, Ok(()));
        assert_eq!(store.writes().len(), 1);
        assert_eq!(store.writes()[0].0, "/scope-a/leader");
        assert!(store.writes()[0].1.contains("\"member_id\":\"node-a\""));
    }

    #[test]
    fn acquire_leader_lease_rejects_existing_leader() {
        let mut store = TestDcsStore::new(true);
        let first = DcsLeaderStore::acquire_leader_lease(
            &mut store,
            "scope-a",
            &MemberId("node-a".to_string()),
        );
        let second = DcsLeaderStore::acquire_leader_lease(
            &mut store,
            "scope-a",
            &MemberId("node-b".to_string()),
        );

        assert_eq!(first, Ok(()));
        assert_eq!(
            second,
            Err(DcsStoreError::AlreadyExists("/scope-a/leader".to_string()))
        );
        assert_eq!(store.writes().len(), 1);
        assert!(store.writes()[0].1.contains("\"member_id\":\"node-a\""));
        assert_eq!(
            store.read_path("/scope-a/leader"),
            Ok(Some(
                "{\"member_id\":\"node-a\",\"generation\":1}".to_string()
            ))
        );
    }

    #[test]
    fn release_leader_lease_deletes_leader_key() {
        let mut store = TestDcsStore::new(true);
        let result = DcsLeaderStore::release_leader_lease(
            &mut store,
            "scope-a",
            &MemberId("node-a".to_string()),
        );
        assert_eq!(result, Ok(()));
        assert_eq!(store.deletes(), &["/scope-a/leader".to_string()]);
    }

    #[test]
    fn clear_switchover_deletes_switchover_key() {
        let mut store = TestDcsStore::new(true);
        let result = DcsLeaderStore::clear_switchover(&mut store, "scope-a");
        assert_eq!(result, Ok(()));
        assert_eq!(store.deletes(), &["/scope-a/switchover".to_string()]);
    }
}
