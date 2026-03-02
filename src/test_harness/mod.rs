#![allow(dead_code)]
// Test harness setup/teardown tests intentionally panic on unrecoverable fixture failures.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;
use std::time::Duration;

use thiserror::Error;

pub(crate) mod auth;
pub(crate) mod etcd3;
pub(crate) mod namespace;
pub(crate) mod pg16;
pub(crate) mod ports;
pub(crate) mod tls;

#[derive(Debug, Error)]
pub(crate) enum HarnessError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("spawn failed for {binary}: {source}")]
    SpawnFailure {
        binary: String,
        #[source]
        source: std::io::Error,
    },
    #[error("{component} did not become ready within {timeout:?}")]
    StartupTimeout { component: &'static str, timeout: Duration },
    #[error("{component} exited before readiness with status {status}")]
    EarlyExit {
        component: &'static str,
        status: std::process::ExitStatus,
    },
    #[error("stale path exists: {path}")]
    StalePath { path: PathBuf },
}

#[cfg(test)]
mod tests {
    use crate::test_harness::namespace::{cleanup_namespace, create_namespace};
    use crate::test_harness::ports::allocate_ports;

    #[test]
    fn concurrent_namespace_and_port_allocations_are_isolated() {
        let mut namespaces = Vec::new();
        let mut reservations = Vec::new();

        for idx in 0..12_u32 {
            let ns = match create_namespace(&format!("isolation-{idx}")) {
                Ok(ns) => ns,
                Err(err) => panic!("namespace create failed: {err}"),
            };
            let reservation = match allocate_ports(3) {
                Ok(res) => res,
                Err(err) => {
                    if let Err(clean_err) = cleanup_namespace(ns) {
                        panic!("port alloc failed: {err}; cleanup also failed: {clean_err}");
                    }
                    panic!("port alloc failed: {err}");
                }
            };
            namespaces.push(ns);
            reservations.push(reservation);
        }

        let mut all_ns = std::collections::BTreeSet::new();
        for ns in &namespaces {
            assert!(all_ns.insert(ns.id.clone()), "duplicate namespace id");
        }

        let mut all_ports = std::collections::BTreeSet::new();
        for reservation in &reservations {
            for port in reservation.as_slice() {
                assert!(all_ports.insert(*port), "duplicate allocated port");
            }
        }

        for ns in namespaces {
            if let Err(err) = cleanup_namespace(ns) {
                panic!("cleanup failed: {err}");
            }
        }
    }
}
