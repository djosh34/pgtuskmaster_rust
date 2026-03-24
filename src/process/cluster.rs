use std::{fs, path::Path};

use thiserror::Error;

use crate::{
    config_v2::{types::Secret, RuntimeConfigV2},
    postgres_managed::{inspect_managed_recovery_state, materialize_managed_postgres_config},
    postgres_managed_conf::ManagedPostgresStartIntent,
    process::{
        jobs::{
            PostgresStartMode, ProcessCommandSpec, ProcessEnvValue, ProcessEnvVar, ProcessError,
            ProcessJobKind, StartPostgresSpec,
        },
        planner::{ClusterProcessPlan, ProcessIntentPlanner},
        state::{
            ensure_start_paths, ProcessExecutionKind, ProcessExecutionRequest,
            ProcessIntentRequest, ProcessObservedSnapshot, ProcessWorkerCtx,
        },
    },
};

const PG_CTL_DEFAULT_WAIT_SECONDS: u64 = 30;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreparedProcessLaunch {
    pub(crate) request: ProcessExecutionRequest,
    pub(crate) command: ProcessCommandSpec,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ProcessPreparationError {
    #[error("planning snapshot failed: {0}")]
    Snapshot(ProcessError),
    #[error("intent materialization failed: {0}")]
    IntentMaterialization(ProcessError),
    #[error("build command failed: {0}")]
    BuildCommand(ProcessError),
}

impl ProcessPreparationError {
    pub(crate) fn into_process_error(self) -> ProcessError {
        match self {
            Self::Snapshot(error)
            | Self::IntentMaterialization(error)
            | Self::BuildCommand(error) => error,
        }
    }
}

pub(crate) fn prepare_process_launch_from_ctx(
    ctx: &ProcessWorkerCtx<'_>,
    request: &ProcessIntentRequest,
) -> Result<PreparedProcessLaunch, ProcessPreparationError> {
    let observed = observed_snapshot_from_ctx(ctx)?;
    prepare_process_launch(ctx.cfg, &observed, request)
}

pub(crate) fn prepare_process_launch(
    cfg: &RuntimeConfigV2,
    observed: &ProcessObservedSnapshot,
    request: &ProcessIntentRequest,
) -> Result<PreparedProcessLaunch, ProcessPreparationError> {
    let plan = ProcessIntentPlanner
        .plan(&cfg.member_id, cfg, observed, &request.intent)
        .map_err(ProcessPreparationError::IntentMaterialization)?;
    let execution_request =
        execution_request_from_plan(request.id.clone(), request.intent.job_kind(), cfg, &plan)
            .map_err(ProcessPreparationError::IntentMaterialization)?;
    let command = build_command(cfg, &execution_request.kind)
        .map_err(ProcessPreparationError::BuildCommand)?;
    Ok(PreparedProcessLaunch {
        request: execution_request,
        command,
    })
}

fn observed_snapshot_from_ctx(
    ctx: &ProcessWorkerCtx<'_>,
) -> Result<ProcessObservedSnapshot, ProcessPreparationError> {
    let managed_recovery_state =
        inspect_managed_recovery_state(ctx.cfg.postgres.data_dir.as_path()).map_err(|err| {
            ProcessPreparationError::Snapshot(ProcessError::InvalidSpec(format!(
                "inspect managed recovery state failed: {err}"
            )))
        })?;
    Ok(ProcessObservedSnapshot {
        dcs: ctx.observed.dcs.latest(),
        managed_recovery_state,
    })
}

fn execution_request_from_plan(
    request_id: crate::state::JobId,
    tracked_job_kind: ProcessJobKind,
    cfg: &RuntimeConfigV2,
    plan: &ClusterProcessPlan,
) -> Result<ProcessExecutionRequest, ProcessError> {
    let kind = match plan {
        ClusterProcessPlan::Bootstrap(spec) => {
            wipe_data_dir(spec.data_dir.as_path())?;
            ProcessExecutionKind::Bootstrap(spec.clone())
        }
        ClusterProcessPlan::BaseBackup(spec) => {
            wipe_data_dir(spec.data_dir.as_path())?;
            ProcessExecutionKind::BaseBackup(spec.clone())
        }
        ClusterProcessPlan::PgRewind(spec) => ProcessExecutionKind::PgRewind(spec.clone()),
        ClusterProcessPlan::StartManagedPostgres(start_intent) => {
            let config = materialize_start_config(cfg, start_intent)?;
            ProcessExecutionKind::StartPostgres(StartPostgresSpec {
                mode: managed_start_mode(start_intent),
                data_dir: cfg.postgres.data_dir.clone(),
                config_file: config.postgresql_conf_path,
                log_file: cfg.postgres.log_file.clone(),
            })
        }
        ClusterProcessPlan::Promote(spec) => ProcessExecutionKind::Promote(spec.clone()),
        ClusterProcessPlan::Demote(spec) => ProcessExecutionKind::Demote(spec.clone()),
    };

    Ok(ProcessExecutionRequest {
        id: request_id,
        tracked_job_kind,
        kind,
    })
}

fn materialize_start_config(
    cfg: &RuntimeConfigV2,
    start_intent: &ManagedPostgresStartIntent,
) -> Result<crate::postgres_managed::ManagedPostgresConfig, ProcessError> {
    ensure_start_paths(cfg)?;
    materialize_managed_postgres_config(cfg, start_intent).map_err(|err| {
        ProcessError::InvalidSpec(format!("materialize managed postgres config failed: {err}"))
    })
}

fn managed_start_mode(start_intent: &ManagedPostgresStartIntent) -> PostgresStartMode {
    match start_intent {
        ManagedPostgresStartIntent::Primary => PostgresStartMode::Primary,
        ManagedPostgresStartIntent::DetachedStandby => PostgresStartMode::DetachedStandby,
        ManagedPostgresStartIntent::Replica { .. } => PostgresStartMode::Replica,
    }
}

fn build_command(
    cfg: &RuntimeConfigV2,
    kind: &ProcessExecutionKind,
) -> Result<ProcessCommandSpec, ProcessError> {
    match kind {
        ProcessExecutionKind::Bootstrap(spec) => {
            validate_non_empty_path("bootstrap.data_dir", &spec.data_dir)?;
            if spec.superuser.as_str().trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "bootstrap.superuser must not be empty".to_string(),
                ));
            }
            let program = cfg.binaries.initdb.clone();
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "-A".to_string(),
                    "trust".to_string(),
                    "-U".to_string(),
                    spec.superuser.as_str().to_string(),
                ],
                env: Vec::new(),
                capture_output: cfg.logging.capture_subprocess_output,
                job_kind: kind.job_kind(),
            })
        }
        ProcessExecutionKind::BaseBackup(spec) => {
            validate_non_empty_path("basebackup.data_dir", &spec.data_dir)?;
            validate_non_empty_pg_endpoint(
                "basebackup.source_conninfo.endpoint",
                spec.source.conninfo.route.endpoint(),
            )?;
            if spec.source.conninfo.user.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "basebackup.source_conninfo.user must not be empty".to_string(),
                ));
            }
            if spec.source.conninfo.dbname.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "basebackup.source_conninfo.dbname must not be empty".to_string(),
                ));
            }
            let program = cfg.binaries.pg_basebackup.clone();
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "--dbname".to_string(),
                    spec.source.conninfo.to_string(),
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "-Fp".to_string(),
                    "-Xs".to_string(),
                ],
                env: role_auth_env(&spec.source.auth),
                capture_output: cfg.logging.capture_subprocess_output,
                job_kind: kind.job_kind(),
            })
        }
        ProcessExecutionKind::PgRewind(spec) => {
            validate_non_empty_path("pg_rewind.target_data_dir", &spec.target_data_dir)?;
            validate_non_empty_pg_endpoint(
                "pg_rewind.source_conninfo.endpoint",
                spec.source.conninfo.route.endpoint(),
            )?;
            if spec.source.conninfo.user.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "pg_rewind.source_conninfo.user must not be empty".to_string(),
                ));
            }
            if spec.source.conninfo.dbname.trim().is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "pg_rewind.source_conninfo.dbname must not be empty".to_string(),
                ));
            }
            let program = cfg.binaries.pg_rewind.clone();
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "--target-pgdata".to_string(),
                    spec.target_data_dir.display().to_string(),
                    "--source-server".to_string(),
                    spec.source.conninfo.to_string(),
                ],
                env: role_auth_env(&spec.source.auth),
                capture_output: cfg.logging.capture_subprocess_output,
                job_kind: kind.job_kind(),
            })
        }
        ProcessExecutionKind::Promote(spec) => {
            validate_non_empty_path("promote.data_dir", &spec.data_dir)?;
            let wait_args = spec
                .wait_seconds
                .into_iter()
                .flat_map(|wait_seconds| ["-t".to_string(), wait_seconds.to_string()])
                .collect::<Vec<_>>();
            let program = cfg.binaries.pg_ctl.clone();
            let args = [
                vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "promote".to_string(),
                    "-w".to_string(),
                ],
                wait_args,
            ]
            .concat();
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args,
                env: Vec::new(),
                capture_output: cfg.logging.capture_subprocess_output,
                job_kind: kind.job_kind(),
            })
        }
        ProcessExecutionKind::Demote(spec) => {
            validate_non_empty_path("demote.data_dir", &spec.data_dir)?;
            let program = cfg.binaries.pg_ctl.clone();
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "stop".to_string(),
                    "-m".to_string(),
                    spec.mode.as_pg_ctl_arg().to_string(),
                    "-w".to_string(),
                ],
                env: Vec::new(),
                capture_output: cfg.logging.capture_subprocess_output,
                job_kind: kind.job_kind(),
            })
        }
        ProcessExecutionKind::StartPostgres(spec) => {
            validate_non_empty_path("start_postgres.data_dir", &spec.data_dir)?;
            validate_non_empty_path("start_postgres.config_file", &spec.config_file)?;
            validate_non_empty_path("start_postgres.log_file", &spec.log_file)?;
            let option_tokens = vec![
                "-c".to_string(),
                format!("config_file={}", spec.config_file.display()),
            ];
            let options = render_pg_ctl_option_string(&option_tokens)?;
            let program = cfg.binaries.pg_ctl.clone();
            Ok(ProcessCommandSpec {
                program: program.clone(),
                args: vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "-l".to_string(),
                    spec.log_file.display().to_string(),
                    "-o".to_string(),
                    options,
                    "start".to_string(),
                    "-w".to_string(),
                    "-t".to_string(),
                    PG_CTL_DEFAULT_WAIT_SECONDS.to_string(),
                ],
                env: Vec::new(),
                capture_output: cfg.logging.capture_subprocess_output,
                job_kind: kind.job_kind(),
            })
        }
    }
}

