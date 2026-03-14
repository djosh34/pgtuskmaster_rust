use std::time::Duration;

use crate::{
    logging::LogHandle,
    state::{new_state_channel, MemberId, WorkerError},
};

use super::state::{
    PgInfoCadence, PgInfoRuntime, PgInfoState, PgInfoStateChannel, PgInfoWorkerBootstrap,
    PgInfoWorkerCtx, PgNodeIdentity, PgProbeTarget,
};

pub(crate) struct PgInfoRuntimeRequest {
    pub(crate) self_id: MemberId,
    pub(crate) probe: PgProbeTarget,
    pub(crate) poll_interval: Duration,
    pub(crate) log: LogHandle,
}

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

pub(crate) fn bootstrap(request: PgInfoRuntimeRequest) -> PgInfoRuntimeBundle {
    let (publisher, state) = new_state_channel(PgInfoState::starting());

    PgInfoRuntimeBundle {
        state,
        worker: PgInfoWorker(PgInfoWorkerCtx::new(PgInfoWorkerBootstrap {
            identity: PgNodeIdentity {
                self_id: request.self_id,
            },
            probe: request.probe,
            cadence: PgInfoCadence {
                poll_interval: request.poll_interval,
            },
            state_channel: PgInfoStateChannel {
                publisher,
                last_emitted_sql_status: None,
            },
            runtime: PgInfoRuntime { log: request.log },
        })),
    }
}
