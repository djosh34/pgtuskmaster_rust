use std::{time::Duration};

use crate::support::{
    error::{Result},
    observer::pgtm::PgtmObserver,
};

#[derive(Debug)]
pub struct PrimaryCountInvariantRunner {
    _poll_interval: Duration,
}

impl PrimaryCountInvariantRunner {
    pub fn start(
        _observer: PgtmObserver,
        poll_interval: Duration,
    ) -> Result<Self> {
        Ok(Self {
            _poll_interval: poll_interval,
        })
    }

    pub fn ensure_healthy(&self) -> Result<()> {
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        Ok(())
    }
}