fn role_auth_env(auth: &Secret) -> Vec<ProcessEnvVar> {
    vec![ProcessEnvVar {
        key: "PGPASSWORD".to_string(),
        value: ProcessEnvValue::Secret(auth.clone()),
    }]
}

fn wipe_data_dir(data_dir: &Path) -> Result<(), ProcessError> {
    if data_dir.as_os_str().is_empty() {
        return Err(ProcessError::InvalidSpec(
            "wipe_data_dir data_dir must not be empty".to_string(),
        ));
    }
    if data_dir.exists() {
        wipe_data_dir_contents(data_dir)?;
    } else {
        fs::create_dir_all(data_dir).map_err(|err| {
            ProcessError::InvalidSpec(format!("wipe_data_dir create_dir_all failed: {err}"))
        })?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(data_dir, fs::Permissions::from_mode(0o700)).map_err(|err| {
            ProcessError::InvalidSpec(format!("wipe_data_dir set_permissions failed: {err}"))
        })?;
    }

    Ok(())
}

fn wipe_data_dir_contents(data_dir: &Path) -> Result<(), ProcessError> {
    let entries = fs::read_dir(data_dir).map_err(|err| {
        ProcessError::InvalidSpec(format!("wipe_data_dir read_dir failed: {err}"))
    })?;
    for entry_result in entries {
        let entry = entry_result.map_err(|err| {
            ProcessError::InvalidSpec(format!("wipe_data_dir read_dir entry failed: {err}"))
        })?;
        let file_type = entry.file_type().map_err(|err| {
            ProcessError::InvalidSpec(format!("wipe_data_dir file_type failed: {err}"))
        })?;
        let path = entry.path();
        if file_type.is_dir() {
            fs::remove_dir_all(&path).map_err(|err| {
                ProcessError::InvalidSpec(format!(
                    "wipe_data_dir remove_dir_all failed for {}: {err}",
                    path.display()
                ))
            })?;
        } else {
            fs::remove_file(&path).map_err(|err| {
                ProcessError::InvalidSpec(format!(
                    "wipe_data_dir remove_file failed for {}: {err}",
                    path.display()
                ))
            })?;
        }
    }
    Ok(())
}

