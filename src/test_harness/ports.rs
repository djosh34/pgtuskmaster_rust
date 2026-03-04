use std::collections::BTreeSet;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};

use super::HarnessError;

#[derive(Debug)]
pub(crate) struct PortReservation {
    listeners: Vec<TcpListener>,
    ports: Vec<u16>,
}

impl PortReservation {
    pub(crate) fn empty() -> Self {
        Self {
            listeners: Vec::new(),
            ports: Vec::new(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[u16] {
        &self.ports
    }

    pub(crate) fn into_vec(self) -> Vec<u16> {
        self.ports
    }

    pub(crate) fn release_port(&mut self, port: u16) -> Result<(), HarnessError> {
        let index = self
            .ports
            .iter()
            .position(|candidate| *candidate == port)
            .ok_or_else(|| {
                HarnessError::InvalidInput(format!(
                    "attempted to release unknown reserved port: {port}"
                ))
            })?;

        self.ports.remove(index);
        self.listeners.remove(index);
        Ok(())
    }

    pub(crate) fn len(&self) -> usize {
        self.listeners.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.listeners.is_empty()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct HaTopologyPorts {
    pub(crate) etcd_client_ports: Vec<u16>,
    pub(crate) etcd_peer_ports: Vec<u16>,
    pub(crate) node_ports: Vec<u16>,
}

#[derive(Debug)]
pub(crate) struct HaTopologyPortReservation {
    reservation: PortReservation,
    layout: HaTopologyPorts,
}

impl HaTopologyPortReservation {
    pub(crate) fn layout(&self) -> &HaTopologyPorts {
        &self.layout
    }

    pub(crate) fn into_layout(self) -> HaTopologyPorts {
        self.layout
    }

    pub(crate) fn release_port(&mut self, port: u16) -> Result<(), HarnessError> {
        self.reservation.release_port(port)
    }

    pub(crate) fn len(&self) -> usize {
        self.reservation.len()
    }
}

pub(crate) fn allocate_ports(count: usize) -> Result<PortReservation, HarnessError> {
    if count == 0 {
        return Err(HarnessError::InvalidInput(
            "allocate_ports count must be greater than zero".to_string(),
        ));
    }

    let mut listeners = Vec::with_capacity(count);
    let mut ports = Vec::with_capacity(count);
    let mut seen = BTreeSet::new();

    for _ in 0..count {
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))?;
        let addr = listener.local_addr()?;
        let port = addr.port();
        if !seen.insert(port) {
            return Err(HarnessError::InvalidInput(format!(
                "duplicate ephemeral port allocated: {port}"
            )));
        }
        listeners.push(listener);
        ports.push(port);
    }

    Ok(PortReservation { listeners, ports })
}

pub(crate) fn allocate_ha_topology_ports(
    node_count: usize,
    etcd_members: usize,
) -> Result<HaTopologyPortReservation, HarnessError> {
    if node_count == 0 {
        return Err(HarnessError::InvalidInput(
            "node_count must be greater than zero".to_string(),
        ));
    }
    if etcd_members == 0 {
        return Err(HarnessError::InvalidInput(
            "etcd_members must be greater than zero".to_string(),
        ));
    }

    let total = node_count
        .checked_add(etcd_members.saturating_mul(2))
        .ok_or_else(|| {
            HarnessError::InvalidInput(
                "topology port count overflowed usize while reserving ports".to_string(),
            )
        })?;

    let reservation = allocate_ports(total)?;
    let ports = reservation.as_slice();

    let etcd_client_end = etcd_members;
    let etcd_peer_end = etcd_client_end + etcd_members;

    let layout = HaTopologyPorts {
        etcd_client_ports: ports[..etcd_client_end].to_vec(),
        etcd_peer_ports: ports[etcd_client_end..etcd_peer_end].to_vec(),
        node_ports: ports[etcd_peer_end..].to_vec(),
    };

    Ok(HaTopologyPortReservation {
        reservation,
        layout,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
    use std::sync::{Arc, Barrier, Mutex};
    use std::thread;
    use std::time::Duration;

    use super::{allocate_ha_topology_ports, allocate_ports, HarnessError};

    #[test]
    fn allocate_ports_rejects_zero() {
        let result = allocate_ports(0);
        assert!(result.is_err());
    }

    #[test]
    fn allocate_ports_returns_unique_ports() -> Result<(), HarnessError> {
        let reservation = allocate_ports(8)?;

        let ports = reservation.as_slice();
        let unique: BTreeSet<u16> = ports.iter().copied().collect();
        assert_eq!(unique.len(), ports.len());
        Ok(())
    }

    #[test]
    fn allocate_ha_topology_ports_rejects_zero_sizes() {
        assert!(allocate_ha_topology_ports(0, 3).is_err());
        assert!(allocate_ha_topology_ports(3, 0).is_err());
    }

    #[test]
    fn allocate_ha_topology_ports_returns_expected_layout() -> Result<(), HarnessError> {
        let reservation = allocate_ha_topology_ports(3, 3)?;
        let layout = reservation.layout();
        assert_eq!(reservation.len(), 9);
        assert_eq!(layout.etcd_client_ports.len(), 3);
        assert_eq!(layout.etcd_peer_ports.len(), 3);
        assert_eq!(layout.node_ports.len(), 3);

        let mut all = BTreeSet::new();
        for port in &layout.etcd_client_ports {
            assert!(all.insert(*port));
        }
        for port in &layout.etcd_peer_ports {
            assert!(all.insert(*port));
        }
        for port in &layout.node_ports {
            assert!(all.insert(*port));
        }
        assert_eq!(all.len(), 9);
        Ok(())
    }

    #[test]
    fn concurrent_allocations_do_not_collide_while_reserved() -> Result<(), HarnessError> {
        let workers = 24usize;
        let per_worker = 2usize;

        let start_barrier = Arc::new(Barrier::new(workers));
        let hold_barrier = Arc::new(Barrier::new(workers + 1));
        let all_ports = Arc::new(Mutex::new(Vec::with_capacity(workers * per_worker)));
        let mut handles = Vec::with_capacity(workers);

        for _ in 0..workers {
            let start_barrier = Arc::clone(&start_barrier);
            let hold_barrier = Arc::clone(&hold_barrier);
            let all_ports = Arc::clone(&all_ports);

            handles.push(thread::spawn(move || {
                start_barrier.wait();
                let reservation = allocate_ports(per_worker)?;

                {
                    let mut lock = all_ports.lock().map_err(|err| {
                        HarnessError::InvalidInput(format!("mutex poisoned: {err}"))
                    })?;
                    lock.extend(reservation.as_slice().iter().copied());
                }

                hold_barrier.wait();
                drop(reservation);
                Ok::<(), HarnessError>(())
            }));
        }

        loop {
            let len = {
                let lock = all_ports
                    .lock()
                    .map_err(|err| HarnessError::InvalidInput(format!("mutex poisoned: {err}")))?;
                lock.len()
            };
            if len == workers * per_worker {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }

        {
            let lock = all_ports
                .lock()
                .map_err(|err| HarnessError::InvalidInput(format!("mutex poisoned: {err}")))?;
            let unique: BTreeSet<u16> = lock.iter().copied().collect();
            assert_eq!(unique.len(), workers * per_worker);
        }

        hold_barrier.wait();

        for handle in handles {
            let thread_result = handle
                .join()
                .map_err(|_| HarnessError::InvalidInput("thread panicked".to_string()))?;
            thread_result?;
        }
        Ok(())
    }

    #[test]
    fn release_port_succeeds_for_reserved_port() -> Result<(), HarnessError> {
        let mut reservation = allocate_ports(2)?;
        let port = *reservation
            .as_slice()
            .first()
            .ok_or_else(|| HarnessError::InvalidInput("missing reserved port".to_string()))?;
        reservation.release_port(port)?;
        Ok(())
    }

    #[test]
    fn release_port_errors_for_unknown_port() -> Result<(), HarnessError> {
        let mut reservation = allocate_ports(2)?;
        let result = reservation.release_port(1);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn release_port_keeps_other_ports_reserved() -> Result<(), HarnessError> {
        let mut reservation = allocate_ports(2)?;
        let ports = reservation.as_slice().to_vec();
        if ports.len() != 2 {
            return Err(HarnessError::InvalidInput(format!(
                "expected 2 ports, got {}",
                ports.len()
            )));
        }

        reservation.release_port(ports[0])?;

        let released_bind = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, ports[0]));
        if released_bind.is_err() {
            return Err(HarnessError::InvalidInput(format!(
                "expected released port to be bindable: port={} err={:?}",
                ports[0], released_bind
            )));
        }

        let still_held_bind = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, ports[1]));
        if still_held_bind.is_ok() {
            return Err(HarnessError::InvalidInput(format!(
                "expected unreleased port to remain reserved: port={}",
                ports[1]
            )));
        }

        Ok(())
    }
}
