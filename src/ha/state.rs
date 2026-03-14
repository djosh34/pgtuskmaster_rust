use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    config::RuntimeConfig,
    dcs::{DcsHandle, DcsView},
    logging::LogHandle,
    pginfo::state::PgInfoState,
    process::state::{ProcessIntentRequest, ProcessState},
    state::{MemberId, StatePublisher, StateSubscriber, UnixMillis, WorkerError, WorkerStatus},
};

use super::types::{AuthorityProjectionState, IdleReason, PlannedActions, TargetRole, WorldView};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HaState {
    pub worker: WorkerStatus,
    pub tick: u64,
    pub required_roles_ready: bool,
    pub publication: AuthorityProjectionState,
    pub role: TargetRole,
    pub world: WorldView,
    pub clear_switchover: bool,
    pub planned_actions: PlannedActions,
}

pub(crate) struct HaWorkerCtx {
    pub(crate) poll_interval: Duration,
    pub(crate) state: HaState,
    pub(crate) publisher: StatePublisher<HaState>,
    pub(crate) config_subscriber: StateSubscriber<RuntimeConfig>,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) dcs_subscriber: StateSubscriber<DcsView>,
    pub(crate) process_subscriber: StateSubscriber<ProcessState>,
    pub(crate) process_intent_inbox: UnboundedSender<ProcessIntentRequest>,
    pub(crate) dcs_handle: DcsHandle,
    pub(crate) scope: String,
    pub(crate) self_id: MemberId,
    pub(crate) now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
    pub(crate) log: LogHandle,
}

pub(crate) struct HaWorkerContractStubInputs {
    pub(crate) publisher: StatePublisher<HaState>,
    pub(crate) config_subscriber: StateSubscriber<RuntimeConfig>,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) dcs_subscriber: StateSubscriber<DcsView>,
    pub(crate) process_subscriber: StateSubscriber<ProcessState>,
    pub(crate) process_intent_inbox: UnboundedSender<ProcessIntentRequest>,
    pub(crate) dcs_handle: DcsHandle,
    pub(crate) scope: String,
    pub(crate) self_id: MemberId,
}

impl HaState {
    pub(crate) fn initial(worker: WorkerStatus) -> Self {
        Self {
            worker,
            tick: 0,
            required_roles_ready: false,
            publication: AuthorityProjectionState::unknown(),
            role: TargetRole::Idle(IdleReason::AwaitingLeader),
            world: WorldView::initial(),
            clear_switchover: false,
            planned_actions: PlannedActions::default(),
        }
    }
}

impl HaWorkerCtx {
    pub(crate) fn contract_stub(inputs: HaWorkerContractStubInputs) -> Self {
        let HaWorkerContractStubInputs {
            publisher,
            config_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            process_intent_inbox,
            dcs_handle,
            scope,
            self_id,
        } = inputs;

        Self {
            poll_interval: Duration::from_millis(10),
            state: HaState::initial(WorkerStatus::Starting),
            publisher,
            config_subscriber,
            pg_subscriber,
            dcs_subscriber,
            process_subscriber,
            process_intent_inbox,
            dcs_handle,
            scope,
            self_id,
            now: Box::new(|| Ok(UnixMillis(0))),
            log: LogHandle::disabled(),
        }
    }
}
