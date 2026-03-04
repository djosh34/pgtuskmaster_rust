use std::collections::{BTreeMap, BTreeSet};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use tokio::task::JoinHandle;

use crate::cli::client::CliApiClient;
use crate::config::{
    ApiAuthConfig, ApiConfig, ApiSecurityConfig, ApiTlsMode, BinaryPaths, ClusterConfig, DcsConfig,
    DebugConfig, HaConfig, InlineOrPath, LogCleanupConfig, LogLevel, LoggingConfig, PgHbaConfig,
    PgIdentConfig, PostgresConnIdentityConfig, PostgresConfig, PostgresLoggingConfig,
    PostgresRoleConfig, PostgresRolesConfig, ProcessConfig, RoleAuthConfig, RuntimeConfig,
    StderrSinkConfig, TlsServerConfig,
};
use crate::pginfo::conninfo::PgSslMode;
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
    http_timeout_ms, parse_http_endpoint, parse_loopback_socket, reserve_non_overlapping_ports,
    wait_for_bootstrap_primary, wait_for_node_api_ready_or_task_exit,
};

struct StartupGuard {
    guard: NamespaceGuard,
    scope: String,
    cluster_name: String,
    mode: Mode,
    binaries: BinaryPaths,
    superuser_username: Option<String>,
    superuser_dbname: Option<String>,
    etcd: Option<EtcdClusterHandle>,
    nodes: Vec<NodeHandle>,
    api_clients: Vec<CliApiClient>,
    tasks: Vec<JoinHandle<Result<(), WorkerError>>>,
    timeline: Vec<String>,
    artifact_root: Option<PathBuf>,
    etcd_links_by_node: BTreeMap<String, Vec<String>>,
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
            scope: self.scope,
            cluster_name: self.cluster_name,
            mode: self.mode,
            timeouts: self.timeouts,
            binaries: self.binaries,
            superuser_username,
            superuser_dbname,
            etcd: self.etcd,
            nodes: self.nodes,
            api_clients: self.api_clients,
            tasks: self.tasks,
            timeline: self.timeline,
            artifact_root: self.artifact_root,
            etcd_links_by_node: self.etcd_links_by_node,
            etcd_proxies: self.etcd_proxies,
            api_proxies: self.api_proxies,
            pg_proxies: self.pg_proxies,
        })
    }
}

