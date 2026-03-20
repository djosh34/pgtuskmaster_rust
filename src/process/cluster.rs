use thiserror::Error;

use crate::{
    config_v2::RuntimeConfigV2,
    postgres_managed::inspect_managed_recovery_state,
    process::{
        planner::ProcessIntentPlanner,
        session::ManagedPostgresSessionMaterializer,
        state::{
            ProcessExecutionRequest, ProcessIntentRequest, ProcessObservedSnapshot,
            ProcessWorkerCtx,
        },
        tools::ExternalToolLowerer,
    },
    state::NodeIdentity,
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

pub(crate) struct ProcessCluster<'a> {
    cfg: &'a RuntimeConfigV2,
    identity: NodeIdentity,
    observed: ProcessObservedSnapshot,
    planner: ProcessIntentPlanner,
    sessions: ManagedPostgresSessionMaterializer,
    tools: ExternalToolLowerer,
}

impl<'a> ProcessCluster<'a> {
    pub(crate) fn production_from_ctx(ctx: &'a ProcessWorkerCtx<'a>) -> Result<Self, ProcessError> {
        let managed_recovery_state =
            inspect_managed_recovery_state(ctx.cfg.postgres.data_dir.as_path()).map_err(|err| {
                ProcessError::InvalidSpec(format!("inspect managed recovery state failed: {err}"))
            })?;
        Ok(Self::from_snapshot(
            ctx.cfg,
            ctx.identity.clone(),
            ProcessObservedSnapshot {
                dcs: ctx.observed.dcs.latest(),
                managed_recovery_state,
            },
        ))
    }

    pub(crate) fn from_snapshot(
        cfg: &'a RuntimeConfigV2,
        identity: NodeIdentity,
        observed: ProcessObservedSnapshot,
    ) -> Self {
        Self {
            cfg,
            identity,
            observed,
            planner: ProcessIntentPlanner,
            sessions: ManagedPostgresSessionMaterializer,
            tools: ExternalToolLowerer,
        }
    }

