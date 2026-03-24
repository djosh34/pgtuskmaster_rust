use thiserror::Error;

use crate::{
    config_v2::RuntimeConfigV2,
    dcs::{DcsMemberState, DcsSnapshot},
    pginfo::state::{PgConnInfo, PgInfoState},
    postgres_managed::{ManagedRecoverySignal, MANAGED_POSTGRESQL_CONF_NAME},
    process::{
        jobs::{
            BaseBackupSpec, BootstrapSpec, DemoteSpec, MandatoryRoleSourceConn,
            MandatorySourceRole, PgRewindSpec, PostgresStartIntent, ProcessError,
            ProcessExecutionKind, ProcessIntent, PromoteSpec, ReplicaProvisionIntent,
            StartPostgresSpec,
        },
        state::ProcessObservedSnapshot,
    },
    state::MemberId,
};

#[derive(Clone, Debug, Error, PartialEq, Eq)]
enum SourceMaterializationError {
    #[error("remote source member `{member_id}` is self")]
    SelfTarget { member_id: String },
    #[error("remote source member `{member_id}` is not a healthy primary")]
    NotHealthyPrimary { member_id: String },
    #[error("remote source member `{member_id}` has an empty postgres host")]
    EmptyHost { member_id: String },
}

#[derive(Default)]
pub(crate) struct ProcessIntentPlanner;

