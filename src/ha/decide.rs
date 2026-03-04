use thiserror::Error;

use crate::{
    dcs::state::DcsTrust,
    pginfo::state::{PgInfoState, SqlStatus},
    process::state::{JobOutcome, ProcessState},
};

use super::{
    actions::HaAction,
    state::{DecideInput, DecideOutput, HaPhase},
};

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum DecideError {
    #[error("decision failed")]
    Failed,
}

pub(crate) fn decide(input: DecideInput) -> Result<DecideOutput, DecideError> {
    let DecideInput { current, world } = input;
    let self_member_id = world.config.value.cluster.member_id.as_str();
    let trust = &world.dcs.value.trust;
    let leader_member_id = world
        .dcs
        .value
        .cache
        .leader
        .as_ref()
        .map(|record| record.member_id.0.as_str());
    let leader_is_available = is_available_primary_leader(&world, leader_member_id);
    let active_leader_member_id = if leader_is_available {
        leader_member_id
    } else {
        None
    };
    let switchover_requested = world.dcs.value.cache.switchover.is_some();
    let i_am_leader = leader_member_id == Some(self_member_id);
    let has_other_leader_record = leader_member_id
        .map(|leader| leader != self_member_id)
        .unwrap_or(false);
    let has_available_other_leader = active_leader_member_id
        .map(|leader| leader != self_member_id)
        .unwrap_or(false);
    let pg_reachable = is_postgres_reachable(&world.pg.value);

    let mut next = current.clone();
    next.tick = current.tick.saturating_add(1);
    let mut candidates = Vec::new();

    if !matches!(trust, DcsTrust::FullQuorum) {
        if !matches!(next.phase, HaPhase::FailSafe) {
            if matches!(next.phase, HaPhase::Primary) {
                candidates.push(HaAction::ReleaseLeaderLease);
            }
            next.phase = HaPhase::FailSafe;
        }
        candidates.push(HaAction::SignalFailSafe);
    } else {
        match next.phase {
            HaPhase::Init => {
                next.phase = HaPhase::WaitingPostgresReachable;
            }
            HaPhase::WaitingPostgresReachable => {
                if pg_reachable {
                    next.phase = HaPhase::WaitingDcsTrusted;
                } else {
                    candidates.push(HaAction::StartPostgres);
                }
            }
            HaPhase::WaitingDcsTrusted => {
                if !pg_reachable {
                    next.phase = HaPhase::WaitingPostgresReachable;
                    candidates.push(HaAction::StartPostgres);
                } else if let Some(leader) = active_leader_member_id {
                    if leader == self_member_id {
                        next.phase = HaPhase::Primary;
                    } else {
                        next.phase = HaPhase::Replica;
                        candidates.push(HaAction::FollowLeader {
                            leader_member_id: leader.to_string(),
                        });
                    }
                } else {
                    next.phase = HaPhase::CandidateLeader;
                    candidates.push(HaAction::AcquireLeaderLease);
                }
            }
            HaPhase::Replica => {
                if !pg_reachable {
                    next.phase = HaPhase::WaitingPostgresReachable;
                    candidates.push(HaAction::StartPostgres);
                } else if let Some(leader) = active_leader_member_id {
                    if leader == self_member_id {
                        next.phase = HaPhase::Primary;
                        candidates.push(HaAction::PromoteToPrimary);
                    } else {
                        candidates.push(HaAction::FollowLeader {
                            leader_member_id: leader.to_string(),
                        });
                    }
                } else {
                    next.phase = HaPhase::CandidateLeader;
                    candidates.push(HaAction::AcquireLeaderLease);
                }
            }
            HaPhase::CandidateLeader => {
                if !pg_reachable {
                    next.phase = HaPhase::WaitingPostgresReachable;
                    candidates.push(HaAction::StartPostgres);
                } else if i_am_leader {
                    next.phase = HaPhase::Primary;
                    candidates.push(HaAction::PromoteToPrimary);
                } else if has_available_other_leader {
                    next.phase = HaPhase::Replica;
                    if let Some(leader) = active_leader_member_id {
                        candidates.push(HaAction::FollowLeader {
                            leader_member_id: leader.to_string(),
                        });
                    }
                } else {
                    candidates.push(HaAction::AcquireLeaderLease);
                }
            }
            HaPhase::Primary => {
                if switchover_requested && i_am_leader {
                    next.phase = HaPhase::Replica;
                    candidates.push(HaAction::DemoteToReplica);
                    candidates.push(HaAction::ReleaseLeaderLease);
                    candidates.push(HaAction::ClearSwitchover);
                } else if !pg_reachable {
                    next.phase = HaPhase::Rewinding;
                    candidates.push(HaAction::StartRewind);
                } else if has_other_leader_record {
                    next.phase = HaPhase::Fencing;
                    candidates.push(HaAction::DemoteToReplica);
                    candidates.push(HaAction::ReleaseLeaderLease);
                    candidates.push(HaAction::FenceNode);
                }
            }
            HaPhase::Rewinding => match &world.process.value {
                ProcessState::Running { .. } => {}
                ProcessState::Idle {
                    last_outcome: Some(JobOutcome::Success { .. }),
                    ..
                } => {
                    next.phase = HaPhase::Replica;
                    if let Some(leader) = active_leader_member_id {
                        if leader != self_member_id {
                            candidates.push(HaAction::FollowLeader {
                                leader_member_id: leader.to_string(),
                            });
                        }
                    }
                }
                ProcessState::Idle {
                    last_outcome: Some(_),
                    ..
                } => {
                    next.phase = HaPhase::Bootstrapping;
                    candidates.push(HaAction::RunBootstrap);
                }
                ProcessState::Idle {
                    last_outcome: None, ..
                } => {
                    candidates.push(HaAction::StartRewind);
                }
            },
            HaPhase::Bootstrapping => match &world.process.value {
                ProcessState::Running { .. } => {}
                ProcessState::Idle {
                    last_outcome: Some(JobOutcome::Success { .. }),
                    ..
                } => {
                    next.phase = HaPhase::Replica;
                    candidates.push(HaAction::StartPostgres);
                }
                ProcessState::Idle {
                    last_outcome: Some(_),
                    ..
                } => {
                    next.phase = HaPhase::Fencing;
                    candidates.push(HaAction::FenceNode);
                }
                ProcessState::Idle {
                    last_outcome: None, ..
                } => {
                    candidates.push(HaAction::RunBootstrap);
                }
            },
            HaPhase::Fencing => match &world.process.value {
                ProcessState::Running { .. } => {}
                ProcessState::Idle {
                    last_outcome: Some(JobOutcome::Success { .. }),
                    ..
                } => {
                    next.phase = HaPhase::WaitingDcsTrusted;
                    candidates.push(HaAction::ReleaseLeaderLease);
                }
                ProcessState::Idle {
                    last_outcome: Some(_),
                    ..
                } => {
                    next.phase = HaPhase::FailSafe;
                    candidates.push(HaAction::SignalFailSafe);
                }
                ProcessState::Idle {
                    last_outcome: None, ..
                } => {
                    candidates.push(HaAction::FenceNode);
                }
            },
            HaPhase::FailSafe => {
                next.phase = HaPhase::WaitingDcsTrusted;
            }
        }
    }

    let mut actions = Vec::new();
    for action in candidates {
        let action_id = action.id();
        if !next.recent_action_ids.contains(&action_id) {
            next.recent_action_ids.insert(action_id);
            actions.push(action);
        }
    }
    next.pending = actions.clone();

    Ok(DecideOutput { next, actions })
}

