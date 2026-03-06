use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::PathBuf;

use tokio::task::JoinHandle;

use crate::config::BinaryPaths;
use crate::state::WorkerError;
use crate::test_harness::etcd3::EtcdClusterHandle;
use crate::test_harness::namespace::NamespaceGuard;
use crate::test_harness::net_proxy::TcpProxyLink;

use super::config::TimeoutConfig;

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
    pub tasks: Vec<JoinHandle<Result<(), WorkerError>>>,
    pub etcd_proxies: BTreeMap<String, TcpProxyLink>,
    pub api_proxies: BTreeMap<String, TcpProxyLink>,
    pub pg_proxies: BTreeMap<String, TcpProxyLink>,
}
