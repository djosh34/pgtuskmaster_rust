use thiserror::Error;

use crate::{
    config::RuntimeConfig,
    process::{jobs::ProcessIntent, state::ProcessIntentRequest},
    state::{JobId, MemberId},
};

use super::state::HaWorkerCtx;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProcessDispatchOutcome {
    Applied,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ProcessDispatchError {
    #[error("process send failed for action `{action}`: {message}")]
    ProcessSend { action: String, message: String },
}

pub(crate) fn dispatch_process_action(
    ctx: &mut HaWorkerCtx,
    ha_tick: u64,
    action_index: usize,
    action: &ProcessIntent,
    _runtime_config: &RuntimeConfig,
) -> Result<ProcessDispatchOutcome, ProcessDispatchError> {
    let request = ProcessIntentRequest {
        id: process_job_id(
            &ctx.identity.scope,
            &ctx.identity.self_id,
            action,
            action_index,
            ha_tick,
        ),
        intent: action.clone(),
    };
    send_process_request(ctx, action.label(), request)?;
    Ok(ProcessDispatchOutcome::Applied)
}

fn send_process_request(
    ctx: &mut HaWorkerCtx,
    action: &str,
    request: ProcessIntentRequest,
) -> Result<(), ProcessDispatchError> {
    ctx.control
        .process_intent_inbox
        .send(request)
        .map_err(|err| ProcessDispatchError::ProcessSend {
            action: action.to_string(),
            message: err.to_string(),
        })
}

fn process_job_id(
    scope: &str,
    self_id: &MemberId,
    action: &ProcessIntent,
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
