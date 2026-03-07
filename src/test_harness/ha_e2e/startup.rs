use std::collections::{BTreeMap, BTreeSet};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use tokio::task::JoinHandle;

use crate::config::{
    BinaryPaths, DcsConfig, DcsInitConfig, DebugConfig, HaConfig, InlineOrPath, LogCleanupConfig,
    LogLevel, LoggingConfig, PgHbaConfig, PgIdentConfig, PostgresConfig, PostgresLoggingConfig,
    ProcessConfig, StderrSinkConfig,
};
use crate::state::WorkerError;
use crate::test_harness::binaries::{
    require_etcd_bin_for_real_tests, require_pg16_process_binaries_for_real_tests,
};
use crate::test_harness::etcd3::{
    prepare_etcd_member_data_dir, spawn_etcd3_cluster, EtcdClusterHandle, EtcdClusterMemberSpec,
    EtcdClusterSpec,
};
use crate::test_harness::namespace::NamespaceGuard;
use crate::test_harness::net_proxy::TcpProxyLink;
use crate::test_harness::pg16::prepare_pgdata_dir;
use crate::test_harness::ports::{allocate_ha_topology_ports, PortReservation};

use super::config::{Mode, TestConfig};
use super::handle::{NodeHandle, TestClusterHandle};
use super::util::{
    parse_http_endpoint, parse_loopback_socket, reserve_non_overlapping_ports,
    wait_for_bootstrap_primary, wait_for_node_api_ready_or_task_exit,
};

const ETCD_CLUSTER_STARTUP_TIMEOUT: Duration = Duration::from_secs(15);
const HARNESS_POSTGRES_CONNECT_TIMEOUT_S: u32 = 2;
const HARNESS_HA_LOOP_INTERVAL_MS: u64 = 100;
const HARNESS_HA_LEASE_TTL_MS: u64 = 2_000;
const HARNESS_PG_REWIND_TIMEOUT_MS: u64 = 5_000;
const HARNESS_BOOTSTRAP_TIMEOUT_MS: u64 = 30_000;
const HARNESS_FENCING_TIMEOUT_MS: u64 = 5_000;
const HARNESS_LOGGING_POLL_INTERVAL_MS: u64 = 200;
const HARNESS_LOGGING_CLEANUP_MAX_FILES: u64 = 50;
const HARNESS_LOGGING_CLEANUP_MAX_AGE_SECONDS: u64 = 7 * 24 * 60 * 60;
const HARNESS_LOGGING_PROTECT_RECENT_SECONDS: u64 = 300;

struct StartupGuard {
    guard: NamespaceGuard,
    binaries: BinaryPaths,
    superuser_username: Option<String>,
    superuser_dbname: Option<String>,
    etcd: Option<EtcdClusterHandle>,
    nodes: Vec<NodeHandle>,
    tasks: Vec<JoinHandle<Result<(), WorkerError>>>,
    etcd_proxies: BTreeMap<String, TcpProxyLink>,
    api_proxies: BTreeMap<String, TcpProxyLink>,
    pg_proxies: BTreeMap<String, TcpProxyLink>,
    timeouts: super::config::TimeoutConfig,
}

impl StartupGuard {
    async fn cleanup_best_effort(&mut self) -> Result<(), WorkerError> {
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
                "startup cleanup failures: {}",
                failures.join("; ")
            )))
        }
    }

    fn into_handle(self) -> Result<TestClusterHandle, WorkerError> {
        let superuser_username = self.superuser_username.ok_or_else(|| {
            WorkerError::Message("startup missing postgres superuser username".to_string())
        })?;
        let superuser_dbname = self.superuser_dbname.ok_or_else(|| {
            WorkerError::Message("startup missing postgres superuser dbname".to_string())
        })?;

        Ok(TestClusterHandle {
            guard: self.guard,
            timeouts: self.timeouts,
            binaries: self.binaries,
            superuser_username,
            superuser_dbname,
            etcd: self.etcd,
            nodes: self.nodes,
            tasks: self.tasks,
            etcd_proxies: self.etcd_proxies,
            api_proxies: self.api_proxies,
            pg_proxies: self.pg_proxies,
        })
    }
}

