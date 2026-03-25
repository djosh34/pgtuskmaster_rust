use std::{fs, path::Path};

use thiserror::Error;

use crate::{
    config_v2::{types::Secret, RuntimeConfigV2},
    dcs::{DcsMemberState, DcsSnapshot},
    pginfo::state::{PgConnInfo, PgInfoState},
    postgres_managed::{
        inspect_managed_recovery_state, managed_postgresql_conf_path,
        materialize_managed_postgres_config, ManagedRecoverySignal,
    },
    process::{
        jobs::{
            PostgresStartIntent, ProcessCommandSpec, ProcessEnvValue, ProcessEnvVar, ProcessError,
            ProcessIntent, ReplicaProvisionIntent, ShutdownMode,
        },
        state::{ProcessIntentRequest, ProcessObservedSnapshot, ProcessWorkerCtx},
    },
    state::MemberId,
};

const PG_CTL_DEFAULT_WAIT_SECONDS: u64 = 30;

#[derive(Clone, Debug, PartialEq, Eq)]
struct SourceConn {
    conninfo: PgConnInfo,
    auth: Secret,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SourceCredentialKind {
    Replicator,
    Rewinder,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
enum SourceMaterializationError {
    #[error("remote source member `{member_id}` is self")]
    SelfTarget { member_id: String },
    #[error("remote source member `{member_id}` is not a healthy primary")]
    NotHealthyPrimary { member_id: String },
    #[error("remote source member `{member_id}` has an empty postgres host")]
    EmptyHost { member_id: String },
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
) -> Result<ProcessCommandSpec, ProcessPreparationError> {
    let observed = observed_snapshot_from_ctx(ctx)?;
    prepare_process_launch(ctx.cfg, &observed, request)
}

pub(crate) fn prepare_process_launch(
    cfg: &RuntimeConfigV2,
    observed: &ProcessObservedSnapshot,
    request: &ProcessIntentRequest,
) -> Result<ProcessCommandSpec, ProcessPreparationError> {
    match &request.intent {
        ProcessIntent::Bootstrap => {
            wipe_data_dir(cfg.postgres.data_dir.as_path())
                .map_err(ProcessPreparationError::IntentMaterialization)?;
            Ok(build_bootstrap_command(cfg))
        }
        ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup { leader }) => {
            let source = source_from_leader(
                &cfg.member_id,
                cfg,
                &observed.dcs,
                leader,
                SourceCredentialKind::Replicator,
            )
            .map_err(ProcessPreparationError::IntentMaterialization)?;
            wipe_data_dir(cfg.postgres.data_dir.as_path())
                .map_err(ProcessPreparationError::IntentMaterialization)?;
            Ok(build_basebackup_command(cfg, &source))
        }
        ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::PgRewind { leader }) => {
            let source = source_from_leader(
                &cfg.member_id,
                cfg,
                &observed.dcs,
                leader,
                SourceCredentialKind::Rewinder,
            )
            .map_err(ProcessPreparationError::IntentMaterialization)?;
            Ok(build_pg_rewind_command(cfg, &source))
        }
        ProcessIntent::Start(start_intent) => {
            prepare_start_postgres_launch(cfg, observed, start_intent)
        }
        ProcessIntent::Promote => Ok(build_promote_command(cfg, None)),
        ProcessIntent::Demote(mode) => Ok(build_demote_command(cfg, mode)),
    }
}

