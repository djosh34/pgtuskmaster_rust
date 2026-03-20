use std::time::Duration;

use tokio::sync::mpsc;

use crate::{
    config_v2::RuntimeConfigV2,
    logging::LogSender,
    state::{new_state_channel, NodeIdentity, WorkerError},
};

use super::{
    state::{
        ProcessCadence, ProcessControlPlane, ProcessIntentRequest, ProcessObservedState,
        ProcessRuntime, ProcessState, ProcessStateChannel, ProcessWorkerCtx,
    },
    worker::{system_now_unix_millis, TokioCommandRunner},
};

const PROCESS_WORKER_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Clone, Debug)]
pub(crate) struct ProcessControlHandle {
    pub(crate) intents: tokio::sync::mpsc::UnboundedSender<ProcessIntentRequest>,
}

pub(crate) struct ProcessRuntimeBundle<'a> {
    pub(crate) state: crate::state::StateSubscriber<ProcessState>,
    pub(crate) control: ProcessControlHandle,
    pub(crate) worker: ProcessWorkerCtx<'a>,
}

pub(crate) async fn run(ctx: ProcessWorkerCtx<'_>) -> Result<(), WorkerError> {
    super::worker::run(ctx).await
}

pub(crate) fn bootstrap<'a>(
    identity: NodeIdentity,
    cfg: &'a RuntimeConfigV2,
    observed: ProcessObservedState,
    log: LogSender,
) -> ProcessRuntimeBundle<'a> {
    let initial_state = ProcessState::starting();
    let (publisher, state) = new_state_channel(initial_state.clone());
    let (intents, inbox) = mpsc::unbounded_channel();

    ProcessRuntimeBundle {
        state,
        control: ProcessControlHandle { intents },
        worker: ProcessWorkerCtx {
            cfg,
            cadence: ProcessCadence {
                poll_interval: PROCESS_WORKER_POLL_INTERVAL,
                now: Box::new(system_now_unix_millis),
            },
            identity,
            observed,
            state_channel: ProcessStateChannel {
                current: initial_state,
                publisher,
                last_rejection: None,
            },
            control: ProcessControlPlane {
                inbox,
                inbox_disconnected_logged: false,
                active_runtime: None,
            },
            runtime: ProcessRuntime {
                log,
                command_runner: Box::new(TokioCommandRunner),
            },
        },
    }
}