pub async fn start_cluster(config: TestConfig) -> Result<TestClusterHandle, WorkerError> {
    let mut config = config;
    config.validate()?;

    let namespace_guard = NamespaceGuard::new(config.test_name.as_str())?;
    let namespace_id = namespace_guard.namespace()?.id.clone();
    config.scope = format!("{}-{}", config.scope, namespace_id);
    config.cluster_name = format!("{}-{}", config.cluster_name, namespace_id);

    let binaries = require_pg16_process_binaries_for_real_tests()?;
    let etcd_bin = require_etcd_bin_for_real_tests()?;

    let mut guard = StartupGuard {
        guard: namespace_guard,
        binaries: binaries.clone(),
        superuser_username: None,
        superuser_dbname: None,
        etcd: None,
        nodes: Vec::new(),
        tasks: Vec::new(),
        etcd_proxies: BTreeMap::new(),
        api_proxies: BTreeMap::new(),
        pg_proxies: BTreeMap::new(),
        timeouts: config.timeouts.clone(),
    };

    match start_cluster_inner(&mut guard, config, etcd_bin, binaries).await {
        Ok(()) => guard.into_handle(),
        Err(start_err) => {
            let cleanup_result = guard.cleanup_best_effort().await;
            match cleanup_result {
                Ok(()) => Err(start_err),
                Err(cleanup_err) => Err(WorkerError::Message(format!(
                    "{start_err}; cleanup failed: {cleanup_err}"
                ))),
            }
        }
    }
}

