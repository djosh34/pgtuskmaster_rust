use std::time::Duration;

use crate::support::observer::pgtm::PgtmObserver;

#[derive(Debug)]
pub struct WriteConvergenceInvariantRunner {
    _write_deadline: Duration,
}

#[derive(Debug, thiserror::Error)]
pub enum WriteConvergenceInvariantError {
    #[error("write-convergence invariant failed")]
    Failed,
}

impl WriteConvergenceInvariantRunner {
    pub async fn start(
        _observer: PgtmObserver,
        _poll_interval: Duration,
        _write_deadline: Duration,
    ) -> Result<Self, WriteConvergenceInvariantError> {
        Err(WriteConvergenceInvariantError::Failed)
    }

    pub fn ensure_healthy(&self) -> Result<(), WriteConvergenceInvariantError> {
        Ok(())
    }
}
