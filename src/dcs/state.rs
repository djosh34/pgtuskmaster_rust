use std::collections::BTreeMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{
    config::{DcsClientConfig, DcsEndpoint},
    logging::LogHandle,
    pginfo::state::{PgInfoState, Readiness},
    state::{
        LeaseEpoch, MemberId, ObservedWalPosition, PgTcpTarget, StatePublisher, StateSubscriber,
        SwitchoverTarget, SystemIdentifier, TimelineId,
    },
};

use super::command::DcsCommandInbox;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DcsMode {
    NotTrusted,
    Degraded,
    Coordinated,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DcsView {
    NotTrusted(NotTrustedView),
    Degraded(ClusterView),
    Coordinated(ClusterView),
}

impl DcsView {
    pub fn mode(&self) -> DcsMode {
        match self {
            Self::NotTrusted(_) => DcsMode::NotTrusted,
            Self::Degraded(_) => DcsMode::Degraded,
            Self::Coordinated(_) => DcsMode::Coordinated,
        }
    }

    pub fn observed_leadership(&self) -> Option<&LeaseEpoch> {
        match self {
            Self::NotTrusted(view) => view.observed_leadership(),
            Self::Degraded(view) | Self::Coordinated(view) => view.leadership().held(),
        }
    }

    pub fn cluster(&self) -> Option<&ClusterView> {
        match self {
            Self::NotTrusted(view) => Some(view.cluster()),
            Self::Degraded(view) | Self::Coordinated(view) => Some(view),
        }
    }

    pub fn is_coordinated(&self) -> bool {
        matches!(self, Self::Coordinated(_))
    }

    pub(crate) fn starting() -> Self {
        Self::NotTrusted(NotTrustedView {
            observed_leadership: None,
            cluster: ClusterView {
                members: BTreeMap::new(),
                leadership: LeadershipObservation::Open,
                switchover: SwitchoverView::None,
            },
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotTrustedView {
    observed_leadership: Option<LeaseEpoch>,
    cluster: ClusterView,
}

impl NotTrustedView {
    pub fn observed_leadership(&self) -> Option<&LeaseEpoch> {
        self.observed_leadership.as_ref()
    }

    pub fn cluster(&self) -> &ClusterView {
        &self.cluster
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterView {
    members: BTreeMap<MemberId, ClusterMemberView>,
    leadership: LeadershipObservation,
    switchover: SwitchoverView,
}

impl ClusterView {
    pub fn members(&self) -> impl Iterator<Item = (&MemberId, &ClusterMemberView)> {
        self.members.iter()
    }

    pub fn member_ids(&self) -> impl Iterator<Item = &MemberId> {
        self.members.keys()
    }

    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    pub fn member(&self, member_id: &MemberId) -> Option<&ClusterMemberView> {
        self.members.get(member_id)
    }

    pub fn leadership(&self) -> &LeadershipObservation {
        &self.leadership
    }

    pub fn switchover(&self) -> &SwitchoverView {
        &self.switchover
    }

    #[cfg(any(test, feature = "internal-test-support"))]
    #[allow(dead_code)]
    pub(crate) fn new(
        members: BTreeMap<MemberId, ClusterMemberView>,
        leadership: LeadershipObservation,
        switchover: SwitchoverView,
    ) -> Self {
        Self {
            members,
            leadership,
            switchover,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterMemberView {
    postgres: MemberPostgresView,
    postgres_target: PgTcpTarget,
}

impl ClusterMemberView {
    pub fn postgres_target(&self) -> &PgTcpTarget {
        &self.postgres_target
    }

    pub fn postgres(&self) -> &MemberPostgresView {
        &self.postgres
    }

    #[cfg(any(test, feature = "internal-test-support"))]
    #[allow(dead_code)]
    pub(crate) fn new(postgres: MemberPostgresView, postgres_target: PgTcpTarget) -> Self {
        Self {
            postgres,
            postgres_target,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MemberPostgresView {
    Unknown {
        readiness: Readiness,
        timeline: Option<TimelineId>,
        system_identifier: Option<SystemIdentifier>,
    },
    Primary {
        readiness: Readiness,
        system_identifier: Option<SystemIdentifier>,
        committed_wal: ObservedWalPosition,
    },
    Replica {
        readiness: Readiness,
        system_identifier: Option<SystemIdentifier>,
        upstream: Option<MemberId>,
        replay_wal: Option<ObservedWalPosition>,
        follow_wal: Option<ObservedWalPosition>,
    },
}

impl MemberPostgresView {
    pub fn readiness(&self) -> Readiness {
        match self {
            Self::Unknown { readiness, .. }
            | Self::Primary { readiness, .. }
            | Self::Replica { readiness, .. } => readiness.clone(),
        }
    }

    pub fn system_identifier(&self) -> Option<SystemIdentifier> {
        match self {
            Self::Unknown {
                system_identifier, ..
            }
            | Self::Primary {
                system_identifier, ..
            }
            | Self::Replica {
                system_identifier, ..
            } => *system_identifier,
        }
    }

    pub fn timeline(&self) -> Option<TimelineId> {
        match self {
            Self::Unknown { timeline, .. } => *timeline,
            Self::Primary { committed_wal, .. } => committed_wal.timeline,
            Self::Replica {
                replay_wal,
                follow_wal,
                ..
            } => replay_wal
                .as_ref()
                .map(|position| position.timeline)
                .or_else(|| follow_wal.as_ref().map(|position| position.timeline))
                .flatten(),
        }
    }

    pub fn is_primary(&self) -> bool {
        matches!(self, Self::Primary { .. })
    }

    pub fn is_ready_replica(&self) -> bool {
        matches!(
            self,
            Self::Replica {
                readiness: Readiness::Ready,
                ..
            }
        )
    }

    pub fn is_ready_non_primary(&self) -> bool {
        matches!(
            self,
            Self::Unknown {
                readiness: Readiness::Ready,
                ..
            }
                | Self::Replica {
                    readiness: Readiness::Ready,
                    ..
                }
        )
    }

    pub fn committed_wal(&self) -> Option<&ObservedWalPosition> {
        match self {
            Self::Primary { committed_wal, .. } => Some(committed_wal),
            Self::Unknown { .. } | Self::Replica { .. } => None,
        }
    }

    pub fn replay_wal(&self) -> Option<&ObservedWalPosition> {
        match self {
            Self::Replica { replay_wal, .. } => replay_wal.as_ref(),
            Self::Unknown { .. } | Self::Primary { .. } => None,
        }
    }

    pub fn follow_wal(&self) -> Option<&ObservedWalPosition> {
        match self {
            Self::Replica { follow_wal, .. } => follow_wal.as_ref(),
            Self::Unknown { .. } | Self::Primary { .. } => None,
        }
    }

    pub fn upstream(&self) -> Option<&MemberId> {
        match self {
            Self::Replica { upstream, .. } => upstream.as_ref(),
            Self::Unknown { .. } | Self::Primary { .. } => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeadershipObservation {
    Open,
    Held(LeaseEpoch),
}

impl LeadershipObservation {
    pub fn held(&self) -> Option<&LeaseEpoch> {
        match self {
            Self::Open => None,
            Self::Held(epoch) => Some(epoch),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "state", content = "target")]
pub enum SwitchoverView {
    None,
    Requested(SwitchoverTarget),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DcsEtcdConfig {
    pub(crate) endpoints: Vec<DcsEndpoint>,
    pub(crate) client: DcsClientConfig,
}

pub(crate) struct DcsWorkerCtx {
    pub(crate) identity: DcsNodeIdentity,
    pub(crate) etcd: DcsEtcdConfig,
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
    pub(crate) postgres: PgTcpTarget,
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
            },
        }
    }
}

pub(crate) struct DcsControlPlane {
    pub(crate) command_inbox: DcsCommandInbox,
}

pub(crate) struct DcsRuntime {
    pub(crate) log: LogHandle,
    pub(crate) last_emitted_mode: Option<DcsMode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberLeaseRecord {
    pub(crate) owner: MemberId,
    pub(crate) ttl_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MemberRecord {
    pub(crate) lease: MemberLeaseRecord,
    pub(crate) postgres_target: PgTcpTarget,
    pub(crate) postgres: MemberPostgresRecord,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum MemberPostgresRecord {
    Unknown {
        readiness: Readiness,
        timeline: Option<TimelineId>,
        system_identifier: Option<SystemIdentifier>,
    },
    Primary {
        readiness: Readiness,
        system_identifier: Option<SystemIdentifier>,
        committed_wal: ObservedWalPosition,
    },
    Replica {
        readiness: Readiness,
        system_identifier: Option<SystemIdentifier>,
        upstream: Option<MemberId>,
        replay_wal: Option<ObservedWalPosition>,
        follow_wal: Option<ObservedWalPosition>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct LeadershipRecord {
    pub(crate) epoch: LeaseEpoch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SwitchoverRecord {
    pub(crate) target: SwitchoverTarget,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DcsCache {
    pub(crate) member_records: BTreeMap<MemberId, MemberRecord>,
    pub(crate) leader_record: Option<LeadershipRecord>,
    pub(crate) switchover_record: Option<SwitchoverRecord>,
}

pub(crate) fn evaluate_mode(etcd_reachable: bool, cache: &DcsCache, self_id: &MemberId) -> DcsMode {
    if !etcd_reachable {
        return DcsMode::NotTrusted;
    }

    if !cache.member_records.contains_key(self_id) {
        return DcsMode::Degraded;
    }

    if !has_member_quorum(cache) {
        return DcsMode::Degraded;
    }

    DcsMode::Coordinated
}

fn has_member_quorum(cache: &DcsCache) -> bool {
    if cache.member_records.len() <= 1 {
        cache.member_records.len() == 1
    } else {
        cache.member_records.len() >= 2
    }
}

pub(crate) fn build_dcs_view(mode: DcsMode, cache: &DcsCache) -> DcsView {
    let authoritative_leader = cache
        .leader_record
        .as_ref()
        .map(|record| record.epoch.holder.clone());
    let cluster = ClusterView {
        members: cache
            .member_records
            .iter()
            .map(|(member_id, record)| {
                (
                    member_id.clone(),
                    build_member_view(member_id, record, authoritative_leader.as_ref()),
                )
            })
            .collect(),
        leadership: cache
            .leader_record
            .as_ref()
            .map(|record| LeadershipObservation::Held(record.epoch.clone()))
            .unwrap_or(LeadershipObservation::Open),
        switchover: cache
            .switchover_record
            .as_ref()
            .map(|record| SwitchoverView::Requested(record.target.clone()))
            .unwrap_or(SwitchoverView::None),
    };

    match mode {
        DcsMode::NotTrusted => DcsView::NotTrusted(NotTrustedView {
            observed_leadership: cache.leader_record.as_ref().map(|record| record.epoch.clone()),
            cluster,
        }),
        DcsMode::Degraded => DcsView::Degraded(cluster),
        DcsMode::Coordinated => DcsView::Coordinated(cluster),
    }
}

fn build_member_view(
    member_id: &MemberId,
    record: &MemberRecord,
    authoritative_leader: Option<&MemberId>,
) -> ClusterMemberView {
    ClusterMemberView {
        postgres: match &record.postgres {
            MemberPostgresRecord::Unknown {
                readiness,
                timeline,
                system_identifier,
            } => MemberPostgresView::Unknown {
                readiness: readiness.clone(),
                timeline: *timeline,
                system_identifier: *system_identifier,
            },
            MemberPostgresRecord::Primary {
                readiness,
                system_identifier,
                committed_wal,
            } => {
                if authoritative_leader.is_some_and(|leader| leader != member_id) {
                    MemberPostgresView::Unknown {
                        readiness: readiness.clone(),
                        timeline: committed_wal.timeline,
                        system_identifier: *system_identifier,
                    }
                } else {
                    MemberPostgresView::Primary {
                        readiness: readiness.clone(),
                        system_identifier: *system_identifier,
                        committed_wal: committed_wal.clone(),
                    }
                }
            }
            MemberPostgresRecord::Replica {
                readiness,
                system_identifier,
                upstream,
                replay_wal,
                follow_wal,
            } => MemberPostgresView::Replica {
                readiness: readiness.clone(),
                system_identifier: *system_identifier,
                upstream: upstream.clone(),
                replay_wal: replay_wal.clone(),
                follow_wal: follow_wal.clone(),
            },
        },
        postgres_target: record.postgres_target.clone(),
    }
}

pub(crate) fn build_local_member_record(
    self_id: &MemberId,
    postgres_target: &PgTcpTarget,
    lease_ttl_ms: u64,
    pg_state: &PgInfoState,
    previous_record: Option<&MemberRecord>,
) -> MemberRecord {
    let lease = MemberLeaseRecord {
        owner: self_id.clone(),
        ttl_ms: lease_ttl_ms,
    };

    let postgres = match pg_state {
        PgInfoState::Unknown { common } => MemberPostgresRecord::Unknown {
            readiness: common.readiness.clone(),
            timeline: common
                .timeline
                .or_else(|| previous_record.and_then(member_record_timeline)),
            system_identifier: common
                .system_identifier
                .or_else(|| previous_record.and_then(member_record_system_identifier)),
        },
        PgInfoState::Primary {
            common, wal_lsn, ..
        } => MemberPostgresRecord::Primary {
            readiness: common.readiness.clone(),
            system_identifier: common.system_identifier,
            committed_wal: ObservedWalPosition {
                timeline: common.timeline,
                lsn: *wal_lsn,
            },
        },
        PgInfoState::Replica {
            common,
            replay_lsn,
            follow_lsn,
            upstream,
        } => MemberPostgresRecord::Replica {
            readiness: common.readiness.clone(),
            system_identifier: common.system_identifier,
            upstream: upstream.as_ref().map(|value| value.member_id.clone()),
            replay_wal: Some(ObservedWalPosition {
                timeline: common.timeline,
                lsn: *replay_lsn,
            }),
            follow_wal: follow_lsn.map(|lsn| ObservedWalPosition {
                timeline: common.timeline,
                lsn,
            }),
        },
    };

    MemberRecord {
        lease,
        postgres_target: postgres_target.clone(),
        postgres,
    }
}

fn member_record_timeline(record: &MemberRecord) -> Option<TimelineId> {
    match &record.postgres {
        MemberPostgresRecord::Unknown { timeline, .. } => *timeline,
        MemberPostgresRecord::Primary { committed_wal, .. } => committed_wal.timeline,
        MemberPostgresRecord::Replica {
            replay_wal,
            follow_wal,
            ..
        } => replay_wal
            .as_ref()
            .and_then(|value| value.timeline)
            .or_else(|| follow_wal.as_ref().and_then(|value| value.timeline)),
    }
}

fn member_record_system_identifier(record: &MemberRecord) -> Option<SystemIdentifier> {
    match &record.postgres {
        MemberPostgresRecord::Unknown {
            system_identifier, ..
        }
        | MemberPostgresRecord::Primary {
            system_identifier, ..
        }
        | MemberPostgresRecord::Replica {
            system_identifier, ..
        } => *system_identifier,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        pginfo::state::{PgInfoState, Readiness},
        state::{LeaseEpoch, MemberId, PgTcpTarget, SystemIdentifier, TimelineId, WalLsn},
    };

    use super::{
        build_dcs_view, build_local_member_record, DcsCache, DcsMode, LeadershipObservation,
        LeadershipRecord,
        MemberLeaseRecord, MemberPostgresRecord, MemberPostgresView, MemberRecord,
        ObservedWalPosition,
    };

    fn member_record(postgres: MemberPostgresRecord) -> Result<MemberRecord, String> {
        Ok(MemberRecord {
            lease: MemberLeaseRecord {
                owner: MemberId("owner".to_string()),
                ttl_ms: 5_000,
            },
            postgres_target: PgTcpTarget::new("127.0.0.1".to_string(), 5432)?,
            postgres,
        })
    }

    #[test]
    fn build_dcs_view_hides_non_leader_primary_records() -> Result<(), String> {
        let mut member_records = BTreeMap::new();
        member_records.insert(
            MemberId("node-a".to_string()),
            member_record(MemberPostgresRecord::Primary {
                readiness: Readiness::Ready,
                system_identifier: None,
                committed_wal: ObservedWalPosition {
                    timeline: None,
                    lsn: WalLsn(42),
                },
            })?,
        );
        member_records.insert(
            MemberId("node-b".to_string()),
            member_record(MemberPostgresRecord::Primary {
                readiness: Readiness::Ready,
                system_identifier: None,
                committed_wal: ObservedWalPosition {
                    timeline: None,
                    lsn: WalLsn(41),
                },
            })?,
        );
        let cache = DcsCache {
            member_records,
            leader_record: Some(LeadershipRecord {
                epoch: LeaseEpoch {
                    holder: MemberId("node-a".to_string()),
                    generation: 7,
                },
            }),
            switchover_record: None,
        };

        let cluster = match build_dcs_view(DcsMode::Coordinated, &cache) {
            super::DcsView::Coordinated(cluster) => cluster,
            other => return Err(format!("expected coordinated view, got {other:?}")),
        };

        if cluster.leadership()
            != &LeadershipObservation::Held(LeaseEpoch {
                holder: MemberId("node-a".to_string()),
                generation: 7,
            })
        {
            return Err("expected node-a leadership to remain authoritative".to_string());
        }

        match cluster
            .member(&MemberId("node-a".to_string()))
            .ok_or_else(|| "missing node-a member".to_string())?
            .postgres()
        {
            MemberPostgresView::Primary { .. } => {}
            other => return Err(format!("expected node-a to remain primary, got {other:?}")),
        }

        match cluster
            .member(&MemberId("node-b".to_string()))
            .ok_or_else(|| "missing node-b member".to_string())?
            .postgres()
        {
            MemberPostgresView::Unknown { readiness, .. } if readiness == &Readiness::Ready => {}
            other => {
                return Err(format!(
                    "expected stale non-leader primary to be downgraded, got {other:?}"
                ))
            }
        }

        Ok(())
    }

    #[test]
    fn build_local_member_record_preserves_last_known_identity_when_pg_is_unknown() -> Result<(), String> {
        let previous = member_record(MemberPostgresRecord::Primary {
            readiness: Readiness::Ready,
            system_identifier: Some(SystemIdentifier(41)),
            committed_wal: ObservedWalPosition {
                timeline: Some(TimelineId(7)),
                lsn: WalLsn(42),
            },
        })?;
        let pg_state = PgInfoState::Unknown {
            common: crate::pginfo::state::PgInfoCommon {
                worker: crate::state::WorkerStatus::Running,
                sql: crate::pginfo::state::SqlStatus::Unreachable,
                readiness: Readiness::NotReady,
                timeline: None,
                system_identifier: None,
                pg_config: crate::pginfo::state::PgConfig {
                    port: None,
                    hot_standby: None,
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: BTreeMap::new(),
                },
                last_refresh_at: None,
            },
        };

        let record = build_local_member_record(
            &MemberId("node-a".to_string()),
            &PgTcpTarget::new("127.0.0.1".to_string(), 5432)?,
            5_000,
            &pg_state,
            Some(&previous),
        );

        match record.postgres {
            MemberPostgresRecord::Unknown {
                timeline,
                system_identifier,
                ..
            } => {
                if timeline != Some(TimelineId(7)) {
                    return Err(format!("expected preserved timeline, got {timeline:?}"));
                }
                if system_identifier != Some(SystemIdentifier(41)) {
                    return Err(format!(
                        "expected preserved system identifier, got {system_identifier:?}"
                    ));
                }
            }
            other => return Err(format!("expected unknown member record, got {other:?}")),
        }

        Ok(())
    }
}
