use thiserror::Error;

use crate::state::JobId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BootstrapSpec;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgRewindSpec;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PromoteSpec;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DemoteSpec;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StartPostgresSpec;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StopPostgresSpec;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RestartPostgresSpec;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FencingSpec;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ActiveJob {
    pub(crate) id: JobId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CancelReason {
    Superseded,
    Shutdown,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum ProcessError {
    #[error("process worker operation failed")]
    OperationFailed,
}
