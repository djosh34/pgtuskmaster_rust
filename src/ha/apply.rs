use thiserror::Error;

use crate::process::state::ProcessState;
use crate::{dcs::store::DcsStoreError, state::WorkerError};

use super::{
    actions::{ActionId, HaAction},
    events::{
        emit_ha_action_dispatch, emit_ha_action_intent, emit_ha_action_result_failed,
        emit_ha_action_result_ok, emit_ha_action_result_skipped, emit_ha_lease_transition,
    },
    lower::{
        HaEffectPlan, LeaseEffect, PostgresEffect, RecoveryEffect, SafetyEffect, SwitchoverEffect,
    },
    process_dispatch::{
        dispatch_process_action, validate_basebackup_source, ProcessDispatchError,
        ProcessDispatchOutcome,
    },
    state::HaWorkerCtx,
};

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ActionDispatchError {
    #[error("process send failed for action `{action:?}`: {message}")]
    ProcessSend { action: ActionId, message: String },
    #[error("managed config materialization failed for action `{action:?}`: {message}")]
    ManagedConfig { action: ActionId, message: String },
    #[error("filesystem operation failed for action `{action:?}`: {message}")]
    Filesystem { action: ActionId, message: String },
    #[error("remote source selection failed for action `{action:?}`: {message}")]
    SourceSelection { action: ActionId, message: String },
    #[error("dcs write failed for action `{action:?}` at `{path}`: {message}")]
    DcsWrite {
        action: ActionId,
        path: String,
        message: String,
    },
    #[error("dcs delete failed for action `{action:?}` at `{path}`: {message}")]
    DcsDelete {
        action: ActionId,
        path: String,
        message: String,
    },
}

pub(crate) fn apply_effect_plan(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    plan: &HaEffectPlan,
) -> Result<Vec<ActionDispatchError>, WorkerError> {
    let runtime_config = ctx.config_subscriber.latest().value;
    let mut errors = Vec::new();
    let mut action_index = 0usize;

    if matches!(plan.lease, LeaseEffect::AcquireLeader) {
        action_index = dispatch_lease_effect(
            ctx,
            ha_tick,
            action_index,
            &plan.lease,
            &runtime_config,
            &mut errors,
        )?;
    }

    let (next_index, postgres_dispatched) = dispatch_postgres_effect(
        ctx,
        ha_tick,
        action_index,
        &plan.postgres,
        &runtime_config,
        &mut errors,
    )?;
    action_index = next_index;
    let recovery_dispatch_blocked =
        recovery_dispatch_is_blocked(&ctx.process_subscriber.latest().value, postgres_dispatched);
    action_index = dispatch_recovery_effect(
        ctx,
        ha_tick,
        action_index,
        &plan.recovery,
        &runtime_config,
        recovery_dispatch_blocked,
        &mut errors,
    )?;
    action_index = dispatch_safety_effect(
        ctx,
        ha_tick,
        action_index,
        &plan.safety,
        &runtime_config,
        &mut errors,
    )?;

    if matches!(plan.lease, LeaseEffect::ReleaseLeader) {
        action_index = dispatch_lease_effect(
            ctx,
            ha_tick,
            action_index,
            &plan.lease,
            &runtime_config,
            &mut errors,
        )?;
    }

    let _ = dispatch_switchover_effect(
        ctx,
        ha_tick,
        action_index,
        &plan.switchover,
        &runtime_config,
        &mut errors,
    )?;

    Ok(errors)
}

pub(crate) fn format_dispatch_errors(errors: &[ActionDispatchError]) -> String {
    let details = errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "ha dispatch failed with {} error(s): {details}",
        errors.len()
    )
}

fn dispatch_postgres_effect(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    effect: &PostgresEffect,
    runtime_config: &crate::config::RuntimeConfig,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<(usize, bool), WorkerError> {
    match effect {
        PostgresEffect::None => Ok((action_index, false)),
        PostgresEffect::StartPrimary => dispatch_process_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::StartPrimary,
            runtime_config,
            errors,
        ),
        PostgresEffect::StartReplica { leader_member_id } => dispatch_process_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::StartReplica {
                leader_member_id: leader_member_id.clone(),
            },
            runtime_config,
            errors,
        ),
        PostgresEffect::Promote => dispatch_process_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::PromoteToPrimary,
            runtime_config,
            errors,
        ),
        PostgresEffect::Demote => dispatch_process_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::DemoteToReplica,
            runtime_config,
            errors,
        ),
    }
}

fn dispatch_lease_effect(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    effect: &LeaseEffect,
    runtime_config: &crate::config::RuntimeConfig,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<usize, WorkerError> {
    match effect {
        LeaseEffect::None => Ok(action_index),
        LeaseEffect::AcquireLeader => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::AcquireLeaderLease,
            runtime_config,
            errors,
        ),
        LeaseEffect::ReleaseLeader => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::ReleaseLeaderLease,
            runtime_config,
            errors,
        ),
    }
}