async fn start_cluster_inner(
    guard: &mut StartupGuard,
    config: TestConfig,
    etcd_bin: PathBuf,
    binaries: BinaryPaths,
) -> Result<(), WorkerError> {
    let namespace = guard.guard.namespace()?.clone();
    let etcd_member_count = config.etcd_members.len();
    let mut topology_reservation =
        allocate_ha_topology_ports(config.node_count, etcd_member_count)?;
    let topology = topology_reservation.layout().clone();
    let node_ports = topology.node_ports.clone();

    let mut forbidden_ports: BTreeSet<u16> = topology
        .etcd_client_ports
        .iter()
        .chain(topology.etcd_peer_ports.iter())
        .chain(node_ports.iter())
        .copied()
        .collect();

    let mut members = Vec::with_capacity(etcd_member_count);
    for (index, member_name) in config.etcd_members.iter().enumerate() {
        let data_dir = prepare_etcd_member_data_dir(&namespace, member_name)?;
        let log_dir = namespace.child_dir(format!("logs/{member_name}"));
        let client_port = *topology.etcd_client_ports.get(index).ok_or_else(|| {
            WorkerError::Message(format!("missing etcd client port for {member_name}"))
        })?;
        let peer_port = *topology.etcd_peer_ports.get(index).ok_or_else(|| {
            WorkerError::Message(format!("missing etcd peer port for {member_name}"))
        })?;

        members.push(EtcdClusterMemberSpec {
            member_name: member_name.clone(),
            data_dir,
            log_dir,
            client_port,
            peer_port,
        });
    }

    let cluster_spec = EtcdClusterSpec {
        etcd_bin,
        namespace_id: namespace.id.clone(),
        startup_timeout: ETCD_CLUSTER_STARTUP_TIMEOUT,
        members,
    };

    for port in topology
        .etcd_client_ports
        .iter()
        .chain(topology.etcd_peer_ports.iter())
    {
        topology_reservation.release_port(*port).map_err(|err| {
            WorkerError::Message(format!("release etcd reserved port failed: {err}"))
        })?;
    }

    let etcd = spawn_etcd3_cluster(cluster_spec).await?;
    let endpoints = etcd.client_endpoints().to_vec();
    let endpoint_count = endpoints.len();
    if endpoint_count == 0 {
        return Err(WorkerError::Message(
            "etcd cluster returned no endpoints".to_string(),
        ));
    }
    guard.etcd = Some(etcd);

    let mut api_reservation = reserve_non_overlapping_ports(config.node_count, &forbidden_ports)?;
    let api_ports = api_reservation.as_slice().to_vec();
    if api_ports.len() != config.node_count {
        return Err(WorkerError::Message(format!(
            "api port reservation mismatch: expected {}, got {}",
            config.node_count,
            api_ports.len()
        )));
    }
    for port in &api_ports {
        forbidden_ports.insert(*port);
    }

    let mut cursor = 0usize;
    let mut proxy_reservation = PortReservation::empty();
    let (dcs_endpoints_by_node, proxy_ports) = match config.mode {
        Mode::Plain => (None, Vec::new()),
        Mode::PartitionProxy => {
            let total_proxy_ports = config.node_count.checked_mul(3).ok_or_else(|| {
                WorkerError::Message("proxy port count overflow for partition mode".to_string())
            })?;
            proxy_reservation = reserve_non_overlapping_ports(total_proxy_ports, &forbidden_ports)?;
            let proxy_ports = proxy_reservation.as_slice().to_vec();
            let dcs_endpoints_by_node = spawn_partition_etcd_proxies(
                guard,
                config.node_count,
                &endpoints,
                proxy_ports.as_slice(),
                &mut cursor,
                &mut proxy_reservation,
            )
            .await?;
            (Some(dcs_endpoints_by_node), proxy_ports)
        }
    };

    let next_proxy_listener = |ports: &[u16],
                               cursor_ref: &mut usize,
                               reservation: &mut PortReservation|
     -> Result<std::net::TcpListener, WorkerError> {
        if *cursor_ref >= ports.len() {
            return Err(WorkerError::Message(
                "proxy port allocation cursor out of bounds".to_string(),
            ));
        }
        let selected = ports[*cursor_ref];
        *cursor_ref = cursor_ref.saturating_add(1);
        reservation.take_listener(selected).map_err(|err| {
            WorkerError::Message(format!(
                "take proxy reserved listener failed for port={selected}: {err}"
            ))
        })
    };

    for (index, (pg_port, api_port)) in node_ports.iter().copied().zip(api_ports).enumerate() {
        let node_id = format!("node-{}", index.saturating_add(1));
        let data_dir = prepare_pgdata_dir(&namespace, &node_id)?;
        let socket_dir = namespace.child_dir(format!("run/{node_id}"));
        let log_file = namespace.child_dir(format!("logs/{node_id}/postgres.log"));
        if let Some(parent) = log_file.parent() {
            std::fs::create_dir_all(parent).map_err(|err| {
                WorkerError::Message(format!(
                    "create postgres log dir failed for node {node_id}: {err}"
                ))
            })?;
        }

        let api_addr: SocketAddr = format!("127.0.0.1:{api_port}")
            .parse()
            .map_err(|err| WorkerError::Message(format!("parse api addr failed: {err}")))?;

        let (api_observe_addr, sql_port) = match config.mode {
            Mode::Plain => (api_addr, pg_port),
            Mode::PartitionProxy => {
                let api_listener = next_proxy_listener(
                    proxy_ports.as_slice(),
                    &mut cursor,
                    &mut proxy_reservation,
                )?;
                let api_proxy = TcpProxyLink::spawn_with_listener(
                    format!("{node_id}-api-proxy"),
                    api_listener,
                    api_addr,
                )
                .await
                .map_err(|err| {
                    WorkerError::Message(format!(
                        "spawn api proxy failed for node {node_id}: {err}"
                    ))
                })?;
                let api_proxy_addr = api_proxy.listen_addr();
                guard.api_proxies.insert(node_id.clone(), api_proxy);

                let pg_listener = next_proxy_listener(
                    proxy_ports.as_slice(),
                    &mut cursor,
                    &mut proxy_reservation,
                )?;
                let pg_target_addr = parse_loopback_socket(pg_port)?;
                let pg_proxy = TcpProxyLink::spawn_with_listener(
                    format!("{node_id}-pg-proxy"),
                    pg_listener,
                    pg_target_addr,
                )
                .await
                .map_err(|err| {
                    WorkerError::Message(format!(
                        "spawn postgres proxy failed for node {node_id}: {err}"
                    ))
                })?;
                let pg_proxy_addr = pg_proxy.listen_addr();
                guard.pg_proxies.insert(node_id.clone(), pg_proxy);

                (api_proxy_addr, pg_proxy_addr.port())
            }
        };

        let dcs_endpoints = match (config.mode, &dcs_endpoints_by_node) {
            (Mode::Plain, _) => endpoints.clone(),
            (Mode::PartitionProxy, Some(map)) => {
                map.get(node_id.as_str()).cloned().ok_or_else(|| {
                    WorkerError::Message(format!(
                        "missing proxy DCS endpoints for node runtime config: {node_id}"
                    ))
                })?
            }
            (Mode::PartitionProxy, None) => {
                return Err(WorkerError::Message(
                    "partition mode missing DCS endpoints map".to_string(),
                ));
            }
        };

        let pg_hba_contents = concat!(
            "# managed by pgtuskmaster test harness\n",
            "local all all trust\n",
            "host all all 127.0.0.1/32 trust\n",
            "host replication replicator 127.0.0.1/32 trust\n",
        )
        .to_string();
        let pg_ident_contents = "# empty\n".to_string();

        let dcs_endpoints_for_check = dcs_endpoints.clone();
        let dcs_init_payload = serde_json::json!({
            "cluster": {
                "name": config.cluster_name.clone(),
                "member_id": node_id.clone(),
            },
            "postgres": {
                "data_dir": data_dir.display().to_string(),
                "connect_timeout_s": HARNESS_POSTGRES_CONNECT_TIMEOUT_S,
                "listen_host": "127.0.0.1",
                "listen_port": pg_port,
                "socket_dir": socket_dir.display().to_string(),
                "log_file": log_file.display().to_string(),
                "local_conn_identity": { "user": "postgres", "dbname": "postgres", "ssl_mode": "prefer" },
                "rewind_conn_identity": { "user": "rewinder", "dbname": "postgres", "ssl_mode": "prefer" },
                "tls": { "mode": "disabled", "identity": null, "client_auth": null },
                "roles": {
                    "superuser": { "username": "postgres", "auth": { "type": "tls" } },
                    "replicator": { "username": "replicator", "auth": { "type": "tls" } },
                    "rewinder": { "username": "rewinder", "auth": { "type": "tls" } },
                },
                "pg_hba": { "source": { "content": pg_hba_contents.clone() } },
                "pg_ident": { "source": { "content": pg_ident_contents.clone() } },
                "extra_gucs": {},
            },
            "dcs": {
                "endpoints": dcs_endpoints_for_check.clone(),
                "scope": config.scope.clone(),
                "init": null,
            },
            "ha": {
                "loop_interval_ms": HARNESS_HA_LOOP_INTERVAL_MS,
                "lease_ttl_ms": HARNESS_HA_LEASE_TTL_MS,
            },
            "process": {
                "pg_rewind_timeout_ms": HARNESS_PG_REWIND_TIMEOUT_MS,
                "bootstrap_timeout_ms": HARNESS_BOOTSTRAP_TIMEOUT_MS,
                "fencing_timeout_ms": HARNESS_FENCING_TIMEOUT_MS,
                "binaries": {
                    "postgres": binaries.postgres.display().to_string(),
                    "pg_ctl": binaries.pg_ctl.display().to_string(),
                    "pg_rewind": binaries.pg_rewind.display().to_string(),
                    "initdb": binaries.initdb.display().to_string(),
                    "pg_basebackup": binaries.pg_basebackup.display().to_string(),
                    "psql": binaries.psql.display().to_string(),
                },
            },
            "logging": {
                "level": "info",
                "capture_subprocess_output": false,
                "postgres": {
                    "enabled": false,
                    "pg_ctl_log_file": null,
                    "log_dir": null,
                    "poll_interval_ms": HARNESS_LOGGING_POLL_INTERVAL_MS,
                    "cleanup": {
                        "enabled": true,
                        "max_files": HARNESS_LOGGING_CLEANUP_MAX_FILES,
                        "max_age_seconds": HARNESS_LOGGING_CLEANUP_MAX_AGE_SECONDS,
                        "protect_recent_seconds": HARNESS_LOGGING_PROTECT_RECENT_SECONDS
                    },
                },
                "sinks": {
                    "stderr": { "enabled": true },
                    "file": { "enabled": false, "path": null, "mode": "append" },
                },
            },
            "api": {
                "listen_addr": api_addr.to_string(),
                "security": {
                    "tls": { "mode": "disabled", "identity": null, "client_auth": null },
                    "auth": { "type": "disabled" },
                },
            },
            "debug": { "enabled": false },
        });
        let dcs_init_payload_json = serde_json::to_string(&dcs_init_payload).map_err(|err| {
            WorkerError::Message(format!(
                "encode dcs.init.payload_json failed for node {node_id}: {err}"
            ))
        })?;

        let runtime_cfg = crate::test_harness::runtime_config::RuntimeConfigBuilder::new()
            .with_cluster_name(config.cluster_name.clone())
            .with_member_id(node_id.clone())
            .transform_postgres(|postgres| PostgresConfig {
                data_dir: data_dir.clone(),
                connect_timeout_s: HARNESS_POSTGRES_CONNECT_TIMEOUT_S,
                listen_port: pg_port,
                socket_dir,
                log_file: log_file.clone(),
                pg_hba: PgHbaConfig {
                    source: InlineOrPath::Inline {
                        content: pg_hba_contents.clone(),
                    },
                },
                pg_ident: PgIdentConfig {
                    source: InlineOrPath::Inline {
                        content: pg_ident_contents.clone(),
                    },
                },
                ..postgres
            })
            .with_dcs(DcsConfig {
                endpoints: dcs_endpoints,
                scope: config.scope.clone(),
                init: Some(DcsInitConfig {
                    payload_json: dcs_init_payload_json.clone(),
                    write_on_bootstrap: true,
                }),
            })
            .with_ha(HaConfig {
                loop_interval_ms: HARNESS_HA_LOOP_INTERVAL_MS,
                lease_ttl_ms: HARNESS_HA_LEASE_TTL_MS,
            })
            .with_process(ProcessConfig {
                pg_rewind_timeout_ms: HARNESS_PG_REWIND_TIMEOUT_MS,
                bootstrap_timeout_ms: HARNESS_BOOTSTRAP_TIMEOUT_MS,
                fencing_timeout_ms: HARNESS_FENCING_TIMEOUT_MS,
                binaries: binaries.clone(),
            })
            .with_logging(LoggingConfig {
                level: LogLevel::Info,
                capture_subprocess_output: false,
                postgres: PostgresLoggingConfig {
                    enabled: false,
                    pg_ctl_log_file: None,
                    log_dir: None,
                    poll_interval_ms: HARNESS_LOGGING_POLL_INTERVAL_MS,
                    cleanup: LogCleanupConfig {
                        enabled: true,
                        max_files: HARNESS_LOGGING_CLEANUP_MAX_FILES,
                        max_age_seconds: HARNESS_LOGGING_CLEANUP_MAX_AGE_SECONDS,
                        protect_recent_seconds: HARNESS_LOGGING_PROTECT_RECENT_SECONDS,
                    },
                },
                sinks: crate::config::LoggingSinksConfig {
                    stderr: StderrSinkConfig { enabled: true },
                    file: crate::config::FileSinkConfig {
                        enabled: false,
                        path: None,
                        mode: crate::config::FileSinkMode::Append,
                    },
                },
            })
            .with_api_listen_addr(api_addr.to_string())
            .with_debug(DebugConfig { enabled: false })
            .build();

        let runtime_superuser_username = runtime_cfg.postgres.roles.superuser.username.clone();
        let runtime_superuser_dbname = runtime_cfg.postgres.local_conn_identity.dbname.clone();
        match (&guard.superuser_username, &guard.superuser_dbname) {
            (None, None) => {
                guard.superuser_username = Some(runtime_superuser_username);
                guard.superuser_dbname = Some(runtime_superuser_dbname);
            }
            (Some(expected_user), Some(expected_dbname)) => {
                if expected_user.as_str() != runtime_superuser_username.as_str()
                    || expected_dbname.as_str() != runtime_superuser_dbname.as_str()
                {
                    return Err(WorkerError::Message(format!(
                        "inconsistent superuser identity across nodes: expected user/dbname {}/{} but got {}/{}",
                        expected_user,
                        expected_dbname,
                        runtime_superuser_username,
                        runtime_superuser_dbname
                    )));
                }
            }
            _ => {
                return Err(WorkerError::Message(
                    "startup guard superuser identity partially initialized".to_string(),
                ));
            }
        }

        topology_reservation.release_port(pg_port).map_err(|err| {
            WorkerError::Message(format!("release postgres reserved port failed: {err}"))
        })?;
        api_reservation.release_port(api_port).map_err(|err| {
            WorkerError::Message(format!("release api reserved port failed: {err}"))
        })?;

        let task_node_id = node_id.clone();
        let runtime_task = tokio::task::spawn_local(async move {
            match crate::runtime::run_node_from_config(runtime_cfg).await {
                Ok(()) => Ok(()),
                Err(err) => Err(WorkerError::Message(format!(
                    "runtime node {task_node_id} exited with error: {err}"
                ))),
            }
        });

        guard.nodes.push(NodeHandle {
            id: node_id.clone(),
            pg_port,
            sql_port,
            api_addr,
            api_observe_addr,
            data_dir,
        });

        let runtime_task = wait_for_node_api_ready_or_task_exit(
            api_observe_addr,
            node_id.as_str(),
            log_file.as_path(),
            runtime_task,
            config.timeouts.http_step_timeout,
            config.timeouts.api_readiness_timeout,
        )
        .await?;
        guard.tasks.push(runtime_task);

        if index == 0 {
            let expected_member_id = format!("node-{}", index.saturating_add(1));
            wait_for_bootstrap_primary(
                api_observe_addr,
                expected_member_id.as_str(),
                config.timeouts.http_step_timeout,
                config.timeouts.bootstrap_primary_timeout,
            )
            .await?;

            // Clone/basebackup connects using the configured replicator role. Ensure that role
            // exists on the elected primary before bringing up other nodes.
            let superuser_username = guard.superuser_username.as_deref().ok_or_else(|| {
                WorkerError::Message("startup missing postgres superuser username".to_string())
            })?;
            let superuser_dbname = guard.superuser_dbname.as_deref().ok_or_else(|| {
                WorkerError::Message("startup missing postgres superuser dbname".to_string())
            })?;

            let create_roles_sql = r#"
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'replicator') THEN
    CREATE ROLE replicator WITH LOGIN REPLICATION;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'rewinder') THEN
    -- pg_rewind typically needs superuser privileges; keep tests conservative.
    CREATE ROLE rewinder WITH LOGIN SUPERUSER;
  END IF;
END
$$;
"#;
            let _ = super::util::run_psql_statement(
                guard.binaries.psql.as_path(),
                sql_port,
                superuser_username,
                superuser_dbname,
                create_roles_sql,
                guard.timeouts.command_timeout,
                guard.timeouts.command_kill_wait_timeout,
            )
            .await?;

            let primary = guard.nodes.last().ok_or_else(|| {
                WorkerError::Message("startup expected primary node handle".to_string())
            })?;
            let expected_hba_file = primary.data_dir.join("pgtm.pg_hba.conf");
            let expected_ident_file = primary.data_dir.join("pgtm.pg_ident.conf");
            let expected_managed_postgresql_conf = primary.data_dir.join("pgtm.postgresql.conf");

            let hba_file_raw = super::util::run_psql_statement(
                guard.binaries.psql.as_path(),
                sql_port,
                superuser_username,
                superuser_dbname,
                "SHOW hba_file;",
                guard.timeouts.command_timeout,
                guard.timeouts.command_kill_wait_timeout,
            )
            .await?;
            let ident_file_raw = super::util::run_psql_statement(
                guard.binaries.psql.as_path(),
                sql_port,
                superuser_username,
                superuser_dbname,
                "SHOW ident_file;",
                guard.timeouts.command_timeout,
                guard.timeouts.command_kill_wait_timeout,
            )
            .await?;
            let config_file_raw = super::util::run_psql_statement(
                guard.binaries.psql.as_path(),
                sql_port,
                superuser_username,
                superuser_dbname,
                "SHOW config_file;",
                guard.timeouts.command_timeout,
                guard.timeouts.command_kill_wait_timeout,
            )
            .await?;

            let expected_hba = expected_hba_file.display().to_string();
            let expected_ident = expected_ident_file.display().to_string();
            let expected_config = expected_managed_postgresql_conf.display().to_string();
            if hba_file_raw.trim() != expected_hba.as_str() {
                return Err(WorkerError::Message(format!(
                    "expected SHOW hba_file to be `{expected_hba}`, got: {:?}",
                    hba_file_raw.trim()
                )));
            }
            if ident_file_raw.trim() != expected_ident.as_str() {
                return Err(WorkerError::Message(format!(
                    "expected SHOW ident_file to be `{expected_ident}`, got: {:?}",
                    ident_file_raw.trim()
                )));
            }
            if config_file_raw.trim() != expected_config.as_str() {
                return Err(WorkerError::Message(format!(
                    "expected SHOW config_file to be `{expected_config}`, got: {:?}",
                    config_file_raw.trim()
                )));
            }

            let disk_hba = std::fs::read_to_string(&expected_hba_file).map_err(|err| {
                WorkerError::Message(format!(
                    "read managed hba file {} failed: {err}",
                    expected_hba_file.display()
                ))
            })?;
            if disk_hba != pg_hba_contents {
                return Err(WorkerError::Message(format!(
                    "managed hba file did not match configured content; file={} expected_len={} actual_len={}",
                    expected_hba_file.display(),
                    pg_hba_contents.len(),
                    disk_hba.len(),
                )));
            }
            let disk_ident = std::fs::read_to_string(&expected_ident_file).map_err(|err| {
                WorkerError::Message(format!(
                    "read managed ident file {} failed: {err}",
                    expected_ident_file.display()
                ))
            })?;
            if disk_ident != pg_ident_contents {
                return Err(WorkerError::Message(format!(
                    "managed ident file did not match configured content; file={} expected_len={} actual_len={}",
                    expected_ident_file.display(),
                    pg_ident_contents.len(),
                    disk_ident.len(),
                )));
            }
            let disk_managed_postgresql_conf =
                std::fs::read_to_string(&expected_managed_postgresql_conf).map_err(|err| {
                    WorkerError::Message(format!(
                        "read managed postgresql conf {} failed: {err}",
                        expected_managed_postgresql_conf.display()
                    ))
                })?;
            if disk_managed_postgresql_conf.contains("archive_mode")
                || disk_managed_postgresql_conf.contains("archive_command")
                || disk_managed_postgresql_conf.contains("restore_command")
            {
                return Err(WorkerError::Message(format!(
                    "managed postgresql conf unexpectedly contains backup settings: {:?}",
                    disk_managed_postgresql_conf
                )));
            }
            if !disk_managed_postgresql_conf.contains(expected_hba.as_str())
                || !disk_managed_postgresql_conf.contains(expected_ident.as_str())
            {
                return Err(WorkerError::Message(format!(
                    "managed postgresql conf did not reference expected managed hba/ident files: {:?}",
                    disk_managed_postgresql_conf
                )));
            }
            if !disk_managed_postgresql_conf.contains("listen_addresses = '127.0.0.1'")
                || !disk_managed_postgresql_conf.contains(format!("port = {pg_port}").as_str())
            {
                return Err(WorkerError::Message(format!(
                    "managed postgresql conf missing expected listen/port settings: {:?}",
                    disk_managed_postgresql_conf
                )));
            }

            let init_key = format!("/{}/init", config.scope.trim_matches('/'));
            let config_key = format!("/{}/config", config.scope.trim_matches('/'));
            let mut etcd_client =
                etcd_client::Client::connect(dcs_endpoints_for_check.clone(), None)
                    .await
                    .map_err(|err| {
                        WorkerError::Message(format!(
                            "etcd connect for init/config check failed: {err}"
                        ))
                    })?;
            let init_response = etcd_client
                .get(init_key.as_str(), None)
                .await
                .map_err(|err| WorkerError::Message(format!("etcd get init key failed: {err}")))?;
            if init_response.kvs().is_empty() {
                return Err(WorkerError::Message(format!(
                    "expected init key to exist at {init_key}"
                )));
            }

            let config_response =
                etcd_client
                    .get(config_key.as_str(), None)
                    .await
                    .map_err(|err| {
                        WorkerError::Message(format!("etcd get config key failed: {err}"))
                    })?;
            let Some(kv) = config_response.kvs().first() else {
                return Err(WorkerError::Message(format!(
                    "expected config key to exist at {config_key}"
                )));
            };
            let raw = std::str::from_utf8(kv.value()).map_err(|err| {
                WorkerError::Message(format!("config value not utf8 at {config_key}: {err}"))
            })?;
            let decoded: serde_json::Value = serde_json::from_str(raw).map_err(|err| {
                WorkerError::Message(format!(
                    "config payload stored in etcd was not valid json at {config_key}: {err}"
                ))
            })?;
            if decoded != dcs_init_payload {
                return Err(WorkerError::Message(format!(
                    "etcd config payload mismatch: expected={dcs_init_payload_json} got={raw}"
                )));
            }
        }
    }

    if config.mode == Mode::PartitionProxy && cursor != proxy_ports.len() {
        return Err(WorkerError::Message(format!(
            "proxy port cursor mismatch: used={cursor} allocated={}",
            proxy_ports.len()
        )));
    }

    // Keep port reservations alive until the entire cluster is ready; the runtime binds
    // ports asynchronously after we release the OS-level reservation sockets.
    drop(proxy_reservation);
    drop(api_reservation);
    drop(topology_reservation);

    Ok(())
}

