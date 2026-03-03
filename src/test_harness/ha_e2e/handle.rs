use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use tokio::task::JoinHandle;

use crate::cli::client::CliApiClient;
use crate::config::BinaryPaths;
use crate::state::WorkerError;
use crate::test_harness::etcd3::EtcdClusterHandle;
use crate::test_harness::namespace::NamespaceGuard;
use crate::test_harness::net_proxy::TcpProxyLink;

use super::config::{Mode, TimeoutConfig};

#[derive(Clone, Debug)]
pub(crate) struct NodeHandle {
    pub(crate) id: String,
    pub(crate) pg_port: u16,
    pub(crate) sql_port: u16,
    pub(crate) api_addr: SocketAddr,
    pub(crate) api_observe_addr: SocketAddr,
    pub(crate) data_dir: PathBuf,
    pub(crate) log_file: PathBuf,
}

impl NodeHandle {
    pub(crate) fn log_file(&self) -> &Path {
        &self.log_file
    }
}

pub(crate) struct TestClusterHandle {
    pub(crate) guard: NamespaceGuard,
    pub(crate) scope: String,
    pub(crate) cluster_name: String,
    pub(crate) mode: Mode,
    pub(crate) timeouts: TimeoutConfig,
    pub(crate) binaries: BinaryPaths,
    pub(crate) etcd: Option<EtcdClusterHandle>,
    pub(crate) nodes: Vec<NodeHandle>,
    pub(crate) api_clients: Vec<CliApiClient>,
    pub(crate) tasks: Vec<JoinHandle<Result<(), WorkerError>>>,
    pub(crate) timeline: Vec<String>,
    pub(crate) artifact_root: Option<PathBuf>,
    pub(crate) etcd_links_by_node: BTreeMap<String, Vec<String>>,
    pub(crate) etcd_proxies: BTreeMap<String, TcpProxyLink>,
    pub(crate) api_proxies: BTreeMap<String, TcpProxyLink>,
    pub(crate) pg_proxies: BTreeMap<String, TcpProxyLink>,
}

