use std::collections::BTreeMap;

use crate::{
    api::{controller::get_ha_state, ClusterModeResponse, DesiredNodeStateResponse, PrimaryPlanResponse},
    debug_api::snapshot::{AppLifecycle, SystemSnapshot},
    dcs::state::{
        ClusterIdentityRecord, ClusterInitializedRecord, DcsState, DcsTrust, DcsView, MemberRecord,
        MemberRole, PostgresRuntimeClass,
    },
    ha::{
        decide::decide,
        state::{
            ClusterMode, DecideInput, DesiredNodeState, HaState, LeadershipTransferState,
            PrimaryPlan, QuiescentReason, WorldSnapshot,
        },
    },
    pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
    process::state::ProcessState,
    state::{MemberId, SystemIdentifier, TimelineId, UnixMillis, Version, Versioned, WalLsn, WorkerStatus},
};

fn sample_member(member_id: &str, role: MemberRole) -> MemberRecord {
    MemberRecord {
        member_id: MemberId(member_id.to_string()),
        postgres_host: "127.0.0.1".to_string(),
        postgres_port: 5432,
        api_url: Some(format!("http://{member_id}:8080")),
        role: role.clone(),
        sql: SqlStatus::Healthy,
        readiness: Readiness::Ready,
        timeline: Some(TimelineId(1)),
        write_lsn: if role == MemberRole::Primary {
            Some(WalLsn(11))
        } else {
            None
        },
        replay_lsn: if role == MemberRole::Replica {
            Some(WalLsn(10))
        } else {
            None
        },
        system_identifier: Some(SystemIdentifier(42)),
        durable_end_lsn: Some(WalLsn(11)),
        state_class: Some(crate::dcs::state::MemberStateClass::Promotable),
        postgres_runtime_class: Some(PostgresRuntimeClass::RunningHealthy),
        updated_at: UnixMillis(100),
        pg_version: Version(16),
    }
}

fn sample_runtime_config() -> crate::config::RuntimeConfig {
    crate::test_harness::runtime_config::sample_runtime_config()
}

fn sample_pg_primary() -> PgInfoState {
    PgInfoState::Primary {
        common: PgInfoCommon {
            worker: WorkerStatus::Running,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
            timeline: Some(TimelineId(1)),
            pg_config: PgConfig {
                port: Some(5432),
                hot_standby: Some(false),
                primary_conninfo: None,
                primary_slot_name: None,
                extra: BTreeMap::new(),
            },
            last_refresh_at: Some(UnixMillis(100)),
        },
        wal_lsn: WalLsn(11),
        slots: Vec::new(),
    }
}

fn sample_pg_replica() -> PgInfoState {
    PgInfoState::Replica {
        common: PgInfoCommon {
            worker: WorkerStatus::Running,
            sql: SqlStatus::Healthy,
            readiness: Readiness::Ready,
            timeline: Some(TimelineId(1)),
            pg_config: PgConfig {
                port: Some(5432),
                hot_standby: Some(true),
                primary_conninfo: None,
                primary_slot_name: None,
                extra: BTreeMap::new(),
            },
            last_refresh_at: Some(UnixMillis(100)),
        },
        replay_lsn: WalLsn(10),
        follow_lsn: None,
        upstream: None,
    }
}

fn sample_world(dcs: DcsState, pg: PgInfoState) -> WorldSnapshot {
    let cfg = sample_runtime_config();
    let now = UnixMillis(100);
    WorldSnapshot {
        config: Versioned::new(Version(1), now, cfg),
        pg: Versioned::new(Version(1), now, pg),
        dcs: Versioned::new(Version(1), now, dcs),
        process: Versioned::new(
            Version(1),
            now,
            ProcessState::Idle {
                worker: WorkerStatus::Running,
                last_outcome: None,
            },
        ),
    }
}

