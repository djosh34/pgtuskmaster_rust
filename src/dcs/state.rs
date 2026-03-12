use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    config::RuntimeConfig,
    logging::LogHandle,
    pginfo::state::{PgInfoState, Readiness},
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
    Degraded,
    NotTrusted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberLease {
    pub(crate) owner: MemberId,
    pub(crate) ttl_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberSlot {
    pub(crate) lease: MemberLease,
    pub(crate) routing: MemberRouting,
    pub(crate) postgres: MemberPostgresView,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberRouting {
    pub(crate) postgres: MemberEndpoint,
    pub(crate) api: Option<MemberApiEndpoint>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberEndpoint {
    pub(crate) host: String,
    pub(crate) port: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberApiEndpoint {
    pub(crate) url: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct WalVector {
    pub(crate) timeline: Option<TimelineId>,
    pub(crate) lsn: WalLsn,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct UnknownPostgresObservation {
    pub(crate) readiness: Readiness,
    pub(crate) timeline: Option<TimelineId>,
    pub(crate) pg_version: Version,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PrimaryObservation {
    pub(crate) readiness: Readiness,
    pub(crate) committed_wal: WalVector,
    pub(crate) pg_version: Version,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ReplicaObservation {
    pub(crate) readiness: Readiness,
    pub(crate) upstream: Option<MemberId>,
    pub(crate) replay_wal: Option<WalVector>,
    pub(crate) follow_wal: Option<WalVector>,
    pub(crate) pg_version: Version,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum MemberPostgresView {
    Unknown(UnknownPostgresObservation),
    Primary(PrimaryObservation),
    Replica(ReplicaObservation),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct LeaderLeaseRecord {
    pub(crate) holder: MemberId,
    pub(crate) generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SwitchoverIntentRecord {
    pub(crate) target: SwitchoverTargetRecord,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SwitchoverTargetRecord {
    AnyHealthyReplica,
    Specific(MemberId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct InitLockRecord {
    pub(crate) holder: MemberId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsCache {
    pub(crate) member_slots: BTreeMap<MemberId, MemberSlot>,
    pub(crate) leader_lease: Option<LeaderLeaseRecord>,
    pub(crate) switchover_intent: Option<SwitchoverIntentRecord>,
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
    pub(crate) local_api_url: Option<String>,
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
) -> DcsTrust {
    if !etcd_healthy {
        return DcsTrust::NotTrusted;
    }

    if !cache.member_slots.contains_key(self_id) {
        return DcsTrust::Degraded;
    }

    if !has_member_quorum(cache) {
        return DcsTrust::Degraded;
    }

    DcsTrust::FullQuorum
}

fn has_member_quorum(cache: &DcsCache) -> bool {
    if cache.member_slots.len() <= 1 {
        cache.member_slots.len() == 1
    } else {
        cache.member_slots.len() >= 2
    }
}

pub(crate) fn build_local_member_slot(
    self_id: &MemberId,
    postgres_host: &str,
    postgres_port: u16,
    api_url: Option<&str>,
    lease_ttl_ms: u64,
    pg_state: &PgInfoState,
    pg_version: Version,
) -> MemberSlot {
    let lease = MemberLease {
        owner: self_id.clone(),
        ttl_ms: lease_ttl_ms,
    };
    let routing = MemberRouting {
        postgres: MemberEndpoint {
            host: postgres_host.to_string(),
            port: postgres_port,
        },
        api: api_url.map(|url| MemberApiEndpoint {
            url: url.to_string(),
        }),
    };

    match pg_state {
        PgInfoState::Unknown { common } => MemberSlot {
            lease,
            routing,
            postgres: MemberPostgresView::Unknown(UnknownPostgresObservation {
                readiness: common.readiness.clone(),
                timeline: common.timeline,
                pg_version,
            }),
        },
        PgInfoState::Primary {
            common, wal_lsn, ..
        } => MemberSlot {
            lease,
            routing,
            postgres: MemberPostgresView::Primary(PrimaryObservation {
                readiness: common.readiness.clone(),
                committed_wal: WalVector {
                    timeline: common.timeline,
                    lsn: *wal_lsn,
                },
                pg_version,
            }),
        },
        PgInfoState::Replica {
            common,
            replay_lsn,
            follow_lsn,
            upstream,
        } => MemberSlot {
            lease,
            routing,
            postgres: MemberPostgresView::Replica(ReplicaObservation {
                readiness: common.readiness.clone(),
                upstream: upstream.as_ref().map(|value| value.member_id.clone()),
                replay_wal: Some(WalVector {
                    timeline: common.timeline,
                    lsn: *replay_lsn,
                }),
                follow_wal: follow_lsn.map(|lsn| WalVector {
                    timeline: common.timeline,
                    lsn,
                }),
                pg_version,
            }),
        },
    }
}
