use std::time::Duration;
use std::{collections::BTreeMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    config::RuntimeConfig,
    logging::LogHandle,
    pginfo::state::{PgInfoState, Readiness, SqlStatus},
    state::{
        MemberId, StatePublisher, StateSubscriber, SystemIdentifier, TimelineId, UnixMillis,
        Version, WalLsn, WorkerStatus,
    },
};

use super::store::DcsStore;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DcsTrust {
    // Trust is derived per tick from store health plus fresh quorum visibility.
    FreshQuorum,
    NoFreshQuorum,
    NotTrusted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum MemberRole {
    Unknown,
    Primary,
    Replica,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MemberStateClass {
    EmptyOrMissingDataDir,
    InitializedInspectable,
    InvalidDataDir,
    ReplicaOnly,
    Promotable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PostgresRuntimeClass {
    RunningHealthy,
    OfflineInspectable,
    UnsafeLocalState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberRecord {
    pub(crate) member_id: MemberId,
    pub(crate) postgres_host: String,
    pub(crate) postgres_port: u16,
    pub(crate) api_url: Option<String>,
    pub(crate) role: MemberRole,
    pub(crate) sql: SqlStatus,
    pub(crate) readiness: Readiness,
    pub(crate) timeline: Option<TimelineId>,
    pub(crate) write_lsn: Option<WalLsn>,
    pub(crate) replay_lsn: Option<WalLsn>,
    pub(crate) system_identifier: Option<SystemIdentifier>,
    pub(crate) durable_end_lsn: Option<WalLsn>,
    pub(crate) state_class: Option<MemberStateClass>,
    pub(crate) postgres_runtime_class: Option<PostgresRuntimeClass>,
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
pub(crate) struct BootstrapLockRecord {
    pub(crate) holder: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ClusterInitializedRecord {
    pub(crate) initialized_by: MemberId,
    pub(crate) initialized_at: UnixMillis,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ClusterIdentityRecord {
    pub(crate) system_identifier: SystemIdentifier,
    pub(crate) bootstrapped_by: MemberId,
    pub(crate) bootstrapped_at: UnixMillis,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsView {
    // These are the only durable or leased cluster facts cached from DCS.
    pub(crate) members: BTreeMap<MemberId, MemberRecord>,
    pub(crate) leader: Option<LeaderRecord>,
    pub(crate) switchover: Option<SwitchoverRequest>,
    pub(crate) config: RuntimeConfig,
    pub(crate) cluster_initialized: Option<ClusterInitializedRecord>,
    pub(crate) cluster_identity: Option<ClusterIdentityRecord>,
    pub(crate) bootstrap_lock: Option<BootstrapLockRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsState {
    pub(crate) worker: WorkerStatus,
    pub(crate) trust: DcsTrust,
    pub(crate) cache: DcsView,
    pub(crate) last_refresh_at: Option<UnixMillis>,
}

pub(crate) struct DcsWorkerCtx {
    pub(crate) self_id: MemberId,
    pub(crate) scope: String,
    pub(crate) poll_interval: Duration,
    pub(crate) local_postgres_host: String,
    pub(crate) local_postgres_port: u16,
    pub(crate) local_api_url: Option<String>,
    pub(crate) local_data_dir: PathBuf,
    pub(crate) local_postgres_binary: PathBuf,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) publisher: StatePublisher<DcsState>,
    pub(crate) store: Box<dyn DcsStore>,
    pub(crate) log: LogHandle,
    pub(crate) cache: DcsView,
    pub(crate) last_published_pg_version: Option<Version>,
    pub(crate) last_emitted_store_healthy: Option<bool>,
    pub(crate) last_emitted_trust: Option<DcsTrust>,
}

pub(crate) struct BuildLocalMemberRecordInput<'a> {
    pub(crate) self_id: &'a MemberId,
    pub(crate) postgres_host: &'a str,
    pub(crate) postgres_port: u16,
    pub(crate) api_url: Option<&'a str>,
    pub(crate) pg_state: &'a PgInfoState,
    pub(crate) previous_record: Option<&'a MemberRecord>,
    pub(crate) now: UnixMillis,
    pub(crate) pg_version: Version,
}

pub(crate) fn evaluate_trust(
    etcd_healthy: bool,
    cache: &DcsView,
    self_id: &MemberId,
    now: UnixMillis,
) -> DcsTrust {
    if !etcd_healthy {
        return DcsTrust::NotTrusted;
    }

    let Some(self_member) = cache.members.get(self_id) else {
        return DcsTrust::NoFreshQuorum;
    };
    if !member_record_is_fresh(self_member, cache, now) {
        return DcsTrust::NoFreshQuorum;
    }

    if !has_fresh_quorum(cache, now) {
        return DcsTrust::NoFreshQuorum;
    }

    DcsTrust::FreshQuorum
}

pub(crate) fn member_record_is_fresh(
    record: &MemberRecord,
    cache: &DcsView,
    now: UnixMillis,
) -> bool {
    let max_age_ms = cache.config.ha.lease_ttl_ms;
    now.0.saturating_sub(record.updated_at.0) <= max_age_ms
}

fn fresh_member_count(cache: &DcsView, now: UnixMillis) -> usize {
    cache
        .members
        .values()
        .filter(|record| member_record_is_fresh(record, cache, now))
        .count()
}

fn has_fresh_quorum(cache: &DcsView, now: UnixMillis) -> bool {
    let fresh_members = fresh_member_count(cache, now);

    // The current runtime only knows the observed DCS member set. Until there is an explicit
    // configured membership source, multi-member quorum stays conservative: one fresh member is
    // only trusted in a single-member view, and any larger observed view requires at least two
    // fresh members.
    if cache.members.len() <= 1 {
        fresh_members == 1
    } else {
        fresh_members >= 2
    }
}

pub(crate) fn build_local_member_record(input: BuildLocalMemberRecordInput<'_>) -> MemberRecord {
    let BuildLocalMemberRecordInput {
        self_id,
        postgres_host,
        postgres_port,
        api_url,
        pg_state,
        previous_record,
        now,
        pg_version,
    } = input;
    match pg_state {
        PgInfoState::Unknown { common } => MemberRecord {
            member_id: self_id.clone(),
            postgres_host: postgres_host.to_string(),
            postgres_port,
            api_url: api_url.map(ToString::to_string),
            role: MemberRole::Unknown,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common
                .timeline
                .or(previous_record.and_then(|record| record.timeline)),
            write_lsn: previous_record.and_then(|record| record.write_lsn),
            replay_lsn: previous_record.and_then(|record| record.replay_lsn),
            system_identifier: previous_record.and_then(|record| record.system_identifier),
            durable_end_lsn: previous_record.and_then(|record| record.durable_end_lsn),
            state_class: previous_record.and_then(|record| record.state_class.clone()),
            postgres_runtime_class: previous_record
                .and_then(|record| record.postgres_runtime_class.clone()),
            updated_at: now,
            pg_version,
        },
        PgInfoState::Primary {
            common, wal_lsn, ..
        } => MemberRecord {
            member_id: self_id.clone(),
            postgres_host: postgres_host.to_string(),
            postgres_port,
            api_url: api_url.map(ToString::to_string),
            role: MemberRole::Primary,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common.timeline,
            write_lsn: Some(*wal_lsn),
            replay_lsn: None,
            system_identifier: previous_record.and_then(|record| record.system_identifier),
            durable_end_lsn: previous_record
                .and_then(|record| record.durable_end_lsn)
                .or(Some(*wal_lsn)),
            state_class: previous_record.and_then(|record| record.state_class.clone()),
            postgres_runtime_class: Some(PostgresRuntimeClass::RunningHealthy),
            updated_at: now,
            pg_version,
        },
        PgInfoState::Replica {
            common, replay_lsn, ..
        } => MemberRecord {
            member_id: self_id.clone(),
            postgres_host: postgres_host.to_string(),
            postgres_port,
            api_url: api_url.map(ToString::to_string),
            role: MemberRole::Replica,
            sql: common.sql.clone(),
            readiness: common.readiness.clone(),
            timeline: common.timeline,
            write_lsn: None,
            replay_lsn: Some(*replay_lsn),
            system_identifier: previous_record.and_then(|record| record.system_identifier),
            durable_end_lsn: previous_record
                .and_then(|record| record.durable_end_lsn)
                .or(Some(*replay_lsn)),
            state_class: previous_record.and_then(|record| record.state_class.clone()),
            postgres_runtime_class: Some(PostgresRuntimeClass::RunningHealthy),
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
        build_local_member_record, evaluate_trust, BuildLocalMemberRecordInput, DcsTrust, DcsView,
        LeaderRecord, MemberRecord, MemberRole,
    };
    use crate::{
        pginfo::state::{PgInfoState, Readiness, SqlStatus},
        state::{MemberId, TimelineId, UnixMillis, WalLsn},
    };

    fn sample_runtime_config() -> RuntimeConfig {
        crate::test_harness::runtime_config::sample_runtime_config()
    }

    fn sample_cache() -> DcsView {
        DcsView {
            members: BTreeMap::new(),
            leader: None,
            switchover: None,
            config: sample_runtime_config(),
            cluster_initialized: None,
            cluster_identity: None,
            bootstrap_lock: None,
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
            DcsTrust::NoFreshQuorum
        );

        cache.members.insert(
            self_id.clone(),
            MemberRecord {
                member_id: self_id.clone(),
                postgres_host: "127.0.0.1".to_string(),
                postgres_port: 5432,
                api_url: None,
                role: MemberRole::Unknown,
                sql: SqlStatus::Unknown,
                readiness: Readiness::Unknown,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
                system_identifier: None,
                durable_end_lsn: None,
                state_class: None,
                postgres_runtime_class: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );
        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(1)),
            DcsTrust::FreshQuorum
        );
        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(20_000)),
            DcsTrust::NoFreshQuorum
        );

        cache.leader = Some(LeaderRecord {
            member_id: MemberId("node-b".to_string()),
        });
        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(1)),
            DcsTrust::FreshQuorum
        );
    }

    #[test]
    fn evaluate_trust_keeps_healthy_majority_when_leader_metadata_is_missing_or_stale() {
        let self_id = MemberId("node-a".to_string());
        let mut cache = sample_cache();
        let fresh_time = UnixMillis(100);

        for member_id in ["node-a", "node-c"] {
            let member_id = MemberId(member_id.to_string());
            cache.members.insert(
                member_id.clone(),
                MemberRecord {
                    member_id,
                    postgres_host: "127.0.0.1".to_string(),
                    postgres_port: 5432,
                    api_url: None,
                    role: MemberRole::Replica,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: None,
                    write_lsn: None,
                    replay_lsn: None,
                    system_identifier: None,
                    durable_end_lsn: None,
                    state_class: None,
                    postgres_runtime_class: None,
                    updated_at: fresh_time,
                    pg_version: Version(1),
                },
            );
        }
        cache.members.insert(
            MemberId("node-b".to_string()),
            MemberRecord {
                member_id: MemberId("node-b".to_string()),
                postgres_host: "127.0.0.2".to_string(),
                postgres_port: 5432,
                api_url: None,
                role: MemberRole::Primary,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
                system_identifier: None,
                durable_end_lsn: None,
                state_class: None,
                postgres_runtime_class: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );
        cache.leader = Some(LeaderRecord {
            member_id: MemberId("node-b".to_string()),
        });

        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(5_000)),
            DcsTrust::FreshQuorum
        );

        cache.members.remove(&MemberId("node-b".to_string()));
        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(5_000)),
            DcsTrust::FreshQuorum
        );
    }

    #[test]
    fn evaluate_trust_stays_fail_safe_without_fresh_quorum() {
        let self_id = MemberId("node-a".to_string());
        let mut cache = sample_cache();

        cache.members.insert(
            self_id.clone(),
            MemberRecord {
                member_id: self_id.clone(),
                postgres_host: "127.0.0.1".to_string(),
                postgres_port: 5432,
                api_url: None,
                role: MemberRole::Replica,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
                system_identifier: None,
                durable_end_lsn: None,
                state_class: None,
                postgres_runtime_class: None,
                updated_at: UnixMillis(100),
                pg_version: Version(1),
            },
        );
        cache.members.insert(
            MemberId("node-b".to_string()),
            MemberRecord {
                member_id: MemberId("node-b".to_string()),
                postgres_host: "127.0.0.2".to_string(),
                postgres_port: 5432,
                api_url: None,
                role: MemberRole::Primary,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
                system_identifier: None,
                durable_end_lsn: None,
                state_class: None,
                postgres_runtime_class: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );
        cache.members.insert(
            MemberId("node-c".to_string()),
            MemberRecord {
                member_id: MemberId("node-c".to_string()),
                postgres_host: "127.0.0.3".to_string(),
                postgres_port: 5432,
                api_url: None,
                role: MemberRole::Replica,
                sql: SqlStatus::Healthy,
                readiness: Readiness::Ready,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
                system_identifier: None,
                durable_end_lsn: None,
                state_class: None,
                postgres_runtime_class: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );
        cache.leader = Some(LeaderRecord {
            member_id: MemberId("node-b".to_string()),
        });

        assert_eq!(
            evaluate_trust(true, &cache, &self_id, UnixMillis(20_000)),
            DcsTrust::NoFreshQuorum
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
        let unknown_record = build_local_member_record(BuildLocalMemberRecordInput {
            self_id: &self_id,
            postgres_host: "10.0.0.11",
            postgres_port: 5433,
            api_url: Some("http://node-a:8080"),
            pg_state: &unknown,
            previous_record: None,
            now: UnixMillis(10),
            pg_version: Version(11),
        });
        assert_eq!(unknown_record.postgres_host, "10.0.0.11".to_string());
        assert_eq!(unknown_record.postgres_port, 5433);
        assert_eq!(
            unknown_record.api_url.as_deref(),
            Some("http://node-a:8080")
        );
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
        let primary_record = build_local_member_record(BuildLocalMemberRecordInput {
            self_id: &self_id,
            postgres_host: "10.0.0.12",
            postgres_port: 5434,
            api_url: Some("http://node-a:8081"),
            pg_state: &primary,
            previous_record: None,
            now: UnixMillis(12),
            pg_version: Version(13),
        });
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
        let replica_record = build_local_member_record(BuildLocalMemberRecordInput {
            self_id: &self_id,
            postgres_host: "10.0.0.13",
            postgres_port: 5435,
            api_url: None,
            pg_state: &replica,
            previous_record: None,
            now: UnixMillis(14),
            pg_version: Version(15),
        });
        assert_eq!(replica_record.postgres_host, "10.0.0.13".to_string());
        assert_eq!(replica_record.postgres_port, 5435);
        assert_eq!(replica_record.api_url, None);
        assert_eq!(replica_record.role, MemberRole::Replica);
        assert_eq!(replica_record.write_lsn, None);
        assert_eq!(replica_record.replay_lsn, Some(WalLsn(22)));
    }

    #[test]
    fn build_local_member_record_retains_last_known_wal_evidence_for_unknown_pg_state() {
        let self_id = MemberId("node-a".to_string());
        let previous = MemberRecord {
            member_id: self_id.clone(),
            postgres_host: "10.0.0.10".to_string(),
            postgres_port: 5432,
            api_url: None,
            role: MemberRole::Primary,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
            timeline: Some(TimelineId(4)),
            write_lsn: Some(WalLsn(99)),
            replay_lsn: None,
            system_identifier: None,
            durable_end_lsn: None,
            state_class: None,
            postgres_runtime_class: None,
            updated_at: UnixMillis(8),
            pg_version: Version(9),
        };
        let unknown = PgInfoState::Unknown {
            common: common(SqlStatus::Unreachable, Readiness::NotReady),
        };

        let unknown_record = build_local_member_record(BuildLocalMemberRecordInput {
            self_id: &self_id,
            postgres_host: "10.0.0.11",
            postgres_port: 5433,
            api_url: None,
            pg_state: &unknown,
            previous_record: Some(&previous),
            now: UnixMillis(10),
            pg_version: Version(11),
        });

        assert_eq!(unknown_record.role, MemberRole::Unknown);
        assert_eq!(unknown_record.timeline, Some(TimelineId(4)));
        assert_eq!(unknown_record.write_lsn, Some(WalLsn(99)));
        assert_eq!(unknown_record.replay_lsn, None);
    }
}
