use std::collections::BTreeSet;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::time::Duration;

use super::HarnessError;

#[cfg(unix)]
use std::fs::File;

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

#[cfg(unix)]
use libc::{flock, LOCK_EX, LOCK_UN};

#[cfg(unix)]
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub(crate) struct PortReservation {
    listeners: Vec<TcpListener>,
    ports: Vec<u16>,
    leased_ports: Vec<u16>,
}

impl PortReservation {
    pub(crate) fn empty() -> Self {
        Self {
            listeners: Vec::new(),
            ports: Vec::new(),
            leased_ports: Vec::new(),
        }
    }

    pub(crate) fn as_slice(&self) -> &[u16] {
        &self.ports
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

    pub(crate) fn take_listener(&mut self, port: u16) -> Result<TcpListener, HarnessError> {
        let index = self
            .ports
            .iter()
            .position(|candidate| *candidate == port)
            .ok_or_else(|| {
                HarnessError::InvalidInput(format!(
                    "attempted to take listener for unknown reserved port: {port}"
                ))
            })?;
        self.ports.remove(index);
        Ok(self.listeners.remove(index))
    }

    pub(crate) fn len(&self) -> usize {
        self.listeners.len()
    }
}

#[cfg(unix)]
impl Drop for PortReservation {
    fn drop(&mut self) {
        if self.leased_ports.is_empty() {
            return;
        }
        let _ = unlease_ports_best_effort(self.leased_ports.as_slice());
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
    let mut leased_ports = Vec::with_capacity(count);

    for _ in 0..count {
        let mut attempts = 0usize;
        loop {
            attempts = attempts.saturating_add(1);
            if attempts > 200 {
                return Err(HarnessError::InvalidInput(
                    "failed to allocate a non-leased ephemeral port after retries".to_string(),
                ));
            }

            let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))?;
            let addr = listener.local_addr()?;
            let port = addr.port();
            if !seen.insert(port) {
                continue;
            }

            #[cfg(unix)]
            {
                if !lease_port_best_effort(port) {
                    // Port is currently leased by another test/process; retry.
                    drop(listener);
                    continue;
                }
            }

            listeners.push(listener);
            ports.push(port);
            leased_ports.push(port);
            break;
        }
    }

    Ok(PortReservation {
        listeners,
        ports,
        leased_ports,
    })
}

#[cfg(unix)]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PortLeaseEntry {
    port: u16,
    expires_at_ms: u64,
}

#[cfg(unix)]
fn lease_registry_path() -> std::path::PathBuf {
    std::path::PathBuf::from("/tmp/pgtuskmaster_rust_port_leases.json")
}

#[cfg(unix)]
fn unix_now_ms() -> Result<u64, HarnessError> {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| {
            HarnessError::InvalidInput(format!("system clock before unix epoch: {err}"))
        })?;
    Ok(duration.as_millis() as u64)
}

#[cfg(unix)]
fn lease_ttl() -> Duration {
    // Keep leases long enough that parallel `cargo test` workers don't race each other.
    Duration::from_secs(15 * 60)
}

#[cfg(unix)]
struct LeaseFileGuard {
    file: File,
}

#[cfg(unix)]
impl LeaseFileGuard {
    fn lock() -> Result<Self, HarnessError> {
        let path = lease_registry_path();
        let file = std::fs::OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(path.as_path())
            .map_err(HarnessError::Io)?;

        let rc = unsafe { flock(file.as_raw_fd(), LOCK_EX) };
        if rc != 0 {
            return Err(HarnessError::Io(std::io::Error::last_os_error()));
        }

        Ok(Self { file })
    }

    fn load_entries(&mut self) -> Vec<PortLeaseEntry> {
        let mut buf = String::new();
        if std::io::Read::read_to_string(&mut self.file, &mut buf).is_err() {
            return Vec::new();
        }
        if buf.trim().is_empty() {
            return Vec::new();
        }
        serde_json::from_str::<Vec<PortLeaseEntry>>(&buf).unwrap_or_default()
    }

    fn store_entries(&mut self, entries: &[PortLeaseEntry]) -> Result<(), HarnessError> {
        let serialized = serde_json::to_string(entries)
            .map_err(|err| HarnessError::InvalidInput(format!("serialize port lease file failed: {err}")))?;
        self.file
            .set_len(0)
            .map_err(HarnessError::Io)?;
        std::io::Seek::seek(&mut self.file, std::io::SeekFrom::Start(0))
            .map_err(HarnessError::Io)?;
        std::io::Write::write_all(&mut self.file, serialized.as_bytes())
            .map_err(HarnessError::Io)?;
        std::io::Write::write_all(&mut self.file, b"\n").map_err(HarnessError::Io)?;
        self.file.sync_data().map_err(HarnessError::Io)?;
        Ok(())
    }
}

#[cfg(unix)]
impl Drop for LeaseFileGuard {
    fn drop(&mut self) {
        let _ = self.file.sync_data();
        let _ = unsafe { flock(self.file.as_raw_fd(), LOCK_UN) };
    }
}

#[cfg(unix)]
fn lease_port_best_effort(port: u16) -> bool {
    let mut guard = match LeaseFileGuard::lock() {
        Ok(guard) => guard,
        Err(_) => return true,
    };

    let now_ms = match unix_now_ms() {
        Ok(value) => value,
        Err(_) => return true,
    };
    let ttl_ms = lease_ttl().as_millis() as u64;
    let expires_at_ms = now_ms.saturating_add(ttl_ms);

    let mut entries = guard.load_entries();
    entries.retain(|entry| entry.expires_at_ms > now_ms);
    if entries.iter().any(|entry| entry.port == port) {
        let _ = guard.store_entries(entries.as_slice());
        return false;
    }

    entries.push(PortLeaseEntry { port, expires_at_ms });
    let _ = guard.store_entries(entries.as_slice());
    true
}

#[cfg(unix)]
fn unlease_ports_best_effort(ports: &[u16]) -> Result<(), HarnessError> {
    if ports.is_empty() {
        return Ok(());
    }

    let mut guard = LeaseFileGuard::lock()?;
    let now_ms = unix_now_ms().ok();

    let mut entries = guard.load_entries();
    entries.retain(|entry| {
        if ports.contains(&entry.port) {
            return false;
        }
        match now_ms {
            Some(now_ms) => entry.expires_at_ms > now_ms,
            None => true,
        }
    });
    guard.store_entries(entries.as_slice())?;
    Ok(())
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

    Ok(HaTopologyPortReservation { reservation, layout })
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
