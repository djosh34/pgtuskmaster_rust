use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::Duration;

use crate::cli::client::HaStateResponse;
use crate::state::WorkerError;

use super::handle::{NodeHandle, TestClusterHandle};
use super::util::{parse_psql_rows, parse_single_u64, run_psql_statement, unix_now};

impl TestClusterHandle {
    pub(crate) fn record(&mut self, message: impl Into<String>) {
        let now = match unix_now() {
            Ok(value) => value.0,
            Err(_) => 0,
        };
        self.timeline.push(format!("[{now}] {}", message.into()));
    }

    pub(crate) fn node_ids(&self) -> Vec<String> {
        self.nodes.iter().map(|node| node.id.clone()).collect()
    }

    pub(crate) fn node_by_id(&self, id: &str) -> Option<&NodeHandle> {
        self.nodes.iter().find(|node| node.id == id)
    }

    pub(crate) fn node_index_by_id(&self, id: &str) -> Option<usize> {
        self.nodes.iter().position(|node| node.id == id)
    }

    pub(crate) fn control_node_id(&self) -> Result<String, WorkerError> {
        self.nodes
            .first()
            .map(|node| node.id.clone())
            .ok_or_else(|| WorkerError::Message("no nodes available".to_string()))
    }

    pub(crate) fn postgres_port_by_id(&self, id: &str) -> Result<u16, WorkerError> {
        let node = self.node_by_id(id).ok_or_else(|| {
            WorkerError::Message(format!("unknown node id for postgres port lookup: {id}"))
        })?;
        Ok(node.pg_port)
    }

    fn sql_port_by_id(&self, id: &str) -> Result<u16, WorkerError> {
        let node = self.node_by_id(id).ok_or_else(|| {
            WorkerError::Message(format!("unknown node id for sql port lookup: {id}"))
        })?;
        Ok(node.sql_port)
    }

    pub(crate) async fn run_sql_on_node(&self, node_id: &str, sql: &str) -> Result<String, WorkerError> {
        let port = self.sql_port_by_id(node_id)?;
        run_psql_statement(
            self.binaries.psql.as_path(),
            port,
            self.superuser_username.as_str(),
            self.superuser_dbname.as_str(),
            sql,
            self.timeouts.command_timeout,
            self.timeouts.command_kill_wait_timeout,
        )
        .await
    }

