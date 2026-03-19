use crate::{
    dcs::{DcsMemberState, DcsSnapshot},
    postgres_managed_conf::ManagedRecoverySignal,
    process::{
        jobs::{
            BaseBackupSpec, BootstrapSpec, DemoteSpec, MandatoryRoleSourceConn,
            MandatorySourceRole, PgRewindSpec, PostgresStartIntent, PostgresStartMode,
            ProcessError, ProcessIntent, PromoteSpec, ReplicaProvisionIntent,
        },
        source::source_from_member,
        state::{ProcessNodeIdentity, ProcessObservedSnapshot, ProcessRuntimePlan},
    },
    state::MemberId,
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
        identity: &ProcessNodeIdentity,
        runtime: &ProcessRuntimePlan,
        observed: &ProcessObservedSnapshot,
        intent: &ProcessIntent,
    ) -> Result<ClusterProcessPlan, ProcessError> {
        match intent {
            ProcessIntent::Bootstrap => Ok(ClusterProcessPlan::Bootstrap(BootstrapSpec {
                data_dir: observed.runtime_config.postgres.paths.data_dir.clone(),
                superuser: observed
                    .runtime_config
                    .postgres
                    .roles
                    .mandatory
                    .superuser
                    .username
                    .clone(),
                timeout_ms: None,
            })),
            ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup { leader }) => {
                let source = basebackup_source_from_leader(
                    &identity.member_id,
                    runtime,
                    &observed.dcs,
                    leader,
                )?;
                Ok(ClusterProcessPlan::BaseBackup(BaseBackupSpec {
                    data_dir: observed.runtime_config.postgres.paths.data_dir.clone(),
                    source,
                    timeout_ms: Some(observed.runtime_config.process.timeouts.bootstrap_ms),
                }))
            }
            ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::PgRewind { leader }) => {
                let (source_member_id, source_member) =
                    resolve_source_member(&observed.dcs, leader)?;
                let source = source_from_member(
                    &identity.member_id,
                    runtime,
                    source_member_id,
                    source_member,
                    MandatorySourceRole::Rewinder,
                )
                .map_err(source_materialization_error)?;
                Ok(ClusterProcessPlan::PgRewind(PgRewindSpec {
                    target_data_dir: observed.runtime_config.postgres.paths.data_dir.clone(),
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
                let source = basebackup_source_from_leader(
                    &identity.member_id,
                    runtime,
                    &observed.dcs,
                    leader,
                )?;
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
                data_dir: observed.runtime_config.postgres.paths.data_dir.clone(),
                wait_seconds: None,
                timeout_ms: None,
            })),
            ProcessIntent::Demote(mode) => Ok(ClusterProcessPlan::Demote(DemoteSpec {
                data_dir: observed.runtime_config.postgres.paths.data_dir.clone(),
                mode: mode.clone(),
                timeout_ms: None,
            })),
        }
    }
}

fn basebackup_source_from_leader(
    self_id: &MemberId,
    runtime: &ProcessRuntimePlan,
    dcs: &DcsSnapshot,
    leader: &MemberId,
) -> Result<MandatoryRoleSourceConn, ProcessError> {
    let (source_member_id, source_member) = resolve_source_member(dcs, leader)?;
    source_from_member(
        self_id,
        runtime,
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

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        dcs::{DcsMemberState, DcsSnapshot},
        dev_support::runtime_config::RuntimeConfigBuilder,
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        postgres_managed_conf::ManagedRecoverySignal,
        process::{
            jobs::{
                MandatorySourceRole, PostgresStartIntent, ProcessIntent, ReplicaProvisionIntent,
                ShutdownMode,
            },
            state::{ProcessNodeIdentity, ProcessObservedSnapshot, ProcessRuntimePlan},
        },
        state::{
            ClusterName, MemberId, NodeIdentity, PgTcpTarget, ScopeName, SwitchoverState,
            SystemIdentifier, TimelineId, UnixMillis, WalLsn, WorkerStatus,
        },
    };

    use super::{ClusterProcessPlan, DesiredManagedPostgresSession, ProcessIntentPlanner};

    fn unique_test_dir(label: &str) -> Result<PathBuf, String> {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("clock error for test dir: {err}"))?
            .as_millis();
        let dir = std::env::temp_dir().join(format!(
            "pgtm-process-planner-{label}-{}-{millis}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir)
            .map_err(|err| format!("create test dir {} failed: {err}", dir.display()))?;
        Ok(dir)
    }

    fn sample_identity() -> ProcessNodeIdentity {
        NodeIdentity {
            cluster_name: ClusterName("cluster-a".to_string()),
            scope: ScopeName("scope-a".to_string()),
            member_id: MemberId("node-a".to_string()),
        }
    }

    fn sample_runtime(data_dir: PathBuf) -> crate::config::RuntimeConfig {
        RuntimeConfigBuilder::new()
            .with_postgres_data_dir(data_dir)
            .build()
    }

    fn primary_member(host: &str, port: u16) -> Result<DcsMemberState, String> {
        Ok(DcsMemberState {
            postgres_endpoint: PgTcpTarget::new(host.to_string(), port)?,
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
        runtime_config: crate::config::RuntimeConfig,
        dcs: DcsSnapshot,
        managed_recovery_state: ManagedRecoverySignal,
    ) -> ProcessObservedSnapshot {
        ProcessObservedSnapshot {
            runtime_config,
            dcs,
            managed_recovery_state,
        }
    }

    #[test]
    fn planner_maps_process_intents_to_expected_plan_variants() -> Result<(), String> {
        let root = unique_test_dir("intent-variants")?;
        let runtime_config = sample_runtime(root.join("data"));
        let leader = MemberId("node-b".to_string());
        let dcs = DcsSnapshot::quorum(
            None,
            SwitchoverState::None,
            BTreeMap::from([(leader.clone(), primary_member("10.0.0.8", 5432)?)]),
        );
        let snapshot = observed_snapshot(runtime_config.clone(), dcs, ManagedRecoverySignal::None);
        let runtime = ProcessRuntimePlan::from_config(&runtime_config);
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
                .plan(&identity, &runtime, &snapshot, &intent)
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
        let root = unique_test_dir("primary-reject")?;
        let runtime_config = sample_runtime(root.join("data"));
        let snapshot = observed_snapshot(
            runtime_config.clone(),
            DcsSnapshot::starting(),
            ManagedRecoverySignal::Standby,
        );
        let runtime = ProcessRuntimePlan::from_config(&runtime_config);
        let planner = ProcessIntentPlanner;
        let error = planner
            .plan(
                &sample_identity(),
                &runtime,
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
        let root = unique_test_dir("source-roles")?;
        let runtime_config = sample_runtime(root.join("data"));
        let leader = MemberId("node-b".to_string());
        let dcs = DcsSnapshot::quorum(
            None,
            SwitchoverState::None,
            BTreeMap::from([(leader.clone(), primary_member("10.0.0.9", 5432)?)]),
        );
        let snapshot = observed_snapshot(runtime_config.clone(), dcs, ManagedRecoverySignal::None);
        let runtime = ProcessRuntimePlan::from_config(&runtime_config);
        let planner = ProcessIntentPlanner;
        let identity = sample_identity();

        let basebackup_plan = planner
            .plan(
                &identity,
                &runtime,
                &snapshot,
                &ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup {
                    leader: leader.clone(),
                }),
            )
            .map_err(|err| format!("plan basebackup failed: {err}"))?;
        let rewind_plan = planner
            .plan(
                &identity,
                &runtime,
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
                &runtime,
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
