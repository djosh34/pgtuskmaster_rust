use thiserror::Error;

use crate::{
    process::{jobs::ProcessIntent, state::ProcessIntentRequest},
    state::JobId,
};

use super::state::HaRuntimeCtx;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ProcessDispatchError {
    #[error("process send failed for action `{action}`: {message}")]
    ProcessSend { action: String, message: String },
}

pub(crate) fn dispatch_process_action(
    ctx: &mut HaRuntimeCtx,
    ha_tick: u64,
    action_index: usize,
    action: &ProcessIntent,
) -> Result<(), ProcessDispatchError> {
    let request = ProcessIntentRequest {
        id: JobId(format!(
            "ha-{}-{}-{}-{}-{}",
            ctx.identity.scope.as_str().trim_matches('/'),
            ctx.identity.member_id.0,
            ha_tick,
            action_index,
            action.label(),
        )),
        intent: action.clone(),
    };
    ctx.control
        .process_intent_inbox
        .send(request)
        .map_err(|err| ProcessDispatchError::ProcessSend {
            action: action.label().to_string(),
            message: err.to_string(),
        })
}
