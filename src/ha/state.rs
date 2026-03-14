use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    config::RuntimeConfig,
    dcs::{DcsHandle, DcsView},
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
    pub(crate) cadence: HaWorkerCadence,
    pub(crate) state_channel: HaStateChannel,
    pub(crate) observed: HaObservedState,
    pub(crate) control: HaControlPlane,
    pub(crate) identity: HaNodeIdentity,
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
    pub(crate) dcs: StateSubscriber<DcsView>,
    pub(crate) process: StateSubscriber<ProcessState>,
}

pub(crate) struct HaControlPlane {
    pub(crate) process_intent_inbox: UnboundedSender<ProcessIntentRequest>,
    pub(crate) dcs_handle: DcsHandle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HaNodeIdentity {
    pub(crate) scope: String,
    pub(crate) self_id: MemberId,
}

pub(crate) struct HaWorkerBootstrap {
    pub(crate) cadence: HaWorkerCadence,
    pub(crate) state_channel: HaStateChannel,
    pub(crate) observed: HaObservedState,
    pub(crate) control: HaControlPlane,
    pub(crate) identity: HaNodeIdentity,
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
    pub(crate) fn new(bootstrap: HaWorkerBootstrap) -> Self {
        let HaWorkerBootstrap {
            cadence,
            state_channel,
            observed,
            control,
            identity,
        } = bootstrap;
        Self {
            cadence,
            state_channel,
            observed,
            control,
            identity,
        }
    }
}
