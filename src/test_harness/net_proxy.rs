use std::{
    collections::BTreeMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread,
};

use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{oneshot, watch, Mutex},
    task::AbortHandle,
    time::Duration,
};

use super::HarnessError;

const PROXY_READ_BUFFER_SIZE: usize = 16 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ProxyMode {
    PassThrough,
    Blocked,
    Latency {
        upstream_delay_ms: u64,
        downstream_delay_ms: u64,
    },
}

impl ProxyMode {
    fn delay_for_direction(&self, direction: Direction) -> Duration {
        match self {
            Self::PassThrough | Self::Blocked => Duration::from_millis(0),
            Self::Latency {
                upstream_delay_ms,
                downstream_delay_ms,
            } => match direction {
                Direction::Upstream => Duration::from_millis(*upstream_delay_ms),
                Direction::Downstream => Duration::from_millis(*downstream_delay_ms),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ProxyLinkSpec {
    pub(crate) name: String,
    pub(crate) listen_addr: SocketAddr,
    pub(crate) target_addr: SocketAddr,
}

pub(crate) struct TcpProxyLink {
    name: String,
    listen_addr: SocketAddr,
    target_addr: SocketAddr,
    mode_tx: watch::Sender<ProxyMode>,
    shutdown_tx: watch::Sender<bool>,
    active: Arc<Mutex<BTreeMap<u64, AbortHandle>>>,
    join: thread::JoinHandle<Result<(), HarnessError>>,
}

impl TcpProxyLink {
    pub(crate) async fn spawn(spec: ProxyLinkSpec) -> Result<Self, HarnessError> {
        if spec.name.trim().is_empty() {
            return Err(HarnessError::InvalidInput(
                "proxy link name must not be empty".to_string(),
            ));
        }

        let (mode_tx, mode_rx) = watch::channel(ProxyMode::PassThrough);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let active = Arc::new(Mutex::new(BTreeMap::<u64, AbortHandle>::new()));
        let conn_seq = Arc::new(AtomicU64::new(1));
        let active_for_task = Arc::clone(&active);
        let conn_seq_for_task = Arc::clone(&conn_seq);
        let target_addr = spec.target_addr;
        let listen_addr = spec.listen_addr;
        let (startup_tx, startup_rx) = oneshot::channel::<Result<SocketAddr, HarnessError>>();

        let join = thread::Builder::new()
            .name(format!("tcp-proxy-{}", spec.name))
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|err| {
                        HarnessError::InvalidInput(format!(
                            "build tokio runtime for proxy listener failed: {err}"
                        ))
                    })?;

                runtime.block_on(async move {
                    let listener = TcpListener::bind(listen_addr).await.map_err(HarnessError::Io)?;
                    let bound_addr = listener.local_addr().map_err(HarnessError::Io)?;
                    let _ = startup_tx.send(Ok(bound_addr));
                    run_listener(
                        listener,
                        target_addr,
                        mode_rx,
                        shutdown_rx,
                        active_for_task,
                        conn_seq_for_task,
                    )
                    .await
                })
            })
            .map_err(|err| {
                HarnessError::InvalidInput(format!("spawn proxy listener thread failed: {err}"))
            })?;

        let listen_addr = startup_rx.await.map_err(|err| {
            HarnessError::InvalidInput(format!("proxy listener startup channel failed: {err}"))
        })??;

        Ok(Self {
            name: spec.name,
            listen_addr,
            target_addr: spec.target_addr,
            mode_tx,
            shutdown_tx,
            active,
            join,
        })
    }

    pub(crate) async fn spawn_with_listener(
        name: String,
        std_listener: std::net::TcpListener,
        target_addr: SocketAddr,
    ) -> Result<Self, HarnessError> {
        if name.trim().is_empty() {
            return Err(HarnessError::InvalidInput(
                "proxy link name must not be empty".to_string(),
            ));
        }

        std_listener
            .set_nonblocking(true)
            .map_err(HarnessError::Io)?;

        let (mode_tx, mode_rx) = watch::channel(ProxyMode::PassThrough);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let active = Arc::new(Mutex::new(BTreeMap::<u64, AbortHandle>::new()));
        let conn_seq = Arc::new(AtomicU64::new(1));
        let active_for_task = Arc::clone(&active);
        let conn_seq_for_task = Arc::clone(&conn_seq);
        let (startup_tx, startup_rx) = oneshot::channel::<Result<SocketAddr, HarnessError>>();

        let name_for_thread = name.clone();
        let join = thread::Builder::new()
            .name(format!("tcp-proxy-{name_for_thread}"))
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|err| {
                        HarnessError::InvalidInput(format!(
                            "build tokio runtime for proxy listener failed: {err}"
                        ))
                    })?;

                runtime.block_on(async move {
                    let listener = TcpListener::from_std(std_listener).map_err(HarnessError::Io)?;
                    let bound_addr = listener.local_addr().map_err(HarnessError::Io)?;
                    let _ = startup_tx.send(Ok(bound_addr));
                    run_listener(
                        listener,
                        target_addr,
                        mode_rx,
                        shutdown_rx,
                        active_for_task,
                        conn_seq_for_task,
                    )
                    .await
                })
            })
            .map_err(|err| {
                HarnessError::InvalidInput(format!("spawn proxy listener thread failed: {err}"))
            })?;

        let listen_addr = startup_rx.await.map_err(|err| {
            HarnessError::InvalidInput(format!("proxy listener startup channel failed: {err}"))
        })??;

        Ok(Self {
            name,
            listen_addr,
            target_addr,
            mode_tx,
            shutdown_tx,
            active,
            join,
        })
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn listen_addr(&self) -> SocketAddr {
        self.listen_addr
    }

    pub(crate) fn target_addr(&self) -> SocketAddr {
        self.target_addr
    }

    pub(crate) async fn set_mode(&self, mode: ProxyMode) -> Result<(), HarnessError> {
        let should_abort = mode == ProxyMode::Blocked;
        self.mode_tx.send(mode).map_err(|err| {
            HarnessError::InvalidInput(format!("proxy mode channel closed: {err}"))
        })?;
        if should_abort {
            abort_all_active(&self.active).await;
        }
        Ok(())
    }

    pub(crate) async fn shutdown(self) -> Result<(), HarnessError> {
        self.shutdown_tx.send(true).map_err(|err| {
            HarnessError::InvalidInput(format!("proxy shutdown channel closed: {err}"))
        })?;
        abort_all_active(&self.active).await;
        let join = self.join;
        let joined = tokio::task::spawn_blocking(move || join.join())
            .await
            .map_err(|err| {
                HarnessError::InvalidInput(format!(
                    "proxy listener join blocking task failed: {err}"
                ))
            })?;
        joined
            .map_err(|_| HarnessError::InvalidInput("proxy listener thread panicked".to_string()))?
    }
}

