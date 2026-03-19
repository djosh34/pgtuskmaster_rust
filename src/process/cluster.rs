use thiserror::Error;

use crate::{
    postgres_managed::inspect_managed_recovery_state,
    process::{
        planner::ProcessIntentPlanner,
        session::ManagedPostgresSessionMaterializer,
        state::{
            ProcessExecutionRequest, ProcessIntentRequest, ProcessObservedSnapshot,
            ProcessRuntimePlan, ProcessWorkerCtx,
        },
        tools::ExternalToolLowerer,
    },
};

use super::jobs::{ProcessCommandSpec, ProcessError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreparedProcessLaunch {
    pub(crate) request: ProcessExecutionRequest,
    pub(crate) command: ProcessCommandSpec,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ProcessPreparationError {
    #[error("process planning failed: {0}")]
    Planning(ProcessError),
    #[error("managed session materialization failed: {0}")]
    SessionMaterialization(ProcessError),
    #[error("external tool lowering failed: {0}")]
    ToolLowering(ProcessError),
}

impl ProcessPreparationError {
    pub(crate) fn into_process_error(self) -> ProcessError {
        match self {
            Self::Planning(error)
            | Self::SessionMaterialization(error)
            | Self::ToolLowering(error) => error,
        }
    }

    pub(crate) fn stage_label(&self) -> &'static str {
        match self {
            Self::Planning(_) => "planning",
            Self::SessionMaterialization(_) => "managed session materialization",
            Self::ToolLowering(_) => "external tool lowering",
        }
    }
}

pub(crate) struct ProcessCluster {
    identity: crate::process::state::ProcessNodeIdentity,
    runtime: ProcessRuntimePlan,
    observed: ProcessObservedSnapshot,
    planner: ProcessIntentPlanner,
    sessions: ManagedPostgresSessionMaterializer,
    tools: ExternalToolLowerer,
}

impl ProcessCluster {
    pub(crate) fn production_from_ctx(ctx: &ProcessWorkerCtx) -> Result<Self, ProcessError> {
        let runtime_config = ctx.observed.runtime_config.latest();
        let managed_recovery_state =
            inspect_managed_recovery_state(runtime_config.postgres.paths.data_dir.as_path())
                .map_err(|err| {
                    ProcessError::InvalidSpec(format!(
                        "inspect managed recovery state failed: {err}"
                    ))
                })?;
        Ok(Self::from_snapshot(
            ctx.identity.clone(),
            ctx.plan.clone(),
            ProcessObservedSnapshot {
                dcs: ctx.observed.dcs.latest(),
                runtime_config,
                managed_recovery_state,
            },
        ))
    }

    pub(crate) fn from_snapshot(
        identity: crate::process::state::ProcessNodeIdentity,
        runtime: ProcessRuntimePlan,
        observed: ProcessObservedSnapshot,
    ) -> Self {
        Self {
            identity,
            runtime,
            observed,
            planner: ProcessIntentPlanner,
            sessions: ManagedPostgresSessionMaterializer,
            tools: ExternalToolLowerer,
        }
    }

    pub(crate) fn prepare(
        &self,
        request: &ProcessIntentRequest,
        config: &crate::config::ProcessConfig,
        capture_output: bool,
    ) -> Result<PreparedProcessLaunch, ProcessPreparationError> {
        let plan = self
            .planner
            .plan(
                &self.identity,
                &self.runtime,
                &self.observed,
                &request.intent,
            )
            .map_err(ProcessPreparationError::Planning)?;
        let prepared_session = self
            .sessions
            .materialize(&self.observed.runtime_config, &self.runtime, &plan)
            .map_err(ProcessPreparationError::SessionMaterialization)?;
        let execution_request = self
            .tools
            .lower_execution_request(
                request.id.clone(),
                &plan,
                &self.runtime,
                &self.observed,
                prepared_session.as_ref(),
            )
            .map_err(ProcessPreparationError::ToolLowering)?;
        let command = self
            .tools
            .build_command(config, &execution_request.kind, capture_output)
            .map_err(ProcessPreparationError::ToolLowering)?;
        Ok(PreparedProcessLaunch {
            request: execution_request,
            command,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        dcs::{DcsMemberState, DcsSnapshot},
        dev_support::runtime_config::{sample_binary_paths, RuntimeConfigBuilder},
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        postgres_managed_conf::ManagedRecoverySignal,
        process::{
            jobs::{PostgresStartIntent, ProcessIntent},
            state::{ProcessIntentRequest, ProcessObservedSnapshot, ProcessRuntimePlan},
        },
        state::{
            ClusterName, JobId, MemberId, NodeIdentity, PgEndpoint, ScopeName, SwitchoverState,
            SystemIdentifier, TimelineId, UnixMillis, WalLsn, WorkerStatus,
        },
    };

    use super::ProcessCluster;

    fn unique_test_dir(label: &str) -> Result<PathBuf, String> {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("clock error for test dir: {err}"))?
            .as_millis();
        let dir = std::env::temp_dir().join(format!(
            "pgtm-process-cluster-{label}-{}-{millis}",
            std::process::id()
        ));
        fs::create_dir_all(&dir)
            .map_err(|err| format!("create test dir {} failed: {err}", dir.display()))?;
        Ok(dir)
    }

    fn sample_identity() -> NodeIdentity {
        NodeIdentity {
            cluster_name: ClusterName("cluster-a".to_string()),
            scope: ScopeName("scope-a".to_string()),
            member_id: MemberId("node-a".to_string()),
        }
    }

    fn sample_runtime_config(data_dir: PathBuf) -> crate::config::RuntimeConfig {
        RuntimeConfigBuilder::new()
            .with_postgres_data_dir(data_dir)
            .build()
    }

    fn primary_member(host: &str, port: u16) -> Result<DcsMemberState, String> {
        Ok(DcsMemberState {
            postgres_endpoint: PgEndpoint::tcp(host.to_string(), port)?,
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
                wal_lsn: WalLsn(91),
                slots: Vec::new(),
            },
        })
    }

    #[test]
    fn prepare_replica_start_runs_through_planner_session_and_tool_layers() -> Result<(), String> {
        let root = unique_test_dir("replica-start")?;
        let data_dir = root.join("data");
        let runtime_config = sample_runtime_config(data_dir.clone());
        let runtime = ProcessRuntimePlan::from_config(&runtime_config);
        let leader = MemberId("node-b".to_string());
        let snapshot = ProcessObservedSnapshot {
            runtime_config: runtime_config.clone(),
            dcs: DcsSnapshot::quorum(
                None,
                SwitchoverState::None,
                BTreeMap::from([(leader.clone(), primary_member("10.0.0.13", 5432)?)]),
            ),
            managed_recovery_state: ManagedRecoverySignal::None,
        };
        let cluster = ProcessCluster::from_snapshot(sample_identity(), runtime, snapshot);
        let request = ProcessIntentRequest {
            id: JobId("job-start-replica".to_string()),
            intent: ProcessIntent::Start(PostgresStartIntent::Replica { leader }),
        };

        let prepared = cluster
            .prepare(
                &request,
                &crate::config::ProcessConfig {
                    binaries: sample_binary_paths(),
                    ..runtime_config.process.clone()
                },
                true,
            )
            .map_err(|err| format!("prepare replica start failed: {err}"))?;

        if prepared.command.job_kind != crate::process::jobs::ProcessJobKind::StartPostgres {
            return Err(format!(
                "unexpected prepared command job kind: {:?}",
                prepared.command.job_kind
            ));
        }
        match prepared.request.kind {
            crate::process::state::ProcessExecutionKind::StartPostgres(spec) => {
                if spec.mode != crate::process::jobs::PostgresStartMode::Replica {
                    return Err(format!("unexpected start mode: {:?}", spec.mode));
                }
                if !spec.config_file.exists() {
                    return Err(format!(
                        "expected prepared managed config file to exist at {}",
                        spec.config_file.display()
                    ));
                }
            }
            other => return Err(format!("unexpected execution request kind: {other:?}")),
        }

        Ok(())
    }
}
