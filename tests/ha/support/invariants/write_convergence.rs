use std::{time::Duration};

use crate::support::{
    error::{Result},
    observer::{pgtm::PgtmObserver},
};

#[derive(Debug)]
pub struct WriteConvergenceInvariantRunner {
    _poll_interval: Duration,
    _write_deadline: Duration,
}

impl WriteConvergenceInvariantRunner {
    pub fn start(
        _observer: PgtmObserver,
        poll_interval: Duration,
        write_deadline: Duration,
    ) -> Result<Self> {
        Ok(Self {
            _poll_interval: poll_interval,
            _write_deadline: write_deadline,
        })
    }

    pub fn ensure_healthy(&self) -> Result<()> {
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        Ok(())
    }
}
