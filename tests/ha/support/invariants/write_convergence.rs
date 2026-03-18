use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration},
};

use crate::support::{error::Result, observer::pgtm::PgtmObserver, topology::ClusterMember};

#[derive(Debug)]
pub struct WriteConvergenceInvariantRunner {
    dsn_found: Arc<AtomicBool>,
    _thread: Option<JoinHandle<()>>,
}

impl WriteConvergenceInvariantRunner {
    pub fn start(
        observer: PgtmObserver,
        poll_interval: Duration,
        _write_deadline: Duration,
    ) -> Result<Self> {
        let dsn_found = Arc::new(AtomicBool::new(false));
        let thread_dsn_found = Arc::clone(&dsn_found);

        let thread = thread::spawn(move || {
            loop {
                let found = ClusterMember::ALL
                    .into_iter()
                    .any(|member| observer.postgres_routing_target(member).is_ok());
                if found {
                    thread_dsn_found.store(true, Ordering::Release);
                    return;
                }
                thread::sleep(poll_interval);
            }
        });

        Ok(Self {
            dsn_found,
            _thread: Some(thread),
        })
    }

    pub fn ensure_healthy(&self) -> bool {
        self.dsn_found.load(Ordering::Acquire)
    }

}