    pub(crate) fn prepare(
        &self,
        request: &ProcessIntentRequest,
    ) -> Result<PreparedProcessLaunch, ProcessPreparationError> {
        let plan = self
            .planner
            .plan(&self.identity, self.cfg, &self.observed, &request.intent)
            .map_err(ProcessPreparationError::Planning)?;
        let prepared_session = self
            .sessions
            .materialize(self.cfg, &plan)
            .map_err(ProcessPreparationError::SessionMaterialization)?;
        let execution_request = self
            .tools
            .lower_execution_request(
                request.id.clone(),
                self.cfg,
                &plan,
                &self.observed,
                prepared_session.as_ref(),
            )
            .map_err(ProcessPreparationError::ToolLowering)?;
        let command = self
            .tools
            .build_command(self.cfg, &execution_request.kind)
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
        config_v2::load_runtime_config,
        dcs::{DcsMemberState, DcsSnapshot},
        dev_support::runtime_config::RuntimeConfigBuilder,
        pginfo::conninfo::PgConnInfo,
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        postgres_managed_conf::ManagedRecoverySignal,
        process::{
            jobs::{PostgresStartIntent, ProcessIntent, ReplicaProvisionIntent},
            state::{ProcessIntentRequest, ProcessObservedSnapshot},
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

    fn write_runtime_config_v2_with_source_ca(root: &std::path::Path) -> Result<PathBuf, String> {
        let data_dir = root.join("data");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        let ca_cert = root.join("source-ca.crt");
        fs::write(&ca_cert, "test ca")
            .map_err(|err| format!("write ca cert {} failed: {err}", ca_cert.display()))?;
        let config_path = root.join("runtime.toml");
        fs::write(
            &config_path,
            format!(
                r#"
[cluster]
name = "cluster-a"
scope = "scope-a"
member_id = "node-a"

[postgres.paths]
data_dir = "{}"

[postgres.rewind.transport]
ssl_mode = "verify-full"
ca_cert = {{ path = "{}" }}

[postgres.roles.mandatory.superuser]
username = "postgres"
auth = {{ type = "password", password = {{ type = "string", value = "postgres" }} }}

[postgres.roles.mandatory.replicator]
username = "replicator"
auth = {{ type = "password", password = {{ type = "string", value = "replicator" }} }}

[postgres.roles.mandatory.rewinder]
username = "rewinder"
auth = {{ type = "password", password = {{ type = "string", value = "rewinder" }} }}

[postgres.access]
hba = {{ content = "host all all 127.0.0.1/32 trust" }}
ident = {{ content = "" }}

[dcs]
endpoints = ["http://127.0.0.1:2379"]

[process.binaries.overrides]
postgres = "/bin/true"
pg_ctl = "/bin/true"
initdb = "/bin/true"
pg_rewind = "/bin/true"
pg_basebackup = "/bin/true"
psql = "/bin/true"
"#,
                data_dir.display(),
                ca_cert.display()
            ),
        )
        .map_err(|err| format!("write runtime config {} failed: {err}", config_path.display()))?;
        Ok(config_path)
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
        let cfg =
            crate::dev_support::runtime_config_v2::from_legacy_runtime_config(runtime_config)?;
        let leader = MemberId("node-b".to_string());
        let snapshot = ProcessObservedSnapshot {
            dcs: DcsSnapshot::quorum(
                None,
                SwitchoverState::None,
                BTreeMap::from([(leader.clone(), primary_member("10.0.0.13", 5432)?)]),
            ),
            managed_recovery_state: ManagedRecoverySignal::None,
        };
        let cluster = ProcessCluster::from_snapshot(&cfg, sample_identity(), snapshot);
        let request = ProcessIntentRequest {
            id: JobId("job-start-replica".to_string()),
            intent: ProcessIntent::Start(PostgresStartIntent::Replica { leader }),
        };

        let prepared = cluster
            .prepare(&request)
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

    #[test]
    fn prepare_basebackup_from_config_v2_preserves_sslrootcert_in_conninfo() -> Result<(), String> {
        let root = unique_test_dir("basebackup-source-ca")?;
        let config_path = write_runtime_config_v2_with_source_ca(root.as_path())?;
        let cfg = load_runtime_config(config_path.as_path()).map_err(|err| err.to_string())?;
        let leader = MemberId("node-b".to_string());
        let snapshot = ProcessObservedSnapshot {
            dcs: DcsSnapshot::quorum(
                None,
                SwitchoverState::None,
                BTreeMap::from([(leader.clone(), primary_member("10.0.0.13", 5432)?)]),
            ),
            managed_recovery_state: ManagedRecoverySignal::None,
        };
        let cluster = ProcessCluster::from_snapshot(&cfg, sample_identity(), snapshot);
        let request = ProcessIntentRequest {
            id: JobId("job-basebackup".to_string()),
            intent: ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup { leader }),
        };

        let prepared = cluster
            .prepare(&request)
            .map_err(|err| format!("prepare basebackup failed: {err}"))?;

        if prepared.command.job_kind != crate::process::jobs::ProcessJobKind::BaseBackup {
            return Err(format!(
                "unexpected prepared command job kind: {:?}",
                prepared.command.job_kind
            ));
        }
        let conninfo_arg = prepared
            .command
            .args
            .windows(2)
            .find_map(|window| {
                if window[0] == "--dbname" {
                    Some(window[1].clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| "basebackup command did not include --dbname".to_string())?;
        let conninfo: PgConnInfo = conninfo_arg.parse()?;
        let expected_root_cert = config_path
            .parent()
            .ok_or_else(|| {
                format!(
                    "config path {} unexpectedly had no parent directory",
                    config_path.display()
                )
            })?
            .join("source-ca.crt");
        if conninfo.tls.root_cert != Some(expected_root_cert.clone()) {
            return Err(format!(
                "basebackup conninfo lost sslrootcert, expected {}, observed {:?}",
                expected_root_cert.display(),
                conninfo.tls.root_cert
            ));
        }

        Ok(())
    }
}
