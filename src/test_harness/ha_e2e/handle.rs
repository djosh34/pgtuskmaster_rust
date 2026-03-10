use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use tokio::process::Child;

use crate::config::BinaryPaths;
use crate::config::RuntimeConfig;
use crate::state::WorkerError;
use crate::test_harness::etcd3::EtcdClusterHandle;
use crate::test_harness::namespace::NamespaceGuard;
use crate::test_harness::net_proxy::TcpProxyLink;

use super::config::TimeoutConfig;
use super::util::{
    kill_child_forcefully, pg_ctl_stop_fast, pg_ctl_stop_immediate, spawn_runtime_node_process,
    stop_child_gracefully, wait_for_node_api_ready_or_process_exit, wait_for_node_api_unavailable,
    wait_for_postgres_unavailable,
};

#[derive(Clone, Debug)]
pub struct NodeHandle {
    pub id: String,
    pub pg_port: u16,
    pub sql_port: u16,
    pub api_addr: SocketAddr,
    pub api_observe_addr: SocketAddr,
    pub data_dir: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WholeNodeOutageKind {
    CleanStop,
    HardKill,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WholeNodeOutageState {
    pub kind: WholeNodeOutageKind,
    pub etcd_member_name: Option<String>,
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
    pub node_etcd_colocation: BTreeMap<String, String>,
    pub whole_node_outages: BTreeMap<String, WholeNodeOutageState>,
}

pub(crate) enum RuntimeNodeState {
    Running(Child),
    Offline,
}

pub struct RuntimeNodeHandle {
    pub runtime_cfg: RuntimeConfig,
    pub runtime_binary_path: PathBuf,
    pub runtime_config_path: PathBuf,
    pub postgres_log_file: PathBuf,
    pub runtime_log_file: PathBuf,
    pub(crate) state: RuntimeNodeState,
}

impl RuntimeNodeHandle {
    fn running_child_mut(&mut self) -> Result<&mut Child, WorkerError> {
        match &mut self.state {
            RuntimeNodeState::Running(child) => Ok(child),
            RuntimeNodeState::Offline => Err(WorkerError::Message(format!(
                "runtime process is intentionally offline for node {}",
                self.runtime_cfg.cluster.member_id
            ))),
        }
    }

    fn set_running(&mut self, child: Child) {
        self.state = RuntimeNodeState::Running(child);
    }

    fn set_offline(&mut self) {
        self.state = RuntimeNodeState::Offline;
    }

    fn is_offline(&self) -> bool {
        matches!(self.state, RuntimeNodeState::Offline)
    }
}

#[derive(Default)]
pub struct RuntimeNodeSet {
    nodes: BTreeMap<String, RuntimeNodeHandle>,
}

impl RuntimeNodeSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        node_id: String,
        handle: RuntimeNodeHandle,
    ) -> Option<RuntimeNodeHandle> {
        self.nodes.insert(node_id, handle)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn is_node_offline(&self, node_id: &str) -> Result<bool, WorkerError> {
        self.nodes
            .get(node_id)
            .map(RuntimeNodeHandle::is_offline)
            .ok_or_else(|| WorkerError::Message(format!("unknown runtime node id: {node_id}")))
    }

    pub fn metadata_for_node(&self, node_id: &str) -> Option<(&RuntimeConfig, &PathBuf)> {
        self.nodes
            .get(node_id)
            .map(|handle| (&handle.runtime_cfg, &handle.postgres_log_file))
    }

    pub async fn ensure_healthy(&mut self) -> Result<(), WorkerError> {
        let finished = self
            .nodes
            .iter_mut()
            .find_map(|(node_id, handle)| match &mut handle.state {
                RuntimeNodeState::Running(child) => match child.try_wait() {
                    Ok(Some(status)) => Some(Ok((node_id.clone(), status))),
                    Ok(None) => None,
                    Err(err) => Some(Err(WorkerError::Message(format!(
                        "runtime process status probe failed for {node_id}: {err}"
                    )))),
                },
                RuntimeNodeState::Offline => None,
            })
            .transpose()?;

        if let Some((node_id, status)) = finished {
            let removed = self.nodes.remove(node_id.as_str()).ok_or_else(|| {
                WorkerError::Message(format!(
                    "runtime process bookkeeping lost finished node record: {node_id}"
                ))
            })?;
            return Err(WorkerError::Message(format!(
                "runtime process for {node_id} exited unexpectedly with status {status}; runtime_log_tail={}",
                super::util::read_log_tail(removed.runtime_log_file.as_path(), 40)
            )));
        }

        Ok(())
    }

    pub async fn shutdown_all(&mut self) -> Result<(), WorkerError> {
        let drained = std::mem::take(&mut self.nodes)
            .into_iter()
            .collect::<Vec<_>>();
        let mut failures = Vec::new();

        for (node_id, mut handle) in drained {
            if let RuntimeNodeState::Running(child) = &mut handle.state {
                let label = format!("runtime shutdown for {node_id}");
                if let Err(err) = kill_child_forcefully(&label, child, Duration::from_secs(3)).await
                {
                    failures.push(err.to_string());
                }
            }
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(WorkerError::Message(format!(
                "runtime shutdown failures: {}",
                failures.join("; ")
            )))
        }
    }

    pub async fn restart_node(
        &mut self,
        node: &NodeHandle,
        http_step_timeout: Duration,
        api_readiness_timeout: Duration,
        command_kill_wait_timeout: Duration,
    ) -> Result<(), WorkerError> {
        let node_id = node.id.clone();
        let mut handle = self.nodes.remove(node_id.as_str()).ok_or_else(|| {
            WorkerError::Message(format!(
                "missing runtime restart metadata for node: {node_id}"
            ))
        })?;
        if !handle.is_offline() {
            self.nodes.insert(node_id.clone(), handle);
            return Err(WorkerError::Message(format!(
                "runtime restart requested for running node: {node_id}"
            )));
        }

        let runtime_child = spawn_runtime_node_process(
            handle.runtime_binary_path.as_path(),
            handle.runtime_config_path.as_path(),
            handle.runtime_log_file.as_path(),
        )?;
        let runtime_child = wait_for_node_api_ready_or_process_exit(
            node.api_observe_addr,
            node_id.as_str(),
            handle.runtime_log_file.as_path(),
            runtime_child,
            http_step_timeout,
            api_readiness_timeout,
            command_kill_wait_timeout,
        )
        .await?;

        handle.set_running(runtime_child);
        self.nodes.insert(node_id, handle);
        Ok(())
    }

    pub async fn stop_node(
        &mut self,
        node: &NodeHandle,
        command_timeout: Duration,
        command_kill_wait_timeout: Duration,
        http_step_timeout: Duration,
        api_readiness_timeout: Duration,
    ) -> Result<(), WorkerError> {
        let node_id = node.id.clone();
        let mut handle = self.nodes.remove(node_id.as_str()).ok_or_else(|| {
            WorkerError::Message(format!("missing runtime stop metadata for node: {node_id}"))
        })?;
        if handle.is_offline() {
            self.nodes.insert(node_id.clone(), handle);
            return Err(WorkerError::Message(format!(
                "runtime stop requested for offline node: {node_id}"
            )));
        }

        let label = format!("runtime graceful stop for {node_id}");
        stop_child_gracefully(
            label.as_str(),
            handle.running_child_mut()?,
            command_timeout,
            command_kill_wait_timeout,
        )
        .await?;
        wait_for_node_api_unavailable(
            node.api_observe_addr,
            node_id.as_str(),
            http_step_timeout,
            api_readiness_timeout,
        )
        .await?;

        handle.set_offline();
        self.nodes.insert(node_id, handle);
        Ok(())
    }

    pub async fn kill_node(
        &mut self,
        node: &NodeHandle,
        command_kill_wait_timeout: Duration,
        http_step_timeout: Duration,
        api_readiness_timeout: Duration,
    ) -> Result<(), WorkerError> {
        let node_id = node.id.clone();
        let mut handle = self.nodes.remove(node_id.as_str()).ok_or_else(|| {
            WorkerError::Message(format!("missing runtime kill metadata for node: {node_id}"))
        })?;
        if handle.is_offline() {
            self.nodes.insert(node_id.clone(), handle);
            return Err(WorkerError::Message(format!(
                "runtime kill requested for offline node: {node_id}"
            )));
        }

        let label = format!("runtime hard kill for {node_id}");
        kill_child_forcefully(
            label.as_str(),
            handle.running_child_mut()?,
            command_kill_wait_timeout,
        )
        .await?;
        wait_for_node_api_unavailable(
            node.api_observe_addr,
            node_id.as_str(),
            http_step_timeout,
            api_readiness_timeout,
        )
        .await?;

        handle.set_offline();
        self.nodes.insert(node_id, handle);
        Ok(())
    }
}

impl TestClusterHandle {
    fn node_by_id(&self, node_id: &str) -> Result<NodeHandle, WorkerError> {
        self.nodes
            .iter()
            .find(|candidate| candidate.id == node_id)
            .cloned()
            .ok_or_else(|| WorkerError::Message(format!("unknown node id: {node_id}")))
    }

