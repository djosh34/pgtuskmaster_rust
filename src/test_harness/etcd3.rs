use std::fs::{self, File};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::process::{Child, Command};
use tokio::time::{sleep, timeout, Instant};

use super::HarnessError;
use crate::test_harness::namespace::TestNamespace;

#[derive(Debug, Clone)]
pub(crate) struct EtcdInstanceSpec {
    pub(crate) etcd_bin: PathBuf,
    pub(crate) namespace_id: String,
    pub(crate) member_name: String,
    pub(crate) data_dir: PathBuf,
    pub(crate) log_dir: PathBuf,
    pub(crate) client_port: u16,
    pub(crate) peer_port: u16,
    pub(crate) startup_timeout: Duration,
}

#[derive(Debug)]
pub(crate) struct EtcdHandle {
    child: Child,
    pub(crate) client_port: u16,
    pub(crate) data_dir: PathBuf,
}

impl EtcdHandle {
    pub(crate) async fn shutdown(&mut self) -> Result<(), HarnessError> {
        if let Some(pid) = self.child.id() {
            let mut term_cmd = Command::new("kill");
            term_cmd.arg("-TERM").arg(pid.to_string());
            let status = timeout(Duration::from_secs(2), term_cmd.status()).await;
            if let Ok(Ok(exit_status)) = status {
                if !exit_status.success() {
                    // Continue with fallback kill path below.
                }
            }
        }

        let wait_result = timeout(Duration::from_secs(5), self.child.wait()).await;
        match wait_result {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(err)) => Err(HarnessError::Io(err)),
            Err(_) => {
                self.child.start_kill().map_err(HarnessError::Io)?;
                self.child.wait().await.map_err(HarnessError::Io)?;
                Ok(())
            }
        }
    }
}

pub(crate) fn prepare_etcd_data_dir(namespace: &TestNamespace) -> Result<PathBuf, HarnessError> {
    let data_dir = namespace.child_dir("etcd3/data");
    if data_dir.exists() {
        return Err(HarnessError::StalePath { path: data_dir });
    }
    fs::create_dir_all(&data_dir)?;
    Ok(data_dir)
}

pub(crate) async fn spawn_etcd3(spec: EtcdInstanceSpec) -> Result<EtcdHandle, HarnessError> {
    if !spec.etcd_bin.exists() {
        return Err(HarnessError::InvalidInput(format!(
            "etcd binary does not exist: {}",
            spec.etcd_bin.display()
        )));
    }

    fs::create_dir_all(&spec.log_dir)?;

    let stdout_path = spec.log_dir.join("etcd.stdout.log");
    let stderr_path = spec.log_dir.join("etcd.stderr.log");
    let stdout_file = File::create(stdout_path)?;
    let stderr_file = File::create(stderr_path)?;

    let client_url = format!("http://127.0.0.1:{}", spec.client_port);
    let peer_url = format!("http://127.0.0.1:{}", spec.peer_port);
    let initial_cluster = format!("{}={peer_url}", spec.member_name);

    let mut command = Command::new(&spec.etcd_bin);
    command
        .arg("--name")
        .arg(&spec.member_name)
        .arg("--data-dir")
        .arg(&spec.data_dir)
        .arg("--listen-client-urls")
        .arg(&client_url)
        .arg("--advertise-client-urls")
        .arg(&client_url)
        .arg("--listen-peer-urls")
        .arg(&peer_url)
        .arg("--initial-advertise-peer-urls")
        .arg(&peer_url)
        .arg("--initial-cluster")
        .arg(&initial_cluster)
        .arg("--initial-cluster-state")
        .arg("new")
        .arg("--initial-cluster-token")
        .arg(&spec.namespace_id)
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file));

    let mut child = command
        .spawn()
        .map_err(|source| HarnessError::SpawnFailure {
            binary: spec.etcd_bin.display().to_string(),
            source,
        })?;

    wait_for_port("etcd", spec.client_port, spec.startup_timeout, &mut child).await?;

    Ok(EtcdHandle {
        child,
        client_port: spec.client_port,
        data_dir: spec.data_dir,
    })
}

async fn wait_for_port(
    component: &'static str,
    port: u16,
    startup_timeout: Duration,
    child: &mut Child,
) -> Result<(), HarnessError> {
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(HarnessError::Io)? {
            return Err(HarnessError::EarlyExit { component, status });
        }

        if TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            return Ok(());
        }

        if started.elapsed() >= startup_timeout {
            let _ = child.start_kill();
            let _ = child.wait().await;
            return Err(HarnessError::StartupTimeout {
                component,
                timeout: startup_timeout,
            });
        }

        sleep(Duration::from_millis(50)).await;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{prepare_etcd_data_dir, spawn_etcd3, EtcdInstanceSpec};
    use crate::test_harness::binaries::require_etcd_bin_for_real_tests;
    use crate::test_harness::namespace::NamespaceGuard;
    use crate::test_harness::ports::allocate_ports;
    use crate::test_harness::HarnessError;

    #[test]
    fn prepare_etcd_data_dir_rejects_reuse() -> Result<(), HarnessError> {
        let guard = NamespaceGuard::new("prepare-etcd")?;
        let ns = guard.namespace()?;

        let first = prepare_etcd_data_dir(ns)?;
        assert!(first.exists());

        let second = prepare_etcd_data_dir(ns);
        assert!(second.is_err());
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn spawn_etcd3_requires_binary_and_spawns() -> Result<(), HarnessError> {
        let etcd_bin = match require_etcd_bin_for_real_tests()? {
            Some(path) => path,
            None => return Ok(()),
        };

        let guard = NamespaceGuard::new("spawn-etcd")?;
        let ns = guard.namespace()?;

        let data_dir = prepare_etcd_data_dir(ns)?;

        let reservation = allocate_ports(2)?;
        let ports = reservation.as_slice();
        let client_port = ports[0];
        let peer_port = ports[1];

        let log_dir = ns.child_dir("logs/etcd3-node-a");

        let spec = EtcdInstanceSpec {
            etcd_bin,
            namespace_id: ns.id.clone(),
            member_name: "node-a".to_string(),
            data_dir,
            log_dir,
            client_port,
            peer_port,
            startup_timeout: Duration::from_secs(10),
        };

        // Release the reserved ports immediately before spawning etcd so the
        // child can bind them.
        drop(reservation);
        let mut handle = spawn_etcd3(spec).await?;
        handle.shutdown().await?;
        Ok(())
    }
}
