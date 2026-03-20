use crate::{
    config_v2::RuntimeConfigV2,
    state::{new_state_channel, NodeIdentity},
};

use super::state::{HaControlPlane, HaObservedState, HaRuntimeCtx, HaState, HaStateChannel};

pub(crate) struct HaRuntimeBundle<'a> {
    pub(crate) state: crate::state::StateSubscriber<HaState>,
    pub(crate) worker: HaRuntimeCtx<'a>,
}

pub(crate) fn bootstrap<'a>(
    identity: NodeIdentity,
    cfg: &'a RuntimeConfigV2,
    observed: HaObservedState,
    control: HaControlPlane,
) -> HaRuntimeBundle<'a> {
    let initial_state = HaState::initial(crate::state::WorkerStatus::Starting);
    let (publisher, state) = new_state_channel(initial_state.clone());
    let ctx = HaRuntimeCtx {
        cfg,
        now: Box::new(crate::process::worker::system_now_unix_millis),
        state_channel: HaStateChannel {
            current: initial_state,
            publisher,
        },
        observed,
        control,
        identity,
    };

    HaRuntimeBundle { state, worker: ctx }
}
