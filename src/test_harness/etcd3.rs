use std::collections::BTreeSet;
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

#[derive(Debug, Clone)]
pub(crate) struct EtcdClusterMemberSpec {
    pub(crate) member_name: String,
    pub(crate) data_dir: PathBuf,
    pub(crate) log_dir: PathBuf,
    pub(crate) client_port: u16,
    pub(crate) peer_port: u16,
}

#[derive(Debug, Clone)]
pub(crate) struct EtcdClusterSpec {
    pub(crate) etcd_bin: PathBuf,
    pub(crate) namespace_id: String,
    pub(crate) startup_timeout: Duration,
    pub(crate) members: Vec<EtcdClusterMemberSpec>,
}

#[derive(Debug)]
pub(crate) struct EtcdHandle {
    child: Child,
    member_name: String,
    pub(crate) client_port: u16,
    pub(crate) data_dir: PathBuf,
}

impl EtcdHandle {
    pub(crate) fn member_name(&self) -> &str {
        &self.member_name
    }

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

#[derive(Debug)]
pub(crate) struct EtcdClusterHandle {
    members: Vec<EtcdHandle>,
    client_endpoints: Vec<String>,
}

impl EtcdClusterHandle {
    pub(crate) fn client_endpoints(&self) -> &[String] {
        &self.client_endpoints
    }

    pub(crate) fn member_names(&self) -> Vec<String> {
        self.members
            .iter()
            .map(|member| member.member_name().to_string())
            .collect()
    }