    fn ensure_node_not_in_whole_outage(&self, node_id: &str) -> Result<(), WorkerError> {
        if self.whole_node_outages.contains_key(node_id) {
            return Err(WorkerError::Message(format!(
                "whole-node outage already active for {node_id}"
            )));
        }
        Ok(())
    }

    pub async fn ensure_runtime_tasks_healthy(&mut self) -> Result<(), WorkerError> {
        self.runtime_nodes.ensure_healthy().await
    }

    pub async fn restart_runtime_node(&mut self, node_id: &str) -> Result<(), WorkerError> {
        if !self.runtime_nodes.is_node_offline(node_id)? {
            self.stop_runtime_node(node_id).await?;
        }
        let node = self.node_by_id(node_id)?;
        self.runtime_nodes
            .restart_node(
                &node,
                self.timeouts.http_step_timeout,
                self.timeouts.api_readiness_timeout,
                self.timeouts.command_kill_wait_timeout,
            )
            .await
    }

    pub async fn stop_runtime_node(&mut self, node_id: &str) -> Result<(), WorkerError> {
        self.ensure_node_not_in_whole_outage(node_id)?;
        let node = self.node_by_id(node_id)?;
        self.runtime_nodes
            .stop_node(
                &node,
                self.timeouts.command_timeout,
                self.timeouts.command_kill_wait_timeout,
                self.timeouts.http_step_timeout,
                self.timeouts.api_readiness_timeout,
            )
            .await
    }

