use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    config::RuntimeConfig,
    dcs::{DcsHandle, DcsSnapshot},
    pginfo::state::PgInfoState,
    process::state::{ProcessIntentRequest, ProcessState},
    state::{NodeIdentity, StatePublisher, StateSubscriber, UnixMillis, WorkerError, WorkerStatus},
};

use super::types::{IdleReason, PlannedActions, PublicationState, TargetRole, WorldView};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HaState {
    pub worker: WorkerStatus,
    pub tick: u64,
    pub managed_roles_reconciled: bool,
    pub publication: PublicationState,
    pub role: TargetRole,
    pub world: WorldView,
    pub clear_switchover: bool,
    pub planned_actions: PlannedActions,
}

pub(crate) struct HaRuntimeCtx {
    pub(crate) cadence: HaWorkerCadence,
    pub(crate) state_channel: HaStateChannel,
    pub(crate) observed: HaObservedState,
    pub(crate) control: HaControlPlane,
    pub(crate) identity: NodeIdentity,
}

pub(crate) struct HaWorkerCadence {
    pub(crate) poll_interval: Duration,
    pub(crate) now: Box<dyn FnMut() -> Result<UnixMillis, WorkerError> + Send>,
}

pub(crate) struct HaStateChannel {
    pub(crate) current: HaState,
    pub(crate) publisher: StatePublisher<HaState>,
}

pub(crate) struct HaObservedState {
    pub(crate) config: StateSubscriber<RuntimeConfig>,
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
            role: TargetRole::Idle(IdleReason::AwaitingLeader),
            world: WorldView::initial(),
            clear_switchover: false,
            planned_actions: PlannedActions::default(),
        }
    }
}