impl ProcessIntentPlanner {
    pub(crate) fn plan(
        &self,
        self_id: &MemberId,
        cfg: &RuntimeConfigV2,
        observed: &ProcessObservedSnapshot,
        intent: &ProcessIntent,
    ) -> Result<ProcessExecutionKind, ProcessError> {
        match intent {
            ProcessIntent::Bootstrap => Ok(ProcessExecutionKind::Bootstrap(BootstrapSpec {
                data_dir: cfg.postgres.data_dir.clone(),
                superuser: cfg.postgres.superuser.username.clone(),
                timeout_ms: None,
            })),
            ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup { leader }) => {
                let source = basebackup_source_from_leader(self_id, cfg, &observed.dcs, leader)?;
                Ok(ProcessExecutionKind::BaseBackup(BaseBackupSpec {
                    data_dir: cfg.postgres.data_dir.clone(),
                    source,
                    timeout_ms: Some(duration_millis_u64(cfg.timing.bootstrap_timeout)),
                }))
            }
            ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::PgRewind { leader }) => {
                let (source_member_id, source_member) =
                    resolve_source_member(&observed.dcs, leader)?;
                let source = source_from_member(
                    self_id,
                    cfg,
                    source_member_id,
                    source_member,
                    MandatorySourceRole::Rewinder,
                )
                .map_err(source_materialization_error)?;
                Ok(ProcessExecutionKind::PgRewind(PgRewindSpec {
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
                Ok(ProcessExecutionKind::StartPostgres(start_postgres_spec(
                    cfg, None, None,
                )))
            }
            ProcessIntent::Start(PostgresStartIntent::DetachedStandby) => Ok(
                ProcessExecutionKind::StartPostgres(start_postgres_spec(cfg, None, None)),
            ),
            ProcessIntent::Start(PostgresStartIntent::Replica { leader }) => {
                let source = basebackup_source_from_leader(self_id, cfg, &observed.dcs, leader)?;
                Ok(ProcessExecutionKind::StartPostgres(start_postgres_spec(
                    cfg,
                    Some(source.conninfo),
                    None,
                )))
            }
            ProcessIntent::Promote => Ok(ProcessExecutionKind::Promote(PromoteSpec {
                data_dir: cfg.postgres.data_dir.clone(),
                wait_seconds: None,
                timeout_ms: None,
            })),
            ProcessIntent::Demote(mode) => Ok(ProcessExecutionKind::Demote(DemoteSpec {
                data_dir: cfg.postgres.data_dir.clone(),
                mode: mode.clone(),
                timeout_ms: None,
            })),
        }
    }
}

fn start_postgres_spec(
    cfg: &RuntimeConfigV2,
    primary_conninfo: Option<PgConnInfo>,
    primary_slot_name: Option<String>,
) -> StartPostgresSpec {
    StartPostgresSpec {
        data_dir: cfg.postgres.data_dir.clone(),
        config_file: cfg.postgres.data_dir.join(MANAGED_POSTGRESQL_CONF_NAME),
        log_file: cfg.postgres.log_file.clone(),
        primary_conninfo,
        primary_slot_name,
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

fn source_materialization_error(error: SourceMaterializationError) -> ProcessError {
    ProcessError::InvalidSpec(error.to_string())
}

fn source_from_member(
    self_id: &MemberId,
    cfg: &RuntimeConfigV2,
    member_id: &MemberId,
    member: &DcsMemberState,
    role: MandatorySourceRole,
) -> Result<MandatoryRoleSourceConn, SourceMaterializationError> {
    if member_id == self_id {
        return Err(SourceMaterializationError::SelfTarget {
            member_id: member_id.0.clone(),
        });
    }

    if member.cluster_postgres_target().host().trim().is_empty() {
        return Err(SourceMaterializationError::EmptyHost {
            member_id: member_id.0.clone(),
        });
    }

    if !matches!(member.postgres(), PgInfoState::Primary { .. }) {
        return Err(SourceMaterializationError::NotHealthyPrimary {
            member_id: member_id.0.clone(),
        });
    }

    let credential = match role {
        MandatorySourceRole::Replicator => &cfg.postgres.replicator,
        MandatorySourceRole::Rewinder => &cfg.postgres.rewinder,
    };
    Ok(MandatoryRoleSourceConn {
        role,
        conninfo: PgConnInfo {
            route: member.cluster_postgres_target().clone(),
            user: credential.username.clone(),
            dbname: cfg.postgres.local_database.clone(),
            application_name: None,
            connect_timeout_s: Some(
                u32::try_from(cfg.postgres.connect_timeout.as_secs()).unwrap_or(u32::MAX),
            ),
            options: None,
            tls: cfg.postgres.source_client_tls.clone(),
        },
        auth: credential.password.clone(),
    })
}

fn duration_millis_u64(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        config_v2::{runtime_test_config_with_data_dir, RuntimeConfigV2},
        dcs::{DcsMemberState, DcsSnapshot},
        dev_support::test_fs::unique_test_dir,
        pginfo::{
            conninfo::{PgClientTls, PgSslMode},
            state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        },
        postgres_managed::ManagedRecoverySignal,
        process::{
            jobs::{
                MandatorySourceRole, PostgresStartIntent, ProcessExecutionKind, ProcessIntent,
                ReplicaProvisionIntent, ShutdownMode, StartPostgresSpec,
            },
            state::ProcessObservedSnapshot,
        },
        state::{
            MemberId, PgRoute, SwitchoverState, SystemIdentifier, TimelineId, UnixMillis, WalLsn,
            WorkerStatus,
        },
    };

    use super::{source_from_member, ProcessIntentPlanner, SourceMaterializationError};

    fn primary_member(host: &str, port: u16) -> Result<DcsMemberState, String> {
        Ok(DcsMemberState {
            cluster_postgres: PgRoute::tcp(host.to_string(), port)?,
            operator_postgres: None,
            operator_api: None,
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

    fn replica_member(host: &str, port: u16) -> Result<DcsMemberState, String> {
        Ok(DcsMemberState {
            cluster_postgres: PgRoute::tcp(host.to_string(), port)?,
            operator_postgres: None,
            operator_api: None,
            postgres: PgInfoState::Replica {
                common: PgInfoCommon {
                    worker: WorkerStatus::Running,
                    sql: SqlStatus::Healthy,
                    readiness: Readiness::Ready,
                    timeline: Some(TimelineId(7)),
                    system_identifier: Some(SystemIdentifier(41)),
                    pg_config: PgConfig {
                        port: Some(port),
                        hot_standby: Some(true),
                        primary_conninfo: None,
                        primary_slot_name: None,
                        extra: BTreeMap::new(),
                    },
                    last_refresh_at: Some(UnixMillis(123)),
                },
                replay_lsn: WalLsn(98),
                follow_lsn: Some(WalLsn(99)),
                upstream: Some(crate::pginfo::state::UpstreamInfo {
                    member_id: MemberId("node-a".to_string()),
                }),
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
        let cfg =
            runtime_test_config_with_data_dir(root.join("data")).map_err(|err| err.to_string())?;
        let leader = MemberId("node-b".to_string());
        let dcs = DcsSnapshot::quorum(
            None,
            SwitchoverState::None,
            BTreeMap::from([(leader.clone(), primary_member("10.0.0.8", 5432)?)]),
        );
        let snapshot = observed_snapshot(dcs, ManagedRecoverySignal::None);
        let planner = ProcessIntentPlanner;
        let self_id = cfg.member_id.clone();

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
                .plan(&self_id, &cfg, &snapshot, &intent)
                .map_err(|err| format!("planning {label} failed: {err}"))?;
            let matches_expected = matches!(
                (&plan, label),
                (ProcessExecutionKind::Bootstrap(_), "bootstrap")
                    | (ProcessExecutionKind::StartPostgres(_), "start-primary")
                    | (
                        ProcessExecutionKind::StartPostgres(_),
                        "start-detached-standby"
                    )
                    | (ProcessExecutionKind::StartPostgres(_), "start-replica")
                    | (ProcessExecutionKind::BaseBackup(_), "basebackup")
                    | (ProcessExecutionKind::PgRewind(_), "pg-rewind")
                    | (ProcessExecutionKind::Promote(_), "promote")
                    | (ProcessExecutionKind::Demote(_), "demote")
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
        let cfg =
            runtime_test_config_with_data_dir(root.join("data")).map_err(|err| err.to_string())?;
        let snapshot = observed_snapshot(DcsSnapshot::starting(), ManagedRecoverySignal::Standby);
        let planner = ProcessIntentPlanner;
        let error = planner
            .plan(
                &cfg.member_id,
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
        let cfg =
            runtime_test_config_with_data_dir(root.join("data")).map_err(|err| err.to_string())?;
        let leader = MemberId("node-b".to_string());
        let dcs = DcsSnapshot::quorum(
            None,
            SwitchoverState::None,
            BTreeMap::from([(leader.clone(), primary_member("10.0.0.9", 5432)?)]),
        );
        let snapshot = observed_snapshot(dcs, ManagedRecoverySignal::None);
        let planner = ProcessIntentPlanner;
        let self_id = cfg.member_id.clone();

        let basebackup_plan = planner
            .plan(
                &self_id,
                &cfg,
                &snapshot,
                &ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup {
                    leader: leader.clone(),
                }),
            )
            .map_err(|err| format!("plan basebackup failed: {err}"))?;
        let rewind_plan = planner
            .plan(
                &self_id,
                &cfg,
                &snapshot,
                &ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::PgRewind { leader }),
            )
            .map_err(|err| format!("plan rewind failed: {err}"))?;

        let basebackup_role = match basebackup_plan {
            ProcessExecutionKind::BaseBackup(spec) => spec.source.role,
            other => return Err(format!("unexpected basebackup plan: {other:?}")),
        };
        let rewind_role = match rewind_plan {
            ProcessExecutionKind::PgRewind(spec) => spec.source.role,
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
                &self_id,
                &cfg,
                &snapshot,
                &ProcessIntent::Start(PostgresStartIntent::Replica {
                    leader: MemberId("node-b".to_string()),
                }),
            )
            .map_err(|err| format!("plan replica start failed: {err}"))?;
        match replica_plan {
            ProcessExecutionKind::StartPostgres(StartPostgresSpec {
                primary_conninfo: Some(primary_conninfo),
                primary_slot_name,
                ..
            }) => {
                if primary_conninfo.user != "replicator" {
                    return Err(format!(
                        "replica start should use replicator user, observed `{}`",
                        primary_conninfo.user
                    ));
                }
                if primary_slot_name.is_some() {
                    return Err(format!(
                        "replica start should not set primary_slot_name, observed {primary_slot_name:?}"
                    ));
                }
            }
            other => return Err(format!("unexpected replica start plan: {other:?}")),
        }

        Ok(())
    }

    #[test]
    fn source_from_member_selects_role_specific_credentials() -> Result<(), String> {
        let root = unique_test_dir("process-planner", "source-credentials")?;
        let cfg =
            runtime_test_config_with_data_dir(root.join("data")).map_err(|err| err.to_string())?;
        let member_id = MemberId("node-b".to_string());
        let member = primary_member("10.0.0.9", 5432)?;

        let replicator = source_from_member(
            &MemberId("node-a".to_string()),
            &cfg,
            &member_id,
            &member,
            MandatorySourceRole::Replicator,
        )
        .map_err(|err| err.to_string())?;
        let rewinder = source_from_member(
            &MemberId("node-a".to_string()),
            &cfg,
            &member_id,
            &member,
            MandatorySourceRole::Rewinder,
        )
        .map_err(|err| err.to_string())?;

        assert_eq!(replicator.role, MandatorySourceRole::Replicator);
        assert_eq!(replicator.conninfo.user, cfg.postgres.replicator.username);
        assert_eq!(
            replicator.conninfo.connect_timeout_s,
            Some(u32::try_from(cfg.postgres.connect_timeout.as_secs()).unwrap_or(u32::MAX))
        );
        assert_eq!(replicator.conninfo.tls, cfg.postgres.source_client_tls);
        assert_eq!(replicator.auth, cfg.postgres.replicator.password.clone());

        assert_eq!(rewinder.role, MandatorySourceRole::Rewinder);
        assert_eq!(rewinder.conninfo.user, cfg.postgres.rewinder.username);
        assert_eq!(rewinder.conninfo.tls, cfg.postgres.source_client_tls);
        assert_eq!(rewinder.auth, cfg.postgres.rewinder.password.clone());
        Ok(())
    }

    #[test]
    fn source_from_member_uses_shared_source_client_tls_for_all_roles() -> Result<(), String> {
        let root = unique_test_dir("process-planner", "source-shared-tls")?;
        let config =
            runtime_test_config_with_data_dir(root.join("data")).map_err(|err| err.to_string())?;
        let cfg = RuntimeConfigV2 {
            postgres: crate::config_v2::types::PostgresConfig {
                source_client_tls: PgClientTls {
                    mode: PgSslMode::VerifyFull,
                    root_cert: Some("/tmp/pgtm/source-ca.crt".into()),
                    client_cert: None,
                    client_key: None,
                },
                ..config.postgres
            },
            ..config
        };
        let member_id = MemberId("node-b".to_string());
        let member = primary_member("10.0.0.9", 5432)?;

        let replicator = source_from_member(
            &MemberId("node-a".to_string()),
            &cfg,
            &member_id,
            &member,
            MandatorySourceRole::Replicator,
        )
        .map_err(|err| err.to_string())?;
        let rewinder = source_from_member(
            &MemberId("node-a".to_string()),
            &cfg,
            &member_id,
            &member,
            MandatorySourceRole::Rewinder,
        )
        .map_err(|err| err.to_string())?;

        assert_eq!(cfg.postgres.source_client_tls.mode, PgSslMode::VerifyFull);
        assert_eq!(
            cfg.postgres.source_client_tls.root_cert,
            Some("/tmp/pgtm/source-ca.crt".into())
        );
        assert_eq!(replicator.conninfo.tls, cfg.postgres.source_client_tls);
        assert_eq!(rewinder.conninfo.tls, cfg.postgres.source_client_tls);
        Ok(())
    }

    #[test]
    fn source_from_member_rejects_empty_host_route() -> Result<(), String> {
        let root = unique_test_dir("process-planner", "source-empty-host")?;
        let cfg =
            runtime_test_config_with_data_dir(root.join("data")).map_err(|err| err.to_string())?;
        let member_id = MemberId("node-b".to_string());
        let member = DcsMemberState {
            cluster_postgres: PgRoute::unix_socket("/tmp/pgtm".into(), 5432)?,
            operator_postgres: None,
            operator_api: None,
            postgres: primary_member("10.0.0.9", 5432)?.postgres,
        };

        assert_eq!(
            source_from_member(
                &MemberId("node-a".to_string()),
                &cfg,
                &member_id,
                &member,
                MandatorySourceRole::Replicator,
            ),
            Err(SourceMaterializationError::EmptyHost {
                member_id: "node-b".to_string(),
            })
        );
        Ok(())
    }

    #[test]
    fn source_from_member_rejects_non_primary_source() -> Result<(), String> {
        let root = unique_test_dir("process-planner", "source-non-primary")?;
        let cfg =
            runtime_test_config_with_data_dir(root.join("data")).map_err(|err| err.to_string())?;
        let member_id = MemberId("node-b".to_string());
        let member = replica_member("10.0.0.9", 5432)?;

        assert_eq!(
            source_from_member(
                &MemberId("node-a".to_string()),
                &cfg,
                &member_id,
                &member,
                MandatorySourceRole::Replicator,
            ),
            Err(SourceMaterializationError::NotHealthyPrimary {
                member_id: "node-b".to_string(),
            })
        );
        Ok(())
    }

    #[test]
    fn source_from_member_rejects_self_target() -> Result<(), String> {
        let root = unique_test_dir("process-planner", "source-self-target")?;
        let cfg =
            runtime_test_config_with_data_dir(root.join("data")).map_err(|err| err.to_string())?;
        let member_id = MemberId("node-a".to_string());
        let member = primary_member("10.0.0.9", 5432)?;

        assert_eq!(
            source_from_member(
                &member_id,
                &cfg,
                &member_id,
                &member,
                MandatorySourceRole::Replicator,
            ),
            Err(SourceMaterializationError::SelfTarget {
                member_id: "node-a".to_string(),
            })
        );
        Ok(())
    }
}
