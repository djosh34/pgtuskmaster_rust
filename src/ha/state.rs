use std::collections::BTreeSet;

use crate::{
    config::RuntimeConfig,
    dcs::state::DcsState,
    pginfo::state::PgInfoState,
    process::state::ProcessState,
    state::{Versioned, WorkerStatus},
};

use super::actions::{ActionId, HaAction};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum HaPhase {
    Init,
    WaitingPostgresReachable,
    WaitingDcsTrusted,
    Replica,
    CandidateLeader,
    Primary,
    Rewinding,
    Bootstrapping,
    Fencing,
    FailSafe,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HaState {
    pub(crate) worker: WorkerStatus,
    pub(crate) phase: HaPhase,
    pub(crate) tick: u64,
    pub(crate) pending: Vec<HaAction>,
    pub(crate) recent_action_ids: BTreeSet<ActionId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorldSnapshot {
    pub(crate) config: Versioned<RuntimeConfig>,
    pub(crate) pg: Versioned<PgInfoState>,
    pub(crate) dcs: Versioned<DcsState>,
    pub(crate) process: Versioned<ProcessState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecideInput {
    pub(crate) current: HaState,
    pub(crate) world: WorldSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecideOutput {
    pub(crate) next: HaState,
    pub(crate) actions: Vec<HaAction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HaWorkerCtx {
    pub(crate) _private: (),
}
