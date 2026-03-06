use thiserror::Error;

pub(crate) mod controller;
pub(crate) mod events;
pub(crate) mod fallback;
pub mod worker;

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
pub(crate) struct AcceptedResponse {
    pub(crate) accepted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HaStateResponse {
    pub(crate) cluster_name: String,
    pub(crate) scope: String,
    pub(crate) self_member_id: String,
    pub(crate) leader: Option<String>,
    pub(crate) switchover_requested_by: Option<String>,
    pub(crate) member_count: usize,
    pub(crate) dcs_trust: String,
    pub(crate) ha_phase: String,
    pub(crate) ha_tick: u64,
    pub(crate) pending_actions: usize,
    pub(crate) snapshot_sequence: u64,
}