async fn run_listener(
    listener: TcpListener,
    target_addr: SocketAddr,
    mut mode_rx: watch::Receiver<ProxyMode>,
    mut shutdown_rx: watch::Receiver<bool>,
    active: Arc<Mutex<BTreeMap<u64, AbortHandle>>>,
    conn_seq: Arc<AtomicU64>,
) -> Result<(), HarnessError> {
    loop {
        tokio::select! {
            changed = shutdown_rx.changed() => {
                match changed {
                    Ok(()) => {
                        if *shutdown_rx.borrow() {
                            abort_all_active(&active).await;
                            break;
                        }
                    }
                    Err(_) => {
                        abort_all_active(&active).await;
                        break;
                    }
                }
            }
            changed = mode_rx.changed() => {
                match changed {
                    Ok(()) => {
                        if *mode_rx.borrow() == ProxyMode::Blocked {
                            abort_all_active(&active).await;
                        }
                    }
                    Err(_) => {
                        abort_all_active(&active).await;
                        break;
                    }
                }
            }
            accept_result = listener.accept() => {
                let (downstream, _) = accept_result?;
                if *mode_rx.borrow() == ProxyMode::Blocked {
                    drop(downstream);
                    continue;
                }
                let conn_id = conn_seq.fetch_add(1, Ordering::SeqCst);
                let mode_for_connection = mode_rx.clone();
                let active_for_connection = Arc::clone(&active);
                let join = tokio::spawn(async move {
                    let result = proxy_connection(downstream, target_addr, mode_for_connection).await;
                    let mut guard = active_for_connection.lock().await;
                    let _ = guard.remove(&conn_id);
                    result
                });
                let abort_handle = join.abort_handle();
                let mut guard = active.lock().await;
                guard.insert(conn_id, abort_handle);
            }
        }
    }

    Ok(())
}

