use crate::{
    process::{jobs::ActiveJobKind, state::ProcessState},
    state::{WorkerError, WorkerStatus},
};

use super::{
    apply::{apply_effect_plan, format_dispatch_errors},
    decide::decide,
    decision::reconcile_facts,
    events::{
        emit_cluster_mode_transition, emit_desired_state_selected, emit_effect_plan_selected,
        emit_node_role_transition, emit_state_transition, node_role_label,
    },
    lower::{lower_desired_state, HaEffectPlan, PostgresEffect, RecoveryEffect, SafetyEffect},
    state::{DecideInput, HaWorkerCtx, WorldSnapshot},
};

pub(crate) async fn run(mut ctx: HaWorkerCtx) -> Result<(), WorkerError> {
    let mut interval = tokio::time::interval(ctx.poll_interval);
    loop {
        tokio::select! {
            changed = ctx.pg_subscriber.changed() => {
                changed.map_err(|err| WorkerError::Message(format!("ha pg subscriber closed: {err}")))?;
            }
            changed = ctx.dcs_subscriber.changed() => {
                changed.map_err(|err| WorkerError::Message(format!("ha dcs subscriber closed: {err}")))?;
            }
            changed = ctx.process_subscriber.changed() => {
                changed.map_err(|err| WorkerError::Message(format!("ha process subscriber closed: {err}")))?;
            }
            changed = ctx.config_subscriber.changed() => {
                changed.map_err(|err| WorkerError::Message(format!("ha config subscriber closed: {err}")))?;
            }
            _ = interval.tick() => {}
        }
        step_once(&mut ctx).await?;
    }
}

pub(crate) async fn step_once(ctx: &mut HaWorkerCtx) -> Result<(), WorkerError> {
    let previous_state = ctx.state.clone();
    let world = world_snapshot(ctx);
    let facts = reconcile_facts(&ctx.state, &world);
    let output = decide(DecideInput {
        current: ctx.state.clone(),
        world,
    });
    let plan = lower_desired_state(&output.next.desired_state, &facts);
    let process_state = ctx.process_subscriber.latest().value;
    let skip_redundant_process_dispatch =
        should_skip_redundant_process_dispatch(&ctx.state, &output.next, &plan, &process_state);

    emit_desired_state_selected(ctx, output.next.tick, &output.next.desired_state, &plan)?;
    emit_effect_plan_selected(ctx, output.next.tick, &plan)?;

    let published_next = crate::ha::state::HaState {
        worker: WorkerStatus::Running,
        ..output.next.clone()
    };
    let now = (ctx.now)()?;

    ctx.publisher
        .publish(published_next.clone(), now)
        .map_err(|err| WorkerError::Message(format!("ha publish failed: {err}")))?;

    if previous_state.cluster_mode != published_next.cluster_mode {
        emit_cluster_mode_transition(
            ctx,
            published_next.tick,
            &previous_state.cluster_mode,
            &published_next.cluster_mode,
        )?;
    }
    if previous_state.desired_state != published_next.desired_state {
        emit_state_transition(
            ctx,
            published_next.tick,
            &previous_state.desired_state,
            &published_next.desired_state,
        )?;
    }

    let previous_role = node_role_label(&previous_state.desired_state);
    let next_role = node_role_label(&published_next.desired_state);
    if previous_role != next_role {
        emit_node_role_transition(ctx, published_next.tick, previous_role, next_role)?;
    }

    ctx.state = published_next.clone();

    let dispatch_errors = if skip_redundant_process_dispatch {
        Vec::new()
    } else {
        apply_effect_plan(ctx, published_next.tick, &plan)?
    };
    if !dispatch_errors.is_empty() {
        let faulted = crate::ha::state::HaState {
            worker: WorkerStatus::Faulted(WorkerError::Message(format_dispatch_errors(
                &dispatch_errors,
            ))),
            ..published_next
        };
        let faulted_now = (ctx.now)()?;
        ctx.publisher
            .publish(faulted.clone(), faulted_now)
            .map_err(|err| WorkerError::Message(format!("ha publish failed: {err}")))?;
        ctx.state = faulted;
    }

    Ok(())
}

fn world_snapshot(ctx: &HaWorkerCtx) -> WorldSnapshot {
    WorldSnapshot {
        config: ctx.config_subscriber.latest(),
        pg: ctx.pg_subscriber.latest(),
        dcs: ctx.dcs_subscriber.latest(),
        process: ctx.process_subscriber.latest(),
    }
}

fn should_skip_redundant_process_dispatch(
    current: &crate::ha::state::HaState,
    next: &crate::ha::state::HaState,
    plan: &HaEffectPlan,
    process_state: &ProcessState,
) -> bool {
    current.cluster_mode == next.cluster_mode
        && current.desired_state == next.desired_state
        && (plan_effect_is_already_active(plan, process_state) || plan_dispatch_is_latched(plan))
}

fn plan_dispatch_is_latched(plan: &HaEffectPlan) -> bool {
    matches!(
        plan.postgres,
        PostgresEffect::StartPrimary | PostgresEffect::StartReplica { .. }
    )
}

fn plan_effect_is_already_active(plan: &HaEffectPlan, process_state: &ProcessState) -> bool {
    let kinds = match (&plan.recovery, &plan.postgres, &plan.safety) {
        (RecoveryEffect::Rewind { .. }, _, _) => vec![ActiveJobKind::PgRewind],
        (RecoveryEffect::Basebackup { .. }, _, _) => vec![ActiveJobKind::BaseBackup],
        (RecoveryEffect::Bootstrap, _, _) => vec![ActiveJobKind::Bootstrap],
        (_, PostgresEffect::StartPrimary | PostgresEffect::StartReplica { .. }, _) => {
            vec![ActiveJobKind::StartPostgres]
        }
        (_, PostgresEffect::Promote, _) => vec![ActiveJobKind::Promote],
        (_, PostgresEffect::Demote, _) => vec![ActiveJobKind::Demote],
        (_, _, SafetyEffect::FenceNode) => vec![ActiveJobKind::Fencing],
        _ => Vec::new(),
    };
    if kinds.is_empty() {
        return false;
    }
    match process_state {
        ProcessState::Running { active, .. } => kinds.contains(&active.kind),
        ProcessState::Idle { .. } => false,
    }
}
