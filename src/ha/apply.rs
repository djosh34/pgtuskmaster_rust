use thiserror::Error;

use crate::{
    dcs::store::{DcsHaWriter, DcsStoreError},
    state::WorkerError,
};

use super::{
    actions::{ActionId, HaAction},
    events::{
        emit_ha_action_dispatch, emit_ha_action_intent, emit_ha_action_result_failed,
        emit_ha_action_result_ok, emit_ha_action_result_skipped, emit_ha_lease_transition,
    },
    lower::{
        HaEffectPlan, LeaseEffect, PostgresEffect, ReplicationEffect, SafetyEffect,
        SwitchoverEffect,
    },
    process_dispatch::{dispatch_process_action, ProcessDispatchError, ProcessDispatchOutcome},
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

    action_index = dispatch_postgres_effect(
        ctx,
        ha_tick,
        action_index,
        &plan.postgres,
        &runtime_config,
        &mut errors,
    )?;
    action_index = dispatch_lease_effect(
        ctx,
        ha_tick,
        action_index,
        &plan.lease,
        &runtime_config,
        &mut errors,
    )?;
    action_index = dispatch_switchover_effect(
        ctx,
        ha_tick,
        action_index,
        &plan.switchover,
        &runtime_config,
        &mut errors,
    )?;
    action_index = dispatch_replication_effect(
        ctx,
        ha_tick,
        action_index,
        &plan.replication,
        &runtime_config,
        &mut errors,
    )?;
    let _ = dispatch_safety_effect(
        ctx,
        ha_tick,
        action_index,
        &plan.safety,
        &runtime_config,
        &mut errors,
    )?;

    Ok(errors)
}

pub(crate) fn format_dispatch_errors(errors: &[ActionDispatchError]) -> String {
    let mut details = String::new();
    for (index, err) in errors.iter().enumerate() {
        if index > 0 {
            details.push_str("; ");
        }
        details.push_str(&err.to_string());
    }
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
) -> Result<usize, WorkerError> {
    match effect {
        PostgresEffect::None => Ok(action_index),
        PostgresEffect::Start => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::StartPostgres,
            runtime_config,
            errors,
        ),
        PostgresEffect::Promote => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::PromoteToPrimary,
            runtime_config,
            errors,
        ),
        PostgresEffect::Demote => dispatch_effect_action(
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

fn dispatch_replication_effect(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    effect: &ReplicationEffect,
    runtime_config: &crate::config::RuntimeConfig,
    errors: &mut Vec<ActionDispatchError>,
) -> Result<usize, WorkerError> {
    match effect {
        ReplicationEffect::None => Ok(action_index),
        ReplicationEffect::FollowLeader { leader_member_id } => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::FollowLeader {
                leader_member_id: leader_member_id.0.clone(),
            },
            runtime_config,
            errors,
        ),
        ReplicationEffect::RecoverReplica { strategy } => match strategy {
            crate::ha::decision::RecoveryStrategy::Rewind { .. } => dispatch_effect_action(
                ctx,
                ha_tick,
                action_index,
                HaAction::StartRewind,
                runtime_config,
                errors,
            ),
            crate::ha::decision::RecoveryStrategy::BaseBackup { .. } => {
                let next_index = dispatch_effect_action(
                    ctx,
                    ha_tick,
                    action_index,
                    HaAction::WipeDataDir,
                    runtime_config,
                    errors,
                )?;
                dispatch_effect_action(
                    ctx,
                    ha_tick,
                    next_index,
                    HaAction::StartBaseBackup,
                    runtime_config,
                    errors,
                )
            }
            crate::ha::decision::RecoveryStrategy::Bootstrap => {
                let next_index = dispatch_effect_action(
                    ctx,
                    ha_tick,
                    action_index,
                    HaAction::WipeDataDir,
                    runtime_config,
                    errors,
                )?;
                dispatch_effect_action(
                    ctx,
                    ha_tick,
                    next_index,
                    HaAction::RunBootstrap,
                    runtime_config,
                    errors,
                )
            }
        },
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
        SafetyEffect::FenceNode => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::FenceNode,
            runtime_config,
            errors,
        ),
        SafetyEffect::SignalFailSafe => dispatch_effect_action(
            ctx,
            ha_tick,
            action_index,
            HaAction::SignalFailSafe,
            runtime_config,
            errors,
        ),
    }
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
            let dispatch_result = acquire_leader_lease(ctx);
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
            let dispatch_result = release_leader_lease(ctx);
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
            let result = clear_switchover_request(ctx);
            dcs_delete_result(ctx, ha_tick, action_index, action, path, result)
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
            let message = err.to_string();
            emit_ha_action_result_failed(ctx, ha_tick, action_index, action, message)?;
            let error = if acquired {
                ActionDispatchError::DcsWrite {
                    action: action.id(),
                    path,
                    message: dcs_error_message(err),
                }
            } else {
                ActionDispatchError::DcsDelete {
                    action: action.id(),
                    path,
                    message: dcs_error_message(err),
                }
            };
            Ok(Some(error))
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
            let message = err.to_string();
            emit_ha_action_result_failed(ctx, ha_tick, action_index, action, message)?;
            Ok(Some(ActionDispatchError::DcsDelete {
                action: action.id(),
                path,
                message: dcs_error_message(err),
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
        Err(ProcessDispatchError::UnsupportedAction { action }) => {
            Err(WorkerError::Message(format!(
                "ha apply routed unsupported process action `{}`",
                action.label()
            )))
        }
        Err(err) => {
            let message = err.to_string();
            emit_ha_action_result_failed(ctx, ha_tick, action_index, action, message)?;
            Ok(Some(map_process_dispatch_error(err)))
        }
    }
}

fn acquire_leader_lease(ctx: &mut HaWorkerCtx) -> Result<(), DcsStoreError> {
    ctx.dcs_store.write_leader_lease(&ctx.scope, &ctx.self_id)
}

fn release_leader_lease(ctx: &mut HaWorkerCtx) -> Result<(), DcsStoreError> {
    ctx.dcs_store.delete_leader(&ctx.scope)
}

fn clear_switchover_request(ctx: &mut HaWorkerCtx) -> Result<(), DcsStoreError> {
    ctx.dcs_store.clear_switchover(&ctx.scope)
}

fn leader_path(scope: &str) -> String {
    format!("/{}/leader", scope.trim_matches('/'))
}

fn switchover_path(scope: &str) -> String {
    format!("/{}/switchover", scope.trim_matches('/'))
}

fn dcs_error_message(error: DcsStoreError) -> String {
    error.to_string()
}

fn map_process_dispatch_error(error: ProcessDispatchError) -> ActionDispatchError {
    match error {
        ProcessDispatchError::ProcessSend { action, message } => {
            ActionDispatchError::ProcessSend { action, message }
        }
        ProcessDispatchError::ManagedConfig { action, message } => {
            ActionDispatchError::ManagedConfig { action, message }
        }
        ProcessDispatchError::Filesystem { action, message } => {
            ActionDispatchError::Filesystem { action, message }
        }
        ProcessDispatchError::UnsupportedAction { action } => ActionDispatchError::ProcessSend {
            action,
            message: "unsupported process action".to_string(),
        },
    }
}
