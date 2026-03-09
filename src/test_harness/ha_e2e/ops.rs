use crate::state::WorkerError;

use super::handle::TestClusterHandle;

impl TestClusterHandle {
    pub async fn shutdown(&mut self) -> Result<(), WorkerError> {
        let mut failures = Vec::new();

        if let Err(err) = self.runtime_nodes.shutdown_all().await {
            failures.push(format!("runtime shutdown failed: {err}"));
        }

        for node in &self.nodes {
            if let Err(err) = super::util::pg_ctl_stop_immediate(
                self.binaries.pg_ctl.as_path(),
                node.data_dir.as_path(),
                self.timeouts.command_timeout,
                self.timeouts.command_kill_wait_timeout,
            )
            .await
            {
                failures.push(format!("postgres stop {} failed: {err}", node.id));
            }
        }

        let etcd_proxy_map = std::mem::take(&mut self.etcd_proxies);
        for (name, proxy) in etcd_proxy_map {
            if let Err(err) = proxy.shutdown().await {
                failures.push(format!("etcd proxy {name} shutdown failed: {err}"));
            }
        }

        let api_proxy_map = std::mem::take(&mut self.api_proxies);
        for (name, proxy) in api_proxy_map {
            if let Err(err) = proxy.shutdown().await {
                failures.push(format!("api proxy {name} shutdown failed: {err}"));
            }
        }

        let pg_proxy_map = std::mem::take(&mut self.pg_proxies);
        for (name, proxy) in pg_proxy_map {
            if let Err(err) = proxy.shutdown().await {
                failures.push(format!("postgres proxy {name} shutdown failed: {err}"));
            }
        }

        if let Some(etcd) = self.etcd.as_mut() {
            if let Err(err) = etcd.shutdown_all().await {
                failures.push(format!("etcd shutdown failed: {err}"));
            }
        }
        self.etcd = None;

        if failures.is_empty() {
            Ok(())
        } else {
            Err(WorkerError::Message(format!(
                "cluster shutdown failures: {}",
                failures.join("; ")
            )))
        }
    }
}