    pub(crate) async fn shutdown_member(
        &mut self,
        member_name: &str,
    ) -> Result<bool, HarnessError> {
        if let Some(position) = self
            .members
            .iter()
            .position(|member| member.member_name() == member_name)
        {
            let mut member = self.members.swap_remove(position);
            member.shutdown().await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub(crate) async fn shutdown_all(&mut self) -> Result<(), HarnessError> {
        let mut failures = Vec::new();

        while let Some(mut member) = self.members.pop() {
            if let Err(err) = member.shutdown().await {
                failures.push(format!("{}: {err}", member.member_name()));
            }
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(HarnessError::InvalidInput(format!(
                "failed shutting down etcd members: {}",
                failures.join("; ")
            )))
        }
    }
}

pub(crate) fn prepare_etcd_member_data_dir(
    namespace: &TestNamespace,
    member_name: &str,
) -> Result<PathBuf, HarnessError> {
    let trimmed = member_name.trim();
    if trimmed.is_empty() {
        return Err(HarnessError::InvalidInput(
            "etcd member name must not be empty".to_string(),
        ));
    }
    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err(HarnessError::InvalidInput(format!(
            "etcd member name contains invalid path separators: {trimmed}"
        )));
    }

    let data_dir = namespace.child_dir(format!("etcd3/{trimmed}/data"));
    if data_dir.exists() {
        return Err(HarnessError::StalePath { path: data_dir });
    }
    fs::create_dir_all(&data_dir)?;
    Ok(data_dir)
}

pub(crate) fn prepare_etcd_data_dir(namespace: &TestNamespace) -> Result<PathBuf, HarnessError> {
    prepare_etcd_member_data_dir(namespace, "node-a")
}

pub(crate) async fn spawn_etcd3(spec: EtcdInstanceSpec) -> Result<EtcdHandle, HarnessError> {
    let cluster = spawn_etcd3_cluster(EtcdClusterSpec {
        etcd_bin: spec.etcd_bin,
        namespace_id: spec.namespace_id,
        startup_timeout: spec.startup_timeout,
        members: vec![EtcdClusterMemberSpec {
            member_name: spec.member_name,
            data_dir: spec.data_dir,
            log_dir: spec.log_dir,
            client_port: spec.client_port,
            peer_port: spec.peer_port,
        }],
    })
    .await?;

    let mut members = cluster.members;
    match members.pop() {
        Some(member) => Ok(member),
        None => Err(HarnessError::InvalidInput(
            "single-member etcd spawn returned no members".to_string(),
        )),
    }
}

pub(crate) async fn spawn_etcd3_cluster(
    spec: EtcdClusterSpec,
) -> Result<EtcdClusterHandle, HarnessError> {
    if !spec.etcd_bin.exists() {
        return Err(HarnessError::InvalidInput(format!(
            "etcd binary does not exist: {}",
            spec.etcd_bin.display()
        )));
    }
    if spec.members.is_empty() {
        return Err(HarnessError::InvalidInput(
            "etcd cluster must include at least one member".to_string(),
        ));
    }

    let mut seen_names = BTreeSet::new();
    let mut seen_ports = BTreeSet::new();
    for member in &spec.members {
        if !seen_names.insert(member.member_name.clone()) {
            return Err(HarnessError::InvalidInput(format!(
                "duplicate etcd member name: {}",
                member.member_name
            )));
        }
        if !seen_ports.insert(member.client_port) {
            return Err(HarnessError::InvalidInput(format!(
                "duplicate etcd client port: {}",
                member.client_port
            )));
        }
        if !seen_ports.insert(member.peer_port) {
            return Err(HarnessError::InvalidInput(format!(
                "duplicate etcd peer port: {}",
                member.peer_port
            )));
        }
    }

    let initial_cluster = build_initial_cluster(&spec.members)?;
    let mut started_members: Vec<EtcdHandle> = Vec::with_capacity(spec.members.len());

    for member in &spec.members {
        match spawn_etcd_member(
            &spec.etcd_bin,
            &spec.namespace_id,
            &initial_cluster,
            spec.startup_timeout,
            member,
        )
        .await
        {
            Ok(handle) => started_members.push(handle),
            Err(start_err) => {
                let cleanup_error = shutdown_started_members(&mut started_members).await;
                return match cleanup_error {
                    Ok(()) => Err(start_err),
                    Err(cleanup_err) => Err(HarnessError::InvalidInput(format!(
                        "{start_err}; cleanup failed: {cleanup_err}"
                    ))),
                };
            }
        }
    }

    let endpoints: Vec<String> = started_members
        .iter()
        .map(|member| format!("http://127.0.0.1:{}", member.client_port))
        .collect();

    if let Err(readiness_err) =
        wait_for_cluster_readiness(&endpoints, &spec.namespace_id, spec.startup_timeout).await
    {
        let cleanup_error = shutdown_started_members(&mut started_members).await;
        return match cleanup_error {
            Ok(()) => Err(readiness_err),
            Err(cleanup_err) => Err(HarnessError::InvalidInput(format!(
                "{readiness_err}; cleanup failed: {cleanup_err}"
            ))),
        };
    }

    Ok(EtcdClusterHandle {
        members: started_members,
        client_endpoints: endpoints,
    })
}

fn build_initial_cluster(members: &[EtcdClusterMemberSpec]) -> Result<String, HarnessError> {
    if members.is_empty() {
        return Err(HarnessError::InvalidInput(
            "cannot build initial-cluster for empty member list".to_string(),
        ));
    }

    let mut entries = Vec::with_capacity(members.len());
    for member in members {
        let name = member.member_name.trim();
        if name.is_empty() {
            return Err(HarnessError::InvalidInput(
                "etcd member name must not be empty".to_string(),
            ));
        }
        entries.push(format!("{name}=http://127.0.0.1:{}", member.peer_port));
    }
    Ok(entries.join(","))
}

async fn spawn_etcd_member(
    etcd_bin: &PathBuf,
    namespace_id: &str,
    initial_cluster: &str,
    startup_timeout: Duration,
    member: &EtcdClusterMemberSpec,
) -> Result<EtcdHandle, HarnessError> {
    fs::create_dir_all(&member.log_dir)?;

    let stdout_path = member.log_dir.join("etcd.stdout.log");
    let stderr_path = member.log_dir.join("etcd.stderr.log");
    let stdout_file = File::create(stdout_path)?;
    let stderr_file = File::create(stderr_path)?;

    let client_url = format!("http://127.0.0.1:{}", member.client_port);
    let peer_url = format!("http://127.0.0.1:{}", member.peer_port);

    let mut command = Command::new(etcd_bin);
    command
        .arg("--name")
        .arg(&member.member_name)
        .arg("--data-dir")
        .arg(&member.data_dir)
        .arg("--listen-client-urls")
        .arg(&client_url)
        .arg("--advertise-client-urls")
        .arg(&client_url)
        .arg("--listen-peer-urls")
        .arg(&peer_url)
        .arg("--initial-advertise-peer-urls")
        .arg(&peer_url)
        .arg("--initial-cluster")
        .arg(initial_cluster)
        .arg("--initial-cluster-state")
        .arg("new")
        .arg("--initial-cluster-token")
        .arg(namespace_id)
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file));

    let mut child = command
        .spawn()
        .map_err(|source| HarnessError::SpawnFailure {
            binary: etcd_bin.display().to_string(),
            source,
        })?;

    wait_for_port("etcd", member.client_port, startup_timeout, &mut child).await?;