async fn abort_all_active(active: &Arc<Mutex<BTreeMap<u64, AbortHandle>>>) {
    let mut guard = active.lock().await;
    for (_, abort_handle) in guard.iter() {
        abort_handle.abort();
    }
    guard.clear();
}

async fn proxy_connection(
    downstream: TcpStream,
    target_addr: SocketAddr,
    mode_rx: watch::Receiver<ProxyMode>,
) -> Result<(), HarnessError> {
    let upstream = TcpStream::connect(target_addr).await?;

    let (downstream_read, downstream_write) = downstream.into_split();
    let (upstream_read, upstream_write) = upstream.into_split();

    let upstream_task = tokio::spawn(pump_direction(
        downstream_read,
        upstream_write,
        mode_rx.clone(),
        Direction::Upstream,
    ));
    let downstream_task = tokio::spawn(pump_direction(
        upstream_read,
        downstream_write,
        mode_rx,
        Direction::Downstream,
    ));

    let upstream_result = upstream_task
        .await
        .map_err(|err| HarnessError::InvalidInput(format!("upstream pump join failed: {err}")))?;
    let downstream_result = downstream_task
        .await
        .map_err(|err| HarnessError::InvalidInput(format!("downstream pump join failed: {err}")))?;

    upstream_result?;
    downstream_result?;
    Ok(())
}

#[derive(Clone, Copy)]
enum Direction {
    Upstream,
    Downstream,
}

