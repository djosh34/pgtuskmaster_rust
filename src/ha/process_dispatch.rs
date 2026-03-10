use std::{fs, path::Path};

use thiserror::Error;

use crate::{
    config::RuntimeConfig,
    dcs::state::MemberRecord,
    local_physical::{inspect_local_physical_state, DataDirKind, SignalFileState},
    pginfo::state::PgInfoState,
    postgres_managed_conf::{
        managed_standby_auth_from_role_auth, render_managed_primary_conninfo,
        ManagedPostgresStartIntent, ManagedStandbyAuth, MANAGED_POSTGRESQL_CONF_NAME,
    },
    process::{
        jobs::{
            BaseBackupSpec, BootstrapSpec, DemoteSpec, FencingSpec, PgRewindSpec, PromoteSpec,
            ShutdownMode, StartPostgresSpec,
        },
        state::{ProcessJobKind, ProcessJobRequest},
    },
    state::{JobId, MemberId},
};

use super::{
    actions::{ActionId, HaAction},
    source_conn::{basebackup_source_from_member, rewind_source_from_member},
    state::HaWorkerCtx,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProcessDispatchOutcome {
    Applied,
    Skipped,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ProcessDispatchError {
    #[error("process send failed for action `{action:?}`: {message}")]
    ProcessSend { action: ActionId, message: String },
    #[error("managed config materialization failed for action `{action:?}`: {message}")]
    ManagedConfig { action: ActionId, message: String },
    #[error("filesystem operation failed for action `{action:?}`: {message}")]
    Filesystem { action: ActionId, message: String },
    #[error("remote source selection failed for action `{action:?}`: {message}")]
    SourceSelection { action: ActionId, message: String },
    #[error("process dispatch does not support action `{action:?}`")]
    UnsupportedAction { action: ActionId },
}

pub(crate) fn dispatch_process_action(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
    runtime_config: &RuntimeConfig,
) -> Result<ProcessDispatchOutcome, ProcessDispatchError> {
    match action {
        HaAction::AcquireLeaderLease | HaAction::ReleaseLeaderLease | HaAction::ClearSwitchover => {
            Err(ProcessDispatchError::UnsupportedAction {
                action: action.id(),
            })
        }
        HaAction::StartPrimary => {
            let start_intent = managed_start_intent_from_dcs(
                ctx,
                action.id(),
                None,
                runtime_config.postgres.data_dir.as_path(),
            )?;
            dispatch_start_postgres(ctx, ha_tick, action_index, action, runtime_config, &start_intent)
        }
        HaAction::StartReplica { leader_member_id } => {
            let start_intent = managed_start_intent_from_dcs(
                ctx,
                action.id(),
                Some(leader_member_id),
                runtime_config.postgres.data_dir.as_path(),
            )?;
            if start_replica_is_already_current_or_pending(
                ctx,
                action.id(),
                runtime_config.postgres.data_dir.as_path(),
                &start_intent,
            )? {
                return Ok(ProcessDispatchOutcome::Skipped);
            }
            dispatch_start_postgres(ctx, ha_tick, action_index, action, runtime_config, &start_intent)
        }
        HaAction::DemoteToReplica => {
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Demote(DemoteSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    mode: ctx.process_defaults.shutdown_mode.clone(),
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::PromoteToPrimary => {
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Promote(PromoteSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    wait_seconds: None,
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::StartRewind { leader_member_id } => {
            let source = validate_rewind_source(ctx, action.id(), leader_member_id)?;
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::PgRewind(PgRewindSpec {
                    target_data_dir: runtime_config.postgres.data_dir.clone(),
                    source,
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::StartBaseBackup { leader_member_id } => {
            let source = validate_basebackup_source(ctx, action.id(), leader_member_id)?;
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::BaseBackup(BaseBackupSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    source,
                    timeout_ms: Some(runtime_config.process.bootstrap_timeout_ms),
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::RunBootstrap => {
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Bootstrap(BootstrapSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    superuser_username: runtime_config.postgres.roles.superuser.username.clone(),
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::FenceNode => {
            let request = ProcessJobRequest {
                id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
                kind: ProcessJobKind::Fencing(FencingSpec {
                    data_dir: runtime_config.postgres.data_dir.clone(),
                    mode: ShutdownMode::Immediate,
                    timeout_ms: None,
                }),
            };
            send_process_request(ctx, action.id(), request)?;
            Ok(ProcessDispatchOutcome::Applied)
        }
        HaAction::WipeDataDir => {
            wipe_data_dir(runtime_config.postgres.data_dir.as_path()).map_err(|message| {
                ProcessDispatchError::Filesystem {
                    action: action.id(),
                    message,
                }
            })?;
            Ok(ProcessDispatchOutcome::Applied)
        }
    }
}

fn dispatch_start_postgres(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
    runtime_config: &RuntimeConfig,
    start_intent: &ManagedPostgresStartIntent,
) -> Result<ProcessDispatchOutcome, ProcessDispatchError> {
    let managed = crate::postgres_managed::materialize_managed_postgres_config(
        runtime_config,
        start_intent,
    )
    .map_err(|err| ProcessDispatchError::ManagedConfig {
        action: action.id(),
        message: err.to_string(),
    })?;
    let request = ProcessJobRequest {
        id: process_job_id(&ctx.scope, &ctx.self_id, action, action_index, ha_tick),
        kind: ProcessJobKind::StartPostgres(StartPostgresSpec {
            data_dir: runtime_config.postgres.data_dir.clone(),
            config_file: managed.postgresql_conf_path,
            log_file: ctx.process_defaults.log_file.clone(),
            wait_seconds: None,
            timeout_ms: None,
        }),
    };
    send_process_request(ctx, action.id(), request)?;
    Ok(ProcessDispatchOutcome::Applied)
}

pub(crate) fn validate_rewind_source(
    ctx: &HaWorkerCtx,
    action: ActionId,
    leader_member_id: &crate::state::MemberId,
) -> Result<crate::process::jobs::RewinderSourceConn, ProcessDispatchError> {
    let member = resolve_source_member(ctx, action.clone(), leader_member_id)?;
    rewind_source_from_member(&ctx.self_id, &member, &ctx.process_defaults).map_err(|err| {
        ProcessDispatchError::SourceSelection {
            action,
            message: err.to_string(),
        }
    })
}

pub(crate) fn validate_basebackup_source(
    ctx: &HaWorkerCtx,
    action: ActionId,
    leader_member_id: &crate::state::MemberId,
) -> Result<crate::process::jobs::ReplicatorSourceConn, ProcessDispatchError> {
    let member = resolve_source_member(ctx, action.clone(), leader_member_id)?;
    basebackup_source_from_member(&ctx.self_id, &member, &ctx.process_defaults).map_err(|err| {
        ProcessDispatchError::SourceSelection {
            action,
            message: err.to_string(),
        }
    })
}

fn resolve_source_member(
    ctx: &HaWorkerCtx,
    action: ActionId,
    leader_member_id: &crate::state::MemberId,
) -> Result<MemberRecord, ProcessDispatchError> {
    let dcs = ctx.dcs_subscriber.latest();
    dcs.value
        .cache
        .members
        .get(leader_member_id)
        .cloned()
        .ok_or_else(|| ProcessDispatchError::SourceSelection {
            action,
            message: format!(
                "target member `{}` not present in DCS cache",
                leader_member_id.0
            ),
        })
}

fn send_process_request(
    ctx: &mut HaWorkerCtx,
    action: ActionId,
    request: ProcessJobRequest,
) -> Result<(), ProcessDispatchError> {
    ctx.process_inbox
        .send(request)
        .map_err(|err| ProcessDispatchError::ProcessSend {
            action,
            message: err.to_string(),
        })
}

fn managed_start_intent_from_dcs(
    ctx: &HaWorkerCtx,
    action: ActionId,
    replica_leader_member_id: Option<&MemberId>,
    data_dir: &Path,
) -> Result<ManagedPostgresStartIntent, ProcessDispatchError> {
    if let Some(leader_member_id) = replica_leader_member_id {
        let leader = resolve_source_member(ctx, action.clone(), leader_member_id)?;
        let source = basebackup_source_from_member(&ctx.self_id, &leader, &ctx.process_defaults)
            .map_err(|err| ProcessDispatchError::SourceSelection {
                action: action.clone(),
                message: err.to_string(),
            })?;
        return Ok(ManagedPostgresStartIntent::replica(
            source.conninfo.clone(),
            managed_standby_auth_from_role_auth(&source.auth, data_dir),
            None,
        ));
    }

    let inspected = inspect_local_physical_state(data_dir, &ctx.process_defaults.postgres_binary)
        .map_err(|err| ProcessDispatchError::ManagedConfig {
            action: action.clone(),
            message: err.to_string(),
        })?;

    if inspected.signal_file_state != SignalFileState::None {
        return Err(ProcessDispatchError::ManagedConfig {
            action,
            message:
                "existing postgres data dir contains managed replica recovery state but no leader-derived source is available to rebuild authoritative managed config"
                    .to_string(),
        });
    }

    Ok(ManagedPostgresStartIntent::primary())
}

fn start_replica_is_already_current_or_pending(
    ctx: &HaWorkerCtx,
    action: ActionId,
    data_dir: &Path,
    start_intent: &ManagedPostgresStartIntent,
) -> Result<bool, ProcessDispatchError> {
    let Some((expected_primary_conninfo, _)) = standby_start_details(start_intent) else {
        return Ok(false);
    };

    let pg = ctx.pg_subscriber.latest();
    let Some(current_primary_conninfo) = current_primary_conninfo(&pg.value) else {
        return managed_config_already_targets_start_intent(action, data_dir, start_intent);
    };
    if current_primary_conninfo.host == expected_primary_conninfo.host
        && current_primary_conninfo.port == expected_primary_conninfo.port
    {
        return Ok(true);
    }
    if pginfo_common(&pg.value).sql == crate::pginfo::state::SqlStatus::Healthy {
        return Ok(false);
    }

    managed_config_already_targets_start_intent(action, data_dir, start_intent)
}

fn pginfo_common(state: &PgInfoState) -> &crate::pginfo::state::PgInfoCommon {
    match state {
        PgInfoState::Unknown { common }
        | PgInfoState::Primary { common, .. }
        | PgInfoState::Replica { common, .. } => common,
    }
}

fn standby_start_details(
    start_intent: &ManagedPostgresStartIntent,
) -> Option<(&crate::pginfo::state::PgConnInfo, &ManagedStandbyAuth)> {
    match start_intent {
        ManagedPostgresStartIntent::Replica {
            primary_conninfo,
            standby_auth,
            ..
        }
        | ManagedPostgresStartIntent::Recovery {
            primary_conninfo,
            standby_auth,
            ..
        } => Some((primary_conninfo, standby_auth)),
        ManagedPostgresStartIntent::Primary => None,
    }
}

fn managed_config_already_targets_start_intent(
    action: ActionId,
    data_dir: &Path,
    start_intent: &ManagedPostgresStartIntent,
) -> Result<bool, ProcessDispatchError> {
    let Some((expected_primary_conninfo, standby_auth)) = standby_start_details(start_intent) else {
        return Ok(false);
    };
    let managed_conf_path = data_dir.join(MANAGED_POSTGRESQL_CONF_NAME);
    let rendered = match fs::read_to_string(&managed_conf_path) {
        Ok(rendered) => rendered,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(err) => {
            return Err(ProcessDispatchError::ManagedConfig {
                action,
                message: format!(
                    "read managed postgres config failed at {}: {err}",
                    managed_conf_path.display()
                ),
            });
        }
    };
    let expected_recovery_state = start_intent.recovery_signal();
    let actual_recovery_state = crate::postgres_managed::inspect_managed_recovery_state(data_dir)
        .map_err(|err| ProcessDispatchError::ManagedConfig {
            action: action.clone(),
            message: err.to_string(),
        })?;
    if actual_recovery_state != expected_recovery_state {
        return Ok(false);
    }

    Ok(rendered.contains(
        render_managed_primary_conninfo(expected_primary_conninfo, standby_auth).as_str(),
    ))
}

fn current_primary_conninfo(state: &PgInfoState) -> Option<&crate::pginfo::state::PgConnInfo> {
    match state {
        PgInfoState::Unknown { common }
        | PgInfoState::Primary { common, .. }
        | PgInfoState::Replica { common, .. } => common.pg_config.primary_conninfo.as_ref(),
    }
}

fn process_job_id(
    scope: &str,
    self_id: &crate::state::MemberId,
    action: &HaAction,
    index: usize,
    tick: u64,
) -> JobId {
    JobId(format!(
        "ha-{}-{}-{}-{}-{}",
        scope.trim_matches('/'),
        self_id.0,
        tick,
        index,
        action.id().label(),
    ))
}

fn wipe_data_dir(data_dir: &Path) -> Result<(), String> {
    if data_dir.as_os_str().is_empty() {
        return Err("wipe_data_dir data_dir must not be empty".to_string());
    }
    if data_dir.exists() {
        fs::remove_dir_all(data_dir)
            .map_err(|err| format!("wipe_data_dir remove_dir_all failed: {err}"))?;
    }
    fs::create_dir_all(data_dir)
        .map_err(|err| format!("wipe_data_dir create_dir_all failed: {err}"))?;
    set_postgres_data_dir_permissions(data_dir)?;
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

#[allow(dead_code)]
fn postgres_data_dir_requires_basebackup(
    postgres_binary: &Path,
    data_dir: &Path,
) -> Result<bool, ProcessDispatchError> {
    let inspected = inspect_local_physical_state(data_dir, postgres_binary).map_err(|err| {
        ProcessDispatchError::Filesystem {
            action: ActionId::StartPrimary,
            message: err.to_string(),
        }
    })?;
    Ok(!matches!(inspected.data_dir_kind, DataDirKind::Initialized))
}
