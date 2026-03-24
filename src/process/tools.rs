use std::{fs, path::Path};

use crate::{
    config_v2::{types::Secret, RuntimeConfigV2},
    process::{
        jobs::{
            ProcessCommandSpec, ProcessEnvValue, ProcessEnvVar, ProcessError, StartPostgresSpec,
        },
        planner::ClusterProcessPlan,
        session::PreparedManagedPostgresSession,
        state::{ProcessExecutionKind, ProcessExecutionRequest, ProcessObservedSnapshot},
    },
};

const PG_CTL_DEFAULT_WAIT_SECONDS: u64 = 30;

#[derive(Default)]
pub(crate) struct ExternalToolLowerer;

impl ExternalToolLowerer {
    pub(crate) fn lower_execution_request(
        &self,
        request_id: crate::state::JobId,
        cfg: &RuntimeConfigV2,
        plan: &ClusterProcessPlan,
        _observed: &ProcessObservedSnapshot,
        prepared_session: Option<&PreparedManagedPostgresSession>,
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
            ClusterProcessPlan::StartManagedPostgres(start) => {
                let prepared_session = prepared_session.ok_or_else(|| {
                    ProcessError::InvalidSpec(
                        "managed postgres start requires prepared session artifacts".to_string(),
                    )
                })?;
                ProcessExecutionKind::StartPostgres(StartPostgresSpec {
                    mode: start.mode,
                    data_dir: cfg.postgres.data_dir.clone(),
                    socket_dir: cfg.postgres.socket_dir.clone(),
                    port: cfg.postgres.listen_port,
                    config_file: prepared_session.config.postgresql_conf_path.clone(),
                    log_file: cfg.postgres.log_file.clone(),
                    wait_seconds: None,
                    timeout_ms: None,
                })
            }
            ClusterProcessPlan::Promote(spec) => ProcessExecutionKind::Promote(spec.clone()),
            ClusterProcessPlan::Demote(spec) => ProcessExecutionKind::Demote(spec.clone()),
        };