    pub async fn kill_runtime_node(&mut self, node_id: &str) -> Result<(), WorkerError> {
        self.ensure_node_not_in_whole_outage(node_id)?;
        let node = self.node_by_id(node_id)?;
        self.runtime_nodes
            .kill_node(
                &node,
                self.timeouts.command_kill_wait_timeout,
                self.timeouts.http_step_timeout,
                self.timeouts.api_readiness_timeout,
            )
            .await
    }

    pub async fn stop_whole_node(&mut self, node_id: &str) -> Result<(), WorkerError> {
        self.ensure_node_not_in_whole_outage(node_id)?;
        let node = self.node_by_id(node_id)?;
        self.runtime_nodes
            .stop_node(
                &node,
                self.timeouts.command_timeout,
                self.timeouts.command_kill_wait_timeout,
                self.timeouts.http_step_timeout,
                self.timeouts.api_readiness_timeout,
            )
            .await?;

        pg_ctl_stop_fast(
            self.binaries.pg_ctl.as_path(),
            node.data_dir.as_path(),
            self.timeouts.command_timeout,
            self.timeouts.command_kill_wait_timeout,
        )
        .await?;
        wait_for_postgres_unavailable(
            self.binaries.psql.as_path(),
            node.pg_port,
            self.superuser_username.as_str(),
            self.superuser_dbname.as_str(),
            self.timeouts.command_timeout,
            self.timeouts.command_kill_wait_timeout,
            self.timeouts.api_readiness_timeout,
        )
        .await?;

        let etcd_member_name = self.node_etcd_colocation.get(node_id).cloned();
        if let Some(member_name) = etcd_member_name.as_deref() {
            let etcd = self.etcd.as_mut().ok_or_else(|| {
                WorkerError::Message(format!(
                    "node {node_id} declares colocated etcd member {member_name} but no etcd cluster is running"
                ))
            })?;
            etcd.shutdown_member(member_name).await.map_err(|err| {
                WorkerError::Message(format!(
                    "failed to stop colocated etcd member {member_name} for node {node_id}: {err}"
                ))
            })?;
        }

        self.whole_node_outages.insert(
            node_id.to_string(),
            WholeNodeOutageState {
                kind: WholeNodeOutageKind::CleanStop,
                etcd_member_name,
            },
        );
        Ok(())
    }

