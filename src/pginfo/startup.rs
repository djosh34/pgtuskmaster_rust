use crate::{
    config_v2::RuntimeConfigV2,
    logging::LogSender,
    state::{new_state_channel, NodeIdentity, WorkerError},
};

use super::state::{PgInfoRuntime, PgInfoState, PgInfoStateChannel, PgInfoWorkerCtx};

pub(crate) struct PgInfoRuntimeBundle<'a> {
    pub(crate) state: crate::state::StateSubscriber<PgInfoState>,
    pub(crate) worker: PgInfoWorkerCtx<'a>,
}

pub(crate) async fn run(ctx: PgInfoWorkerCtx<'_>) -> Result<(), WorkerError> {
    super::worker::run(ctx).await
}

pub(crate) fn bootstrap<'a>(
    identity: NodeIdentity,
    cfg: &'a RuntimeConfigV2,
    log: LogSender,
) -> PgInfoRuntimeBundle<'a> {
    let (publisher, state) = new_state_channel(PgInfoState::starting());

    PgInfoRuntimeBundle {
        state,
        worker: PgInfoWorkerCtx {
            cfg,
            identity,
            state_channel: PgInfoStateChannel {
                publisher,
                last_emitted_sql_status: None,
            },
            runtime: PgInfoRuntime { log },
        },
    }
}
