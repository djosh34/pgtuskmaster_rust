use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    logging::LogHandle,
    pginfo::state::{PgInfoState, Readiness},
    state::{
        MemberId, StatePublisher, StateSubscriber, SystemIdentifier, TimelineId, UnixMillis,
        WalLsn, WorkerStatus,
    },
};

use super::{command::DcsCommandInbox, store::DcsStore};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DcsTrust {
    FullQuorum,
    Degraded,
    NotTrusted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DcsMemberLeaseView {
    pub ttl_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DcsMemberView {
    pub member_id: MemberId,
    pub lease: DcsMemberLeaseView,
    pub routing: DcsMemberRoutingView,
    pub postgres: DcsMemberPostgresView,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DcsMemberRoutingView {
    pub postgres: DcsMemberEndpointView,
    pub api: Option<DcsMemberApiView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DcsMemberEndpointView {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DcsMemberApiView {
    pub url: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalVector {
    pub timeline: Option<TimelineId>,
    pub lsn: WalLsn,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DcsUnknownPostgresView {
    pub readiness: Readiness,
    pub timeline: Option<TimelineId>,
    pub system_identifier: Option<SystemIdentifier>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DcsPrimaryPostgresView {
    pub readiness: Readiness,
    pub system_identifier: Option<SystemIdentifier>,
    pub committed_wal: WalVector,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DcsReplicaPostgresView {
    pub readiness: Readiness,
    pub system_identifier: Option<SystemIdentifier>,
    pub upstream: Option<MemberId>,
    pub replay_wal: Option<WalVector>,
    pub follow_wal: Option<WalVector>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DcsMemberPostgresView {
    Unknown(DcsUnknownPostgresView),
    Primary(DcsPrimaryPostgresView),
    Replica(DcsReplicaPostgresView),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DcsLeaderView {
    pub holder: MemberId,
    pub generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DcsLeaderStateView {
    Unheld,
    Held(DcsLeaderView),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DcsSwitchoverTargetView {
    AnyHealthyReplica,
    Specific(MemberId),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DcsSwitchoverView {
    pub target: DcsSwitchoverTargetView,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DcsSwitchoverStateView {
    None,
    Requested(DcsSwitchoverView),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DcsView {
    pub worker: WorkerStatus,
    pub trust: DcsTrust,
    pub members: BTreeMap<MemberId, DcsMemberView>,
    pub leader: DcsLeaderStateView,
    pub switchover: DcsSwitchoverStateView,
    pub last_observed_at: Option<UnixMillis>,
}

impl DcsView {
    pub fn empty(worker: WorkerStatus) -> Self {
        Self {
            worker,
            trust: DcsTrust::NotTrusted,
            members: BTreeMap::new(),
            leader: DcsLeaderStateView::Unheld,
            switchover: DcsSwitchoverStateView::None,
            last_observed_at: None,
        }
    }

    pub(crate) fn starting() -> Self {
        Self::empty(WorkerStatus::Starting)
    }
}

pub(crate) struct DcsWorkerCtx {
    pub(crate) identity: DcsNodeIdentity,
    pub(crate) cadence: DcsCadence,
    pub(crate) advertisement: DcsLocalMemberAdvertisement,
    pub(crate) observed: DcsObservedState,
    pub(crate) state_channel: DcsStateChannel,
    pub(crate) control: DcsControlPlane,
    pub(crate) runtime: DcsRuntime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsNodeIdentity {
    pub(crate) self_id: MemberId,
    pub(crate) scope: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsCadence {
    pub(crate) poll_interval: Duration,
    pub(crate) member_ttl_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsLocalMemberAdvertisement {
    pub(crate) postgres: DcsMemberEndpointView,
    pub(crate) api: Option<DcsMemberApiView>,
}

#[derive(Clone, Debug)]
pub(crate) struct DcsObservedState {
    pub(crate) pg: StateSubscriber<PgInfoState>,
}

pub(crate) struct DcsStateChannel {
    pub(crate) publisher: StatePublisher<DcsView>,
    pub(crate) cache: DcsCache,
}

impl DcsStateChannel {
    pub(crate) fn new(publisher: StatePublisher<DcsView>) -> Self {
        Self {
            publisher,
            cache: DcsCache {
                member_records: BTreeMap::new(),
                leader_record: None,
                switchover_record: None,
                init_lock: None,
            },
        }
    }
}

pub(crate) struct DcsControlPlane {
    pub(crate) command_inbox: DcsCommandInbox,
    pub(crate) store: Box<dyn DcsStore>,
}

pub(crate) struct DcsRuntime {
    pub(crate) log: LogHandle,
    pub(crate) last_emitted_store_healthy: Option<bool>,
    pub(crate) last_emitted_trust: Option<DcsTrust>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberLeaseRecord {
    pub(crate) owner: MemberId,
    pub(crate) ttl_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberRecord {
    pub(crate) lease: MemberLeaseRecord,
    pub(crate) routing: MemberRoutingRecord,
    pub(crate) postgres: MemberPostgresRecord,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberRoutingRecord {
    pub(crate) postgres: MemberEndpointRecord,
    pub(crate) api: Option<MemberApiRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberEndpointRecord {
    pub(crate) host: String,
    pub(crate) port: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberApiRecord {
    pub(crate) url: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct UnknownPostgresRecord {
    pub(crate) readiness: Readiness,
    pub(crate) timeline: Option<TimelineId>,
    pub(crate) system_identifier: Option<SystemIdentifier>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PrimaryPostgresRecord {
    pub(crate) readiness: Readiness,
    pub(crate) system_identifier: Option<SystemIdentifier>,
    pub(crate) committed_wal: WalVector,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ReplicaPostgresRecord {
    pub(crate) readiness: Readiness,
    pub(crate) system_identifier: Option<SystemIdentifier>,
    pub(crate) upstream: Option<MemberId>,
    pub(crate) replay_wal: Option<WalVector>,
    pub(crate) follow_wal: Option<WalVector>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum MemberPostgresRecord {
    Unknown(UnknownPostgresRecord),
    Primary(PrimaryPostgresRecord),
    Replica(ReplicaPostgresRecord),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct LeaderLeaseRecord {
    pub(crate) holder: MemberId,
    pub(crate) generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SwitchoverRecord {
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DcsCache {
    pub(crate) member_records: BTreeMap<MemberId, MemberRecord>,
    pub(crate) leader_record: Option<LeaderLeaseRecord>,
    pub(crate) switchover_record: Option<SwitchoverRecord>,
    pub(crate) init_lock: Option<InitLockRecord>,
}

pub(crate) fn evaluate_trust(etcd_healthy: bool, cache: &DcsCache, self_id: &MemberId) -> DcsTrust {
    if !etcd_healthy {
        return DcsTrust::NotTrusted;
    }

    if !cache.member_records.contains_key(self_id) {
        return DcsTrust::Degraded;
    }

    if !has_any_members(cache) {
        return DcsTrust::Degraded;
    }

    DcsTrust::FullQuorum
}

fn has_any_members(cache: &DcsCache) -> bool {
    !cache.member_records.is_empty()
}

pub(crate) fn build_dcs_view(
    worker: WorkerStatus,
    trust: DcsTrust,
    cache: &DcsCache,
    last_observed_at: Option<UnixMillis>,
) -> DcsView {
    DcsView {
        worker,
        trust,
        members: cache
            .member_records
            .iter()
            .map(|(member_id, record)| (member_id.clone(), build_member_view(record)))
            .collect(),
        leader: cache
            .leader_record
            .as_ref()
            .map(|record| {
                DcsLeaderStateView::Held(DcsLeaderView {
                    holder: record.holder.clone(),
                    generation: record.generation,
                })
            })
            .unwrap_or(DcsLeaderStateView::Unheld),
        switchover: cache
            .switchover_record
            .as_ref()
            .map(|record| {
                DcsSwitchoverStateView::Requested(DcsSwitchoverView {
                    target: match &record.target {
                        SwitchoverTargetRecord::AnyHealthyReplica => {
                            DcsSwitchoverTargetView::AnyHealthyReplica
                        }
                        SwitchoverTargetRecord::Specific(member_id) => {
                            DcsSwitchoverTargetView::Specific(member_id.clone())
                        }
                    },
                })
            })
            .unwrap_or(DcsSwitchoverStateView::None),
        last_observed_at,
    }
}

fn build_member_view(record: &MemberRecord) -> DcsMemberView {
    DcsMemberView {
        member_id: record.lease.owner.clone(),
        lease: DcsMemberLeaseView {
            ttl_ms: record.lease.ttl_ms,
        },
        routing: DcsMemberRoutingView {
            postgres: DcsMemberEndpointView {
                host: record.routing.postgres.host.clone(),
                port: record.routing.postgres.port,
            },
            api: record.routing.api.as_ref().map(|api| DcsMemberApiView {
                url: api.url.clone(),
            }),
        },
        postgres: match &record.postgres {
            MemberPostgresRecord::Unknown(observation) => {
                DcsMemberPostgresView::Unknown(DcsUnknownPostgresView {
                    readiness: observation.readiness.clone(),
                    timeline: observation.timeline,
                    system_identifier: observation.system_identifier,
                })
            }
            MemberPostgresRecord::Primary(observation) => {
                DcsMemberPostgresView::Primary(DcsPrimaryPostgresView {
                    readiness: observation.readiness.clone(),
                    system_identifier: observation.system_identifier,
                    committed_wal: observation.committed_wal.clone(),
                })
            }
            MemberPostgresRecord::Replica(observation) => {
                DcsMemberPostgresView::Replica(DcsReplicaPostgresView {
                    readiness: observation.readiness.clone(),
                    system_identifier: observation.system_identifier,
                    upstream: observation.upstream.clone(),
                    replay_wal: observation.replay_wal.clone(),
                    follow_wal: observation.follow_wal.clone(),
                })
            }
        },
    }
}

pub(crate) fn build_local_member_record(
    self_id: &MemberId,
    postgres_host: &str,
    postgres_port: u16,
    api_url: Option<&str>,
    lease_ttl_ms: u64,
    pg_state: &PgInfoState,
) -> MemberRecord {
    let lease = MemberLeaseRecord {
        owner: self_id.clone(),
        ttl_ms: lease_ttl_ms,
    };
    let routing = MemberRoutingRecord {
        postgres: MemberEndpointRecord {
            host: postgres_host.to_string(),
            port: postgres_port,
        },
        api: api_url.map(|url| MemberApiRecord {
            url: url.to_string(),
        }),
    };

    match pg_state {
        PgInfoState::Unknown { common } => MemberRecord {
            lease,
            routing,
            postgres: MemberPostgresRecord::Unknown(UnknownPostgresRecord {
                readiness: common.readiness.clone(),
                timeline: common.timeline,
                system_identifier: common.system_identifier,
            }),
        },
        PgInfoState::Primary {
            common, wal_lsn, ..
        } => MemberRecord {
            lease,
            routing,
            postgres: MemberPostgresRecord::Primary(PrimaryPostgresRecord {
                readiness: common.readiness.clone(),
                system_identifier: common.system_identifier,
                committed_wal: WalVector {
                    timeline: common.timeline,
                    lsn: *wal_lsn,
                },
            }),
        },
        PgInfoState::Replica {
            common,
            replay_lsn,
            follow_lsn,
            upstream,
        } => MemberRecord {
            lease,
            routing,
            postgres: MemberPostgresRecord::Replica(ReplicaPostgresRecord {
                readiness: common.readiness.clone(),
                system_identifier: common.system_identifier,
                upstream: upstream.as_ref().map(|value| value.member_id.clone()),
                replay_wal: Some(WalVector {
                    timeline: common.timeline,
                    lsn: *replay_lsn,
                }),
                follow_wal: follow_lsn.map(|lsn| WalVector {
                    timeline: common.timeline,
                    lsn,
                }),
            }),
        },
    }
}
