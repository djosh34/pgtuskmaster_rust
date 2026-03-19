use std::time::Duration;

use tokio::sync::mpsc;

use crate::{
    config::RuntimeConfig,
    logging::LogSender,
    state::{new_state_channel, NodeIdentity, WorkerError},
};

use super::{
    state::{
        ProcessCadence, ProcessControlPlane, ProcessIntentRequest, ProcessObservedState,
        ProcessRuntime, ProcessRuntimePlan, ProcessState, ProcessStateChannel, ProcessWorkerCtx,
    },
    worker::{system_now_unix_millis, TokioCommandRunner},
};

const PROCESS_WORKER_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Clone, Debug)]
pub(crate) struct ProcessControlHandle {
    pub(crate) intents: tokio::sync::mpsc::UnboundedSender<ProcessIntentRequest>,
}

pub(crate) struct ProcessRuntimeBundle {
    pub(crate) state: crate::state::StateSubscriber<ProcessState>,
    pub(crate) control: ProcessControlHandle,
    pub(crate) worker: ProcessWorkerCtx,
}

pub(crate) async fn run(ctx: ProcessWorkerCtx) -> Result<(), WorkerError> {
    super::worker::run(ctx).await
}

pub(crate) fn bootstrap(
    identity: NodeIdentity,
    cfg: &RuntimeConfig,
    observed: ProcessObservedState,
    plan: ProcessRuntimePlan,
    log: LogSender,
) -> ProcessRuntimeBundle {
    let initial_state = ProcessState::starting();
    let (publisher, state) = new_state_channel(initial_state.clone());
    let (intents, inbox) = mpsc::unbounded_channel();

    ProcessRuntimeBundle {
        state,
        control: ProcessControlHandle { intents },
        worker: ProcessWorkerCtx {
            cadence: ProcessCadence {
                poll_interval: PROCESS_WORKER_POLL_INTERVAL,
                now: Box::new(system_now_unix_millis),
            },
            config: cfg.process.clone(),
            identity,
            observed,
            plan,
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
                capture_subprocess_output: cfg.logging.capture_subprocess_output,
                command_runner: Box::new(TokioCommandRunner),
            },
        },
    }
}