    pub async fn kill_whole_node(&mut self, node_id: &str) -> Result<(), WorkerError> {
        self.ensure_node_not_in_whole_outage(node_id)?;
        let node = self.node_by_id(node_id)?;
        self.runtime_nodes
            .kill_node(
                &node,
                self.timeouts.command_kill_wait_timeout,
                self.timeouts.http_step_timeout,
                self.timeouts.api_readiness_timeout,
            )
            .await?;

        pg_ctl_stop_immediate(
            self.binaries.pg_ctl.as_path(),
            node.data_dir.as_path(),
            self.timeouts.command_timeout,
            self.timeouts.command_kill_wait_timeout,
        )
        .await?;
        wait_for_postgres_unavailable(
            self.binaries.psql.as_path(),
            node.pg_port,
            self.superuser_username.as_str(),
            self.superuser_dbname.as_str(),
            self.timeouts.command_timeout,
            self.timeouts.command_kill_wait_timeout,
            self.timeouts.api_readiness_timeout,
        )
        .await?;

        let etcd_member_name = self.node_etcd_colocation.get(node_id).cloned();
        if let Some(member_name) = etcd_member_name.as_deref() {
            let etcd = self.etcd.as_mut().ok_or_else(|| {
                WorkerError::Message(format!(
                    "node {node_id} declares colocated etcd member {member_name} but no etcd cluster is running"
                ))
            })?;
            etcd.shutdown_member(member_name).await.map_err(|err| {
                WorkerError::Message(format!(
                    "failed to stop colocated etcd member {member_name} for node {node_id}: {err}"
                ))
            })?;
        }

        self.whole_node_outages.insert(
            node_id.to_string(),
            WholeNodeOutageState {
                kind: WholeNodeOutageKind::HardKill,
                etcd_member_name,
            },
        );
        Ok(())
    }

    pub async fn restart_whole_node(&mut self, node_id: &str) -> Result<(), WorkerError> {
        let outage = self.whole_node_outages.remove(node_id).ok_or_else(|| {
            WorkerError::Message(format!(
                "whole-node restart requested for node without active outage: {node_id}"
            ))
        })?;

        if let Some(member_name) = outage.etcd_member_name.clone() {
            let etcd = self.etcd.as_mut().ok_or_else(|| {
                WorkerError::Message(format!(
                    "node {node_id} requires colocated etcd member {member_name} restart but no etcd cluster is running"
                ))
            })?;
            if let Err(err) = etcd.restart_member(member_name.as_str()).await {
                self.whole_node_outages.insert(node_id.to_string(), outage);
                return Err(WorkerError::Message(format!(
                    "failed to restart colocated etcd member {member_name} for node {node_id}: {err}"
                )));
            }
        }

        let node = self.node_by_id(node_id)?;
        if let Err(err) = self
            .runtime_nodes
            .restart_node(
                &node,
                self.timeouts.http_step_timeout,
                self.timeouts.api_readiness_timeout,
                self.timeouts.command_kill_wait_timeout,
            )
            .await
        {
            self.whole_node_outages.insert(node_id.to_string(), outage);
            return Err(err);
        }

        Ok(())
    }
}