    Ok(EtcdHandle {
        child,
        member_name: member.member_name.clone(),
        client_port: member.client_port,
        data_dir: member.data_dir.clone(),
    })
}

async fn shutdown_started_members(
    started_members: &mut Vec<EtcdHandle>,
) -> Result<(), HarnessError> {
    let mut cleanup_failures = Vec::new();

    while let Some(mut member) = started_members.pop() {
        if let Err(err) = member.shutdown().await {
            cleanup_failures.push(format!("{}: {err}", member.member_name()));
        }
    }

    if cleanup_failures.is_empty() {
        Ok(())
    } else {
        Err(HarnessError::InvalidInput(format!(
            "failed to cleanup partially-started etcd cluster members: {}",
            cleanup_failures.join("; ")
        )))
    }
}

async fn wait_for_cluster_readiness(
    endpoints: &[String],
    namespace_id: &str,
    startup_timeout: Duration,
) -> Result<(), HarnessError> {
    if endpoints.is_empty() {
        return Err(HarnessError::InvalidInput(
            "cluster readiness endpoints cannot be empty".to_string(),
        ));
    }

    let started = Instant::now();
    let probe_key = format!("/__pgtuskmaster_harness_probe/{namespace_id}");

    loop {
        if started.elapsed() >= startup_timeout {
            return Err(HarnessError::StartupTimeout {
                component: "etcd",
                timeout: startup_timeout,
            });
        }

        let remaining = startup_timeout
            .checked_sub(started.elapsed())
            .unwrap_or(Duration::from_millis(1));
        let attempt_timeout = remaining.min(Duration::from_secs(2));

        let connect_result = timeout(
            attempt_timeout,
            etcd_client::Client::connect(endpoints.to_vec(), None),
        )
        .await;

        if let Ok(Ok(mut client)) = connect_result {
            let put_result =
                timeout(attempt_timeout, client.put(probe_key.clone(), "ok", None)).await;

            if let Ok(Ok(_)) = put_result {
                let get_result =
                    timeout(attempt_timeout, client.get(probe_key.clone(), None)).await;
                if let Ok(Ok(_)) = get_result {
                    let _ = timeout(attempt_timeout, client.delete(probe_key.clone(), None)).await;
                    return Ok(());
                }
            }
        }

        sleep(Duration::from_millis(50)).await;
    }
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

    use super::{
        build_initial_cluster, prepare_etcd_data_dir, prepare_etcd_member_data_dir, spawn_etcd3,
        EtcdClusterMemberSpec, EtcdInstanceSpec,
    };
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

    #[test]
    fn prepare_etcd_member_data_dir_isolated_per_member() -> Result<(), HarnessError> {
        let guard = NamespaceGuard::new("prepare-etcd-member")?;
        let ns = guard.namespace()?;

        let node_a = prepare_etcd_member_data_dir(ns, "node-a")?;
        let node_b = prepare_etcd_member_data_dir(ns, "node-b")?;
        assert!(node_a.exists());
        assert!(node_b.exists());
        assert_ne!(node_a, node_b);

        let reused = prepare_etcd_member_data_dir(ns, "node-a");
        assert!(reused.is_err());
        Ok(())
    }

    #[test]
    fn build_initial_cluster_rejects_empty_member_name() {
        let result = build_initial_cluster(&[EtcdClusterMemberSpec {
            member_name: " ".to_string(),
            data_dir: PathBuf::from("/tmp/data"),
            log_dir: PathBuf::from("/tmp/log"),
            client_port: 1111,
            peer_port: 2222,
        }]);
        assert!(result.is_err());
    }

    #[test]
    fn build_initial_cluster_formats_entries() -> Result<(), HarnessError> {
        let rendered = build_initial_cluster(&[
            EtcdClusterMemberSpec {
                member_name: "node-a".to_string(),
                data_dir: PathBuf::from("/tmp/a"),
                log_dir: PathBuf::from("/tmp/a-log"),
                client_port: 1234,
                peer_port: 2234,
            },
            EtcdClusterMemberSpec {
                member_name: "node-b".to_string(),
                data_dir: PathBuf::from("/tmp/b"),
                log_dir: PathBuf::from("/tmp/b-log"),
                client_port: 1235,
                peer_port: 2235,
            },
        ])?;
        assert_eq!(
            rendered,
            "node-a=http://127.0.0.1:2234,node-b=http://127.0.0.1:2235"
        );
        Ok(())
    }

    use std::path::PathBuf;

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
