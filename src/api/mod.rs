use thiserror::Error;

pub(crate) mod controller;
pub mod worker;

use crate::{
    dcs::state::DcsState, ha::state::HaState, pginfo::state::PgInfoState,
    process::state::ProcessState,
};

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub(crate) enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("dcs store error: {0}")]
    DcsStore(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl ApiError {
    pub(crate) fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub(crate) fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

pub(crate) type ApiResult<T> = Result<T, ApiError>;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptedResponse {
    pub accepted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeState {
    pub cluster_name: String,
    pub scope: String,
    pub self_member_id: String,
    pub pg: PgInfoState,
    pub process: ProcessState,
    pub dcs: DcsState,
    pub ha: HaState,
}
