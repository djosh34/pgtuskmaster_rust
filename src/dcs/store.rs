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

pub(crate) trait DcsHaWriter: Send {
    fn write_leader_lease(
        &mut self,
        scope: &str,
        member_id: &MemberId,
    ) -> Result<(), DcsStoreError>;
    fn delete_leader(&mut self, scope: &str) -> Result<(), DcsStoreError>;
    fn clear_switchover(&mut self, scope: &str) -> Result<(), DcsStoreError>;
}

impl<T> DcsHaWriter for T
where
    T: DcsStore + ?Sized,
{
    fn write_leader_lease(
        &mut self,
        scope: &str,
        member_id: &MemberId,
    ) -> Result<(), DcsStoreError> {
        let path = leader_path(scope);
        let encoded = serde_json::to_string(&LeaderRecord {
            member_id: member_id.clone(),
        })
        .map_err(|err| DcsStoreError::Decode {
            key: path.clone(),
            message: err.to_string(),
        })?;
        self.write_path(&path, encoded)
    }

    fn delete_leader(&mut self, scope: &str) -> Result<(), DcsStoreError> {
        self.delete_path(&leader_path(scope))
    }

    fn clear_switchover(&mut self, scope: &str) -> Result<(), DcsStoreError> {
        self.delete_path(&switchover_path(scope))
    }
}

fn leader_path(scope: &str) -> String {
    format!("/{}/leader", scope.trim_matches('/'))
}

fn switchover_path(scope: &str) -> String {
    format!("/{}/switchover", scope.trim_matches('/'))
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
            schema::{ClusterConfig, DebugConfig, HaConfig, PostgresConfig},
            ApiAuthConfig, ApiConfig, ApiSecurityConfig, ApiTlsMode, BinaryPaths, DcsConfig,
            InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig, PgHbaConfig, PgIdentConfig,
            PostgresConnIdentityConfig, PostgresLoggingConfig, PostgresRoleConfig,
            PostgresRolesConfig, ProcessConfig, RoleAuthConfig, RuntimeConfig, StderrSinkConfig,
            TlsServerConfig,
        },
        dcs::{
            state::{DcsCache, MemberRecord, MemberRole},
            worker::DcsValue,
        },
        pginfo::state::{Readiness, SqlStatus},
        state::{MemberId, UnixMillis, Version},
    };
    use crate::pginfo::conninfo::PgSslMode;

    use super::{
        refresh_from_etcd_watch, write_local_member, DcsHaWriter, DcsStore, DcsStoreError,
        RefreshResult, TestDcsStore, WatchEvent, WatchOp,
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
                binaries: BinaryPaths {
                    postgres: "/usr/bin/postgres".into(),
                    pg_ctl: "/usr/bin/pg_ctl".into(),
                    pg_rewind: "/usr/bin/pg_rewind".into(),
                    initdb: "/usr/bin/initdb".into(),
                    pg_basebackup: "/usr/bin/pg_basebackup".into(),
                    psql: "/usr/bin/psql".into(),
                },
            },
            logging: LoggingConfig {
                level: LogLevel::Info,
                capture_subprocess_output: true,
                postgres: PostgresLoggingConfig {
                    enabled: true,
                    pg_ctl_log_file: None,
                    log_dir: None,
                    archive_command_log_file: None,
                    poll_interval_ms: 200,
                    cleanup: LogCleanupConfig {
                        enabled: true,
                        max_files: 10,
                        max_age_seconds: 60,
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
    fn refresh_applies_member_put_and_delete() -> Result<(), Box<dyn std::error::Error>> {
        let mut cache = sample_cache();
        let mut store = TestDcsStore::new(true);
        let encoded = serde_json::to_string(&MemberRecord {
            member_id: MemberId("node-a".to_string()),
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
                    value: Some("{\"member_id\":\"node-a\"}".to_string()),
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
                member_id: MemberId("node-a".to_string())
            })
        );
    }

    #[test]
    fn dcs_value_type_is_exercised_to_keep_contracts_live() {
        let _value = DcsValue::Leader(crate::dcs::state::LeaderRecord {
            member_id: MemberId("node-a".to_string()),
        });
    }

    #[test]
    fn write_leader_lease_writes_leader_path_and_payload() {
        let mut store = TestDcsStore::new(true);
        let result =
            DcsHaWriter::write_leader_lease(&mut store, "scope-a", &MemberId("node-a".to_string()));
        assert_eq!(result, Ok(()));
        assert_eq!(store.writes().len(), 1);
        assert_eq!(store.writes()[0].0, "/scope-a/leader");
        assert!(store.writes()[0].1.contains("\"member_id\":\"node-a\""));
    }

    #[test]
    fn delete_leader_deletes_leader_key() {
        let mut store = TestDcsStore::new(true);
        let result = DcsHaWriter::delete_leader(&mut store, "scope-a");
        assert_eq!(result, Ok(()));
        assert_eq!(store.deletes(), &["/scope-a/leader".to_string()]);
    }

    #[test]
    fn clear_switchover_deletes_switchover_key() {
        let mut store = TestDcsStore::new(true);
        let result = DcsHaWriter::clear_switchover(&mut store, "scope-a");
        assert_eq!(result, Ok(()));
        assert_eq!(store.deletes(), &["/scope-a/switchover".to_string()]);
    }
}