async fn pump_direction<R, W>(
    mut reader: R,
    mut writer: W,
    mode_rx: watch::Receiver<ProxyMode>,
    direction: Direction,
) -> Result<(), HarnessError>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buf = vec![0_u8; PROXY_READ_BUFFER_SIZE];
    loop {
        let read_bytes = reader.read(&mut buf).await?;
        if read_bytes == 0 {
            writer.shutdown().await?;
            return Ok(());
        }

        let mode = mode_rx.borrow().clone();
        if mode == ProxyMode::Blocked {
            writer.shutdown().await?;
            return Ok(());
        }

        let delay = mode.delay_for_direction(direction);
        if !delay.is_zero() {
            tokio::time::sleep(delay).await;
        }

        writer.write_all(&buf[..read_bytes]).await?;
    }
}

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        time::{Duration, Instant},
    };

    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::{TcpListener, TcpStream},
        sync::watch,
        task::JoinHandle,
    };

    use super::{ProxyLinkSpec, ProxyMode, TcpProxyLink};
    use crate::test_harness::HarnessError;
    use crate::test_harness::{
        binaries::require_etcd_bin_for_real_tests,
        etcd3::{
            prepare_etcd_member_data_dir, spawn_etcd3_cluster, EtcdClusterMemberSpec,
            EtcdClusterSpec,
        },
        namespace::NamespaceGuard,
    };

    struct EchoServer {
        addr: std::net::SocketAddr,
        shutdown_tx: watch::Sender<bool>,
        join: JoinHandle<Result<(), HarnessError>>,
    }

    impl EchoServer {
        async fn spawn() -> Result<Self, HarnessError> {
            let listener = TcpListener::bind("127.0.0.1:0").await?;
            let addr = listener.local_addr()?;
            let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

            let join = tokio::spawn(async move {
                loop {
                    tokio::select! {
                        changed = shutdown_rx.changed() => {
                            match changed {
                                Ok(()) => {
                                    if *shutdown_rx.borrow() {
                                        break;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                        accept_result = listener.accept() => {
                            let (mut stream, _) = accept_result?;
                            tokio::spawn(async move {
                                let mut buf = vec![0_u8; 4096];
                                loop {
                                    let read_bytes = match stream.read(&mut buf).await {
                                        Ok(read_bytes) => read_bytes,
                                        Err(_) => return,
                                    };
                                    if read_bytes == 0 {
                                        return;
                                    }
                                    if stream.write_all(&buf[..read_bytes]).await.is_err() {
                                        return;
                                    }
                                }
                            });
                        }
                    }
                }
                Ok(())
            });

            Ok(Self {
                addr,
                shutdown_tx,
                join,
            })
        }

        async fn shutdown(self) -> Result<(), HarnessError> {
            self.shutdown_tx.send(true).map_err(|err| {
                HarnessError::InvalidInput(format!("echo shutdown channel closed: {err}"))
            })?;
            self.join
                .await
                .map_err(|err| HarnessError::InvalidInput(format!("echo join failed: {err}")))?
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn proxy_link_forwards_bytes_in_passthrough() -> Result<(), HarnessError> {
        let server = EchoServer::spawn().await?;
        let proxy = TcpProxyLink::spawn(ProxyLinkSpec {
            name: "passthrough".to_string(),
            listen_addr: "127.0.0.1:0".parse().map_err(|err| {
                HarnessError::InvalidInput(format!("parse listen addr failed: {err}"))
            })?,
            target_addr: server.addr,
        })
        .await?;

        let mut client = TcpStream::connect(proxy.listen_addr()).await?;
        client.write_all(b"hello").await?;
        client.shutdown().await?;

        let mut response = Vec::new();
        let _ = client.read_to_end(&mut response).await?;
        if response != b"hello" {
            return Err(HarnessError::InvalidInput(format!(
                "passthrough response mismatch: {response:?}"
            )));
        }

        proxy.shutdown().await?;
        server.shutdown().await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn blocking_mode_closes_existing_flows_and_new_accepts() -> Result<(), HarnessError> {
        let server = EchoServer::spawn().await?;
        let proxy = TcpProxyLink::spawn(ProxyLinkSpec {
            name: "blocked".to_string(),
            listen_addr: "127.0.0.1:0".parse().map_err(|err| {
                HarnessError::InvalidInput(format!("parse listen addr failed: {err}"))
            })?,
            target_addr: server.addr,
        })
        .await?;

        let mut client = TcpStream::connect(proxy.listen_addr()).await?;
        client.write_all(b"first").await?;

        proxy.set_mode(ProxyMode::Blocked).await?;

        let mut existing_buf = vec![0_u8; 8];
        let read_existing =
            tokio::time::timeout(Duration::from_secs(3), client.read(&mut existing_buf)).await;
        match read_existing {
            Ok(Ok(0)) => {}
            Ok(Ok(read_bytes)) => {
                if read_bytes != 0 {
                    return Err(HarnessError::InvalidInput(format!(
                        "expected blocked existing stream close, got read_bytes={read_bytes}"
                    )));
                }
            }
            Ok(Err(err)) => match err.kind() {
                std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::BrokenPipe
                | std::io::ErrorKind::NotConnected => {}
                _ => {
                    return Err(HarnessError::InvalidInput(format!(
                        "existing stream read failed while blocked: {err}"
                    )));
                }
            },
            Err(_) => {
                return Err(HarnessError::InvalidInput(
                    "timed out waiting for blocked existing stream close".to_string(),
                ));
            }
        }

        let mut new_client = TcpStream::connect(proxy.listen_addr()).await?;
        let write_result = new_client.write_all(b"second").await;
        if let Err(err) = write_result {
            let _ = proxy.shutdown().await;
            let _ = server.shutdown().await;
            return Err(HarnessError::InvalidInput(format!(
                "new stream write failed unexpectedly: {err}"
            )));
        }

        let mut blocked_buf = vec![0_u8; 8];
        let blocked_read =
            tokio::time::timeout(Duration::from_secs(3), new_client.read(&mut blocked_buf)).await;
        match blocked_read {
            Ok(Ok(0)) => {}
            Ok(Ok(read_bytes)) => {
                return Err(HarnessError::InvalidInput(format!(
                    "expected blocked new stream close (0 bytes), got {read_bytes}"
                )));
            }
            Ok(Err(err)) => match err.kind() {
                std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::BrokenPipe
                | std::io::ErrorKind::NotConnected => {}
                _ => {
                    return Err(HarnessError::InvalidInput(format!(
                        "new blocked stream read failed: {err}"
                    )));
                }
            },
            Err(_) => {
                return Err(HarnessError::InvalidInput(
                    "timed out waiting for blocked new stream close".to_string(),
                ));
            }
        }

        proxy.shutdown().await?;
        server.shutdown().await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn latency_mode_delays_forwarding() -> Result<(), HarnessError> {
        let server = EchoServer::spawn().await?;
        let proxy = TcpProxyLink::spawn(ProxyLinkSpec {
            name: "latency".to_string(),
            listen_addr: "127.0.0.1:0".parse().map_err(|err| {
                HarnessError::InvalidInput(format!("parse listen addr failed: {err}"))
            })?,
            target_addr: server.addr,
        })
        .await?;

        proxy
            .set_mode(ProxyMode::Latency {
                upstream_delay_ms: 150,
                downstream_delay_ms: 0,
            })
            .await?;

        let mut client = TcpStream::connect(proxy.listen_addr()).await?;
        let start = Instant::now();
        client.write_all(b"x").await?;
        client.shutdown().await?;
        let mut response = Vec::new();
        let _ = client.read_to_end(&mut response).await?;
        let elapsed = start.elapsed();
        if response != b"x" {
            return Err(HarnessError::InvalidInput(format!(
                "latency response mismatch: {response:?}"
            )));
        }
        if elapsed < Duration::from_millis(120) {
            return Err(HarnessError::InvalidInput(format!(
                "latency mode did not add expected delay: elapsed={elapsed:?}"
            )));
        }

        proxy.shutdown().await?;
        server.shutdown().await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn proxy_link_supports_etcd_client_traffic() -> Result<(), HarnessError> {
        let etcd_bin: PathBuf = require_etcd_bin_for_real_tests()?;
        let guard = NamespaceGuard::new("proxy-etcd-link")?;
        let namespace = guard.namespace()?;
        let member_name = "etcd-a";
        let data_dir = prepare_etcd_member_data_dir(namespace, member_name)?;
        let log_dir = namespace.child_dir(format!("logs/{member_name}"));
        let reservation = crate::test_harness::ports::allocate_ports(2)?;
        let ports = reservation.as_slice();
        let client_port = *ports.first().ok_or_else(|| {
            HarnessError::InvalidInput("missing etcd client port reservation".to_string())
        })?;
        let peer_port = *ports.get(1).ok_or_else(|| {
            HarnessError::InvalidInput("missing etcd peer port reservation".to_string())
        })?;
        drop(reservation);

        let mut cluster = spawn_etcd3_cluster(EtcdClusterSpec {
            etcd_bin,
            namespace_id: namespace.id.clone(),
            startup_timeout: Duration::from_secs(15),
            members: vec![EtcdClusterMemberSpec {
                member_name: member_name.to_string(),
                data_dir,
                log_dir,
                client_port,
                peer_port,
            }],
        })
        .await?;

        let direct_endpoint = cluster
            .client_endpoints()
            .first()
            .cloned()
            .ok_or_else(|| HarnessError::InvalidInput("missing etcd endpoint".to_string()))?;
        let target_addr: std::net::SocketAddr = direct_endpoint
            .strip_prefix("http://")
            .ok_or_else(|| {
                HarnessError::InvalidInput(format!("invalid etcd endpoint: {direct_endpoint}"))
            })?
            .parse()
            .map_err(|err| {
                HarnessError::InvalidInput(format!("parse etcd endpoint failed: {err}"))
            })?;

        let proxy = TcpProxyLink::spawn(ProxyLinkSpec {
            name: "etcd-proxy".to_string(),
            listen_addr: "127.0.0.1:0".parse().map_err(|err| {
                HarnessError::InvalidInput(format!("parse listen addr failed: {err}"))
            })?,
            target_addr,
        })
        .await?;

        let proxied_endpoint = format!("http://{}", proxy.listen_addr());
        let mut client = etcd_client::Client::connect(vec![proxied_endpoint.clone()], None)
            .await
            .map_err(|err| {
                HarnessError::InvalidInput(format!("connect etcd via proxy failed: {err}"))
            })?;
        client
            .put("/proxy-check/key", "value", None)
            .await
            .map_err(|err| {
                HarnessError::InvalidInput(format!("etcd put via proxy failed: {err}"))
            })?;
        let response = client.get("/proxy-check/key", None).await.map_err(|err| {
            HarnessError::InvalidInput(format!("etcd get via proxy failed: {err}"))
        })?;
        if response.count() != 1 {
            return Err(HarnessError::InvalidInput(format!(
                "unexpected etcd get count via proxy endpoint {proxied_endpoint}: {}",
                response.count()
            )));
        }

        proxy.shutdown().await?;
        cluster.shutdown_all().await?;
        Ok(())
    }
}