fn validate_non_empty_path(field: &str, value: &Path) -> Result<(), ProcessError> {
    if value.as_os_str().is_empty() {
        return Err(ProcessError::InvalidSpec(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
}

fn validate_non_empty_pg_endpoint(
    field: &str,
    value: &crate::state::PgEndpoint,
) -> Result<(), ProcessError> {
    if value.host().trim().is_empty() {
        return Err(ProcessError::InvalidSpec(format!(
            "{field}.host must not be empty"
        )));
    }
    Ok(())
}

fn render_pg_ctl_option_string(tokens: &[String]) -> Result<String, ProcessError> {
    if tokens.is_empty() {
        return Err(ProcessError::InvalidSpec(
            "pg_ctl options must not be empty".to_string(),
        ));
    }

    let rendered = tokens
        .iter()
        .map(|token| {
            if token.is_empty() {
                return Err(ProcessError::InvalidSpec(
                    "pg_ctl option token must not be empty".to_string(),
                ));
            }

            if token
                .chars()
                .any(|ch| ch == '\0' || ch == '\n' || ch == '\r')
            {
                return Err(ProcessError::InvalidSpec(format!(
                    "pg_ctl option token contains control characters: `{token}`"
                )));
            }

            if token
                .chars()
                .all(|ch| !ch.is_whitespace() && ch != '\'' && ch != '"' && ch != '\\')
            {
                Ok(token.clone())
            } else {
                let escaped = token
                    .chars()
                    .map(|ch| match ch {
                        '\\' => "\\\\".to_string(),
                        '"' => "\\\"".to_string(),
                        other => other.to_string(),
                    })
                    .collect::<String>();
                Ok(format!("\"{escaped}\""))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rendered.join(" "))
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, fs, path::Path, path::PathBuf};

    use crate::{
        config_v2::{
            load_runtime_config_contents, render_runtime_test_config_toml,
            runtime_test_config_with_data_dir, toml_path_source, RuntimeConfigV2,
        },
        dcs::{DcsMemberState, DcsSnapshot},
        dev_support::test_fs::unique_test_dir,
        pginfo::{
            conninfo::{PgClientTls, PgConnInfo},
            state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        },
        postgres_managed_conf::{
            managed_standby_passfile_path, ManagedPostgresStartIntent, ManagedRecoverySignal,
            MANAGED_POSTGRESQL_CONF_NAME,
        },
        process::{
            jobs::{PostgresStartIntent, ProcessIntent, ProcessJobKind, ReplicaProvisionIntent},
            state::ProcessIntentRequest,
        },
        state::{
            JobId, MemberId, PgRoute, SwitchoverState, SystemIdentifier, TimelineId, UnixMillis,
            WalLsn, WorkerStatus,
        },
    };

    use super::prepare_process_launch;

    fn runtime_config_v2_with_source_ca(
        root: &std::path::Path,
    ) -> Result<(RuntimeConfigV2, PathBuf), String> {
        let data_dir = root.join("data");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        let ca_cert = root.join("source-ca.crt");
        let cfg = load_runtime_config_contents(
            render_runtime_test_config_toml(
                "cluster-a",
                "scope-a",
                "node-a",
                (
                    data_dir.as_path(),
                    Path::new("/tmp/pgtm-socket"),
                    Path::new("/tmp/pgtm.log"),
                ),
                ["http://127.0.0.1:2379"],
                [
                    format!(
                        r#"[postgres.rewind.transport]
ssl_mode = "verify-full"
ca_cert = {}"#,
                        toml_path_source(ca_cert.as_path()),
                    ),
                    r#"[process.binaries.overrides]
postgres = "/bin/true"
pg_ctl = "/bin/true"
initdb = "/bin/true"
pg_rewind = "/bin/true"
pg_basebackup = "/bin/true"
psql = "/bin/true""#
                        .to_string(),
                ],
            )
            .map_err(|err| err.to_string())?
            .as_str(),
        )
        .map_err(|err| err.to_string())?;
        Ok((cfg, ca_cert))
    }

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
                wal_lsn: WalLsn(91),
                slots: Vec::new(),
            },
        })
    }

    #[test]
    fn prepare_replica_start_materializes_managed_files_and_builds_start_command(
    ) -> Result<(), String> {
        let root = unique_test_dir("process-cluster", "replica-start")?;
        let data_dir = root.join("data");
        let cfg =
            runtime_test_config_with_data_dir(data_dir.clone()).map_err(|err| err.to_string())?;
        let leader = MemberId("node-b".to_string());
        let observed = crate::process::state::ProcessObservedSnapshot {
            dcs: DcsSnapshot::quorum(
                None,
                SwitchoverState::None,
                BTreeMap::from([(leader.clone(), primary_member("10.0.0.13", 5432)?)]),
            ),
            managed_recovery_state: ManagedRecoverySignal::None,
        };
        let request = ProcessIntentRequest {
            id: JobId("job-start-replica".to_string()),
            intent: ProcessIntent::Start(PostgresStartIntent::Replica { leader }),
        };

        let prepared = prepare_process_launch(&cfg, &observed, &request)
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
                let managed_conf_name = spec
                    .config_file
                    .file_name()
                    .and_then(|value| value.to_str())
                    .ok_or_else(|| "managed postgres config path is not valid UTF-8".to_string())?;
                if managed_conf_name != MANAGED_POSTGRESQL_CONF_NAME {
                    return Err(format!(
                        "unexpected managed config file name: {managed_conf_name}"
                    ));
                }
                let passfile_path = managed_standby_passfile_path(&data_dir);
                if !passfile_path.exists() {
                    return Err(format!(
                        "expected standby passfile to exist at {}",
                        passfile_path.display()
                    ));
                }
            }
            other => return Err(format!("unexpected execution request kind: {other:?}")),
        }

        Ok(())
    }

    #[test]
    fn prepare_basebackup_from_config_v2_preserves_sslrootcert_in_conninfo() -> Result<(), String> {
        let root = unique_test_dir("process-cluster", "basebackup-source-ca")?;
        let (cfg, expected_root_cert) = runtime_config_v2_with_source_ca(root.as_path())?;
        let leader = MemberId("node-b".to_string());
        let observed = crate::process::state::ProcessObservedSnapshot {
            dcs: DcsSnapshot::quorum(
                None,
                SwitchoverState::None,
                BTreeMap::from([(leader.clone(), primary_member("10.0.0.13", 5432)?)]),
            ),
            managed_recovery_state: ManagedRecoverySignal::None,
        };
        let request = ProcessIntentRequest {
            id: JobId("job-basebackup".to_string()),
            intent: ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup { leader }),
        };

        let prepared = prepare_process_launch(&cfg, &observed, &request)
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
        if conninfo.tls.root_cert != Some(expected_root_cert.clone()) {
            return Err(format!(
                "basebackup conninfo lost sslrootcert, expected {}, observed {:?}",
                expected_root_cert.display(),
                conninfo.tls.root_cert
            ));
        }

        Ok(())
    }

    #[test]
    fn prepare_basebackup_wipes_existing_data_dir_contents() -> Result<(), String> {
        let root = unique_test_dir("process-cluster", "wipe-basebackup")?;
        let data_dir = root.join("data");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        let stale = data_dir.join("stale.txt");
        fs::write(&stale, "stale")
            .map_err(|err| format!("write stale file {} failed: {err}", stale.display()))?;

        let cfg =
            runtime_test_config_with_data_dir(data_dir.clone()).map_err(|err| err.to_string())?;
        let observed = crate::process::state::ProcessObservedSnapshot {
            dcs: crate::dcs::DcsSnapshot::starting(),
            managed_recovery_state: ManagedRecoverySignal::None,
        };
        let request = ProcessIntentRequest {
            id: JobId("job-basebackup".to_string()),
            intent: ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup {
                leader: MemberId("node-b".to_string()),
            }),
        };

        let error = prepare_process_launch(&cfg, &observed, &request)
            .err()
            .ok_or_else(|| "expected basebackup prepare to fail without quorum DCS".to_string())?;
        if !stale.exists() {
            return Err(format!(
                "stale file should remain when planning fails before wipe at {}",
                stale.display()
            ));
        }
        if !matches!(
            error,
            super::ProcessPreparationError::IntentMaterialization(_)
        ) {
            return Err(format!(
                "unexpected error type for failed prepare: {error:?}"
            ));
        }

        let leader = MemberId("node-b".to_string());
        let observed = crate::process::state::ProcessObservedSnapshot {
            dcs: crate::dcs::DcsSnapshot::quorum(
                None,
                SwitchoverState::None,
                BTreeMap::from([(leader.clone(), primary_member("10.0.0.11", 5432)?)]),
            ),
            managed_recovery_state: ManagedRecoverySignal::None,
        };
        let request = ProcessIntentRequest {
            id: JobId("job-basebackup-success".to_string()),
            intent: ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup { leader }),
        };

        let prepared = prepare_process_launch(&cfg, &observed, &request)
            .map_err(|err| format!("prepare basebackup failed: {err}"))?;
        if !matches!(
            prepared.request.kind,
            crate::process::state::ProcessExecutionKind::BaseBackup(_)
        ) {
            return Err(format!(
                "unexpected execution request kind: {:?}",
                prepared.request.kind
            ));
        }
        if stale.exists() {
            return Err(format!(
                "basebackup prepare should wipe stale data dir contents at {}",
                stale.display()
            ));
        }

        Ok(())
    }

    #[test]
    fn prepare_non_start_plan_skips_managed_session_materialization() -> Result<(), String> {
        let root = unique_test_dir("process-cluster", "skip-non-start")?;
        let cfg =
            runtime_test_config_with_data_dir(root.join("data")).map_err(|err| err.to_string())?;
        let observed = crate::process::state::ProcessObservedSnapshot {
            dcs: DcsSnapshot::starting(),
            managed_recovery_state: ManagedRecoverySignal::None,
        };
        let request = ProcessIntentRequest {
            id: JobId("job-promote".to_string()),
            intent: ProcessIntent::Promote,
        };

        let prepared = prepare_process_launch(&cfg, &observed, &request)
            .map_err(|err| format!("prepare non-start plan failed: {err}"))?;
        if !matches!(
            prepared.request.kind,
            crate::process::state::ProcessExecutionKind::Promote(_)
        ) {
            return Err(format!(
                "unexpected execution request kind for promote: {:?}",
                prepared.request.kind
            ));
        }
        let passfile_path = managed_standby_passfile_path(&cfg.postgres.data_dir);
        if passfile_path.exists() {
            return Err(format!(
                "non-start plan should not materialize standby passfile at {}",
                passfile_path.display()
            ));
        }

        Ok(())
    }

    #[test]
    fn prepare_start_command_uses_materialized_config_path() -> Result<(), String> {
        let root = unique_test_dir("process-cluster", "start-command")?;
        let data_dir = root.join("data");
        let cfg =
            runtime_test_config_with_data_dir(data_dir.clone()).map_err(|err| err.to_string())?;
        let start_intent = ManagedPostgresStartIntent::replica(
            PgConnInfo {
                route: crate::state::PgRoute::tcp("10.0.0.12".to_string(), 5432)?,
                user: "replicator".to_string(),
                dbname: "postgres".to_string(),
                application_name: None,
                connect_timeout_s: Some(5),
                options: None,
                tls: PgClientTls {
                    mode: crate::pginfo::state::PgSslMode::Prefer,
                    root_cert: None,
                    client_cert: None,
                    client_key: None,
                },
            },
            None,
        );
        let observed = crate::process::state::ProcessObservedSnapshot {
            dcs: crate::dcs::DcsSnapshot::starting(),
            managed_recovery_state: ManagedRecoverySignal::None,
        };
        let request = ProcessIntentRequest {
            id: JobId("job-start".to_string()),
            intent: ProcessIntent::Start(PostgresStartIntent::DetachedStandby),
        };

        let prepared = super::PreparedProcessLaunch {
            request: super::execution_request_from_plan(
                request.id.clone(),
                request.intent.job_kind(),
                &cfg,
                &crate::process::planner::ClusterProcessPlan::StartManagedPostgres(start_intent),
            )
            .map_err(|err| format!("lower start execution request failed: {err}"))?,
            command: super::build_command(
                &cfg,
                &super::execution_request_from_plan(
                    JobId("job-start".to_string()),
                    ProcessJobKind::StartReplica,
                    &cfg,
                    &crate::process::planner::ClusterProcessPlan::StartManagedPostgres(
                        ManagedPostgresStartIntent::replica(
                            PgConnInfo {
                                route: crate::state::PgRoute::tcp("10.0.0.12".to_string(), 5432)?,
                                user: "replicator".to_string(),
                                dbname: "postgres".to_string(),
                                application_name: None,
                                connect_timeout_s: Some(5),
                                options: None,
                                tls: PgClientTls {
                                    mode: crate::pginfo::state::PgSslMode::Prefer,
                                    root_cert: None,
                                    client_cert: None,
                                    client_key: None,
                                },
                            },
                            None,
                        ),
                    ),
                )
                .map_err(|err| format!("lower start execution request failed: {err}"))?
                .kind,
            )
            .map_err(|err| format!("build start command failed: {err}"))?,
        };

        let config_file = match prepared.request.kind {
            crate::process::state::ProcessExecutionKind::StartPostgres(ref spec) => {
                spec.config_file.clone()
            }
            ref other => return Err(format!("unexpected execution request kind: {other:?}")),
        };
        let has_config_file = prepared
            .command
            .args
            .iter()
            .any(|arg| arg.contains(config_file.display().to_string().as_str()));
        if !has_config_file {
            return Err(format!(
                "start command did not include prepared config path {}",
                config_file.display()
            ));
        }
        let passfile_path = managed_standby_passfile_path(&data_dir);
        if !passfile_path.exists() {
            return Err(format!(
                "expected standby passfile to exist at {}",
                passfile_path.display()
            ));
        }
        if !matches!(observed.managed_recovery_state, ManagedRecoverySignal::None) {
            return Err("test setup expected managed recovery state none".to_string());
        }

        Ok(())
    }

    #[test]
    fn execution_request_from_replica_start_uses_replicator_user_conninfo() -> Result<(), String> {
        let root = unique_test_dir("process-cluster", "replica-plan")?;
        let cfg =
            runtime_test_config_with_data_dir(root.join("data")).map_err(|err| err.to_string())?;
        let execution_request = super::execution_request_from_plan(
            JobId("job-start".to_string()),
            ProcessJobKind::StartReplica,
            &cfg,
            &crate::process::planner::ClusterProcessPlan::StartManagedPostgres(
                ManagedPostgresStartIntent::replica(
                    PgConnInfo {
                        route: crate::state::PgRoute::tcp("10.0.0.14".to_string(), 5432)?,
                        user: "replicator".to_string(),
                        dbname: "postgres".to_string(),
                        application_name: None,
                        connect_timeout_s: Some(5),
                        options: None,
                        tls: PgClientTls {
                            mode: crate::pginfo::state::PgSslMode::Prefer,
                            root_cert: None,
                            client_cert: None,
                            client_key: None,
                        },
                    },
                    None,
                ),
            ),
        )
        .map_err(|err| format!("execution request from start plan failed: {err}"))?;

        match execution_request.kind {
            crate::process::state::ProcessExecutionKind::StartPostgres(spec) => {
                if spec.mode != crate::process::jobs::PostgresStartMode::Replica {
                    return Err(format!("unexpected start mode: {:?}", spec.mode));
                }
            }
            other => return Err(format!("unexpected execution request kind: {other:?}")),
        }

        Ok(())
    }
}
