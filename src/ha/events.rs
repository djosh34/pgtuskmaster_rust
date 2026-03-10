use crate::{
    logging::{AppEvent, AppEventHeader, SeverityText, StructuredFields},
    state::WorkerError,
};

use super::{
    actions::HaAction,
    lower::HaEffectPlan,
    state::{ClusterMode, DesiredNodeState, HaWorkerCtx},
};

fn ha_append_base_fields(fields: &mut StructuredFields, ctx: &HaWorkerCtx, ha_tick: u64) {
    fields.insert("scope", ctx.scope.clone());
    fields.insert("member_id", ctx.self_id.0.clone());
    fields.insert("ha_tick", ha_tick);
    fields.insert("ha_dispatch_seq", ha_tick);
}

fn ha_append_action_fields(fields: &mut StructuredFields, action_index: usize, action: &HaAction) {
    fields.insert("action_index", action_index);
    fields.insert("action_id", action.id().label());
    match action {
        HaAction::StartReplica { leader_member_id }
        | HaAction::StartRewind { leader_member_id }
        | HaAction::StartBaseBackup { leader_member_id } => {
            fields.insert("leader_member_id", leader_member_id.0.clone());
        }
        _ => {}
    }
}

fn ha_insert_serialized<T: serde::Serialize>(
    fields: &mut StructuredFields,
    key: &str,
    value: &T,
) -> Result<(), WorkerError> {
    fields
        .insert_serialized(key, value)
        .map_err(|err| WorkerError::Message(format!("ha attr serialization failed: {err}")))
}

fn ha_event(severity: SeverityText, message: &str, name: &str, result: &str) -> AppEvent {
    AppEvent::new(severity, message, AppEventHeader::new(name, "ha", result))
}

fn emit_ha_event(
    ctx: &HaWorkerCtx,
    origin: &str,
    event: AppEvent,
    error_prefix: &str,
) -> Result<(), WorkerError> {
    ctx.log
        .emit_app_event(origin, event)
        .map_err(|err| WorkerError::Message(format!("{error_prefix}: {err}")))
}

pub(crate) fn emit_ha_action_intent(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Debug,
        "ha action intent",
        "ha.action.intent",
        "ok",
    );
    let fields = event.fields_mut();
    ha_append_base_fields(fields, ctx, ha_tick);
    ha_append_action_fields(fields, action_index, action);
    emit_ha_event(
        ctx,
        "ha_worker::step_once",
        event,
        "ha intent log emit failed",
    )
}

pub(crate) fn emit_desired_state_selected(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    desired_state: &DesiredNodeState,
    plan: &HaEffectPlan,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Debug,
        "ha desired state selected",
        "ha.desired_state.selected",
        "ok",
    );
    let fields = event.fields_mut();
    ha_append_base_fields(fields, ctx, ha_tick);
    ha_insert_serialized(fields, "desired_state", desired_state)?;
    fields.insert("planned_dispatch_step_count", plan.dispatch_step_count());
    emit_ha_event(
        ctx,
        "ha_worker::step_once",
        event,
        "ha desired_state log emit failed",
    )
}

pub(crate) fn emit_effect_plan_selected(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    plan: &HaEffectPlan,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Debug,
        "ha effect plan selected",
        "ha.effect_plan.selected",
        "ok",
    );
    let fields = event.fields_mut();
    ha_append_base_fields(fields, ctx, ha_tick);
    ha_insert_serialized(fields, "effect_plan", plan)?;
    emit_ha_event(
        ctx,
        "ha_worker::step_once",
        event,
        "ha effect plan log emit failed",
    )
}

pub(crate) fn emit_ha_action_dispatch(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Debug,
        "ha action dispatch",
        "ha.action.dispatch",
        "ok",
    );
    let fields = event.fields_mut();
    ha_append_base_fields(fields, ctx, ha_tick);
    ha_append_action_fields(fields, action_index, action);
    emit_ha_event(
        ctx,
        "ha_worker::dispatch_actions",
        event,
        "ha dispatch log emit failed",
    )
}

