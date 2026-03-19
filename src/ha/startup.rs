use std::time::Duration;

use crate::state::{new_state_channel, NodeIdentity};

use super::state::{
    HaControlPlane, HaObservedState, HaRuntimeCtx, HaState, HaStateChannel, HaWorkerCadence,
};

pub(crate) struct HaRuntimeBundle {
    pub(crate) state: crate::state::StateSubscriber<HaState>,
    pub(crate) worker: HaRuntimeCtx,
}

pub(crate) fn bootstrap(
    identity: NodeIdentity,
    poll_interval: Duration,
    observed: HaObservedState,
    control: HaControlPlane,
) -> HaRuntimeBundle {
    let initial_state = HaState::initial(crate::state::WorkerStatus::Starting);
    let (publisher, state) = new_state_channel(initial_state.clone());
    let ctx = HaRuntimeCtx {
        cadence: HaWorkerCadence {
            poll_interval,
            now: Box::new(crate::process::worker::system_now_unix_millis),
        },
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