fn prepare_start_postgres_launch(
    cfg: &RuntimeConfigV2,
    observed: &ProcessObservedSnapshot,
    start_intent: &PostgresStartIntent,
) -> Result<ProcessCommandSpec, ProcessPreparationError> {
    let source = match start_intent {
        PostgresStartIntent::Primary => {
            if observed.managed_recovery_state != ManagedRecoverySignal::None {
                return Err(ProcessPreparationError::IntentMaterialization(
                    ProcessError::InvalidSpec(
                        "existing postgres data dir contains managed replica recovery state but no leader-derived source is available to rebuild authoritative managed config".to_string(),
                    ),
                ));
            }
            None
        }
        PostgresStartIntent::DetachedStandby => None,
        PostgresStartIntent::Replica { leader } => Some(
            source_from_leader(
                &cfg.member_id,
                cfg,
                &observed.dcs,
                leader,
                SourceCredentialKind::Replicator,
            )
            .map_err(ProcessPreparationError::IntentMaterialization)?,
        ),
    };

    materialize_start_config(
        cfg,
        start_intent,
        source.as_ref().map(|remote_source| &remote_source.conninfo),
    )
    .map_err(ProcessPreparationError::IntentMaterialization)?;
    build_start_postgres_command(cfg).map_err(ProcessPreparationError::BuildCommand)
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

fn materialize_start_config(
    cfg: &RuntimeConfigV2,
    start_intent: &PostgresStartIntent,
    primary_conninfo: Option<&PgConnInfo>,
) -> Result<(), ProcessError> {
    materialize_managed_postgres_config(cfg, start_intent, primary_conninfo).map_err(|err| {
        ProcessError::InvalidSpec(format!("materialize managed postgres config failed: {err}"))
    })
}

fn build_command(
    cfg: &RuntimeConfigV2,
    program: std::path::PathBuf,
    args: Vec<String>,
    env: Vec<ProcessEnvVar>,
) -> ProcessCommandSpec {
    ProcessCommandSpec {
        program,
        args,
        env,
        capture_output: cfg.logging.capture_subprocess_output,
    }
}

fn build_pg_ctl_command(cfg: &RuntimeConfigV2, args: Vec<String>) -> ProcessCommandSpec {
    build_command(cfg, cfg.process.binaries.pg_ctl.clone(), args, Vec::new())
}

fn source_conninfo_and_env(source: &SourceConn) -> (String, Vec<ProcessEnvVar>) {
    (source.conninfo.to_string(), role_auth_env(&source.auth))
}

fn build_bootstrap_command(cfg: &RuntimeConfigV2) -> ProcessCommandSpec {
    build_command(
        cfg,
        cfg.process.binaries.initdb.clone(),
        vec![
            "-D".to_string(),
            cfg.postgres.data_dir.display().to_string(),
            "-A".to_string(),
            "trust".to_string(),
            "-U".to_string(),
            cfg.postgres.superuser.username.clone(),
        ],
        Vec::new(),
    )
}

fn build_basebackup_command(cfg: &RuntimeConfigV2, source: &SourceConn) -> ProcessCommandSpec {
    let (conninfo, env) = source_conninfo_and_env(source);
    build_command(
        cfg,
        cfg.process.binaries.pg_basebackup.clone(),
        vec![
            "--dbname".to_string(),
            conninfo,
            "-D".to_string(),
            cfg.postgres.data_dir.display().to_string(),
            "-Fp".to_string(),
            "-Xs".to_string(),
        ],
        env,
    )
}

fn build_pg_rewind_command(cfg: &RuntimeConfigV2, source: &SourceConn) -> ProcessCommandSpec {
    let (conninfo, env) = source_conninfo_and_env(source);
    build_command(
        cfg,
        cfg.process.binaries.pg_rewind.clone(),
        vec![
            "--target-pgdata".to_string(),
            cfg.postgres.data_dir.display().to_string(),
            "--source-server".to_string(),
            conninfo,
        ],
        env,
    )
}

fn build_promote_command(cfg: &RuntimeConfigV2, wait_seconds: Option<u64>) -> ProcessCommandSpec {
    let wait_args = wait_seconds
        .into_iter()
        .flat_map(|seconds| ["-t".to_string(), seconds.to_string()])
        .collect::<Vec<_>>();
    let args = [
        vec![
            "-D".to_string(),
            cfg.postgres.data_dir.display().to_string(),
            "promote".to_string(),
            "-w".to_string(),
        ],
        wait_args,
    ]
    .concat();
    build_pg_ctl_command(cfg, args)
}

fn build_demote_command(cfg: &RuntimeConfigV2, mode: &ShutdownMode) -> ProcessCommandSpec {
    build_pg_ctl_command(
        cfg,
        vec![
            "-D".to_string(),
            cfg.postgres.data_dir.display().to_string(),
            "stop".to_string(),
            "-m".to_string(),
            mode.as_pg_ctl_arg().to_string(),
            "-w".to_string(),
        ],
    )
}

fn build_start_postgres_command(cfg: &RuntimeConfigV2) -> Result<ProcessCommandSpec, ProcessError> {
    let config_file = managed_postgresql_conf_path(cfg.postgres.data_dir.as_path());
    let option_tokens = vec![
        "-c".to_string(),
        format!("config_file={}", config_file.display()),
    ];
    let options = render_pg_ctl_option_string(&option_tokens)?;
    Ok(build_pg_ctl_command(
        cfg,
        vec![
            "-D".to_string(),
            cfg.postgres.data_dir.display().to_string(),
            "-l".to_string(),
            cfg.postgres.log_file.display().to_string(),
            "-o".to_string(),
            options,
            "start".to_string(),
            "-w".to_string(),
            "-t".to_string(),
            PG_CTL_DEFAULT_WAIT_SECONDS.to_string(),
        ],
    ))
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

fn source_from_leader(
    self_id: &MemberId,
    cfg: &RuntimeConfigV2,
    dcs: &DcsSnapshot,
    leader: &MemberId,
    credential_kind: SourceCredentialKind,
) -> Result<SourceConn, ProcessError> {
    let (source_member_id, source_member) = resolve_source_member(dcs, leader)?;
    source_from_member(
        self_id,
        cfg,
        source_member_id,
        source_member,
        credential_kind,
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
    credential_kind: SourceCredentialKind,
) -> Result<SourceConn, SourceMaterializationError> {
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

    let credential = match credential_kind {
        SourceCredentialKind::Replicator => &cfg.postgres.replicator,
        SourceCredentialKind::Rewinder => &cfg.postgres.rewinder,
    };
    Ok(SourceConn {
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
    use std::{collections::BTreeMap, fs, path::Path};

    use crate::{
        config_v2::{runtime_test_config_with_data_dir, RuntimeConfigV2},
        dcs::{DcsMemberState, DcsSnapshot},
        dev_support::test_fs::unique_test_dir,
        pginfo::{
            conninfo::{PgClientTls, PgSslMode},
            state::{PgConfig, PgConnInfo, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        },
        postgres_managed::{
            managed_standby_passfile_path, ManagedRecoverySignal, MANAGED_POSTGRESQL_CONF_NAME,
        },
        process::{
            jobs::{
                PostgresStartIntent, ProcessIntent, ProcessJobKind, ReplicaProvisionIntent,
                ShutdownMode,
            },
            state::ProcessIntentRequest,
        },
        state::{
            JobId, MemberId, PgRoute, SwitchoverState, SystemIdentifier, TimelineId, UnixMillis,
            WalLsn, WorkerStatus,
        },
    };

    use super::{
        prepare_process_launch, source_from_member, SourceCredentialKind,
        SourceMaterializationError,
    };

    fn test_cfg(data_dir: &Path) -> Result<RuntimeConfigV2, String> {
        runtime_test_config_with_data_dir(data_dir.to_path_buf()).map_err(|err| err.to_string())
    }

    fn observed(dcs: DcsSnapshot) -> crate::process::state::ProcessObservedSnapshot {
        crate::process::state::ProcessObservedSnapshot {
            dcs,
            managed_recovery_state: ManagedRecoverySignal::None,
        }
    }

    fn test_request(id: &str, intent: ProcessIntent) -> ProcessIntentRequest {
        ProcessIntentRequest {
            id: JobId(id.to_string()),
            intent,
        }
    }

    fn leader_observed(
        host: &str,
        port: u16,
    ) -> Result<(MemberId, crate::process::state::ProcessObservedSnapshot), String> {
        let leader = MemberId("node-b".to_string());
        let dcs = DcsSnapshot::quorum(
            None,
            SwitchoverState::None,
            BTreeMap::from([(leader.clone(), primary_member(host, port)?)]),
        );
        Ok((leader, observed(dcs)))
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

    #[test]
    fn prepare_replica_start_materializes_managed_files_and_builds_start_command(
    ) -> Result<(), String> {
        let root = unique_test_dir("process-cluster", "replica-start")?;
        let data_dir = root.join("data");
        let cfg = test_cfg(&data_dir)?;
        let (leader, observed) = leader_observed("10.0.0.13", 5432)?;
        let request = test_request(
            "job-start-replica",
            ProcessIntent::Start(PostgresStartIntent::Replica { leader }),
        );

        let prepared = prepare_process_launch(&cfg, &observed, &request)
            .map_err(|err| format!("prepare replica start failed: {err}"))?;

        if request.intent.job_kind() != ProcessJobKind::StartReplica {
            return Err(format!(
                "unexpected canonical job kind: {:?}",
                request.intent.job_kind()
            ));
        }
        if request.intent.timeout_ms(&cfg) == 0 {
            return Err("replica start should carry a non-zero timeout".to_string());
        }
        let options = prepared
            .args
            .windows(2)
            .find_map(|window| (window[0] == "-o").then(|| window[1].clone()))
            .ok_or_else(|| "start command missing -o options".to_string())?;
        if !options.contains(MANAGED_POSTGRESQL_CONF_NAME) {
            return Err(format!(
                "start command did not reference managed config file: {options}"
            ));
        }
        let passfile_path = managed_standby_passfile_path(&data_dir);
        if !passfile_path.exists() {
            return Err(format!(
                "expected standby passfile to exist at {}",
                passfile_path.display()
            ));
        }

        Ok(())
    }

    #[test]
    fn prepare_basebackup_from_config_v2_preserves_sslrootcert_in_conninfo() -> Result<(), String> {
        let root = unique_test_dir("process-cluster", "basebackup-source-ca")?;
        let data_dir = root.join("data");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        let expected_root_cert = root.join("source-ca.crt");
        let true_path = Path::new("/bin/true").to_path_buf();
        let cfg = test_cfg(&data_dir)
            .map(|cfg| RuntimeConfigV2 {
                postgres: crate::config_v2::types::PostgresConfig {
                    source_client_tls: PgClientTls {
                        mode: PgSslMode::VerifyFull,
                        root_cert: Some(expected_root_cert.clone()),
                        client_cert: None,
                        client_key: None,
                    },
                    ..cfg.postgres
                },
                process: crate::config_v2::types::ProcessConfig {
                    binaries: crate::config_v2::types::ProcessBinariesConfig {
                        pg_ctl: true_path.clone(),
                        initdb: true_path.clone(),
                        pg_rewind: true_path.clone(),
                        pg_basebackup: true_path,
                    },
                    ..cfg.process
                },
                ..cfg
            })
            .map_err(|err| err.to_string())?;
        let (leader, observed) = leader_observed("10.0.0.13", 5432)?;
        let request = test_request(
            "job-basebackup",
            ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup { leader }),
        );

        let prepared = prepare_process_launch(&cfg, &observed, &request)
            .map_err(|err| format!("prepare basebackup failed: {err}"))?;

        let conninfo_arg = prepared
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

        let cfg = test_cfg(&data_dir)?;
        let observed = observed(crate::dcs::DcsSnapshot::starting());
        let request = test_request(
            "job-basebackup",
            ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup {
                leader: MemberId("node-b".to_string()),
            }),
        );

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

        let (leader, observed) = leader_observed("10.0.0.11", 5432)?;
        let request = test_request(
            "job-basebackup-success",
            ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup { leader }),
        );

        prepare_process_launch(&cfg, &observed, &request)
            .map_err(|err| format!("prepare basebackup failed: {err}"))?;
        if request.intent.job_kind() != ProcessJobKind::BaseBackup {
            return Err(format!(
                "unexpected canonical job kind: {:?}",
                request.intent.job_kind()
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
        let cfg = test_cfg(&root.join("data"))?;
        let observed = observed(DcsSnapshot::starting());
        let request = test_request("job-promote", ProcessIntent::Promote);

        prepare_process_launch(&cfg, &observed, &request)
            .map_err(|err| format!("prepare non-start plan failed: {err}"))?;
        if request.intent.job_kind() != ProcessJobKind::Promote {
            return Err(format!(
                "unexpected canonical job kind for promote: {:?}",
                request.intent.job_kind()
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
    fn prepare_primary_start_rejects_existing_managed_replica_state() -> Result<(), String> {
        let root = unique_test_dir("process-cluster", "primary-reject")?;
        let cfg = test_cfg(&root.join("data"))?;
        let observed = crate::process::state::ProcessObservedSnapshot {
            dcs: DcsSnapshot::starting(),
            managed_recovery_state: ManagedRecoverySignal::Standby,
        };
        let request = test_request(
            "job-start-primary",
            ProcessIntent::Start(PostgresStartIntent::Primary),
        );

        let error = prepare_process_launch(&cfg, &observed, &request)
            .err()
            .ok_or_else(|| "expected primary start to be rejected".to_string())?;
        if !error.to_string().contains("managed replica recovery state") {
            return Err(format!("unexpected primary-start rejection: {error}"));
        }

        Ok(())
    }

    #[test]
    fn prepare_intents_map_to_expected_canonical_job_kinds() -> Result<(), String> {
        let root = unique_test_dir("process-cluster", "intent-variants")?;
        let cfg = test_cfg(&root.join("data"))?;
        let (leader, observed) = leader_observed("10.0.0.8", 5432)?;
        let cases = [
            (ProcessIntent::Bootstrap, ProcessJobKind::Bootstrap),
            (
                ProcessIntent::Start(PostgresStartIntent::Primary),
                ProcessJobKind::StartPrimary,
            ),
            (
                ProcessIntent::Start(PostgresStartIntent::DetachedStandby),
                ProcessJobKind::StartDetachedStandby,
            ),
            (
                ProcessIntent::Start(PostgresStartIntent::Replica {
                    leader: leader.clone(),
                }),
                ProcessJobKind::StartReplica,
            ),
            (
                ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::BaseBackup {
                    leader: leader.clone(),
                }),
                ProcessJobKind::BaseBackup,
            ),
            (
                ProcessIntent::ProvisionReplica(ReplicaProvisionIntent::PgRewind {
                    leader: leader.clone(),
                }),
                ProcessJobKind::PgRewind,
            ),
            (ProcessIntent::Promote, ProcessJobKind::Promote),
            (
                ProcessIntent::Demote(ShutdownMode::Fast),
                ProcessJobKind::Demote,
            ),
        ];

        for (intent, job_kind) in cases {
            let request = test_request(&format!("job-{}", job_kind.as_str()), intent);
            let prepared = prepare_process_launch(&cfg, &observed, &request)
                .map_err(|err| format!("prepare {} failed: {err}", job_kind.as_str()))?;
            assert_eq!(request.intent.job_kind(), job_kind);
            if matches!(&request.intent, ProcessIntent::Start(_))
                && !prepared.args.iter().any(|arg| arg == "start")
            {
                return Err(format!(
                    "prepared start command should include start verb for {}",
                    job_kind.as_str()
                ));
            }
            assert!(request.intent.timeout_ms(&cfg) > 0);
        }

        Ok(())
    }

    #[test]
    fn source_from_member_selects_role_specific_credentials() -> Result<(), String> {
        let root = unique_test_dir("process-cluster", "source-credentials")?;
        let cfg = test_cfg(&root.join("data"))?;
        let member_id = MemberId("node-b".to_string());
        let member = primary_member("10.0.0.9", 5432)?;

        let replicator = source_from_member(
            &MemberId("node-a".to_string()),
            &cfg,
            &member_id,
            &member,
            SourceCredentialKind::Replicator,
        )
        .map_err(|err| err.to_string())?;
        let rewinder = source_from_member(
            &MemberId("node-a".to_string()),
            &cfg,
            &member_id,
            &member,
            SourceCredentialKind::Rewinder,
        )
        .map_err(|err| err.to_string())?;

        assert_eq!(replicator.conninfo.user, cfg.postgres.replicator.username);
        assert_eq!(
            replicator.conninfo.connect_timeout_s,
            Some(u32::try_from(cfg.postgres.connect_timeout.as_secs()).unwrap_or(u32::MAX))
        );
        assert_eq!(replicator.conninfo.tls, cfg.postgres.source_client_tls);
        assert_eq!(replicator.auth, cfg.postgres.replicator.password.clone());

        assert_eq!(rewinder.conninfo.user, cfg.postgres.rewinder.username);
        assert_eq!(rewinder.conninfo.tls, cfg.postgres.source_client_tls);
        assert_eq!(rewinder.auth, cfg.postgres.rewinder.password.clone());
        Ok(())
    }

    #[test]
    fn source_from_member_uses_shared_source_client_tls_for_all_roles() -> Result<(), String> {
        let root = unique_test_dir("process-cluster", "source-shared-tls")?;
        let config = test_cfg(&root.join("data"))?;
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
            SourceCredentialKind::Replicator,
        )
        .map_err(|err| err.to_string())?;
        let rewinder = source_from_member(
            &MemberId("node-a".to_string()),
            &cfg,
            &member_id,
            &member,
            SourceCredentialKind::Rewinder,
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
        let root = unique_test_dir("process-cluster", "source-empty-host")?;
        let cfg = test_cfg(&root.join("data"))?;
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
                SourceCredentialKind::Replicator,
            ),
            Err(SourceMaterializationError::EmptyHost {
                member_id: "node-b".to_string(),
            })
        );
        Ok(())
    }

    #[test]
    fn source_from_member_rejects_non_primary_source() -> Result<(), String> {
        let root = unique_test_dir("process-cluster", "source-non-primary")?;
        let cfg = test_cfg(&root.join("data"))?;
        let member_id = MemberId("node-b".to_string());
        let member = replica_member("10.0.0.9", 5432)?;

        assert_eq!(
            source_from_member(
                &MemberId("node-a".to_string()),
                &cfg,
                &member_id,
                &member,
                SourceCredentialKind::Replicator,
            ),
            Err(SourceMaterializationError::NotHealthyPrimary {
                member_id: "node-b".to_string(),
            })
        );
        Ok(())
    }

    #[test]
    fn source_from_member_rejects_self_target() -> Result<(), String> {
        let root = unique_test_dir("process-cluster", "source-self-target")?;
        let cfg = test_cfg(&root.join("data"))?;
        let member_id = MemberId("node-a".to_string());
        let member = primary_member("10.0.0.9", 5432)?;

        assert_eq!(
            source_from_member(
                &member_id,
                &cfg,
                &member_id,
                &member,
                SourceCredentialKind::Replicator,
            ),
            Err(SourceMaterializationError::SelfTarget {
                member_id: "node-a".to_string(),
            })
        );
        Ok(())
    }
}
