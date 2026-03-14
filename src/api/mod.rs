use thiserror::Error;

pub(crate) mod controller;
pub(crate) mod startup;
pub mod worker;

use crate::{
    dcs::DcsView, ha::state::HaState, pginfo::state::PgInfoState, process::state::ProcessState,
};

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub(crate) enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("dcs command error: {0}")]
    DcsCommand(String),
}

impl ApiError {
    pub(crate) fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }
}

pub(crate) type ApiResult<T> = Result<T, ApiError>;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptedResponse {
    pub accepted: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiCertificateReloadStep {
    HttpTransportUnchanged,
    HttpsConfigurationReloaded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostgresReloadSignal {
    Sighup,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresCertificateReloadStep {
    pub signal: PostgresReloadSignal,
    pub postmaster_pid: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReloadCertificatesResponse {
    pub api: ApiCertificateReloadStep,
    pub postgres: PostgresCertificateReloadStep,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeState {
    pub cluster_name: String,
    pub scope: String,
    pub self_member_id: String,
    pub pg: PgInfoState,
    pub process: ProcessState,
    pub dcs: DcsView,
    pub ha: HaState,
}