fn is_postgres_reachable(state: &PgInfoState) -> bool {
    let sql = match state {
        PgInfoState::Unknown { common } => &common.sql,
        PgInfoState::Primary { common, .. } => &common.sql,
        PgInfoState::Replica { common, .. } => &common.sql,
    };
    matches!(sql, SqlStatus::Healthy)
}

fn is_available_primary_leader(
    world: &super::state::WorldSnapshot,
    leader_member_id: Option<&str>,
) -> bool {
    let Some(leader_id) = leader_member_id else {
        return false;
    };

    let leader_record = world
        .dcs
        .value
        .cache
        .members
        .values()
        .find(|member| member.member_id.0 == leader_id);
    let Some(member) = leader_record else {
        // Preserve current behavior when leader member metadata is not yet observed.
        return true;
    };

    matches!(member.sql, SqlStatus::Healthy)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::{
        config::{
            schema::{
                ApiConfig, ClusterConfig, DcsConfig, DebugConfig, HaConfig, PostgresConfig,
                SecurityConfig,
            },
            BinaryPaths, LogCleanupConfig, LogLevel, LoggingConfig, PostgresLoggingConfig,
            ProcessConfig, RuntimeConfig,
        },
        dcs::state::{
            DcsCache, DcsState, DcsTrust, LeaderRecord, MemberRecord, MemberRole, SwitchoverRequest,
        },
        ha::{
            actions::{ActionId, HaAction},
            state::{DecideInput, HaPhase, HaState, WorldSnapshot},
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::{
            jobs::{ActiveJob, ActiveJobKind},
            state::{JobOutcome, ProcessState},
        },
        state::{JobId, MemberId, UnixMillis, Version, Versioned, WorkerStatus},
    };

    use super::{decide, DecideError};

    struct Case {
        name: &'static str,
        current_phase: HaPhase,
        trust: DcsTrust,
        pg: PgInfoState,
        leader: Option<&'static str>,
        process: ProcessState,
        recent_action_ids: BTreeSet<ActionId>,
        expected_phase: HaPhase,
        expected_actions: Vec<HaAction>,
    }

    #[test]
    fn transition_matrix_cases() -> Result<(), DecideError> {
        let cases = vec![
            Case {
                name: "init moves to waiting postgres",
                current_phase: HaPhase::Init,
                trust: DcsTrust::FullQuorum,
                pg: pg_unknown(SqlStatus::Unknown),
                leader: None,
                process: process_idle(None),
                recent_action_ids: BTreeSet::new(),
                expected_phase: HaPhase::WaitingPostgresReachable,
                expected_actions: vec![],
            },
            Case {
                name: "waiting postgres emits start when unreachable",
                current_phase: HaPhase::WaitingPostgresReachable,
                trust: DcsTrust::FullQuorum,
                pg: pg_unknown(SqlStatus::Unreachable),
                leader: None,
                process: process_idle(None),
                recent_action_ids: BTreeSet::new(),
                expected_phase: HaPhase::WaitingPostgresReachable,
                expected_actions: vec![HaAction::StartPostgres],
            },
            Case {
                name: "waiting postgres enters dcs trusted when healthy",
                current_phase: HaPhase::WaitingPostgresReachable,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: None,
                process: process_idle(None),
                recent_action_ids: BTreeSet::new(),
                expected_phase: HaPhase::WaitingDcsTrusted,
                expected_actions: vec![],
            },
            Case {
                name: "waiting dcs to replica with known leader",
                current_phase: HaPhase::WaitingDcsTrusted,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(None),
                recent_action_ids: BTreeSet::new(),
                expected_phase: HaPhase::Replica,
                expected_actions: vec![HaAction::FollowLeader {
                    leader_member_id: "node-b".to_string(),
                }],
            },
            Case {
                name: "waiting dcs becomes candidate when no leader",
                current_phase: HaPhase::WaitingDcsTrusted,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: None,
                process: process_idle(None),
                recent_action_ids: BTreeSet::new(),
                expected_phase: HaPhase::CandidateLeader,
                expected_actions: vec![HaAction::AcquireLeaderLease],
            },
            Case {
                name: "candidate becomes primary when lease self",
                current_phase: HaPhase::CandidateLeader,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-a"),
                process: process_idle(None),
                recent_action_ids: BTreeSet::new(),
                expected_phase: HaPhase::Primary,
                expected_actions: vec![HaAction::PromoteToPrimary],
            },
            Case {
                name: "primary split brain fences",
                current_phase: HaPhase::Primary,
                trust: DcsTrust::FullQuorum,
                pg: pg_primary(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(None),
                recent_action_ids: BTreeSet::new(),
                expected_phase: HaPhase::Fencing,
                expected_actions: vec![
                    HaAction::DemoteToReplica,
                    HaAction::ReleaseLeaderLease,
                    HaAction::FenceNode,
                ],
            },
            Case {
                name: "no quorum enters fail safe",
                current_phase: HaPhase::CandidateLeader,
                trust: DcsTrust::FailSafe,
                pg: pg_replica(SqlStatus::Healthy),
                leader: None,
                process: process_idle(None),
                recent_action_ids: BTreeSet::new(),
                expected_phase: HaPhase::FailSafe,
                expected_actions: vec![HaAction::SignalFailSafe],
            },
            Case {
                name: "rewinding success re-enters replica",
                current_phase: HaPhase::Rewinding,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(Some(JobOutcome::Success {
                    id: JobId("job-1".to_string()),
                    finished_at: UnixMillis(10),
                })),
                recent_action_ids: BTreeSet::new(),
                expected_phase: HaPhase::Replica,
                expected_actions: vec![HaAction::FollowLeader {
                    leader_member_id: "node-b".to_string(),
                }],
            },
            Case {
                name: "rewinding failure goes bootstrap",
                current_phase: HaPhase::Rewinding,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(Some(JobOutcome::Cancelled {
                    id: JobId("job-1".to_string()),
                    finished_at: UnixMillis(10),
                })),
                recent_action_ids: BTreeSet::new(),
                expected_phase: HaPhase::Bootstrapping,
                expected_actions: vec![HaAction::RunBootstrap],
            },
            Case {
                name: "bootstrap failure goes fencing",
                current_phase: HaPhase::Bootstrapping,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(Some(JobOutcome::Timeout {
                    id: JobId("job-1".to_string()),
                    finished_at: UnixMillis(11),
                })),
                recent_action_ids: BTreeSet::new(),
                expected_phase: HaPhase::Fencing,
                expected_actions: vec![HaAction::FenceNode],
            },
            Case {
                name: "fencing success returns waiting dcs",
                current_phase: HaPhase::Fencing,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(Some(JobOutcome::Success {
                    id: JobId("job-2".to_string()),
                    finished_at: UnixMillis(12),
                })),
                recent_action_ids: BTreeSet::new(),
                expected_phase: HaPhase::WaitingDcsTrusted,
                expected_actions: vec![HaAction::ReleaseLeaderLease],
            },
            Case {
                name: "fencing failure enters fail safe",
                current_phase: HaPhase::Fencing,
                trust: DcsTrust::FullQuorum,
                pg: pg_replica(SqlStatus::Healthy),
                leader: Some("node-b"),
                process: process_idle(Some(JobOutcome::Failure {
                    id: JobId("job-2".to_string()),
                    error: crate::process::jobs::ProcessError::OperationFailed,
                    finished_at: UnixMillis(12),
                })),
                recent_action_ids: BTreeSet::new(),
                expected_phase: HaPhase::FailSafe,
                expected_actions: vec![HaAction::SignalFailSafe],
            },
        ];

        for case in cases {
            let input = DecideInput {
                current: HaState {
                    worker: WorkerStatus::Running,
                    phase: case.current_phase.clone(),
                    tick: 41,
                    pending: vec![],
                    recent_action_ids: case.recent_action_ids.clone(),
                },
                world: world(
                    case.trust,
                    case.pg.clone(),
                    case.leader,
                    case.process.clone(),
                ),
            };

            let output = decide(input)?;
            assert_eq!(
                output.next.phase, case.expected_phase,
                "case: {}",
                case.name
            );
            assert_eq!(output.actions, case.expected_actions, "case: {}", case.name);
            assert_eq!(
                output.next.pending, case.expected_actions,
                "case: {}",
                case.name
            );
            assert_eq!(output.next.tick, 42, "case: {}", case.name);
        }
        Ok(())
    }

    #[test]
    fn idempotency_suppresses_duplicate_actions() -> Result<(), DecideError> {
        let current = HaState {
            worker: WorkerStatus::Running,
            phase: HaPhase::WaitingDcsTrusted,
            tick: 0,
            pending: vec![],
            recent_action_ids: BTreeSet::new(),
        };
        let world = world(
            DcsTrust::FullQuorum,
            pg_replica(SqlStatus::Healthy),
            None,
            process_idle(None),
        );

        let first = decide(DecideInput {
            current: current.clone(),
            world: world.clone(),
        })?;
        assert_eq!(first.actions, vec![HaAction::AcquireLeaderLease]);

        let second = decide(DecideInput {
            current: first.next,
            world,
        })?;
        assert_eq!(second.actions, vec![]);
        Ok(())
    }

    #[test]
    fn idempotency_emits_only_new_mixed_actions() -> Result<(), DecideError> {
        let mut known_ids = BTreeSet::new();
        known_ids.insert(ActionId::DemoteToReplica);

        let current = HaState {
            worker: WorkerStatus::Running,
            phase: HaPhase::Primary,
            tick: 5,
            pending: vec![],
            recent_action_ids: known_ids,
        };
        let output = decide(DecideInput {
            current,
            world: world(
                DcsTrust::FullQuorum,
                pg_primary(SqlStatus::Healthy),
                Some("node-b"),
                process_idle(None),
            ),
        })?;

        assert_eq!(
            output.actions,
            vec![HaAction::ReleaseLeaderLease, HaAction::FenceNode]
        );
        Ok(())
    }

    #[test]
    fn fail_safe_holds_without_quorum_and_exits_when_restored() -> Result<(), DecideError> {
        let start = HaState {
            worker: WorkerStatus::Running,
            phase: HaPhase::FailSafe,
            tick: 100,
            pending: vec![],
            recent_action_ids: BTreeSet::new(),
        };

        let held = decide(DecideInput {
            current: start.clone(),
            world: world(
                DcsTrust::NotTrusted,
                pg_replica(SqlStatus::Healthy),
                None,
                process_idle(None),
            ),
        })?;
        assert_eq!(held.next.phase, HaPhase::FailSafe);
        assert_eq!(held.actions, vec![HaAction::SignalFailSafe]);

        let recovered = decide(DecideInput {
            current: start,
            world: world(
                DcsTrust::FullQuorum,
                pg_replica(SqlStatus::Healthy),
                None,
                process_idle(None),
            ),
        })?;
        assert_eq!(recovered.next.phase, HaPhase::WaitingDcsTrusted);
        Ok(())
    }

    #[test]
    fn primary_with_switchover_demotes_releases_and_clears_request() -> Result<(), DecideError> {
        let mut snapshot = world(
            DcsTrust::FullQuorum,
            pg_primary(SqlStatus::Healthy),
            Some("node-a"),
            process_idle(None),
        );
        snapshot.dcs.value.cache.switchover = Some(SwitchoverRequest {
            requested_by: MemberId("node-b".to_string()),
        });

        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Primary,
                tick: 10,
                pending: vec![],
                recent_action_ids: BTreeSet::new(),
            },
            world: snapshot,
        })?;

        assert_eq!(output.next.phase, HaPhase::Replica);
        assert_eq!(
            output.actions,
            vec![
                HaAction::DemoteToReplica,
                HaAction::ReleaseLeaderLease,
                HaAction::ClearSwitchover,
            ]
        );
        Ok(())
    }

    fn process_idle(last_outcome: Option<JobOutcome>) -> ProcessState {
        ProcessState::Idle {
            worker: WorkerStatus::Running,
            last_outcome,
        }
    }

    fn process_running() -> ProcessState {
        ProcessState::Running {
            worker: WorkerStatus::Running,
            active: ActiveJob {
                id: JobId("active-1".to_string()),
                kind: ActiveJobKind::StartPostgres,
                started_at: UnixMillis(1),
                deadline_at: UnixMillis(2),
            },
        }
    }

    fn pg_unknown(sql: SqlStatus) -> PgInfoState {
        PgInfoState::Unknown {
            common: pg_common(sql),
        }
    }

    fn pg_primary(sql: SqlStatus) -> PgInfoState {
        PgInfoState::Primary {
            common: pg_common(sql),
            wal_lsn: crate::state::WalLsn(10),
            slots: vec![],
        }
    }

    fn pg_replica(sql: SqlStatus) -> PgInfoState {
        PgInfoState::Replica {
            common: pg_common(sql),
            replay_lsn: crate::state::WalLsn(10),
            follow_lsn: None,
            upstream: None,
        }
    }

    fn pg_common(sql: SqlStatus) -> PgInfoCommon {
        PgInfoCommon {
            worker: WorkerStatus::Running,
            sql,
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
        }
    }

    fn world(
        trust: DcsTrust,
        pg: PgInfoState,
        leader: Option<&str>,
        process: ProcessState,
    ) -> WorldSnapshot {
        let cfg = RuntimeConfig {
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
        };

        let leader_record = leader.map(|member| LeaderRecord {
            member_id: MemberId(member.to_string()),
        });

        WorldSnapshot {
            config: Versioned::new(Version(1), UnixMillis(1), cfg.clone()),
            pg: Versioned::new(Version(1), UnixMillis(1), pg),
            dcs: Versioned::new(
                Version(1),
                UnixMillis(1),
                DcsState {
                    worker: WorkerStatus::Running,
                    trust,
                    cache: DcsCache {
                        members: BTreeMap::new(),
                        leader: leader_record,
                        switchover: None,
                        config: cfg,
                        init_lock: None,
                    },
                    last_refresh_at: Some(UnixMillis(1)),
                },
            ),
            process: Versioned::new(Version(1), UnixMillis(1), process),
        }
    }

    #[test]
    fn rewinding_while_running_emits_nothing() -> Result<(), DecideError> {
        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Rewinding,
                tick: 8,
                pending: vec![],
                recent_action_ids: BTreeSet::new(),
            },
            world: world(
                DcsTrust::FullQuorum,
                pg_replica(SqlStatus::Healthy),
                Some("node-b"),
                process_running(),
            ),
        })?;

        assert_eq!(output.next.phase, HaPhase::Rewinding);
        assert!(output.actions.is_empty());
        Ok(())
    }

    #[test]
    fn replica_with_unhealthy_leader_becomes_candidate() -> Result<(), DecideError> {
        let mut snapshot = world(
            DcsTrust::FullQuorum,
            pg_replica(SqlStatus::Healthy),
            Some("node-b"),
            process_idle(None),
        );
        snapshot.dcs.value.cache.members.insert(
            MemberId("node-b".to_string()),
            MemberRecord {
                member_id: MemberId("node-b".to_string()),
                role: MemberRole::Unknown,
                sql: SqlStatus::Unreachable,
                readiness: Readiness::NotReady,
                timeline: None,
                write_lsn: None,
                replay_lsn: None,
                updated_at: UnixMillis(1),
                pg_version: Version(1),
            },
        );

        let output = decide(DecideInput {
            current: HaState {
                worker: WorkerStatus::Running,
                phase: HaPhase::Replica,
                tick: 11,
                pending: vec![],
                recent_action_ids: BTreeSet::new(),
            },
            world: snapshot,
        })?;

        assert_eq!(output.next.phase, HaPhase::CandidateLeader);
        assert_eq!(output.actions, vec![HaAction::AcquireLeaderLease]);
        Ok(())
    }
}