#[test]
fn ha_state_response_exposes_cluster_mode_and_desired_state() {
    let cfg = sample_runtime_config();
    let ha = Versioned::new(
        Version(1),
        UnixMillis(100),
        HaState {
            worker: WorkerStatus::Running,
            cluster_mode: ClusterMode::InitializedLeaderPresent {
                leader: MemberId("node-a".to_string()),
            },
            desired_state: DesiredNodeState::Primary {
                plan: PrimaryPlan::KeepLeader,
            },
            leadership_transfer: LeadershipTransferState::None,
            tick: 7,
        },
    );
    let dcs = Versioned::new(
        Version(1),
        UnixMillis(100),
        DcsState {
            worker: WorkerStatus::Running,
            trust: DcsTrust::FreshQuorum,
            cache: DcsView {
                members: BTreeMap::new(),
                leader: Some(crate::dcs::state::LeaderRecord {
                    member_id: MemberId("node-a".to_string()),
                }),
                switchover: None,
                config: cfg.clone(),
                cluster_initialized: Some(ClusterInitializedRecord {
                    initialized_by: MemberId("node-a".to_string()),
                    initialized_at: UnixMillis(1),
                }),
                cluster_identity: Some(ClusterIdentityRecord {
                    system_identifier: SystemIdentifier(42),
                    bootstrapped_by: MemberId("node-a".to_string()),
                    bootstrapped_at: UnixMillis(1),
                }),
                bootstrap_lock: None,
            },
            last_refresh_at: Some(UnixMillis(100)),
        },
    );
    let snapshot = Versioned::new(
        Version(1),
        UnixMillis(100),
        SystemSnapshot {
            app: AppLifecycle::Running,
            generated_at: UnixMillis(100),
            sequence: 9,
            config: Versioned::new(Version(1), UnixMillis(100), cfg.clone()),
            pg: Versioned::new(Version(1), UnixMillis(100), sample_pg_primary()),
            dcs,
            process: Versioned::new(
                Version(1),
                UnixMillis(100),
                ProcessState::Idle {
                    worker: WorkerStatus::Running,
                    last_outcome: None,
                },
            ),
            ha,
            changes: Vec::new(),
            timeline: Vec::new(),
        },
    );

    let response = get_ha_state(&snapshot);
    assert_eq!(
        response.cluster_mode,
        ClusterModeResponse::InitializedLeaderPresent {
            leader: "node-a".to_string(),
        }
    );
    assert_eq!(
        response.desired_state,
        DesiredNodeStateResponse::Primary {
            plan: PrimaryPlanResponse::KeepLeader,
        }
    );
}

#[test]
fn decide_promotes_best_candidate_when_initialized_without_leader() {
    let cfg = sample_runtime_config();
    let mut members = BTreeMap::new();
    members.insert(MemberId("node-a".to_string()), sample_member("node-a", MemberRole::Replica));
    members.insert(MemberId("node-b".to_string()), sample_member("node-b", MemberRole::Replica));

    let world = sample_world(
        DcsState {
            worker: WorkerStatus::Running,
            trust: DcsTrust::FreshQuorum,
            cache: DcsView {
                members,
                leader: None,
                switchover: None,
                config: cfg,
                cluster_initialized: Some(ClusterInitializedRecord {
                    initialized_by: MemberId("node-a".to_string()),
                    initialized_at: UnixMillis(1),
                }),
                cluster_identity: Some(ClusterIdentityRecord {
                    system_identifier: SystemIdentifier(42),
                    bootstrapped_by: MemberId("node-a".to_string()),
                    bootstrapped_at: UnixMillis(1),
                }),
                bootstrap_lock: None,
            },
            last_refresh_at: Some(UnixMillis(100)),
        },
        sample_pg_replica(),
    );

    let output = decide(DecideInput {
        current: HaState {
            worker: WorkerStatus::Running,
            cluster_mode: ClusterMode::InitializedNoLeaderFreshQuorum,
            desired_state: DesiredNodeState::Quiescent {
                reason: QuiescentReason::WaitingForAuthoritativeLeader,
            },
            leadership_transfer: LeadershipTransferState::None,
            tick: 0,
        },
        world,
    });

    assert_eq!(
        output.next.desired_state,
        DesiredNodeState::Primary {
            plan: PrimaryPlan::AcquireLeaderThenPromote,
        }
    );
}
