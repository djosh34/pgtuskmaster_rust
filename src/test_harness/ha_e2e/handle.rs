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
pub(crate) struct NodeHandle {
    pub(crate) id: String,
    pub(crate) pg_port: u16,
    pub(crate) sql_port: u16,
    pub(crate) api_addr: SocketAddr,
    pub(crate) api_observe_addr: SocketAddr,
    pub(crate) data_dir: PathBuf,
}

pub(crate) struct TestClusterHandle {
    pub(crate) guard: NamespaceGuard,
    pub(crate) timeouts: TimeoutConfig,
    pub(crate) binaries: BinaryPaths,
    pub(crate) superuser_username: String,
    pub(crate) superuser_dbname: String,
    pub(crate) etcd: Option<EtcdClusterHandle>,
    pub(crate) nodes: Vec<NodeHandle>,
    pub(crate) tasks: Vec<JoinHandle<Result<(), WorkerError>>>,
    pub(crate) etcd_proxies: BTreeMap<String, TcpProxyLink>,
    pub(crate) api_proxies: BTreeMap<String, TcpProxyLink>,
    pub(crate) pg_proxies: BTreeMap<String, TcpProxyLink>,
}