pub(crate) fn emit_ha_action_result_ok(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Debug,
        "ha action result",
        "ha.action.result",
        "ok",
    );
    let fields = event.fields_mut();
    ha_append_base_fields(fields, ctx, ha_tick);
    ha_append_action_fields(fields, action_index, action);
    emit_ha_event(
        ctx,
        "ha_worker::dispatch_actions",
        event,
        "ha result log emit failed",
    )
}

pub(crate) fn emit_ha_action_result_skipped(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Debug,
        "ha action skipped",
        "ha.action.result",
        "skipped",
    );
    let fields = event.fields_mut();
    ha_append_base_fields(fields, ctx, ha_tick);
    ha_append_action_fields(fields, action_index, action);
    emit_ha_event(
        ctx,
        "ha_worker::dispatch_actions",
        event,
        "ha result log emit failed",
    )
}

pub(crate) fn emit_ha_action_result_failed(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &HaAction,
    error: String,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Warn,
        "ha action failed",
        "ha.action.result",
        "failed",
    );
    let fields = event.fields_mut();
    ha_append_base_fields(fields, ctx, ha_tick);
    ha_append_action_fields(fields, action_index, action);
    fields.insert("error", error);
    emit_ha_event(
        ctx,
        "ha_worker::dispatch_actions",
        event,
        "ha result log emit failed",
    )
}

pub(crate) fn emit_ha_lease_transition(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    acquired: bool,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Info,
        if acquired {
            "ha leader lease acquired"
        } else {
            "ha leader lease released"
        },
        if acquired {
            "ha.lease.acquired"
        } else {
            "ha.lease.released"
        },
        "ok",
    );
    ha_append_base_fields(event.fields_mut(), ctx, ha_tick);
    emit_ha_event(
        ctx,
        "ha_worker::dispatch_actions",
        event,
        "ha lease log emit failed",
    )
}

pub(crate) fn emit_cluster_mode_transition(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    previous: &ClusterMode,
    next: &ClusterMode,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Info,
        "ha cluster mode transition",
        "ha.cluster_mode.transition",
        "ok",
    );
    let fields = event.fields_mut();
    ha_append_base_fields(fields, ctx, ha_tick);
    ha_insert_serialized(fields, "cluster_mode_prev", previous)?;
    ha_insert_serialized(fields, "cluster_mode_next", next)?;
    emit_ha_event(
        ctx,
        "ha_worker::step_once",
        event,
        "ha cluster_mode log emit failed",
    )
}

pub(crate) fn emit_state_transition(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    previous: &DesiredNodeState,
    next: &DesiredNodeState,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Info,
        "ha desired state transition",
        "ha.desired_state.transition",
        "ok",
    );
    let fields = event.fields_mut();
    ha_append_base_fields(fields, ctx, ha_tick);
    ha_insert_serialized(fields, "desired_state_prev", previous)?;
    ha_insert_serialized(fields, "desired_state_next", next)?;
    emit_ha_event(
        ctx,
        "ha_worker::step_once",
        event,
        "ha desired_state transition log emit failed",
    )
}

pub(crate) fn emit_node_role_transition(
    ctx: &HaWorkerCtx,
    ha_tick: u64,
    previous_role: &str,
    next_role: &str,
) -> Result<(), WorkerError> {
    let mut event = ha_event(
        SeverityText::Info,
        "ha node role transition",
        "ha.node_role.transition",
        "ok",
    );
    let fields = event.fields_mut();
    ha_append_base_fields(fields, ctx, ha_tick);
    fields.insert("role_prev", previous_role.to_string());
    fields.insert("role_next", next_role.to_string());
    emit_ha_event(
        ctx,
        "ha_worker::step_once",
        event,
        "ha node role log emit failed",
    )
}

pub(crate) fn node_role_label(desired_state: &DesiredNodeState) -> &'static str {
    match desired_state {
        DesiredNodeState::Primary { .. } => "primary",
        DesiredNodeState::Replica { .. } => "replica",
        _ => "unknown",
    }
}