    pub(crate) async fn run_sql_on_node_with_retry(
        &self,
        node_id: &str,
        sql: &str,
        timeout: Duration,
    ) -> Result<String, WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            match self.run_sql_on_node(node_id, sql).await {
                Ok(output) => return Ok(output),
                Err(err) => {
                    if tokio::time::Instant::now() >= deadline {
                        return Err(WorkerError::Message(format!(
                            "timed out running SQL on {node_id}; last_error={err}"
                        )));
                    }
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
            }
        }
    }

    pub(crate) async fn ensure_runtime_tasks_healthy(&mut self) -> Result<(), WorkerError> {
        let mut index = 0usize;
        while index < self.tasks.len() {
            if !self.tasks[index].is_finished() {
                index = index.saturating_add(1);
                continue;
            }

            let node_id = self
                .nodes
                .get(index)
                .map(|node| node.id.clone())
                .unwrap_or_else(|| format!("index-{index}"));
            let task = self.tasks.swap_remove(index);
            let joined = task
                .await
                .map_err(|err| WorkerError::Message(format!("runtime task join failed: {err}")))?;
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

    pub(crate) async fn shutdown(&mut self) -> Result<(), WorkerError> {
        let mut failures = Vec::new();

        for task in &self.tasks {
            task.abort();
        }
        while let Some(task) = self.tasks.pop() {
            let _ = task.await;
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

    pub(crate) fn primary_members(states: &[HaStateResponse]) -> Vec<String> {
        states
            .iter()
            .filter(|state| state.ha_phase == "Primary")
            .map(|state| state.self_member_id.clone())
            .collect()
    }

    pub(crate) fn format_phase_history(phase_history: &BTreeMap<String, BTreeSet<String>>) -> String {
        let mut entries = Vec::new();
        for (phase, members) in phase_history {
            let mut member_list: Vec<String> = members.iter().cloned().collect();
            member_list.sort();
            entries.push(format!("{phase}={}", member_list.join(",")));
        }
        entries.join(" | ")
    }

    pub(crate) fn write_timeline_artifact(&self, scenario: &str) -> Result<PathBuf, WorkerError> {
        let root = match &self.artifact_root {
            Some(path) => path.clone(),
            None => PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(".ralph/evidence/13-e2e-multi-node"),
        };
        std::fs::create_dir_all(&root).map_err(|err| {
            WorkerError::Message(format!("create timeline artifact dir failed: {err}"))
        })?;
        let stamp = unix_now()?.0;
        let safe_scenario = sanitize_component(scenario);
        let artifact_path = root.join(format!("{safe_scenario}-{stamp}.timeline.log"));
        std::fs::write(&artifact_path, self.timeline.join("\n")).map_err(|err| {
            WorkerError::Message(format!("write timeline artifact failed: {err}"))
        })?;
        Ok(artifact_path)
    }
}

fn sanitize_component(raw: &str) -> String {
    let mut safe: String = raw
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect();
    if safe.is_empty() {
        safe = "unknown".to_string();
    }
    safe
}

impl TestClusterHandle {
    pub(crate) async fn wait_for_rows_on_node(
        &self,
        node_id: &str,
        sql: &str,
        expected_rows: &[String],
        timeout: Duration,
    ) -> Result<(), WorkerError> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let output = self
                .run_sql_on_node_with_retry(node_id, sql, Duration::from_secs(5))
                .await?;
            let rows = parse_psql_rows(output.as_str());
            if rows == expected_rows {
                return Ok(());
            }
            let observed = format!("{rows:?}");
            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for expected rows on {node_id}; expected={expected_rows:?}; observed={observed}"
                )));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    pub(crate) async fn wait_for_table_digest_convergence(
        &mut self,
        node_ids: &[String],
        table_name: &str,
        timeout: Duration,
    ) -> Result<u64, WorkerError> {
        if node_ids.is_empty() {
            return Err(WorkerError::Message(
                "wait_for_table_digest_convergence requires at least one node id".to_string(),
            ));
        }

        let deadline = tokio::time::Instant::now() + timeout;
        let sql = format!(
            "SELECT COALESCE(SUM((worker_id::bigint * 1000000007 + seq::bigint) # payload::bigint), 0)::bigint FROM {}",
            table_name
        );

        loop {
            self.ensure_runtime_tasks_healthy().await?;

            let mut digests = BTreeMap::new();
            let mut errors = Vec::new();
            for node_id in node_ids {
                match self.run_sql_on_node_with_retry(node_id.as_str(), sql.as_str(), Duration::from_secs(10)).await {
                    Ok(output) => match parse_single_u64(output.as_str()) {
                        Ok(value) => {
                            digests.insert(node_id.clone(), value);
                        }
                        Err(err) => {
                            errors.push(format!("node={node_id} parse_error={err} output={}", output.trim()));
                        }
                    },
                    Err(err) => errors.push(format!("node={node_id} sql_error={err}")),
                }
            }

            if errors.is_empty() && !digests.is_empty() {
                let mut unique = BTreeSet::new();
                for digest in digests.values() {
                    unique.insert(*digest);
                }
                if unique.len() == 1 {
                    if let Some(value) = unique.into_iter().next() {
                        return Ok(value);
                    }
                }
            }

            let observed = format!("digests={digests:?} errors={}", errors.join(" | "));

            if tokio::time::Instant::now() >= deadline {
                return Err(WorkerError::Message(format!(
                    "timed out waiting for table digest convergence; last_observation={observed}"
                )));
            }

            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }
}
