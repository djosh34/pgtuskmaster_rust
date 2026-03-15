// Stub implementation – see DESIGN.md for the planned interface.
#![allow(clippy::unimplemented, dead_code)]

use std::time::Duration;

use crate::{
    config::{DcsClientConfig, DcsEndpoint, RuntimeConfig},
    logging::LogSender,
    pginfo::state::PgInfoState,
    state::{NodeIdentity, PgTcpTarget, StateSubscriber, WorkerError},
};

use super::{DcsHandle, DcsView};

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum DcsError {
    #[error("store I/O error: {0}")]
    Io(String),
}

pub(crate) struct DcsAdvertisedEndpoints {
    pub(crate) postgres: PgTcpTarget,
}

pub(crate) struct DcsRuntimeRequest {
    pub(crate) identity: NodeIdentity,
    pub(crate) endpoints: Vec<DcsEndpoint>,
    pub(crate) client: DcsClientConfig,
    pub(crate) poll_interval: Duration,
    pub(crate) member_ttl_ms: u64,
    pub(crate) advertised: DcsAdvertisedEndpoints,
    pub(crate) pg_subscriber: StateSubscriber<PgInfoState>,
    pub(crate) log: LogSender,
}

pub(crate) struct DcsRuntime {
    pub(crate) state: StateSubscriber<DcsView>,
    pub(crate) handle: DcsHandle,
    pub(crate) worker: DcsWorker,
}

pub(crate) struct DcsWorker;

impl DcsAdvertisedEndpoints {
    pub(crate) fn from_config(_cfg: &RuntimeConfig) -> Result<Self, DcsError> {
        unimplemented!("dcs module not implemented")
    }
}

impl DcsWorker {
    pub(crate) async fn run(self) -> Result<(), WorkerError> {
        unimplemented!("dcs module not implemented")
    }
}

pub(crate) fn bootstrap(_request: DcsRuntimeRequest) -> Result<DcsRuntime, DcsError> {
    unimplemented!("dcs module not implemented")
}
