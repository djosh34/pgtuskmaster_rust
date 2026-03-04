use std::path::PathBuf;
use std::time::Duration;

use thiserror::Error;

pub(crate) mod auth;
pub(crate) mod binaries;
pub(crate) mod etcd3;
#[cfg(test)]
pub(crate) mod ha_e2e;
pub(crate) mod namespace;
pub(crate) mod net_proxy;
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
    StartupTimeout {
        component: &'static str,
        timeout: Duration,
    },
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
    use crate::test_harness::namespace::NamespaceGuard;
    use crate::test_harness::ports::allocate_ports;
    use crate::test_harness::HarnessError;

    #[test]
    fn concurrent_namespace_and_port_allocations_are_isolated() -> Result<(), HarnessError> {
        let mut namespaces = Vec::new();
        let mut reservations = Vec::new();

        for idx in 0..12_u32 {
            let guard = NamespaceGuard::new(&format!("isolation-{idx}"))?;
            let namespace = guard.namespace()?.clone();
            let reservation = allocate_ports(3)?;
            namespaces.push((guard, namespace));
            reservations.push(reservation);
        }

        let mut all_ns = std::collections::BTreeSet::new();
        for (_, ns) in &namespaces {
            assert!(all_ns.insert(ns.id.clone()), "duplicate namespace id");
        }

        let mut all_ports = std::collections::BTreeSet::new();
        for reservation in &reservations {
            for port in reservation.as_slice() {
                assert!(all_ports.insert(*port), "duplicate allocated port");
            }
        }

        Ok(())
    }
}