async fn spawn_partition_etcd_proxies(
    guard: &mut StartupGuard,
    node_count: usize,
    endpoints: &[String],
    proxy_ports: &[u16],
    cursor: &mut usize,
    proxy_reservation: &mut PortReservation,
) -> Result<BTreeMap<String, Vec<String>>, WorkerError> {
    let member_names = guard
        .etcd
        .as_ref()
        .ok_or_else(|| WorkerError::Message("missing etcd cluster handle".to_string()))?
        .member_names();
    if member_names.len() != endpoints.len() {
        return Err(WorkerError::Message(format!(
            "etcd members/endpoints mismatch: members={} endpoints={}",
            member_names.len(),
            endpoints.len()
        )));
    }

    let next_listener = |ports: &[u16],
                         cursor_ref: &mut usize,
                         reservation: &mut PortReservation|
     -> Result<std::net::TcpListener, WorkerError> {
        if *cursor_ref >= ports.len() {
            return Err(WorkerError::Message(
                "proxy port allocation cursor out of bounds".to_string(),
            ));
        }
        let selected = ports[*cursor_ref];
        *cursor_ref = cursor_ref.saturating_add(1);
        reservation.take_listener(selected).map_err(|err| {
            WorkerError::Message(format!(
                "take proxy reserved listener failed for port={selected}: {err}"
            ))
        })
    };

    let mut dcs_endpoints_by_node: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for node_index in 0..node_count {
        let node_id = format!("node-{}", node_index.saturating_add(1));
        let endpoint_index = node_index % endpoints.len();
        let member_name = member_names.get(endpoint_index).ok_or_else(|| {
            WorkerError::Message(format!(
                "missing etcd member name for endpoint index {endpoint_index}"
            ))
        })?;
        let endpoint = endpoints.get(endpoint_index).ok_or_else(|| {
            WorkerError::Message(format!("missing etcd endpoint for index {endpoint_index}"))
        })?;
        let target_addr = parse_http_endpoint(endpoint.as_str())?;
        let link_name = format!("{node_id}-to-{member_name}-etcd");
        let listener = next_listener(proxy_ports, cursor, proxy_reservation)?;
        let link = TcpProxyLink::spawn_with_listener(link_name.clone(), listener, target_addr)
            .await
            .map_err(|err| {
                WorkerError::Message(format!(
                    "spawn etcd proxy failed for node {node_id} link={link_name}: {err}"
                ))
            })?;

        let proxy_url = format!("http://{}", link.listen_addr());
        guard.etcd_proxies.insert(link_name.clone(), link);
        dcs_endpoints_by_node.insert(node_id, vec![proxy_url]);
    }

    Ok(dcs_endpoints_by_node)
}
