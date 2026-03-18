use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::support::{
    error::Result,
    observer::pgtm::{PostgresRoutingTarget, PgtmObserver},
    topology::ClusterMember,
};

#[derive(Debug)]
pub struct WriteConvergenceInvariantRunner {
    inner: Arc<WriteConvergenceInvariantState>,
    _thread: Option<JoinHandle<()>>,
}

#[derive(Debug)]
struct WriteConvergenceInvariantState {
    poll_interval: Duration,
    observer: PgtmObserver,
    dsn_values: Mutex<Option<Vec<PostgresRoutingTarget>>>,
}

impl WriteConvergenceInvariantRunner {
    pub fn start(observer: PgtmObserver, poll_interval: Duration, _write_deadline: Duration) -> Result<Self> {
        let inner = Arc::new(WriteConvergenceInvariantState {
            poll_interval,
            observer,
            dsn_values: Mutex::new(None),
        });

        let thread = thread::spawn({
            let inner = Arc::clone(&inner);
            move || inner.run()
        });

        Ok(Self {
            inner,
            _thread: Some(thread),
        })
    }

    pub fn ensure_healthy(&self) -> bool {
        match self.inner.dsn_values.lock() {
            Ok(guard) => guard.is_some(),
            Err(err) => panic!("dsn-values lock poisoned while checking health: {err}"),
        }
    }
}

impl WriteConvergenceInvariantState {
    fn run(self: Arc<Self>) -> ! {
        loop {
            let maybe_targets = self.collect_all_targets();
            let mut dsn_values = self
                .dsn_values
                .lock()
                .unwrap_or_else(|err| panic!("dsn-values lock poisoned during initialization: {err}"));

            match maybe_targets {
                Ok(targets) => {
                    *dsn_values = Some(targets);
                    break;
                }
                Err(err) => {
                    *dsn_values = None;
                    eprintln!("write_convergence invariant not ready: {err}");
                }
            }

            thread::sleep(self.poll_interval);
        }

        loop {
            let maybe_targets = self.collect_all_targets();
            let mut dsn_values = self
                .dsn_values
                .lock()
                .unwrap_or_else(|err| panic!("dsn-values lock poisoned during refresh: {err}"));

            match maybe_targets {
                Ok(targets) => *dsn_values = Some(targets),
                Err(err) => {
                    *dsn_values = None;
                    eprintln!("write_convergence invariant refresh failed: {err}");
                }
            }

            thread::sleep(self.poll_interval);
        }
    }

    fn collect_all_targets(&self) -> Result<Vec<PostgresRoutingTarget>> {
        ClusterMember::ALL
            .into_iter()
            .map(|member| self.observer.postgres_routing_target(member))
            .collect()
    }
}
