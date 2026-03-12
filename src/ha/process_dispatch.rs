use std::{fs, path::Path};

use thiserror::Error;

use crate::{
    config::RuntimeConfig,
    dcs::state::MemberSlot,
    postgres_managed_conf::{managed_standby_auth_from_role_auth, ManagedPostgresStartIntent},
    process::{
        jobs::{
            BaseBackupSpec, BootstrapSpec, DemoteSpec, PgRewindSpec, PromoteSpec, StartPostgresSpec,
        },
        state::{ProcessJobKind, ProcessJobRequest},
    },
    state::{JobId, MemberId},
};

use super::{
    source_conn::{basebackup_source_from_member, rewind_source_from_member},
    state::HaWorkerCtx,
    types::{ReconcileAction, ShutdownMode},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProcessDispatchOutcome {
    Applied,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ProcessDispatchError {
    #[error("process send failed for action `{action}`: {message}")]
    ProcessSend { action: String, message: String },
    #[error("managed config materialization failed for action `{action}`: {message}")]
    ManagedConfig { action: String, message: String },
    #[error("filesystem operation failed for action `{action}`: {message}")]
    Filesystem { action: String, message: String },
    #[error("remote source selection failed for action `{action}`: {message}")]
    SourceSelection { action: String, message: String },
    #[error("process dispatch does not support action `{action}`")]
    UnsupportedAction { action: String },
}

pub(crate) fn dispatch_process_action(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &ReconcileAction,
    runtime_config: &RuntimeConfig,
) -> Result<ProcessDispatchOutcome, ProcessDispatchError> {
    match action {
        ReconcileAction::InitDb => {
            wipe_data_dir(runtime_config.postgres.data_dir.as_path()).map_err(|message| {
                ProcessDispatchError::Filesystem {
                    action: action.label().to_string(),
                    message,
                }
            })?;
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Bootstrap(BootstrapSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    superuser_username: runtime_config.postgres.roles.superuser.username.clone(),
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.label(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        ReconcileAction::BaseBackup(leader_member_id) => {
            wipe_data_dir(runtime_config.postgres.data_dir.as_path()).map_err(|message| {
                ProcessDispatchError::Filesystem {
                    action: action.label().to_string(),
                    message,
                }
            })?;
            let source = validate_basebackup_source(ctx, action.label(), leader_member_id)?;
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::BaseBackup(BaseBackupSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    source,
                    timeout_ms: Some(runtime_config.process.bootstrap_timeout_ms),
                }),
            };
            send_process_request(ctx, action.label(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        ReconcileAction::PgRewind(leader_member_id) => {
            let source = validate_rewind_source(ctx, action.label(), leader_member_id)?;
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::PgRewind(PgRewindSpec {
                    target_data_dir: runtime_config.postgres.data_dir.clone(),
                    source,
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.label(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        ReconcileAction::StartPrimary => {
            let start_intent =
                start_intent_from_dcs(ctx, None, runtime_config.postgres.data_dir.as_path())?;
            let managed = crate::postgres_managed::materialize_managed_postgres_config(
                runtime_config,
                &start_intent,
            )
            .map_err(|err| ProcessDispatchError::ManagedConfig {
                action: action.label().to_string(),
                message: err.to_string(),
            })?;
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::StartPostgres(StartPostgresSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    socket_dir: runtime_config.postgres.socket_dir.clone(),
                    port: runtime_config.postgres.listen_port,
                    config_file: managed.postgresql_conf_path,
                    log_file: ctx.process_defaults.log_file.clone(),
                    wait_seconds: None,
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.label(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        ReconcileAction::StartReplica(leader_member_id) => {
            let start_intent = start_intent_from_dcs(
                ctx,
                Some(leader_member_id),
                runtime_config.postgres.data_dir.as_path(),
            )?;
            let managed = crate::postgres_managed::materialize_managed_postgres_config(
                runtime_config,
                &start_intent,
            )
            .map_err(|err| ProcessDispatchError::ManagedConfig {
                action: action.label().to_string(),
                message: err.to_string(),
            })?;
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::StartPostgres(StartPostgresSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    socket_dir: runtime_config.postgres.socket_dir.clone(),
                    port: runtime_config.postgres.listen_port,
                    config_file: managed.postgresql_conf_path,
                    log_file: ctx.process_defaults.log_file.clone(),
                    wait_seconds: None,
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.label(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        ReconcileAction::Promote => {
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Promote(PromoteSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    wait_seconds: None,
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.label(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        ReconcileAction::Demote(mode) => {
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Demote(DemoteSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    mode: shutdown_mode(*mode).to_process_mode(),
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.label(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        ReconcileAction::AcquireLease(_)
        | ReconcileAction::ReleaseLease
        | ReconcileAction::EnsureRequiredRoles
        | ReconcileAction::Publish(_)
        | ReconcileAction::ClearSwitchover => Err(ProcessDispatchError::UnsupportedAction {
            action: action.label().to_string(),
        }),
    }
}

pub(crate) fn validate_rewind_source(
    ctx: &HaWorkerCtx,
    action: &str,
    leader_member_id: &MemberId,
) -> Result<crate::process::jobs::RewinderSourceConn, ProcessDispatchError> {
    let member = resolve_source_member(ctx, action, leader_member_id)?;
    rewind_source_from_member(&ctx.self_id, &member, &ctx.process_defaults).map_err(|err| {
        ProcessDispatchError::SourceSelection {
            action: action.to_string(),
            message: err.to_string(),
        }
    })
}

pub(crate) fn validate_basebackup_source(
    ctx: &HaWorkerCtx,
    action: &str,
    leader_member_id: &MemberId,
) -> Result<crate::process::jobs::ReplicatorSourceConn, ProcessDispatchError> {
    let member = resolve_source_member(ctx, action, leader_member_id)?;
    basebackup_source_from_member(&ctx.self_id, &member, &ctx.process_defaults).map_err(|err| {
        ProcessDispatchError::SourceSelection {
            action: action.to_string(),
            message: err.to_string(),
        }
    })
}

fn resolve_source_member(
    ctx: &HaWorkerCtx,
    action: &str,
    leader_member_id: &MemberId,
) -> Result<MemberSlot, ProcessDispatchError> {
    let dcs = ctx.dcs_subscriber.latest();
    dcs.value
        .cache
        .member_slots
        .get(leader_member_id)
        .cloned()
        .ok_or_else(|| ProcessDispatchError::SourceSelection {
            action: action.to_string(),
            message: format!(
                "target member `{}` not present in DCS cache",
                leader_member_id.0
            ),
        })
}

fn send_process_request(
    ctx: &mut HaWorkerCtx,
    action: &str,
    request: ProcessJobRequest,
) -> Result<(), ProcessDispatchError> {
    ctx.process_inbox
        .send(request)
        .map_err(|err| ProcessDispatchError::ProcessSend {
            action: action.to_string(),
            message: err.to_string(),
        })
}

fn start_intent_from_dcs(
    ctx: &HaWorkerCtx,
    replica_leader_member_id: Option<&MemberId>,
    data_dir: &Path,
) -> Result<ManagedPostgresStartIntent, ProcessDispatchError> {
    if let Some(leader_member_id) = replica_leader_member_id {
        let leader = resolve_source_member(ctx, "start_replica", leader_member_id)?;
        let source = basebackup_source_from_member(&ctx.self_id, &leader, &ctx.process_defaults)
            .map_err(|err| ProcessDispatchError::SourceSelection {
                action: "start_replica".to_string(),
                message: err.to_string(),
            })?;
        return Ok(ManagedPostgresStartIntent::replica(
            source.conninfo.clone(),
            managed_standby_auth_from_role_auth(&source.auth, data_dir),
            None,
        ));
    }

    let managed_recovery_state = crate::postgres_managed::inspect_managed_recovery_state(data_dir)
        .map_err(|err| ProcessDispatchError::ManagedConfig {
            action: "start_primary".to_string(),
            message: err.to_string(),
        })?;
    if managed_recovery_state != crate::postgres_managed_conf::ManagedRecoverySignal::None {
        return Err(ProcessDispatchError::ManagedConfig {
            action: "start_primary".to_string(),
            message:
                "existing postgres data dir contains managed replica recovery state but no leader-derived source is available to rebuild authoritative managed config"
                    .to_string(),
        });
    }

    Ok(ManagedPostgresStartIntent::primary())
}

fn process_job_id(
    scope: &str,
    self_id: &MemberId,
    action: &ReconcileAction,
    index: usize,
    tick: u64,
) -> JobId {
    JobId(format!(
        "ha-{}-{}-{}-{}-{}",
        scope.trim_matches('/'),
        self_id.0,
        tick,
        index,
        action.label(),
    ))
}

fn shutdown_mode(mode: ShutdownMode) -> ShutdownMode {
    mode
}

fn wipe_data_dir(data_dir: &Path) -> Result<(), String> {
    if data_dir.as_os_str().is_empty() {
        return Err("wipe_data_dir data_dir must not be empty".to_string());
    }
    if data_dir.exists() {
        wipe_data_dir_contents(data_dir)?;
    } else {
        fs::create_dir_all(data_dir)
            .map_err(|err| format!("wipe_data_dir create_dir_all failed: {err}"))?;
    }
    set_postgres_data_dir_permissions(data_dir)?;
    Ok(())
}

fn wipe_data_dir_contents(data_dir: &Path) -> Result<(), String> {
    let entries =
        fs::read_dir(data_dir).map_err(|err| format!("wipe_data_dir read_dir failed: {err}"))?;
    for entry_result in entries {
        let entry =
            entry_result.map_err(|err| format!("wipe_data_dir read_dir entry failed: {err}"))?;
        let file_type = entry
            .file_type()
            .map_err(|err| format!("wipe_data_dir file_type failed: {err}"))?;
        let entry_path = entry.path();
        if file_type.is_dir() {
            fs::remove_dir_all(entry_path.as_path()).map_err(|err| {
                format!(
                    "wipe_data_dir remove_dir_all failed for {}: {err}",
                    entry_path.display()
                )
            })?;
        } else {
            fs::remove_file(entry_path.as_path()).map_err(|err| {
                format!(
                    "wipe_data_dir remove_file failed for {}: {err}",
                    entry_path.display()
                )
            })?;
        }
    }
    Ok(())
}

fn set_postgres_data_dir_permissions(data_dir: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(data_dir, fs::Permissions::from_mode(0o700))
            .map_err(|err| format!("wipe_data_dir set_permissions failed: {err}"))?;
    }

    #[cfg(not(unix))]
    {
        let _ = data_dir;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::wipe_data_dir;

    fn temp_data_dir(label: &str) -> Result<PathBuf, String> {
        let unique = format!(
            "{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|err| format!("read current time failed: {err}"))?
                .as_millis()
        );
        Ok(std::env::temp_dir().join(format!("pgtuskmaster-process-dispatch-{unique}")))
    }

    #[test]
    fn wipe_data_dir_removes_contents_but_preserves_root_directory() -> Result<(), String> {
        let data_dir = temp_data_dir("wipe-contents")?;
        let nested_dir = data_dir.join("nested");
        let nested_file = nested_dir.join("file.txt");
        let top_level_file = data_dir.join("postgresql.auto.conf");

        fs::create_dir_all(&nested_dir).map_err(|err| {
            format!(
                "create nested dir failed for {}: {err}",
                nested_dir.display()
            )
        })?;
        fs::write(&nested_file, "x").map_err(|err| {
            format!(
                "write nested file failed for {}: {err}",
                nested_file.display()
            )
        })?;
        fs::write(&top_level_file, "y").map_err(|err| {
            format!(
                "write top-level file failed for {}: {err}",
                top_level_file.display()
            )
        })?;

        wipe_data_dir(&data_dir)?;

        let entries = fs::read_dir(&data_dir)
            .map_err(|err| format!("read_dir failed for {}: {err}", data_dir.display()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| format!("collect read_dir entries failed: {err}"))?;
        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove data dir failed for {}: {err}", data_dir.display()))?;

        assert!(entries.is_empty());
        Ok(())
    }

    #[test]
    fn wipe_data_dir_creates_missing_directory() -> Result<(), String> {
        let data_dir = temp_data_dir("wipe-create")?;

        wipe_data_dir(&data_dir)?;

        let exists = data_dir.exists();
        fs::remove_dir_all(&data_dir)
            .map_err(|err| format!("remove data dir failed for {}: {err}", data_dir.display()))?;

        assert!(exists);
        Ok(())
    }
}
