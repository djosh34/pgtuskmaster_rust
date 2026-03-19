use std::time::Duration;

use crate::{
    config::RuntimeConfig,
    logging::LogSender,
    process::state::ProcessRuntimePlan,
    state::{new_state_channel, NodeIdentity, WorkerError},
};

use super::state::{
    PgInfoCadence, PgInfoRuntime, PgInfoState, PgInfoStateChannel, PgInfoWorkerCtx, PgProbeTarget,
};

pub(crate) struct PgInfoRuntimeBundle {
    pub(crate) state: crate::state::StateSubscriber<PgInfoState>,
    pub(crate) worker: PgInfoWorker,
}

pub(crate) struct PgInfoWorker(PgInfoWorkerCtx);

impl PgInfoWorker {
    pub(crate) async fn run(self) -> Result<(), WorkerError> {
        super::worker::run(self.0).await
    }
}

pub(crate) fn bootstrap(
    identity: NodeIdentity,
    cfg: &RuntimeConfig,
    process_plan: &ProcessRuntimePlan,
    log: LogSender,
) -> PgInfoRuntimeBundle {
    let (publisher, state) = new_state_channel(PgInfoState::starting());

    PgInfoRuntimeBundle {
        state,
        worker: PgInfoWorker(PgInfoWorkerCtx {
            identity,
            probe: PgProbeTarget::local_from_config(cfg, process_plan),
            cadence: PgInfoCadence {
                poll_interval: Duration::from_millis(cfg.ha.loop_interval_ms),
            },
            state_channel: PgInfoStateChannel {
                publisher,
                last_emitted_sql_status: None,
            },
            runtime: PgInfoRuntime { log },
        }),
    }
}