fn dispatch_switchover_effect(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    effect: &SwitchoverEffect,
    runtime_config: &crate::config::RuntimeConfig,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<usize, WorkerError> {
    match effect {
        SwitchoverEffect::None => Ok(action_index),
        SwitchoverEffect::ClearRequest => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::ClearSwitchover,
            runtime_config,
            errors,
        ),
    }
}

fn dispatch_recovery_effect(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    effect: &RecoveryEffect,
    runtime_config: &crate::config::RuntimeConfig,
    recovery_dispatch_blocked: bool,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<usize, WorkerError> {
    if recovery_dispatch_blocked {
        return Ok(action_index);
    }
    match effect {
        RecoveryEffect::None => Ok(action_index),
        RecoveryEffect::Rewind { leader_member_id } => dispatch_process_only_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::StartRewind {
                leader_member_id: leader_member_id.clone(),
            },
            runtime_config,
            errors,
        ),
        RecoveryEffect::Basebackup { leader_member_id } => {
            if let Err(err) =
                validate_basebackup_source(ctx, ActionId::StartBaseBackup, leader_member_id)
            {
                errors.push(map_process_dispatch_error(err));
                return Ok(action_index);
            }
            let next_index = dispatch_effect_action(
                ctx,
                ha_tick,
                action_index,
                HaAction::WipeDataDir,
                runtime_config,
                errors,
            )?;
            dispatch_process_only_effect_action(
                ctx,
                ha_tick,
                next_index,
                HaAction::StartBaseBackup {
                    leader_member_id: leader_member_id.clone(),
                },
                runtime_config,
                errors,
            )
        }
        RecoveryEffect::Bootstrap => {
            let next_index = dispatch_effect_action(
                ctx,
                ha_tick,
                action_index,
                HaAction::WipeDataDir,
                runtime_config,
                errors,
            )?;
            dispatch_process_only_effect_action(
                ctx,
                ha_tick,
                next_index,
                HaAction::RunBootstrap,
                runtime_config,
                errors,
            )
        }
    }
}

fn dispatch_safety_effect(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    effect: &SafetyEffect,
    runtime_config: &crate::config::RuntimeConfig,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<usize, WorkerError> {
    match effect {
        SafetyEffect::None => Ok(action_index),
        SafetyEffect::FenceNode => dispatch_process_only_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::FenceNode,
            runtime_config,
            errors,
        ),
    }
}

