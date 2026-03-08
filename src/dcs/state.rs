use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    config::RuntimeConfig,
    logging::LogHandle,
    pginfo::state::{PgInfoState, Readiness, SqlStatus},
    state::{
        MemberId, StatePublisher, StateSubscriber, TimelineId, UnixMillis, Version, WalLsn,
        WorkerStatus,
    },
};

use super::store::DcsStore;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DcsTrust {
    FullQuorum,
    FailSafe,
    NotTrusted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum MemberRole {
    Unknown,
    Primary,
    Replica,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberRecord {
    pub(crate) member_id: MemberId,
    pub(crate) postgres_host: String,
    pub(crate) postgres_port: u16,
    pub(crate) role: MemberRole,
    pub(crate) sql: SqlStatus,
    pub(crate) readiness: Readiness,
    pub(crate) timeline: Option<TimelineId>,
    pub(crate) write_lsn: Option<WalLsn>,
    pub(crate) replay_lsn: Option<WalLsn>,
    pub(crate) updated_at: UnixMillis,
    pub(crate) pg_version: Version,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct LeaderRecord {
    pub(crate) member_id: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SwitchoverRequest {
    #[serde(default)]
    pub(crate) switchover_to: Option<MemberId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct InitLockRecord {
    pub(crate) holder: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsCache {
    pub(crate) members: BTreeMap<MemberId, MemberRecord>,
    pub(crate) leader: Option<LeaderRecord>,
    pub(crate) switchover: Option<SwitchoverRequest>,
    pub(crate) config: RuntimeConfig,
    pub(crate) init_lock: Option<InitLockRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsState {
    pub(crate) worker: WorkerStatus,
    pub(crate) trust: DcsTrust,
    pub(crate) cache: DcsCache,
    pub(crate) last_refresh_at: Option<UnixMillis>,
}

pub(crate) struct DcsWorkerCtx {
    pub(crate) self_id: MemberId,
    pub(crate) scope: String,
    pub(crate) poll_interval: Duration,
    pub(crate) local_postgres_host: String,
    pub(crate) local_postgres_port: u16,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) publisher: StatePublisher<DcsState>,
    pub(crate) store: Box<dyn DcsStore>,
    pub(crate) log: LogHandle,
    pub(crate) cache: DcsCache,
    pub(crate) last_published_pg_version: Option<Version>,
    pub(crate) last_emitted_store_healthy: Option<bool>,
    pub(crate) last_emitted_trust: Option<DcsTrust>,
}

pub(crate) fn evaluate_trust(
    etcd_healthy: bool,
    cache: &DcsCache,
    self_id: &MemberId,
    now: UnixMillis,
) -> DcsTrust {
    if !etcd_healthy {
        return DcsTrust::NotTrusted;
    }

    let Some(self_member) = cache.members.get(self_id) else {
        return DcsTrust::FailSafe;
    };
    if !member_record_is_fresh(self_member, cache, now) {
        return DcsTrust::FailSafe;
    }

    if let Some(leader) = &cache.leader {
        let Some(leader_member) = cache.members.get(&leader.member_id) else {
            return DcsTrust::FailSafe;
        };
        if !member_record_is_fresh(leader_member, cache, now) {
            return DcsTrust::FailSafe;
        }
    }

    if cache.members.len() > 1 && fresh_member_count(cache, now) < 2 {
        return DcsTrust::FailSafe;
    }

    DcsTrust::FullQuorum
}

fn member_record_is_fresh(record: &MemberRecord, cache: &DcsCache, now: UnixMillis) -> bool {
    let max_age_ms = cache.config.ha.lease_ttl_ms;
    now.0.saturating_sub(record.updated_at.0) <= max_age_ms
}

fn fresh_member_count(cache: &DcsCache, now: UnixMillis) -> usize {
    cache
        .members
        .values()
        .filter(|record| member_record_is_fresh(record, cache, now))
        .count()
}

pub(crate) fn build_local_member_record(
    self_id: &MemberId,
    postgres_host: &str,
    postgres_port: u16,
    pg_state: &PgInfoState,
    now: UnixMillis,
    pg_version: Version,
) -> MemberRecord {
    match pg_state {
        PgInfoState::Unknown { common } => MemberRecord {
            member_id: self_id.clone(),
            postgres_host: postgres_host.to_string(),
            postgres_port,
            role: MemberRole::Unknown,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common.timeline,
            write_lsn: None,
            replay_lsn: None,
            updated_at: now,
            pg_version,
        },
        PgInfoState::Primary {
            common, wal_lsn, ..
        } => MemberRecord {
            member_id: self_id.clone(),
            postgres_host: postgres_host.to_string(),
            postgres_port,
            role: MemberRole::Primary,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common.timeline,
            write_lsn: Some(*wal_lsn),
            replay_lsn: None,
            updated_at: now,
            pg_version,
        },
        PgInfoState::Replica {
            common, replay_lsn, ..
        } => MemberRecord {
            member_id: self_id.clone(),
            postgres_host: postgres_host.to_string(),
            postgres_port,
            role: MemberRole::Replica,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common.timeline,
            write_lsn: None,
            replay_lsn: Some(*replay_lsn),
            updated_at: now,
            pg_version,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        config::RuntimeConfig,
        pginfo::state::{PgConfig, PgInfoCommon, ReplicationSlotInfo},
        state::{Version, WorkerStatus},
    };

    use super::{
        build_local_member_record, evaluate_trust, DcsCache, DcsTrust, LeaderRecord, MemberRecord,
        MemberRole,
    };
    use crate::{
        pginfo::state::{PgInfoState, Readiness, SqlStatus},
        state::{MemberId, TimelineId, UnixMillis, WalLsn},
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
    fn evaluate_trust_covers_all_outcomes() {
        let self_id = MemberId("node-a".to_string());
        let mut cache = sample_cache();

        assert_eq!(
            evaluate_trust(false, &cache, &self_id, UnixMillis(1)),
            DcsTrust::NotTrusted
        );
        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(1)),
            DcsTrust::FailSafe
        );

        cache.members.insert(
            self_id.clone(),
            MemberRecord {
                member_id: self_id.clone(),
                postgres_host: "127.0.0.1".to_string(),
                postgres_port: 5432,
                role: MemberRole::Unknown,
                sql: SqlStatus::Unknown,
                readiness: Readiness::Unknown,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );
        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(1)),
            DcsTrust::FullQuorum
        );
        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(20_000)),
            DcsTrust::FailSafe
        );

        cache.leader = Some(LeaderRecord {
            member_id: MemberId("node-b".to_string()),
        });
        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(1)),
            DcsTrust::FailSafe
        );
    }

    fn common(sql: SqlStatus, readiness: Readiness) -> PgInfoCommon {
        PgInfoCommon {
            worker: WorkerStatus::Running,
            sql,
            readiness,
            timeline: Some(TimelineId(4)),
            pg_config: PgConfig {
                port: None,
                hot_standby: None,
                primary_conninfo: None,
                primary_slot_name: None,
                extra: BTreeMap::new(),
            },
            last_refresh_at: Some(UnixMillis(9)),
        }
    }

    #[test]
    fn build_local_member_record_maps_pg_variants() {
        let self_id = MemberId("node-a".to_string());
        let unknown = PgInfoState::Unknown {
            common: common(SqlStatus::Unknown, Readiness::Unknown),
        };
        let unknown_record = build_local_member_record(
            &self_id,
            "10.0.0.11",
            5433,
            &unknown,
            UnixMillis(10),
            Version(11),
        );
        assert_eq!(unknown_record.postgres_host, "10.0.0.11".to_string());
        assert_eq!(unknown_record.postgres_port, 5433);
        assert_eq!(unknown_record.role, MemberRole::Unknown);
        assert_eq!(unknown_record.write_lsn, None);
        assert_eq!(unknown_record.replay_lsn, None);

        let primary = PgInfoState::Primary {
            common: common(SqlStatus::Healthy, Readiness::Ready),
            wal_lsn: WalLsn(101),
            slots: vec![ReplicationSlotInfo {
                name: "slot-a".to_string(),
            }],
        };
        let primary_record = build_local_member_record(
            &self_id,
            "10.0.0.12",
            5434,
            &primary,
            UnixMillis(12),
            Version(13),
        );
        assert_eq!(primary_record.postgres_host, "10.0.0.12".to_string());
        assert_eq!(primary_record.postgres_port, 5434);
        assert_eq!(primary_record.role, MemberRole::Primary);
        assert_eq!(primary_record.write_lsn, Some(WalLsn(101)));
        assert_eq!(primary_record.replay_lsn, None);

        let replica = PgInfoState::Replica {
            common: common(SqlStatus::Healthy, Readiness::Ready),
            replay_lsn: WalLsn(22),
            follow_lsn: Some(WalLsn(23)),
            upstream: None,
        };
        let replica_record = build_local_member_record(
            &self_id,
            "10.0.0.13",
            5435,
            &replica,
            UnixMillis(14),
            Version(15),
        );
        assert_eq!(replica_record.postgres_host, "10.0.0.13".to_string());
        assert_eq!(replica_record.postgres_port, 5435);
        assert_eq!(replica_record.role, MemberRole::Replica);
        assert_eq!(replica_record.write_lsn, None);
        assert_eq!(replica_record.replay_lsn, Some(WalLsn(22)));
    }
}
