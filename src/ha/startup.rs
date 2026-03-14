use std::time::Duration;

use crate::{
    config::RuntimeConfig,
    dcs::{DcsHandle, DcsView},
    pginfo::state::PgInfoState,
    process::{startup::ProcessControlHandle, state::ProcessState},
    state::{new_state_channel, NodeIdentity, StateSubscriber, WorkerError},
};

use super::state::{
    HaControlPlane, HaNodeIdentity, HaObservedState, HaState, HaStateChannel,
    HaWorkerBootstrap, HaWorkerCadence, HaWorkerCtx,
};

pub(crate) struct HaRuntimeRequest {
    pub(crate) identity: NodeIdentity,
    pub(crate) poll_interval: Duration,
    pub(crate) config_subscriber: StateSubscriber<RuntimeConfig>,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) dcs_subscriber: StateSubscriber<DcsView>,
    pub(crate) process_subscriber: StateSubscriber<ProcessState>,
    pub(crate) process_control: ProcessControlHandle,
    pub(crate) dcs_handle: DcsHandle,
}

pub(crate) struct HaRuntimeBundle {
    pub(crate) state: crate::state::StateSubscriber<HaState>,
    pub(crate) worker: HaWorker,
}

pub(crate) struct HaWorker(HaWorkerCtx);

impl HaWorker {
    pub(crate) async fn run(self) -> Result<(), WorkerError> {
        super::worker::run(self.0).await
    }
}

pub(crate) fn bootstrap(request: HaRuntimeRequest) -> HaRuntimeBundle {
    let initial_state = HaState::initial(crate::state::WorkerStatus::Starting);
    let (publisher, state) = new_state_channel(initial_state.clone());
    let ctx = HaWorkerCtx::new(HaWorkerBootstrap {
        cadence: HaWorkerCadence {
            poll_interval: request.poll_interval,
            now: Box::new(crate::process::worker::system_now_unix_millis),
        },
        state_channel: HaStateChannel {
            current: initial_state,
            publisher,
        },
        observed: HaObservedState {
            config: request.config_subscriber,
            pg: request.pg_subscriber,
            dcs: request.dcs_subscriber,
            process: request.process_subscriber,
        },
        control: HaControlPlane {
            process_intent_inbox: request.process_control.intents,
            dcs_handle: request.dcs_handle,
        },
        identity: HaNodeIdentity {
            scope: request.identity.scope.0,
            self_id: request.identity.member_id,
        },
    });

    HaRuntimeBundle {
        state,
        worker: HaWorker(ctx),
    }
}