fn dispatch_process_effect_action(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: HaAction,
    runtime_config: &crate::config::RuntimeConfig,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<(usize, bool), WorkerError> {
    let next_index =
        dispatch_effect_action(ctx, ha_tick, action_index, action, runtime_config, errors)?;
    Ok((next_index, true))
}

fn dispatch_process_only_effect_action(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: HaAction,
    runtime_config: &crate::config::RuntimeConfig,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<usize, WorkerError> {
    dispatch_effect_action(ctx, ha_tick, action_index, action, runtime_config, errors)
}

fn recovery_dispatch_is_blocked(
    process_state: &ProcessState,
    postgres_dispatch_started: bool,
) -> bool {
    postgres_dispatch_started || matches!(process_state, ProcessState::Running { .. })
}

fn dispatch_effect_action(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: HaAction,
    runtime_config: &crate::config::RuntimeConfig,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<usize, WorkerError> {
    emit_ha_action_intent(ctx, ha_tick, action_index, &action)?;
    emit_ha_action_dispatch(ctx, ha_tick, action_index, &action)?;

    if let Some(error) = dispatch_action(ctx, ha_tick, action_index, &action, runtime_config)? {
        errors.push(error);
    }

    Ok(action_index.saturating_add(1))
}

fn dispatch_action(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
    runtime_config: &crate::config::RuntimeConfig,
) -> Result<Option<ActionDispatchError>, WorkerError> {
    match action {
        HaAction::AcquireLeaderLease => {
            let dispatch_result = ctx.dcs_store.acquire_leader_lease(&ctx.scope, &ctx.self_id);
            dcs_dispatch_result(
                ctx,
                ha_tick,
                action_index,
                action,
                leader_path(&ctx.scope),
                dispatch_result,
                true,
            )
        }
        HaAction::ReleaseLeaderLease => {
            let dispatch_result = ctx.dcs_store.release_leader_lease(&ctx.scope, &ctx.self_id);
            dcs_dispatch_result(
                ctx,
                ha_tick,
                action_index,
                action,
                leader_path(&ctx.scope),
                dispatch_result,
                false,
            )
        }
        HaAction::ClearSwitchover => {
            let path = switchover_path(&ctx.scope);
            let dispatch_result = ctx.dcs_store.clear_switchover(&ctx.scope);
            dcs_delete_result(ctx, ha_tick, action_index, action, path, dispatch_result)
        }
        _ => {
            let result =
                dispatch_process_action(ctx, ha_tick, action_index, action, runtime_config);
            process_dispatch_result(ctx, ha_tick, action_index, action, result)
        }
    }
}

fn dcs_dispatch_result(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
    path: String,
    result: Result<(), DcsStoreError>,
    acquired: bool,
) -> Result<Option<ActionDispatchError>, WorkerError> {
    match result {
        Ok(()) => {
            emit_ha_action_result_ok(ctx, ha_tick, action_index, action)?;
            emit_ha_lease_transition(ctx, ha_tick, acquired)?;
            Ok(None)
        }
        Err(err) => {
            emit_ha_action_result_failed(ctx, ha_tick, action_index, action, err.to_string())?;
            Ok(Some(if acquired {
                ActionDispatchError::DcsWrite {
                    action: action.id(),
                    path,
                    message: err.to_string(),
                }
            } else {
                ActionDispatchError::DcsDelete {
                    action: action.id(),
                    path,
                    message: err.to_string(),
                }
            }))
        }
    }
}

fn dcs_delete_result(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
    path: String,
    result: Result<(), DcsStoreError>,
) -> Result<Option<ActionDispatchError>, WorkerError> {
    match result {
        Ok(()) => {
            emit_ha_action_result_ok(ctx, ha_tick, action_index, action)?;
            Ok(None)
        }
        Err(err) => {
            emit_ha_action_result_failed(ctx, ha_tick, action_index, action, err.to_string())?;
            Ok(Some(ActionDispatchError::DcsDelete {
                action: action.id(),
                path,
                message: err.to_string(),
            }))
        }
    }
}

fn process_dispatch_result(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
    result: Result<ProcessDispatchOutcome, ProcessDispatchError>,
) -> Result<Option<ActionDispatchError>, WorkerError> {
    match result {
        Ok(ProcessDispatchOutcome::Applied) => {
            emit_ha_action_result_ok(ctx, ha_tick, action_index, action)?;
            Ok(None)
        }
        Ok(ProcessDispatchOutcome::Skipped) => {
            emit_ha_action_result_skipped(ctx, ha_tick, action_index, action)?;
            Ok(None)
        }
        Err(err) => {
            emit_ha_action_result_failed(ctx, ha_tick, action_index, action, err.to_string())?;
            Ok(Some(map_process_dispatch_error(err)))
        }
    }
}

fn map_process_dispatch_error(err: ProcessDispatchError) -> ActionDispatchError {
    match err {
        ProcessDispatchError::ProcessSend { action, message } => {
            ActionDispatchError::ProcessSend { action, message }
        }
        ProcessDispatchError::ManagedConfig { action, message } => {
            ActionDispatchError::ManagedConfig { action, message }
        }
        ProcessDispatchError::Filesystem { action, message } => {
            ActionDispatchError::Filesystem { action, message }
        }
        ProcessDispatchError::SourceSelection { action, message } => {
            ActionDispatchError::SourceSelection { action, message }
        }
        ProcessDispatchError::UnsupportedAction { action } => ActionDispatchError::ProcessSend {
            action,
            message: "unsupported process action".to_string(),
        },
    }
}

fn leader_path(scope: &str) -> String {
    format!("/{}/leader", scope.trim_matches('/'))
}

fn switchover_path(scope: &str) -> String {
    format!("/{}/switchover", scope.trim_matches('/'))
}

#[cfg(test)]
mod tests {
    use crate::{
        process::{
            jobs::{ActiveJob, ActiveJobKind},
            state::ProcessState,
        },
        state::{JobId, UnixMillis, WorkerStatus},
    };

    use super::recovery_dispatch_is_blocked;

    #[test]
    fn running_process_job_blocks_new_recovery_dispatch() {
        let process_state = ProcessState::Running {
            worker: WorkerStatus::Running,
            active: ActiveJob {
                id: JobId("job-1".to_string()),
                kind: ActiveJobKind::Demote,
                started_at: UnixMillis(1),
                deadline_at: UnixMillis(2),
            },
        };

        assert!(recovery_dispatch_is_blocked(&process_state, false));
    }

    #[test]
    fn started_postgres_dispatch_blocks_later_recovery_phase_in_same_tick() {
        let process_state = ProcessState::Idle {
            worker: WorkerStatus::Running,
            last_outcome: None,
        };

        assert!(recovery_dispatch_is_blocked(&process_state, true));
    }
}