        Ok(ProcessExecutionRequest {
            id: request_id,
            kind,
        })
    }

    pub(crate) fn build_command(
        &self,
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
                    job_kind: kind.process_job_kind(),
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
                    job_kind: kind.process_job_kind(),
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
                    job_kind: kind.process_job_kind(),
                })
            }
            ProcessExecutionKind::Promote(spec) => {
                validate_non_empty_path("promote.data_dir", &spec.data_dir)?;
                let mut args = vec![
                    "-D".to_string(),
                    spec.data_dir.display().to_string(),
                    "promote".to_string(),
                    "-w".to_string(),
                ];
                if let Some(wait_seconds) = spec.wait_seconds {
                    args.push("-t".to_string());
                    args.push(wait_seconds.to_string());
                }
                let program = cfg.binaries.pg_ctl.clone();
                Ok(ProcessCommandSpec {
                    program: program.clone(),
                    args,
                    env: Vec::new(),
                    capture_output: cfg.logging.capture_subprocess_output,
                    job_kind: kind.process_job_kind(),
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
                    job_kind: kind.process_job_kind(),
                })
            }
            ProcessExecutionKind::StartPostgres(spec) => {
                validate_non_empty_path("start_postgres.data_dir", &spec.data_dir)?;
                validate_non_empty_path("start_postgres.config_file", &spec.config_file)?;
                validate_non_empty_path("start_postgres.log_file", &spec.log_file)?;
                let wait_seconds = spec.wait_seconds.unwrap_or(PG_CTL_DEFAULT_WAIT_SECONDS);
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
                        wait_seconds.to_string(),
                    ],
                    env: Vec::new(),
                    capture_output: cfg.logging.capture_subprocess_output,
                    job_kind: kind.process_job_kind(),
                })
            }
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
    use std::{fs, path::PathBuf};

    use crate::{
        dev_support::{runtime_config::RuntimeConfigBuilder, test_fs::unique_test_dir},
        pginfo::{conninfo::PgClientTls, state::PgConnInfo},
        postgres_managed::ManagedPostgresConfig,
        postgres_managed_conf::ManagedRecoverySignal,
        process::{
            jobs::{MandatoryRoleSourceConn, MandatorySourceRole},
            planner::{
                ClusterProcessPlan, DesiredManagedPostgresSession, ManagedStartPlan,
                ReplicaFollowPlan,
            },
            session::PreparedManagedPostgresSession,
            state::ProcessObservedSnapshot,
        },
    };

    use super::ExternalToolLowerer;

    fn sample_runtime_config(data_dir: PathBuf) -> crate::config_v2::RuntimeConfigV2 {
        RuntimeConfigBuilder::new()
            .with_postgres_data_dir(data_dir)
            .build()
    }

    #[test]
    fn lower_execution_request_for_basebackup_wipes_existing_data_dir_contents(
    ) -> Result<(), String> {
        let root = unique_test_dir("process-tools", "wipe-basebackup")?;
        let data_dir = root.join("data");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        let stale = data_dir.join("stale.txt");
        fs::write(&stale, "stale")
            .map_err(|err| format!("write stale file {} failed: {err}", stale.display()))?;

        let cfg = sample_runtime_config(data_dir.clone());
        let observed = ProcessObservedSnapshot {
            dcs: crate::dcs::DcsSnapshot::starting(),
            managed_recovery_state: ManagedRecoverySignal::None,
        };
        let plan = ClusterProcessPlan::BaseBackup(crate::process::jobs::BaseBackupSpec {
            data_dir: data_dir.clone(),
            source: MandatoryRoleSourceConn {
                role: MandatorySourceRole::Replicator,
                conninfo: PgConnInfo {
                    route: crate::state::PgRoute::tcp("10.0.0.11".to_string(), 5432)?,
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
                auth: cfg.postgres.replicator.password.clone(),
            },
            timeout_ms: None,
        });

        let request = ExternalToolLowerer
            .lower_execution_request(
                crate::state::JobId("job-basebackup".to_string()),
                &cfg,
                &plan,
                &observed,
                None,
            )
            .map_err(|err| format!("lower execution request failed: {err}"))?;

        if !matches!(
            request.kind,
            crate::process::state::ProcessExecutionKind::BaseBackup(_)
        ) {
            return Err(format!(
                "unexpected execution request kind: {:?}",
                request.kind
            ));
        }
        if stale.exists() {
            return Err(format!(
                "basebackup lowering should wipe stale data dir contents at {}",
                stale.display()
            ));
        }

        Ok(())
    }

    #[test]
    fn build_command_for_start_postgres_uses_prepared_session_paths() -> Result<(), String> {
        let root = unique_test_dir("process-tools", "start-command")?;
        let data_dir = root.join("data");
        let cfg = sample_runtime_config(data_dir.clone());
        let observed = ProcessObservedSnapshot {
            dcs: crate::dcs::DcsSnapshot::starting(),
            managed_recovery_state: ManagedRecoverySignal::None,
        };
        let config_file = data_dir.join("pgtm.postgresql.conf");
        let prepared_session = PreparedManagedPostgresSession {
            config: ManagedPostgresConfig {
                postgresql_conf_path: config_file.clone(),
                hba_path: data_dir.join("pgtm.pg_hba.conf"),
                ident_path: data_dir.join("pgtm.pg_ident.conf"),
                standby_passfile_path: None,
                standby_signal_path: data_dir.join("standby.signal"),
                recovery_signal_path: data_dir.join("recovery.signal"),
                postgresql_auto_conf_path: data_dir.join("postgresql.auto.conf"),
                quarantined_postgresql_auto_conf_path: data_dir
                    .join("pgtm.unmanaged.postgresql.auto.conf"),
            },
        };
        let plan = ClusterProcessPlan::StartManagedPostgres(ManagedStartPlan {
            mode: crate::process::jobs::PostgresStartMode::Replica,
            desired_session: DesiredManagedPostgresSession::Follow(Box::new(ReplicaFollowPlan {
                source: MandatoryRoleSourceConn {
                    role: MandatorySourceRole::Replicator,
                    conninfo: PgConnInfo {
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
                    auth: cfg.postgres.replicator.password.clone(),
                },
                primary_slot_name: None,
            })),
        });

        let execution_request = ExternalToolLowerer
            .lower_execution_request(
                crate::state::JobId("job-start".to_string()),
                &cfg,
                &plan,
                &observed,
                Some(&prepared_session),
            )
            .map_err(|err| format!("lower start execution request failed: {err}"))?;
        let command = ExternalToolLowerer
            .build_command(&cfg, &execution_request.kind)
            .map_err(|err| format!("build start command failed: {err}"))?;

        if command.job_kind != crate::process::jobs::ProcessJobKind::StartPostgres {
            return Err(format!("unexpected start job kind: {:?}", command.job_kind));
        }
        let has_config_file = command
            .args
            .iter()
            .any(|arg| arg.contains(config_file.display().to_string().as_str()));
        if !has_config_file {
            return Err(format!(
                "start command did not include prepared config path {}",
                config_file.display()
            ));
        }

        Ok(())
    }
}