pub(crate) async fn start_cluster(config: TestConfig) -> Result<TestClusterHandle, WorkerError> {
    let mut config = config;
    config.validate()?;

    let namespace_guard = NamespaceGuard::new(config.test_name.as_str())?;
    let namespace_id = namespace_guard
        .namespace()?
        .id
        .clone();
    config.scope = format!("{}-{}", config.scope, namespace_id);
    config.cluster_name = format!("{}-{}", config.cluster_name, namespace_id);

    let binaries = require_pg16_process_binaries_for_real_tests()?;
    let etcd_bin = require_etcd_bin_for_real_tests()?;

    let mut guard = StartupGuard {
        guard: namespace_guard,
        scope: config.scope.clone(),
        cluster_name: config.cluster_name.clone(),
        mode: config.mode,
        binaries: binaries.clone(),
        superuser_username: None,
        superuser_dbname: None,
        etcd: None,
        nodes: Vec::new(),
        api_clients: Vec::new(),
        tasks: Vec::new(),
        timeline: Vec::new(),
        artifact_root: config.artifact_root.clone(),
        etcd_links_by_node: BTreeMap::new(),
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
    let mut topology_reservation = allocate_ha_topology_ports(config.node_count, etcd_member_count)?;
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
        startup_timeout: Duration::from_secs(15),
        members,
    };

    for port in topology
        .etcd_client_ports
        .iter()
        .chain(topology.etcd_peer_ports.iter())
    {
        topology_reservation
            .release_port(*port)
            .map_err(|err| WorkerError::Message(format!("release etcd reserved port failed: {err}")))?;
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

    let mut api_reservation =
        reserve_non_overlapping_ports(config.node_count, &forbidden_ports)?;
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

    let rewind_source_port = *node_ports.first().ok_or_else(|| {
        WorkerError::Message("missing postgres ports for cluster startup".to_string())
    })?;

    let mut cursor = 0usize;
    let mut proxy_reservation = PortReservation::empty();
    let (dcs_endpoints_by_node, proxy_ports) = match config.mode {
        Mode::Plain => (None, Vec::new()),
        Mode::PartitionProxy => {
            let total_proxy_ports = config
                .node_count
                .checked_mul(3)
                .ok_or_else(|| {
                    WorkerError::Message("proxy port count overflow for partition mode".to_string())
                })?;
            proxy_reservation =
                reserve_non_overlapping_ports(total_proxy_ports, &forbidden_ports)?;
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

    let http_timeout_ms = http_timeout_ms(config.timeouts.http_step_timeout)?;

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
            (Mode::PartitionProxy, Some(map)) => map.get(node_id.as_str()).cloned().ok_or_else(|| {
                WorkerError::Message(format!(
                    "missing proxy DCS endpoints for node runtime config: {node_id}"
                ))
            })?,
            (Mode::PartitionProxy, None) => {
                return Err(WorkerError::Message(
                    "partition mode missing DCS endpoints map".to_string(),
                ));
            }
        };

        let api_client = CliApiClient::new(
            format!("http://{api_observe_addr}"),
            http_timeout_ms,
            None,
            None,
        )
        .map_err(|err| {
            WorkerError::Message(format!(
                "build CliApiClient failed for startup node {node_id}: {err}"
            ))
        })?;

        let runtime_cfg = RuntimeConfig {
            cluster: ClusterConfig {
                name: config.cluster_name.clone(),
                member_id: node_id.clone(),
            },
            postgres: PostgresConfig {
                data_dir: data_dir.clone(),
                connect_timeout_s: 2,
                listen_host: "127.0.0.1".to_string(),
                listen_port: pg_port,
                socket_dir,
                log_file: log_file.clone(),
                rewind_source_host: "127.0.0.1".to_string(),
                rewind_source_port,
                local_conn_identity: PostgresConnIdentityConfig {
                    user: "postgres".to_string(),
                    dbname: "postgres".to_string(),
                    ssl_mode: PgSslMode::Prefer,
                },
                rewind_conn_identity: PostgresConnIdentityConfig {
                    user: "rewinder".to_string(),
                    dbname: "postgres".to_string(),
                    ssl_mode: PgSslMode::Prefer,
                },
                tls: TlsServerConfig {
                    mode: ApiTlsMode::Disabled,
                    identity: None,
                    client_auth: None,
                },
                roles: PostgresRolesConfig {
                    superuser: PostgresRoleConfig {
                        username: "postgres".to_string(),
                        auth: RoleAuthConfig::Tls,
                    },
                    replicator: PostgresRoleConfig {
                        username: "replicator".to_string(),
                        auth: RoleAuthConfig::Tls,
                    },
                    rewinder: PostgresRoleConfig {
                        username: "rewinder".to_string(),
                        auth: RoleAuthConfig::Tls,
                    },
                },
                pg_hba: PgHbaConfig {
                    source: InlineOrPath::Inline {
                        content: String::new(),
                    },
                },
                pg_ident: PgIdentConfig {
                    source: InlineOrPath::Inline {
                        content: String::new(),
                    },
                },
            },
            dcs: DcsConfig {
                endpoints: dcs_endpoints,
                scope: config.scope.clone(),
                init: None,
            },
            ha: HaConfig {
                loop_interval_ms: 100,
                lease_ttl_ms: 2_000,
            },
            process: ProcessConfig {
                pg_rewind_timeout_ms: 5_000,
                bootstrap_timeout_ms: 30_000,
                fencing_timeout_ms: 5_000,
                binaries: binaries.clone(),
            },
            logging: LoggingConfig {
                level: LogLevel::Info,
                capture_subprocess_output: false,
                postgres: PostgresLoggingConfig {
                    enabled: false,
                    pg_ctl_log_file: None,
                    log_dir: None,
                    archive_command_log_file: None,
                    poll_interval_ms: 200,
                    cleanup: LogCleanupConfig {
                        enabled: true,
                        max_files: 50,
                        max_age_seconds: 7 * 24 * 60 * 60,
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
            },
            api: ApiConfig {
                listen_addr: api_addr.to_string(),
                security: ApiSecurityConfig {
                    tls: TlsServerConfig {
                        mode: ApiTlsMode::Disabled,
                        identity: None,
                        client_auth: None,
                    },
                    auth: ApiAuthConfig::Disabled,
                },
            },
            debug: DebugConfig { enabled: false },
        };

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

        topology_reservation
            .release_port(pg_port)
            .map_err(|err| {
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
            log_file: log_file.clone(),
        });
        guard.api_clients.push(api_client);

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
        guard
            .etcd_links_by_node
            .entry(node_id.clone())
            .or_default()
            .push(link_name);
        dcs_endpoints_by_node.insert(node_id, vec![proxy_url]);
    }

    Ok(dcs_endpoints_by_node)
}
