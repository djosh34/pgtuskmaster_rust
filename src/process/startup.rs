use std::time::Duration;

use tokio::sync::mpsc;

use crate::{
    config::{ProcessConfig, RuntimeConfig},
    dcs::DcsView,
    logging::LogHandle,
    state::{new_state_channel, NodeIdentity, StateSubscriber, WorkerError},
};

use super::{
    state::{
        ProcessCadence, ProcessControlPlane, ProcessIntentRequest, ProcessNodeIdentity,
        ProcessObservedState, ProcessRuntime, ProcessRuntimePlan, ProcessState,
        ProcessStateChannel, ProcessWorkerBootstrap, ProcessWorkerCtx,
    },
    worker::{system_now_unix_millis, TokioCommandRunner},
};

const PROCESS_WORKER_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Clone, Debug)]
pub(crate) struct ProcessRuntimeRequest {
    pub(crate) identity: NodeIdentity,
    pub(crate) runtime_config: StateSubscriber<RuntimeConfig>,
    pub(crate) dcs_subscriber: StateSubscriber<DcsView>,
    pub(crate) plan: ProcessRuntimePlan,
    pub(crate) config: ProcessConfig,
    pub(crate) capture_subprocess_output: bool,
    pub(crate) log: LogHandle,
}

#[derive(Clone, Debug)]
pub(crate) struct ProcessControlHandle {
    pub(crate) intents: tokio::sync::mpsc::UnboundedSender<ProcessIntentRequest>,
}

pub(crate) struct ProcessRuntimeBundle {
    pub(crate) state: crate::state::StateSubscriber<ProcessState>,
    pub(crate) control: ProcessControlHandle,
    pub(crate) worker: ProcessWorker,
}

pub(crate) struct ProcessWorker(ProcessWorkerCtx);

impl ProcessWorker {
    pub(crate) async fn run(self) -> Result<(), WorkerError> {
        super::worker::run(self.0).await
    }
}

pub(crate) fn bootstrap(request: ProcessRuntimeRequest) -> ProcessRuntimeBundle {
    let initial_state = ProcessState::starting();
    let (publisher, state) = new_state_channel(initial_state.clone());
    let (intents, inbox) = mpsc::unbounded_channel();

    ProcessRuntimeBundle {
        state,
        control: ProcessControlHandle { intents },
        worker: ProcessWorker(ProcessWorkerCtx::new(ProcessWorkerBootstrap {
            cadence: ProcessCadence {
                poll_interval: PROCESS_WORKER_POLL_INTERVAL,
                now: Box::new(system_now_unix_millis),
            },
            config: request.config,
            identity: ProcessNodeIdentity {
                self_id: request.identity.member_id,
            },
            observed: ProcessObservedState {
                runtime_config: request.runtime_config,
                dcs: request.dcs_subscriber,
            },
            plan: request.plan,
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
                log: request.log,
                capture_subprocess_output: request.capture_subprocess_output,
                command_runner: Box::new(TokioCommandRunner),
            },
        })),
    }
}
