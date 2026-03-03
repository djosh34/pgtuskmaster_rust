use crate::state::WorkerError;
use crate::test_harness::net_proxy::ProxyMode;

use super::config::Mode;
use super::handle::TestClusterHandle;

impl TestClusterHandle {
    async fn set_etcd_mode_for_node(&self, node_id: &str, mode: ProxyMode) -> Result<(), WorkerError> {
        if self.mode != Mode::PartitionProxy {
            return Err(WorkerError::Message(format!(
                "etcd proxy manipulation requires PartitionProxy mode (node={node_id})"
            )));
        }

        let links = self.etcd_links_by_node.get(node_id).ok_or_else(|| {
            WorkerError::Message(format!("unknown node for etcd partition: {node_id}"))
        })?;
        for link_name in links {
            let link = self.etcd_proxies.get(link_name.as_str()).ok_or_else(|| {
                WorkerError::Message(format!(
                    "missing etcd proxy link for node {node_id}: {link_name}"
                ))
            })?;
            link.set_mode(mode.clone()).await?;
        }
        Ok(())
    }

    pub(crate) async fn partition_node_from_etcd(&mut self, node_id: &str) -> Result<(), WorkerError> {
        self.record(format!(
            "network fault: partition node from etcd majority node={node_id}"
        ));
        self.set_etcd_mode_for_node(node_id, ProxyMode::Blocked).await
    }

    pub(crate) async fn partition_primary_from_etcd(
        &mut self,
        node_id: &str,
    ) -> Result<(), WorkerError> {
        self.record(format!(
            "network fault: partition current primary from etcd node={node_id}"
        ));
        self.set_etcd_mode_for_node(node_id, ProxyMode::Blocked).await
    }

    pub(crate) async fn isolate_api_path(&mut self, node_id: &str) -> Result<(), WorkerError> {
        if self.mode != Mode::PartitionProxy {
            return Err(WorkerError::Message(format!(
                "api isolation requires PartitionProxy mode (node={node_id})"
            )));
        }

        self.record(format!(
            "network fault: isolate API path for node={node_id}"
        ));
        let link = self.api_proxies.get(node_id).ok_or_else(|| {
            WorkerError::Message(format!("missing api proxy for node: {node_id}"))
        })?;
        link.set_mode(ProxyMode::Blocked).await?;
        Ok(())
    }

    pub(crate) async fn heal_all_network_faults(&mut self) -> Result<(), WorkerError> {
        if self.mode != Mode::PartitionProxy {
            return Ok(());
        }

        self.record("network heal: reset all proxy links to pass-through".to_string());
        for link in self.etcd_proxies.values() {
            link.set_mode(ProxyMode::PassThrough).await?;
        }
        for link in self.api_proxies.values() {
            link.set_mode(ProxyMode::PassThrough).await?;
        }
        for link in self.pg_proxies.values() {
            link.set_mode(ProxyMode::PassThrough).await?;
        }
        Ok(())
    }
}

