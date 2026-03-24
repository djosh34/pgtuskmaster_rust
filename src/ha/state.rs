use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    config_v2::RuntimeConfigV2,
    dcs::{DcsHandle, DcsSnapshot},
    pginfo::state::PgInfoState,
    process::state::{ProcessIntentRequest, ProcessState},
    state::{StatePublisher, StateSubscriber, UnixMillis, WorkerError, WorkerStatus},
};

use super::types::{HaDecision, HaObservation, HaPlan, PublicationState};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HaState {
    pub worker: WorkerStatus,
    pub tick: u64,
    pub managed_roles_reconciled: bool,
    pub publication: PublicationState,
    pub decision: HaDecision,
    pub observation: HaObservation,
    pub clear_switchover: bool,
    pub steps: HaPlan,
}

pub(crate) struct HaRuntimeCtx<'a> {
    pub(crate) cfg: &'a RuntimeConfigV2,
    pub(crate) now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
    pub(crate) state_channel: HaStateChannel,
    pub(crate) observed: HaObservedState,
    pub(crate) control: HaControlPlane,
}

pub(crate) struct HaStateChannel {
    pub(crate) current: HaState,
    pub(crate) publisher: StatePublisher<HaState>,
}

pub(crate) struct HaObservedState {
    pub(crate) pg: StateSubscriber<PgInfoState>,
    pub(crate) dcs: StateSubscriber<DcsSnapshot>,
    pub(crate) process: StateSubscriber<ProcessState>,
}

pub(crate) struct HaControlPlane {
    pub(crate) process_intent_inbox: UnboundedSender<ProcessIntentRequest>,
    pub(crate) dcs_handle: DcsHandle,
}

impl HaState {
    pub(crate) fn initial(worker: WorkerStatus) -> Self {
        Self {
            worker,
            tick: 0,
            managed_roles_reconciled: false,
            publication: PublicationState::unknown(),
            decision: HaDecision::initial(),
            observation: HaObservation::initial(),
            clear_switchover: false,
            steps: Vec::new(),
        }
    }
}
