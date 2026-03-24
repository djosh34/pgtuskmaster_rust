use crate::{
    config_v2::RuntimeConfigV2,
    dcs::{DcsMemberState, DcsSnapshot},
    postgres_managed_conf::ManagedRecoverySignal,
    process::{
        jobs::{
            BaseBackupSpec, BootstrapSpec, DemoteSpec, MandatoryRoleSourceConn,
            MandatorySourceRole, PgRewindSpec, PostgresStartIntent, PostgresStartMode,
            ProcessError, ProcessIntent, PromoteSpec, ReplicaProvisionIntent,
        },
        source::source_from_member,
        state::ProcessObservedSnapshot,
    },
    state::{MemberId, NodeIdentity},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ClusterProcessPlan {
    Bootstrap(BootstrapSpec),
    BaseBackup(BaseBackupSpec),
    PgRewind(PgRewindSpec),
    StartManagedPostgres(ManagedStartPlan),
    Promote(PromoteSpec),
    Demote(DemoteSpec),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManagedStartPlan {
    pub(crate) mode: PostgresStartMode,
    pub(crate) desired_session: DesiredManagedPostgresSession,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DesiredManagedPostgresSession {
    Primary,
    DetachedStandby,
    Follow(Box<ReplicaFollowPlan>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReplicaFollowPlan {
    pub(crate) source: MandatoryRoleSourceConn,
    pub(crate) primary_slot_name: Option<String>,
}

#[derive(Default)]
pub(crate) struct ProcessIntentPlanner;

impl ProcessIntentPlanner {
    pub(crate) fn plan(
        &self,
        identity: &NodeIdentity,
        cfg: &RuntimeConfigV2,
        observed: &ProcessObservedSnapshot,
        intent: &ProcessIntent,
    ) -> Result<ClusterProcessPlan, ProcessError> {
        match intent {
            ProcessIntent::Bootstrap => Ok(ClusterProcessPlan::Bootstrap(BootstrapSpec {
                data_dir: cfg.postgres.data_dir.clone(),
                superuser: cfg.postgres.superuser.username.clone(),
                timeout_ms: None,
            })),
            ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup { leader }) => {
                let source =
                    basebackup_source_from_leader(&identity.member_id, cfg, &observed.dcs, leader)?;
                Ok(ClusterProcessPlan::BaseBackup(BaseBackupSpec {
                    data_dir: cfg.postgres.data_dir.clone(),
                    source,
                    timeout_ms: Some(duration_millis_u64(cfg.timing.bootstrap_timeout)),
                }))
            }
            ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::PgRewind { leader }) => {
                let (source_member_id, source_member) =
                    resolve_source_member(&observed.dcs, leader)?;
                let source = source_from_member(
                    &identity.member_id,
                    cfg,
                    source_member_id,
                    source_member,
                    MandatorySourceRole::Rewinder,
                )
                .map_err(source_materialization_error)?;
                Ok(ClusterProcessPlan::PgRewind(PgRewindSpec {
                    target_data_dir: cfg.postgres.data_dir.clone(),
                    source,
                    timeout_ms: None,
                }))
            }
            ProcessIntent::Start(PostgresStartIntent::Primary) => {
                if observed.managed_recovery_state != ManagedRecoverySignal::None {
                    return Err(ProcessError::InvalidSpec(
                        "existing postgres data dir contains managed replica recovery state but no leader-derived source is available to rebuild authoritative managed config".to_string(),
                    ));
                }
                Ok(ClusterProcessPlan::StartManagedPostgres(ManagedStartPlan {
                    mode: PostgresStartMode::Primary,
                    desired_session: DesiredManagedPostgresSession::Primary,
                }))
            }
            ProcessIntent::Start(PostgresStartIntent::DetachedStandby) => {
                Ok(ClusterProcessPlan::StartManagedPostgres(ManagedStartPlan {
                    mode: PostgresStartMode::DetachedStandby,
                    desired_session: DesiredManagedPostgresSession::DetachedStandby,
                }))
            }
            ProcessIntent::Start(PostgresStartIntent::Replica { leader }) => {
                let source =
                    basebackup_source_from_leader(&identity.member_id, cfg, &observed.dcs, leader)?;
                Ok(ClusterProcessPlan::StartManagedPostgres(ManagedStartPlan {
                    mode: PostgresStartMode::Replica,
                    desired_session: DesiredManagedPostgresSession::Follow(Box::new(
                        ReplicaFollowPlan {
                            source,
                            primary_slot_name: None,
                        },
                    )),
                }))
            }
            ProcessIntent::Promote => Ok(ClusterProcessPlan::Promote(PromoteSpec {
                data_dir: cfg.postgres.data_dir.clone(),
                wait_seconds: None,
                timeout_ms: None,
            })),
            ProcessIntent::Demote(mode) => Ok(ClusterProcessPlan::Demote(DemoteSpec {
                data_dir: cfg.postgres.data_dir.clone(),
                mode: mode.clone(),
                timeout_ms: None,
            })),
        }
    }
}

fn basebackup_source_from_leader(
    self_id: &MemberId,
    cfg: &RuntimeConfigV2,
    dcs: &DcsSnapshot,
    leader: &MemberId,
) -> Result<MandatoryRoleSourceConn, ProcessError> {
    let (source_member_id, source_member) = resolve_source_member(dcs, leader)?;
    source_from_member(
        self_id,
        cfg,
        source_member_id,
        source_member,
        MandatorySourceRole::Replicator,
    )
    .map_err(source_materialization_error)
}

fn resolve_source_member<'a>(
    dcs: &'a DcsSnapshot,
    leader: &'a MemberId,
) -> Result<(&'a MemberId, &'a DcsMemberState), ProcessError> {
    let cluster = dcs.quorum_state().ok_or_else(|| {
        ProcessError::InvalidSpec(
            "source member resolution requires a DCS cluster view, but DCS is currently not trusted"
                .to_string(),
        )
    })?;
    cluster
        .member(leader)
        .map(|member| (leader, member))
        .ok_or_else(|| {
            ProcessError::InvalidSpec(format!(
                "target member `{}` not present in DCS view",
                leader.0
            ))
        })
}

fn source_materialization_error(error: super::source::SourceMaterializationError) -> ProcessError {
    ProcessError::InvalidSpec(error.to_string())
}

fn duration_millis_u64(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use crate::{
        dcs::{DcsMemberState, DcsSnapshot},
        dev_support::{runtime_config::RuntimeConfigBuilder, test_fs::unique_test_dir},
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        postgres_managed_conf::ManagedRecoverySignal,
        process::{
            jobs::{
                MandatorySourceRole, PostgresStartIntent, ProcessIntent, ReplicaProvisionIntent,
                ShutdownMode,
            },
            state::ProcessObservedSnapshot,
        },
        state::{
            ClusterName, MemberId, NodeIdentity, PgRoute, ScopeName, SwitchoverState,
            SystemIdentifier, TimelineId, UnixMillis, WalLsn, WorkerStatus,
        },
    };

    use super::{ClusterProcessPlan, DesiredManagedPostgresSession, ProcessIntentPlanner};

    fn sample_identity() -> NodeIdentity {
        NodeIdentity {
            cluster_name: ClusterName("cluster-a".to_string()),
            scope: ScopeName("scope-a".to_string()),
            member_id: MemberId("node-a".to_string()),
        }
    }

    fn sample_runtime(data_dir: PathBuf) -> crate::config_v2::RuntimeConfigV2 {
        RuntimeConfigBuilder::new()
            .with_postgres_data_dir(data_dir)
            .build()
    }

    fn primary_member(host: &str, port: u16) -> Result<DcsMemberState, String> {
        Ok(DcsMemberState {
            cluster_postgres: PgRoute::tcp(host.to_string(), port)?,
            operator_postgres: None,
            postgres: PgInfoState::Primary {
                common: PgInfoCommon {
                    worker: WorkerStatus::Running,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: Some(TimelineId(7)),
                    system_identifier: Some(SystemIdentifier(41)),
                    pg_config: PgConfig {
                        port: Some(port),
                        hot_standby: Some(false),
                        primary_conninfo: None,
                        primary_slot_name: None,
                        extra: BTreeMap::new(),
                    },
                    last_refresh_at: Some(UnixMillis(123)),
                },
                wal_lsn: WalLsn(99),
                slots: Vec::new(),
            },
        })
    }

    fn observed_snapshot(
        dcs: DcsSnapshot,
        managed_recovery_state: ManagedRecoverySignal,
    ) -> ProcessObservedSnapshot {
        ProcessObservedSnapshot {
            dcs,
            managed_recovery_state,
        }
    }

    #[test]
    fn planner_maps_process_intents_to_expected_plan_variants() -> Result<(), String> {
        let root = unique_test_dir("process-planner", "intent-variants")?;
        let cfg = sample_runtime(root.join("data"));
        let leader = MemberId("node-b".to_string());
        let dcs = DcsSnapshot::quorum(
            None,
            SwitchoverState::None,
            BTreeMap::from([(leader.clone(), primary_member("10.0.0.8", 5432)?)]),
        );
        let snapshot = observed_snapshot(dcs, ManagedRecoverySignal::None);
        let planner = ProcessIntentPlanner;
        let identity = sample_identity();

        let cases = [
            (ProcessIntent::Bootstrap, "bootstrap"),
            (
                ProcessIntent::Start(PostgresStartIntent::Primary),
                "start-primary",
            ),
            (
                ProcessIntent::Start(PostgresStartIntent::DetachedStandby),
                "start-detached-standby",
            ),
            (
                ProcessIntent::Start(PostgresStartIntent::Replica {
                    leader: leader.clone(),
                }),
                "start-replica",
            ),
            (
                ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup {
                    leader: leader.clone(),
                }),
                "basebackup",
            ),
            (
                ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::PgRewind {
                    leader: leader.clone(),
                }),
                "pg-rewind",
            ),
            (ProcessIntent::Promote, "promote"),
            (ProcessIntent::Demote(ShutdownMode::Fast), "demote"),
        ];

        for (intent, label) in cases {
            let plan = planner
                .plan(&identity, &cfg, &snapshot, &intent)
                .map_err(|err| format!("planning {label} failed: {err}"))?;
            let matches_expected = matches!(
                (&plan, label),
                (ClusterProcessPlan::Bootstrap(_), "bootstrap")
                    | (ClusterProcessPlan::StartManagedPostgres(_), "start-primary")
                    | (
                        ClusterProcessPlan::StartManagedPostgres(_),
                        "start-detached-standby"
                    )
                    | (ClusterProcessPlan::StartManagedPostgres(_), "start-replica")
                    | (ClusterProcessPlan::BaseBackup(_), "basebackup")
                    | (ClusterProcessPlan::PgRewind(_), "pg-rewind")
                    | (ClusterProcessPlan::Promote(_), "promote")
                    | (ClusterProcessPlan::Demote(_), "demote")
            );
            if !matches_expected {
                return Err(format!("unexpected plan for {label}: {plan:?}"));
            }
        }

        Ok(())
    }

    #[test]
    fn planner_rejects_primary_start_with_existing_managed_replica_state() -> Result<(), String> {
        let root = unique_test_dir("process-planner", "primary-reject")?;
        let cfg = sample_runtime(root.join("data"));
        let snapshot = observed_snapshot(DcsSnapshot::starting(), ManagedRecoverySignal::Standby);
        let planner = ProcessIntentPlanner;
        let error = planner
            .plan(
                &sample_identity(),
                &cfg,
                &snapshot,
                &ProcessIntent::Start(PostgresStartIntent::Primary),
            )
            .err()
            .ok_or_else(|| "expected primary start to be rejected".to_string())?;

        if !error.to_string().contains("managed replica recovery state") {
            return Err(format!("unexpected primary-start rejection: {error}"));
        }

        Ok(())
    }

    #[test]
    fn planner_uses_distinct_source_roles_for_basebackup_and_rewind() -> Result<(), String> {
        let root = unique_test_dir("process-planner", "source-roles")?;
        let cfg = sample_runtime(root.join("data"));
        let leader = MemberId("node-b".to_string());
        let dcs = DcsSnapshot::quorum(
            None,
            SwitchoverState::None,
            BTreeMap::from([(leader.clone(), primary_member("10.0.0.9", 5432)?)]),
        );
        let snapshot = observed_snapshot(dcs, ManagedRecoverySignal::None);
        let planner = ProcessIntentPlanner;
        let identity = sample_identity();

        let basebackup_plan = planner
            .plan(
                &identity,
                &cfg,
                &snapshot,
                &ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup {
                    leader: leader.clone(),
                }),
            )
            .map_err(|err| format!("plan basebackup failed: {err}"))?;
        let rewind_plan = planner
            .plan(
                &identity,
                &cfg,
                &snapshot,
                &ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::PgRewind { leader }),
            )
            .map_err(|err| format!("plan rewind failed: {err}"))?;

        let basebackup_role = match basebackup_plan {
            ClusterProcessPlan::BaseBackup(spec) => spec.source.role,
            other => return Err(format!("unexpected basebackup plan: {other:?}")),
        };
        let rewind_role = match rewind_plan {
            ClusterProcessPlan::PgRewind(spec) => spec.source.role,
            other => return Err(format!("unexpected rewind plan: {other:?}")),
        };

        if basebackup_role != MandatorySourceRole::Replicator {
            return Err(format!(
                "basebackup should use replicator role, observed {basebackup_role:?}"
            ));
        }
        if rewind_role != MandatorySourceRole::Rewinder {
            return Err(format!(
                "rewind should use rewinder role, observed {rewind_role:?}"
            ));
        }

        let replica_plan = planner
            .plan(
                &identity,
                &cfg,
                &snapshot,
                &ProcessIntent::Start(PostgresStartIntent::Replica {
                    leader: MemberId("node-b".to_string()),
                }),
            )
            .map_err(|err| format!("plan replica start failed: {err}"))?;
        match replica_plan {
            ClusterProcessPlan::StartManagedPostgres(start) => match start.desired_session {
                DesiredManagedPostgresSession::Follow(follow) => {
                    if follow.source.role != MandatorySourceRole::Replicator {
                        return Err(format!(
                            "replica start should use replicator role, observed {:?}",
                            follow.source.role
                        ));
                    }
                }
                other => return Err(format!("unexpected replica desired session: {other:?}")),
            },
            other => return Err(format!("unexpected replica start plan: {other:?}")),
        }

        Ok(())
    }
}
