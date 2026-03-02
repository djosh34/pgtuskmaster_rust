use std::collections::BTreeSet;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};

use super::HarnessError;

#[derive(Debug)]
pub(crate) struct PortReservation {
    listeners: Vec<TcpListener>,
    ports: Vec<u16>,
}

impl PortReservation {
    pub(crate) fn as_slice(&self) -> &[u16] {
        &self.ports
    }

    pub(crate) fn into_vec(self) -> Vec<u16> {
        self.ports
    }

    pub(crate) fn len(&self) -> usize {
        self.listeners.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.listeners.is_empty()
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::sync::{Arc, Barrier, Mutex};
    use std::thread;
    use std::time::Duration;

    use super::{allocate_ports, HarnessError};

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
}
