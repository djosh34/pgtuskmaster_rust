use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use tokio::task::JoinHandle;

use crate::config::BinaryPaths;
use crate::config::RuntimeConfig;
use crate::state::WorkerError;
use crate::test_harness::etcd3::EtcdClusterHandle;
use crate::test_harness::namespace::NamespaceGuard;
use crate::test_harness::net_proxy::TcpProxyLink;

use super::config::TimeoutConfig;
use super::util::{wait_for_node_api_ready_or_task_exit, wait_for_node_api_unavailable};

#[derive(Clone, Debug)]
pub struct NodeHandle {
    pub id: String,
    pub pg_port: u16,
    pub sql_port: u16,
    pub api_addr: SocketAddr,
    pub api_observe_addr: SocketAddr,
    pub data_dir: PathBuf,
}

pub struct TestClusterHandle {
    pub guard: NamespaceGuard,
    pub timeouts: TimeoutConfig,
    pub binaries: BinaryPaths,
    pub superuser_username: String,
    pub superuser_dbname: String,
    pub etcd: Option<EtcdClusterHandle>,
    pub nodes: Vec<NodeHandle>,
    pub runtime_nodes: RuntimeNodeSet,
    pub etcd_proxies: BTreeMap<String, TcpProxyLink>,
    pub api_proxies: BTreeMap<String, TcpProxyLink>,
    pub pg_proxies: BTreeMap<String, TcpProxyLink>,
}

pub struct RuntimeNodeHandle {
    pub runtime_cfg: RuntimeConfig,
    pub postgres_log_file: PathBuf,
    pub task: JoinHandle<Result<(), WorkerError>>,
}

#[derive(Default)]
pub struct RuntimeNodeSet {
    nodes: BTreeMap<String, RuntimeNodeHandle>,
}

impl RuntimeNodeSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, node_id: String, handle: RuntimeNodeHandle) -> Option<RuntimeNodeHandle> {
        self.nodes.insert(node_id, handle)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn metadata_for_node(&self, node_id: &str) -> Option<(&RuntimeConfig, &PathBuf)> {
        self.nodes
            .get(node_id)
            .map(|handle| (&handle.runtime_cfg, &handle.postgres_log_file))
    }

    pub async fn ensure_healthy(&mut self) -> Result<(), WorkerError> {
        let finished_node_ids = self
            .nodes
            .iter()
            .filter_map(|(node_id, handle)| handle.task.is_finished().then_some(node_id.clone()))
            .collect::<Vec<_>>();

        if let Some(node_id) = finished_node_ids.into_iter().next() {
            let handle = self.nodes.remove(node_id.as_str()).ok_or_else(|| {
                WorkerError::Message(format!(
                    "runtime task bookkeeping lost finished node record: {node_id}"
                ))
            })?;
            let joined = handle.task.await.map_err(|err| {
                WorkerError::Message(format!("runtime task join failed for {node_id}: {err}"))
            })?;
            match joined {
                Ok(()) => {
                    return Err(WorkerError::Message(format!(
                        "runtime task for {node_id} exited unexpectedly"
                    )));
                }
                Err(err) => {
                    return Err(WorkerError::Message(format!(
                        "runtime task for {node_id} failed: {err}"
                    )));
                }
            }
        }

        Ok(())
    }

    pub async fn shutdown_all(&mut self) {
        let mut drained = std::mem::take(&mut self.nodes)
            .into_iter()
            .collect::<Vec<_>>();
        for (_, handle) in &drained {
            handle.task.abort();
        }
        while let Some((_, handle)) = drained.pop() {
            let _ = handle.task.await;
        }
    }

    pub fn replace_task(
        &mut self,
        node_id: &str,
        task: JoinHandle<Result<(), WorkerError>>,
    ) -> Result<(), WorkerError> {
        let handle = self.nodes.get_mut(node_id).ok_or_else(|| {
            WorkerError::Message(format!("missing runtime task record for node: {node_id}"))
        })?;
        handle.task = task;
        Ok(())
    }

    pub async fn restart_node(
        &mut self,
        node: &NodeHandle,
        http_step_timeout: Duration,
        api_readiness_timeout: Duration,
    ) -> Result<(), WorkerError> {
        let node_id = node.id.clone();
        let mut handle = self.nodes.remove(node_id.as_str()).ok_or_else(|| {
            WorkerError::Message(format!("missing runtime restart metadata for node: {node_id}"))
        })?;

        handle.task.abort();
        match handle.task.await {
            Ok(Ok(())) => {
                return Err(WorkerError::Message(format!(
                    "runtime task for {node_id} exited cleanly before restart, expected a running node"
                )));
            }
            Ok(Err(err)) => {
                return Err(WorkerError::Message(format!(
                    "runtime task for {node_id} failed before restart: {err}"
                )));
            }
            Err(err) => {
                if !err.is_cancelled() {
                    return Err(WorkerError::Message(format!(
                        "runtime task join failed for {node_id} during restart: {err}"
                    )));
                }
            }
        }

        wait_for_node_api_unavailable(
            node.api_observe_addr,
            node_id.as_str(),
            http_step_timeout,
            api_readiness_timeout,
        )
        .await?;

        let runtime_cfg = handle.runtime_cfg.clone();
        let postgres_log_file = handle.postgres_log_file.clone();
        let task_node_id = node_id.clone();
        let runtime_task = tokio::task::spawn_local(async move {
            match crate::runtime::run_node_from_config(runtime_cfg).await {
                Ok(()) => Ok(()),
                Err(err) => Err(WorkerError::Message(format!(
                    "runtime node {task_node_id} exited with error: {err}"
                ))),
            }
        });

        let runtime_task = wait_for_node_api_ready_or_task_exit(
            node.api_observe_addr,
            node_id.as_str(),
            postgres_log_file.as_path(),
            runtime_task,
            http_step_timeout,
            api_readiness_timeout,
        )
        .await?;

        handle.task = runtime_task;
        self.nodes.insert(node_id, handle);
        Ok(())
    }
}

impl TestClusterHandle {
    pub async fn ensure_runtime_tasks_healthy(&mut self) -> Result<(), WorkerError> {
        self.runtime_nodes.ensure_healthy().await
    }

    pub async fn restart_runtime_node(&mut self, node_id: &str) -> Result<(), WorkerError> {
        let node = self
            .nodes
            .iter()
            .find(|candidate| candidate.id == node_id)
            .cloned()
            .ok_or_else(|| {
                WorkerError::Message(format!("unknown node id for runtime restart: {node_id}"))
            })?;
        self.runtime_nodes
            .restart_node(
                &node,
                self.timeouts.http_step_timeout,
                self.timeouts.api_readiness_timeout,
            )
            .await
    }
}
