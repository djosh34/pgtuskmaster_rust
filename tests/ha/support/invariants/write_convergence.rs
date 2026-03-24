use std::{
    collections::BTreeSet,
    fs,
    future::Future,
    io::Cursor,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex, MutexGuard,
    },
    thread::{self, JoinHandle as ThreadJoinHandle},
    time::Duration,
};

use rustls::{
    self,
    pki_types::{CertificateDer, PrivateKeyDer},
    RootCertStore,
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    runtime::Builder,
    task::JoinHandle,
    time::Instant,
};
use tokio_postgres::{Client, Connection, NoTls, Socket};
use tokio_postgres_rustls::MakeRustlsConnect;

use pgtuskmaster_rust::{
    api::authoritative_primary_member,
    pginfo::{
        conninfo::PgClientTls,
        state::{PgConnInfo, PgSslMode},
    },
};

use crate::support::{
    block_on_support_future,
    observer::pgtm::{PgtmObserver, PostgresRoutingTarget},
    topology::ClusterMember,
};

const FIXTURE_TABLE: &str = "public.write_convergence_invariant";
const FIXTURE_ROW_ID: i32 = 1;
const CREATE_FIXTURE_TABLE_SQL: &str = "
CREATE TABLE IF NOT EXISTS public.write_convergence_invariant (
    id integer PRIMARY KEY,
    written_count bigint NOT NULL
)";
const ENSURE_FIXTURE_ROW_SQL: &str = "
INSERT INTO public.write_convergence_invariant (id, written_count)
VALUES ($1, 0)
ON CONFLICT (id) DO NOTHING";
const INCREMENT_FIXTURE_ROW_SQL: &str = "
UPDATE public.write_convergence_invariant
SET written_count = written_count + 1
WHERE id = $1
RETURNING written_count";
const SELECT_FIXTURE_ROW_SQL: &str = "
SELECT written_count
FROM public.write_convergence_invariant
WHERE id = $1";

type ConnectionTask = JoinHandle<std::result::Result<(), tokio_postgres::Error>>;

pub struct WriteConvergenceInvariantRunner {
    observer: Option<PgtmObserver>,
    poll_interval: Duration,
    write_deadline: Duration,
    write_gate: Arc<WriteGate>,
    routing_targets: Vec<PostgresRoutingTarget>,
    stop_requested: Arc<AtomicBool>,
    worker: Mutex<Option<ThreadJoinHandle<Result<(), String>>>>,
}

#[derive(Debug, thiserror::Error)]
pub enum WriteConvergenceInvariantError {
    #[error("write-convergence invariant failed: {0}")]
    Failed(String),
}

#[derive(Default)]
struct WriteGate {
    state: Mutex<WriteGateState>,
    drained: Condvar,
}

#[derive(Default)]
struct WriteGateState {
    accepted_count: Option<u64>,
    last_error: Option<String>,
    closed: bool,
    in_flight: usize,
}

struct WritePermit {
    gate: Arc<WriteGate>,
}

impl WriteConvergenceInvariantRunner {
    pub async fn start(
        observer: PgtmObserver,
        poll_interval: Duration,
        write_deadline: Duration,
    ) -> Result<Self, WriteConvergenceInvariantError> {
        let routing_targets = ClusterMember::ALL
            .into_iter()
            .map(|member| observer.postgres_routing_target(member))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| {
                WriteConvergenceInvariantError::Failed(format!(
                    "resolve postgres routing targets failed: {err}"
                ))
            })?;

        let write_gate = Arc::new(WriteGate::new());
        let stop_requested = Arc::new(AtomicBool::new(false));
        let worker = Some(spawn_authoritative_worker(
            observer.clone(),
            Arc::clone(&write_gate),
            Arc::clone(&stop_requested),
            poll_interval,
        )?);
        Ok(Self::new(
            Some(observer),
            poll_interval,
            write_deadline,
            write_gate,
            routing_targets,
            stop_requested,
            worker,
        ))
    }

    pub fn ensure_healthy(
        &self,
        online_members: &[ClusterMember],
    ) -> Result<(), WriteConvergenceInvariantError> {
        let members = self.selected_routing_targets(online_members)?;
        self.ensure_running()?;
        if members.is_empty() {
            return Ok(());
        }
        let write_deadline = self.write_deadline;
        let poll_interval = self.poll_interval;
        let observer = self.observer.clone();
        let write_gate = Arc::clone(&self.write_gate);
        let future = async move {
            let (expected_count, last_write_error) = convergence_expectation(
                write_gate.as_ref(),
                members.as_slice(),
                observer.as_ref(),
                poll_interval,
            )
            .await?;
            wait_for_convergence(
                members.as_slice(),
                observer.as_ref(),
                poll_interval,
                write_deadline,
                expected_count,
                last_write_error.as_deref(),
            )
            .await
        };

        block_on_support_future(
            future,
            "build runtime for write convergence invariant failed",
            "write convergence invariant health check thread panicked",
        )
        .map_err(WriteConvergenceInvariantError::Failed)
    }

    pub fn ensure_running(&self) -> Result<(), WriteConvergenceInvariantError> {
        self.write_gate.close_and_drain();
        self.stop_worker()
    }
}

pub fn probe_routing_target_connectivity(
    routing_target: &PostgresRoutingTarget,
    connect_timeout: Duration,
) -> Result<(), WriteConvergenceInvariantError> {
    let routing_target = routing_target.clone();
    let future = async move {
        let (_client, connection_task) = connect_member(&routing_target, connect_timeout).await?;
        connection_task.abort();
        Ok::<(), String>(())
    };

    block_on_support_future(
        future,
        "build runtime for write convergence probe failed",
        "write convergence probe thread panicked",
    )
    .map_err(WriteConvergenceInvariantError::Failed)
}

impl std::fmt::Debug for WriteConvergenceInvariantRunner {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WriteConvergenceInvariantRunner")
            .field("poll_interval", &self.poll_interval)
            .field("write_deadline", &self.write_deadline)
            .field("routing_targets", &self.routing_targets)
            .finish()
    }
}

impl Drop for WriteConvergenceInvariantRunner {
    fn drop(&mut self) {
        if let Err(err) = self.stop_worker() {
            assert!(
                thread::panicking(),
                "write convergence invariant cleanup failed: {err}"
            );
        }
    }
}

impl WriteConvergenceInvariantRunner {
    fn new(
        observer: Option<PgtmObserver>,
        poll_interval: Duration,
        write_deadline: Duration,
        write_gate: Arc<WriteGate>,
        routing_targets: Vec<PostgresRoutingTarget>,
        stop_requested: Arc<AtomicBool>,
        worker: Option<ThreadJoinHandle<Result<(), String>>>,
    ) -> Self {
        Self {
            observer,
            poll_interval,
            write_deadline,
            write_gate,
            routing_targets,
            stop_requested,
            worker: Mutex::new(worker),
        }
    }

    fn stop_worker(&self) -> Result<(), WriteConvergenceInvariantError> {
        self.stop_requested.store(true, Ordering::SeqCst);
        let worker = {
            let mut worker = self.worker.lock().map_err(|_| {
                WriteConvergenceInvariantError::Failed(
                    "write convergence worker mutex was poisoned".to_string(),
                )
            })?;
            worker.take()
        };
        worker.map_or(Ok(()), |worker| {
            worker
                .join()
                .map_err(|_| {
                    WriteConvergenceInvariantError::Failed(
                        "write convergence worker thread panicked".to_string(),
                    )
                })?
                .map_err(WriteConvergenceInvariantError::Failed)
        })
    }

    fn selected_routing_targets(
        &self,
        online_members: &[ClusterMember],
    ) -> Result<Vec<PostgresRoutingTarget>, WriteConvergenceInvariantError> {
        let selected_members = online_members.iter().copied().collect::<BTreeSet<_>>();
        let routing_targets = self
            .routing_targets
            .iter()
            .filter(|target| selected_members.contains(&target.member))
            .cloned()
            .collect::<Vec<_>>();
        if routing_targets.len() == selected_members.len() {
            return Ok(routing_targets);
        }

        let missing_members = selected_members
            .into_iter()
            .filter(|member| {
                !routing_targets
                    .iter()
                    .any(|target| target.member == *member)
            })
            .map(|member| format!("`{member}`"))
            .collect::<Vec<_>>()
            .join(", ");
        Err(WriteConvergenceInvariantError::Failed(format!(
            "write convergence invariant is missing routing targets for selected members: {missing_members}"
        )))
    }
}

impl WriteGate {
    fn new() -> Self {
        Self::default()
    }

    fn try_start_write(self: &Arc<Self>) -> Option<WritePermit> {
        let mut state = self.lock_state();
        if state.closed {
            return None;
        }
        state.in_flight += 1;
        Some(WritePermit {
            gate: Arc::clone(self),
        })
    }

    fn close_and_drain(&self) {
        let mut state = self.lock_state();
        state.closed = true;
        while state.in_flight > 0 {
            state = self.wait_for_drain(state);
        }
    }

    fn accepted_count(&self) -> Option<u64> {
        self.lock_state().accepted_count
    }

    fn record_accepted_count(&self, count: u64) {
        let mut state = self.lock_state();
        state.accepted_count = Some(
            state
                .accepted_count
                .map_or(count, |current| current.max(count)),
        );
        state.last_error = None;
    }

    fn record_last_error(&self, message: String) {
        self.lock_state().last_error = Some(message);
    }

    fn clear_last_error(&self) {
        self.lock_state().last_error = None;
    }

    fn last_error(&self) -> Option<String> {
        self.lock_state().last_error.clone()
    }

    fn lock_state(&self) -> MutexGuard<'_, WriteGateState> {
        match self.state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn wait_for_drain<'guard>(
        &self,
        state: MutexGuard<'guard, WriteGateState>,
    ) -> MutexGuard<'guard, WriteGateState> {
        match self.drained.wait(state) {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

impl Drop for WritePermit {
    fn drop(&mut self) {
        let mut state = self.gate.lock_state();
        state.in_flight -= 1;
        if state.in_flight == 0 {
            self.gate.drained.notify_all();
        }
    }
}

async fn connect_member(
    routing_target: &PostgresRoutingTarget,
    connect_timeout: Duration,
) -> Result<(Arc<Client>, ConnectionTask), String> {
    let connect_dsn = connectable_conninfo(&routing_target.conninfo).to_string();
    if conninfo_uses_tls_files(&routing_target.conninfo) {
        let tls = build_tls_connector(&routing_target.conninfo).map_err(|err| err.to_string())?;
        return connect_and_probe_member(
            &routing_target.member,
            connect_timeout,
            tokio_postgres::connect(connect_dsn.as_str(), tls),
        )
        .await;
    }

    connect_and_probe_member(
        &routing_target.member,
        connect_timeout,
        tokio_postgres::connect(connect_dsn.as_str(), NoTls),
    )
    .await
}

async fn connect_and_probe_member<S, F>(
    member: &ClusterMember,
    connect_timeout: Duration,
    connect_future: F,
) -> Result<(Arc<Client>, ConnectionTask), String>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    F: Future<Output = std::result::Result<(Client, Connection<Socket, S>), tokio_postgres::Error>>,
{
    let (client, connection) = tokio::time::timeout(connect_timeout, connect_future)
        .await
        .map_err(|_| {
            format!(
                "connect to `{}` timed out after {:?}",
                member, connect_timeout
            )
        })?
        .map_err(|err| format!("connect to `{}` failed: {err}", member))?;
    let client: Arc<Client> = Arc::new(client);
    let connection_task = tokio::spawn(connection);
    match tokio::time::timeout(connect_timeout, client.simple_query("SELECT 1")).await {
        Ok(Ok(_)) => {}
        Ok(Err(err)) => {
            connection_task.abort();
            return Err(format!("connect to `{}` failed: {err}", member));
        }
        Err(_) => {
            connection_task.abort();
            return Err(format!(
                "connect to `{}` probe timed out after {:?}",
                member, connect_timeout
            ));
        }
    }
    Ok((client, connection_task))
}

async fn apply_fixture_row_setup(
    client: &Client,
) -> std::result::Result<(), tokio_postgres::Error> {
    client.batch_execute(CREATE_FIXTURE_TABLE_SQL).await?;
    client
        .execute(ENSURE_FIXTURE_ROW_SQL, &[&FIXTURE_ROW_ID])
        .await
        .map(|_| ())
}

fn spawn_authoritative_worker(
    observer: PgtmObserver,
    write_gate: Arc<WriteGate>,
    stop_requested: Arc<AtomicBool>,
    poll_interval: Duration,
) -> Result<ThreadJoinHandle<Result<(), String>>, WriteConvergenceInvariantError> {
    let thread = thread::Builder::new()
        .name("write-convergence-authoritative".to_string())
        .spawn(move || {
            Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|err| format!("build runtime for write convergence worker failed: {err}"))?
                .block_on(run_authoritative_worker(
                    observer,
                    write_gate,
                    stop_requested,
                    poll_interval,
                ))
        })
        .map_err(|err| {
            WriteConvergenceInvariantError::Failed(format!(
                "spawn authoritative write worker thread failed: {err}",
            ))
        })?;
    Ok(thread)
}

async fn run_authoritative_worker(
    observer: PgtmObserver,
    write_gate: Arc<WriteGate>,
    stop_requested: Arc<AtomicBool>,
    poll_interval: Duration,
) -> Result<(), String> {
    loop {
        if stop_requested.load(Ordering::SeqCst) {
            return Ok(());
        }
        let write_permit = match write_gate.try_start_write() {
            Some(write_permit) => write_permit,
            None => return Ok(()),
        };
        let write_result =
            attempt_authoritative_write(observer.clone(), Arc::clone(&write_gate), poll_interval)
                .await;
        drop(write_permit);
        match write_result {
            Ok(()) => write_gate.clear_last_error(),
            Err(err) => write_gate.record_last_error(err),
        }
        if stop_requested.load(Ordering::SeqCst) {
            return Ok(());
        }
        tokio::time::sleep(poll_interval).await;
    }
}

async fn attempt_authoritative_write(
    observer: PgtmObserver,
    write_gate: Arc<WriteGate>,
    query_timeout: Duration,
) -> Result<(), String> {
    let routing_target = match authoritative_primary_routing_target(&observer)? {
        Some(routing_target) => routing_target,
        None => return Ok(()),
    };
    let (client, connection_task) = connect_member(&routing_target, query_timeout).await?;
    let write_result =
        perform_authoritative_write(client.as_ref(), write_gate.as_ref(), query_timeout)
            .await
            .map_err(|err| {
                format!(
                    "authoritative write via `{}` failed: {err}",
                    routing_target.member
                )
            });
    connection_task.abort();
    write_result
}

fn authoritative_primary_routing_target(
    observer: &PgtmObserver,
) -> Result<Option<PostgresRoutingTarget>, String> {
    let authoritative_members = observer
        .observe_states()
        .map_err(|err| format!("observe authoritative primary failed: {err}"))?
        .into_values()
        .filter_map(|state| state.ok())
        .filter_map(|state| {
            authoritative_primary_member(&state)
                .and_then(|member_id| ClusterMember::parse(member_id.as_str()).ok())
        })
        .collect::<BTreeSet<_>>();
    if authoritative_members.len() != 1 {
        return Ok(None);
    }
    let member = authoritative_members
        .iter()
        .copied()
        .next()
        .ok_or_else(|| "authoritative primary disappeared unexpectedly".to_string())?;
    observer
        .postgres_routing_target(member)
        .map(Some)
        .map_err(|err| format!("resolve authoritative primary routing target failed: {err}"))
}

async fn perform_authoritative_write(
    client: &Client,
    write_gate: &WriteGate,
    query_timeout: Duration,
) -> Result<(), WriteConvergenceInvariantError> {
    tokio::time::timeout(query_timeout, apply_fixture_row_setup(client))
        .await
        .map_err(|_| {
            WriteConvergenceInvariantError::Failed(format!(
                "prepare fixture row timed out after {:?}",
                query_timeout
            ))
        })?
        .map_err(|err| {
            WriteConvergenceInvariantError::Failed(format!("prepare fixture row failed: {err}"))
        })?;
    if write_gate.accepted_count().is_none() {
        write_gate.record_accepted_count(read_count(client, query_timeout).await?);
    }
    write_gate.record_accepted_count(increment_fixture_row(client, query_timeout).await?);
    Ok(())
}

async fn increment_fixture_row(
    client: &Client,
    query_timeout: Duration,
) -> Result<u64, WriteConvergenceInvariantError> {
    let row = tokio::time::timeout(
        query_timeout,
        client.query_one(INCREMENT_FIXTURE_ROW_SQL, &[&FIXTURE_ROW_ID]),
    )
    .await
    .map_err(|_| {
        WriteConvergenceInvariantError::Failed(format!(
            "increment fixture row timed out after {:?}",
            query_timeout
        ))
    })?
    .map_err(|err| {
        WriteConvergenceInvariantError::Failed(format!("increment fixture row failed: {err}"))
    })?;
    u64::try_from(row.get::<_, i64>(0)).map_err(|err| {
        WriteConvergenceInvariantError::Failed(format!("fixture count was negative: {err}"))
    })
}

async fn read_member_count_via_fresh_connection(
    member: &PostgresRoutingTarget,
    observer: Option<&PgtmObserver>,
    query_timeout: Duration,
) -> MemberCountObservation {
    let routing_target = match observer.map_or_else(
        || Ok(member.clone()),
        |observer| {
            observer
                .postgres_routing_target(member.member)
                .map_err(|err| err.to_string())
        },
    ) {
        Ok(routing_target) => routing_target,
        Err(err) => {
            return MemberCountObservation::Failed {
                member: member.member,
                message: err,
            };
        }
    };
    match read_count_via_fresh_connection_target(&routing_target, query_timeout).await {
        Ok(count) => MemberCountObservation::Observed {
            member: member.member,
            count,
        },
        Err(err) => MemberCountObservation::Failed {
            member: member.member,
            message: err,
        },
    }
}

async fn read_count_via_fresh_connection_target(
    routing_target: &PostgresRoutingTarget,
    query_timeout: Duration,
) -> Result<u64, String> {
    match connect_member(routing_target, query_timeout).await {
        Ok((client, connection_task)) => {
            let read_result = read_count(client.as_ref(), query_timeout)
                .await
                .map_err(|err| err.to_string());
            connection_task.abort();
            read_result
        }
        Err(err) => Err(err),
    }
}

async fn wait_for_convergence(
    members: &[PostgresRoutingTarget],
    observer: Option<&PgtmObserver>,
    poll_interval: Duration,
    write_deadline: Duration,
    expected_count: Option<u64>,
    last_write_error: Option<&str>,
) -> Result<(), WriteConvergenceInvariantError> {
    let deadline = Instant::now() + write_deadline;
    loop {
        let observations =
            futures::future::join_all(members.iter().map(|member| {
                read_member_count_via_fresh_connection(member, observer, poll_interval)
            }))
            .await;
        if observations_are_converged(observations.as_slice(), expected_count) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(convergence_failure(
                observations.as_slice(),
                write_deadline,
                expected_count,
                last_write_error,
            ));
        }
        tokio::time::sleep(poll_interval).await;
    }
}

async fn convergence_expectation(
    write_gate: &WriteGate,
    members: &[PostgresRoutingTarget],
    observer: Option<&PgtmObserver>,
    query_timeout: Duration,
) -> Result<(Option<u64>, Option<String>), WriteConvergenceInvariantError> {
    let last_write_error = write_gate.last_error();
    let accepted_count = write_gate.accepted_count();
    let expected_count = if last_write_error_is_ambiguous_timeout(last_write_error.as_deref()) {
        let routing_target = match observer {
            Some(observer) => authoritative_primary_routing_target(observer)
                .map_err(WriteConvergenceInvariantError::Failed)?
                .or_else(|| members.first().cloned()),
            None => members.first().cloned(),
        }
        .ok_or_else(|| {
            WriteConvergenceInvariantError::Failed(
                "write convergence invariant has no selected members to reconcile".to_string(),
            )
        })?;
        let authoritative_count =
            read_count_via_fresh_connection_target(&routing_target, query_timeout)
                .await
                .map_err(WriteConvergenceInvariantError::Failed)?;
        Some(accepted_count.map_or(authoritative_count, |count| count.max(authoritative_count)))
    } else {
        accepted_count
    };
    Ok((expected_count, last_write_error))
}

fn last_write_error_is_ambiguous_timeout(last_write_error: Option<&str>) -> bool {
    last_write_error.is_some_and(|error| error.contains("increment fixture row timed out after"))
}

async fn read_count(
    client: &Client,
    query_timeout: Duration,
) -> Result<u64, WriteConvergenceInvariantError> {
    let row = tokio::time::timeout(
        query_timeout,
        client.query_opt(SELECT_FIXTURE_ROW_SQL, &[&FIXTURE_ROW_ID]),
    )
    .await
    .map_err(|_| {
        WriteConvergenceInvariantError::Failed(format!(
            "select fixture row timed out after {:?}",
            query_timeout
        ))
    })?
    .map_err(|err| {
        WriteConvergenceInvariantError::Failed(format!("select fixture row failed: {err}"))
    })?
    .ok_or_else(|| {
        WriteConvergenceInvariantError::Failed(format!(
            "fixture row `{FIXTURE_ROW_ID}` missing from `{FIXTURE_TABLE}`"
        ))
    })?;
    u64::try_from(row.get::<_, i64>(0)).map_err(|err| {
        WriteConvergenceInvariantError::Failed(format!("fixture count was negative: {err}"))
    })
}

fn build_tls_connector(
    conninfo: &PgConnInfo,
) -> Result<MakeRustlsConnect, WriteConvergenceInvariantError> {
    let root_cert_path =
        required_tls_path(conninfo.tls.root_cert.as_ref(), "sslrootcert", conninfo)?;
    let client_cert_path =
        required_tls_path(conninfo.tls.client_cert.as_ref(), "sslcert", conninfo)?;
    let client_key_path = required_tls_path(conninfo.tls.client_key.as_ref(), "sslkey", conninfo)?;

    let mut roots = RootCertStore::empty();
    for cert in load_cert_chain(root_cert_path.as_str())? {
        roots.add(cert).map_err(|err| {
            WriteConvergenceInvariantError::Failed(format!(
                "add root certificate `{root_cert_path}` failed: {err}"
            ))
        })?;
    }

    let builder = rustls::ClientConfig::builder_with_provider(
        rustls::crypto::ring::default_provider().into(),
    )
    .with_safe_default_protocol_versions()
    .map_err(|err| {
        WriteConvergenceInvariantError::Failed(format!("build rustls client config failed: {err}"))
    })?
    .with_root_certificates(roots);
    let client_config = builder
        .with_client_auth_cert(
            load_cert_chain(client_cert_path.as_str())?,
            load_private_key(client_key_path.as_str())?,
        )
        .map_err(|err| {
            WriteConvergenceInvariantError::Failed(format!(
                "configure rustls client identity failed: {err}"
            ))
        })?;

    Ok(MakeRustlsConnect::new(client_config))
}

fn required_tls_path(
    path: Option<&PathBuf>,
    key: &str,
    conninfo: &PgConnInfo,
) -> Result<String, WriteConvergenceInvariantError> {
    path.map(|value| value.display().to_string())
        .ok_or_else(|| {
            WriteConvergenceInvariantError::Failed(format!(
                "conninfo did not contain `{key}`: {conninfo}"
            ))
        })
}

fn conninfo_uses_tls_files(conninfo: &PgConnInfo) -> bool {
    conninfo.tls.root_cert.is_some()
        || conninfo.tls.client_cert.is_some()
        || conninfo.tls.client_key.is_some()
}

fn connectable_conninfo(conninfo: &PgConnInfo) -> PgConnInfo {
    let tls_mode = match conninfo.tls.mode {
        PgSslMode::VerifyCa | PgSslMode::VerifyFull => PgSslMode::Require,
        mode => mode,
    };

    PgConnInfo {
        tls: PgClientTls {
            mode: tls_mode,
            root_cert: None,
            client_cert: None,
            client_key: None,
        },
        ..conninfo.clone()
    }
}

fn load_cert_chain(
    path: &str,
) -> Result<Vec<CertificateDer<'static>>, WriteConvergenceInvariantError> {
    let pem = fs::read(path).map_err(|err| {
        WriteConvergenceInvariantError::Failed(format!("read certificate `{path}` failed: {err}"))
    })?;
    let mut reader = Cursor::new(pem);
    rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| {
            WriteConvergenceInvariantError::Failed(format!(
                "parse certificate `{path}` failed: {err}"
            ))
        })
}

fn load_private_key(path: &str) -> Result<PrivateKeyDer<'static>, WriteConvergenceInvariantError> {
    let pem = fs::read(path).map_err(|err| {
        WriteConvergenceInvariantError::Failed(format!("read private key `{path}` failed: {err}"))
    })?;
    let mut reader = Cursor::new(pem);
    rustls_pemfile::private_key(&mut reader)
        .map_err(|err| {
            WriteConvergenceInvariantError::Failed(format!(
                "parse private key `{path}` failed: {err}"
            ))
        })?
        .ok_or_else(|| {
            WriteConvergenceInvariantError::Failed(format!("private key `{path}` was empty"))
        })
}

enum MemberCountObservation {
    Observed {
        member: ClusterMember,
        count: u64,
    },
    Failed {
        member: ClusterMember,
        message: String,
    },
}

impl MemberCountObservation {
    fn observed_count(&self) -> Option<u64> {
        match self {
            Self::Observed { count, .. } => Some(*count),
            Self::Failed { .. } => None,
        }
    }

    fn render(&self) -> String {
        match self {
            Self::Observed { member, count } => format!("`{member}`={count}"),
            Self::Failed { member, message } => format!("`{member}` error={message}"),
        }
    }
}

fn observations_are_converged(
    observations: &[MemberCountObservation],
    expected_count: Option<u64>,
) -> bool {
    let mut shared_count = expected_count;
    for observation in observations {
        let count = match observation.observed_count() {
            Some(count) => count,
            None => return false,
        };
        if let Some(expected) = shared_count {
            if expected != count {
                return false;
            }
            continue;
        }
        shared_count = Some(count);
    }
    shared_count.is_some()
}

fn render_observations(observations: &[MemberCountObservation]) -> String {
    observations
        .iter()
        .map(MemberCountObservation::render)
        .collect::<Vec<_>>()
        .join(", ")
}

fn convergence_failure(
    observations: &[MemberCountObservation],
    write_deadline: Duration,
    expected_count: Option<u64>,
    last_write_error: Option<&str>,
) -> WriteConvergenceInvariantError {
    let expectation = expected_count.map_or_else(
        || format!("`{FIXTURE_TABLE}` row `{FIXTURE_ROW_ID}`"),
        |count| format!("accepted count `{count}` on `{FIXTURE_TABLE}` row `{FIXTURE_ROW_ID}`"),
    );
    let last_write_error = last_write_error.map_or_else(String::new, |error| {
        format!("; last write attempt error: {error}")
    });
    WriteConvergenceInvariantError::Failed(format!(
        "expected selected members to converge on {expectation} before {:?}; observed: {}{}",
        write_deadline,
        render_observations(observations),
        last_write_error,
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        connectable_conninfo, convergence_failure, observations_are_converged,
        perform_authoritative_write, read_count, required_tls_path, ConnectionTask,
        MemberCountObservation, ThreadJoinHandle, WriteConvergenceInvariantError,
        WriteConvergenceInvariantRunner, WriteGate, CREATE_FIXTURE_TABLE_SQL, FIXTURE_ROW_ID,
    };
    use crate::support::{observer::pgtm::PostgresRoutingTarget, topology::ClusterMember};
    use pgtuskmaster_rust::{
        pginfo::{
            conninfo::PgClientTls,
            state::{PgConnInfo, PgSslMode},
        },
        state::PgRoute,
    };
    use pgtuskmaster_test_support::{
        binaries::require_pg16_bin_for_real_tests,
        namespace::NamespaceGuard,
        pg16::{prepare_pgdata_dir, spawn_pg16, PgHandle, PgInstanceSpec},
        ports::allocate_ports,
    };
    use std::{
        error::Error,
        fs,
        io::Error as IoError,
        sync::mpsc::{sync_channel, SyncSender},
        sync::{
            atomic::{AtomicBool, AtomicU64, Ordering},
            Arc,
        },
        thread,
        time::Duration,
    };
    use tokio::{runtime::Builder, sync::oneshot, time::Instant};
    use tokio_postgres::{Client, NoTls};

    fn sample_routing_conninfo() -> Result<PgConnInfo, String> {
        Ok(PgConnInfo {
            route: PgRoute::tcp_hostaddr(
                "node-a".to_string(),
                5432,
                Some(std::net::Ipv4Addr::LOCALHOST.into()),
            )?,
            user: "postgres".to_string(),
            dbname: "postgres db".to_string(),
            application_name: None,
            connect_timeout_s: None,
            options: None,
            tls: PgClientTls {
                mode: PgSslMode::VerifyFull,
                root_cert: Some("/tmp/ca bundle.pem".into()),
                client_cert: Some("/tmp/client.crt".into()),
                client_key: Some("/tmp/client.key".into()),
            },
        })
    }

    #[test]
    fn convergence_failure_reports_dual_primary_style_divergence() {
        let observations = vec![
            MemberCountObservation::Observed {
                member: ClusterMember::NodeA,
                count: 1,
            },
            MemberCountObservation::Observed {
                member: ClusterMember::NodeB,
                count: 2,
            },
            MemberCountObservation::Observed {
                member: ClusterMember::NodeC,
                count: 1,
            },
        ];

        assert!(!observations_are_converged(observations.as_slice(), None));

        let err = convergence_failure(
            observations.as_slice(),
            Duration::from_millis(10),
            None,
            None,
        );

        match err {
            WriteConvergenceInvariantError::Failed(message) => {
                assert!(message.contains("selected members to converge"));
                assert!(message.contains("`node-a`=1"));
                assert!(message.contains("`node-b`=2"));
                assert!(message.contains("`node-c`=1"));
            }
        }
    }

    #[test]
    fn required_tls_path_accepts_all_tls_fields() -> Result<(), Box<dyn Error + Send + Sync>> {
        let conninfo = sample_routing_conninfo()?;

        assert_eq!(
            required_tls_path(conninfo.tls.root_cert.as_ref(), "sslrootcert", &conninfo)?,
            "/tmp/ca bundle.pem".to_string()
        );
        assert_eq!(
            required_tls_path(conninfo.tls.client_cert.as_ref(), "sslcert", &conninfo)?,
            "/tmp/client.crt".to_string()
        );
        assert_eq!(
            required_tls_path(conninfo.tls.client_key.as_ref(), "sslkey", &conninfo)?,
            "/tmp/client.key".to_string()
        );
        Ok(())
    }

    #[test]
    fn connectable_conninfo_strips_tls_path_fields_but_preserves_general_conninfo(
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let connect_conninfo = connectable_conninfo(&sample_routing_conninfo()?);
        let connect_dsn = connect_conninfo.to_string();

        assert!(connect_dsn.contains("host=node-a"));
        assert!(connect_dsn.contains("hostaddr=127.0.0.1"));
        assert!(connect_dsn.contains("port=5432"));
        assert!(connect_dsn.contains("user=postgres"));
        assert!(connect_dsn.contains("dbname='postgres db'"));
        assert!(connect_dsn.contains("sslmode=require"));
        assert!(!connect_dsn.contains("sslrootcert"));
        assert!(!connect_dsn.contains("sslcert"));
        assert!(!connect_dsn.contains("sslkey"));
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn one_primary_and_two_replicas_are_determined_healthy(
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut fixture = RealPostgresFixture::spawn("write-convergence-loop").await?;
        let run_result = async {
            let write_gate = Arc::new(WriteGate::new());
            initialize_fixture_row_via_dsn(fixture.dsn.as_str(), 0).await?;
            let primary_probe = connect_session(fixture.dsn.as_str(), false).await?;
            let replica_probe = connect_session(fixture.dsn.as_str(), true).await?;
            let stop_requested = Arc::new(AtomicBool::new(false));
            let worker = Some(build_authoritative_write_worker(
                fixture.dsn.as_str(),
                Arc::clone(&write_gate),
                Arc::clone(&stop_requested),
                Duration::from_millis(10),
            )?);
            let runner = WriteConvergenceInvariantRunner::new(
                None,
                Duration::from_millis(10),
                Duration::from_millis(250),
                Arc::clone(&write_gate),
                ClusterMember::ALL
                    .into_iter()
                    .map(|member| routing_target_for_dsn(member, fixture.dsn.as_str()))
                    .collect::<Result<Vec<_>, _>>()?,
                stop_requested,
                worker,
            );

            wait_for_row_count_at_least(primary_probe.client.as_ref(), 3, Duration::from_secs(5))
                .await?;
            wait_for_row_count_at_least(replica_probe.client.as_ref(), 3, Duration::from_secs(5))
                .await?;

            runner.ensure_healthy(ClusterMember::ALL.as_slice())?;
            let primary_count =
                read_count(primary_probe.client.as_ref(), Duration::from_millis(250)).await?;
            let replica_count =
                read_count(replica_probe.client.as_ref(), Duration::from_millis(250)).await?;
            drop(runner);
            drop(primary_probe);
            drop(replica_probe);
            assert_eq!(primary_count, replica_count);
            assert!(primary_count >= 3);
            Ok(())
        }
        .await;

        fixture.handle.shutdown().await?;
        run_result
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn three_replicas_are_not_determined_healthy() -> Result<(), Box<dyn Error + Send + Sync>>
    {
        let mut fixture_a = RealPostgresFixture::spawn("write-convergence-standalone-a").await?;
        let mut fixture_b = RealPostgresFixture::spawn("write-convergence-standalone-b").await?;
        let run_result = async {
            let write_gate = Arc::new(WriteGate::new());
            initialize_fixture_row_via_dsn(fixture_a.dsn.as_str(), 0).await?;
            initialize_fixture_row_via_dsn(fixture_b.dsn.as_str(), 0).await?;
            let writable_probe = connect_session(fixture_b.dsn.as_str(), false).await?;
            let stop_requested = Arc::new(AtomicBool::new(false));
            let worker = Some(build_authoritative_write_worker(
                fixture_b.dsn.as_str(),
                Arc::clone(&write_gate),
                Arc::clone(&stop_requested),
                Duration::from_millis(10),
            )?);
            let runner = WriteConvergenceInvariantRunner::new(
                None,
                Duration::from_millis(10),
                Duration::from_millis(150),
                Arc::clone(&write_gate),
                vec![
                    routing_target_for_dsn(ClusterMember::NodeA, fixture_a.dsn.as_str())?,
                    routing_target_for_dsn(ClusterMember::NodeB, fixture_b.dsn.as_str())?,
                    routing_target_for_dsn(ClusterMember::NodeC, fixture_b.dsn.as_str())?,
                ],
                stop_requested,
                worker,
            );

            wait_for_row_count_at_least(writable_probe.client.as_ref(), 1, Duration::from_secs(5))
                .await?;
            let result = runner.ensure_healthy(ClusterMember::ALL.as_slice());
            drop(runner);
            drop(writable_probe);
            match result {
                Ok(()) => Err(IoError::other("expected ensure_healthy to fail").into()),
                Err(WriteConvergenceInvariantError::Failed(message)) => {
                    assert!(message.contains("selected members to converge"));
                    assert!(message.contains("`node-a`="));
                    assert!(message.contains("`node-b`="));
                    assert!(message.contains("`node-c`="));
                    Ok(())
                }
            }
        }
        .await;

        fixture_a.handle.shutdown().await?;
        fixture_b.handle.shutdown().await?;
        run_result
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn local_write_on_doomed_member_is_not_treated_as_global_baseline(
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut doomed = RealPostgresFixture::spawn("write-convergence-doomed-member").await?;
        let mut surviving =
            RealPostgresFixture::spawn("write-convergence-surviving-majority").await?;
        let run_result = async {
            initialize_fixture_row_via_dsn(doomed.dsn.as_str(), 1).await?;
            initialize_fixture_row_via_dsn(surviving.dsn.as_str(), 0).await?;
            let write_gate = Arc::new(WriteGate::new());
            write_gate.record_accepted_count(0);
            let runner = WriteConvergenceInvariantRunner::new(
                None,
                Duration::from_millis(10),
                Duration::from_millis(150),
                Arc::clone(&write_gate),
                vec![
                    routing_target_for_dsn(ClusterMember::NodeA, doomed.dsn.as_str())?,
                    routing_target_for_dsn(ClusterMember::NodeB, surviving.dsn.as_str())?,
                    routing_target_for_dsn(ClusterMember::NodeC, surviving.dsn.as_str())?,
                ],
                Arc::new(AtomicBool::new(false)),
                None,
            );

            runner.ensure_healthy(&[ClusterMember::NodeB, ClusterMember::NodeC])?;
            Ok(())
        }
        .await;

        doomed.handle.shutdown().await?;
        surviving.handle.shutdown().await?;
        run_result
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn three_read_only_members_with_zero_shared_writes_still_count_as_converged(
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut fixture = RealPostgresFixture::spawn("write-convergence-zero-writes").await?;
        let run_result = async {
            let write_gate = Arc::new(WriteGate::new());
            initialize_fixture_row_via_dsn(fixture.dsn.as_str(), 0).await?;
            write_gate.record_accepted_count(0);
            let runner = WriteConvergenceInvariantRunner::new(
                None,
                Duration::from_millis(10),
                Duration::from_millis(150),
                Arc::clone(&write_gate),
                ClusterMember::ALL
                    .into_iter()
                    .map(|member| routing_target_for_dsn(member, fixture.dsn.as_str()))
                    .collect::<Result<Vec<_>, _>>()?,
                Arc::new(AtomicBool::new(false)),
                None,
            );

            runner.ensure_healthy(ClusterMember::ALL.as_slice())?;
            drop(runner);
            Ok(())
        }
        .await;

        fixture.handle.shutdown().await?;
        run_result
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn ambiguous_timeout_reconciles_expected_count_before_health_check(
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut fixture = RealPostgresFixture::spawn("write-convergence-ambiguous-timeout").await?;
        let run_result = async {
            initialize_fixture_row_via_dsn(fixture.dsn.as_str(), 4).await?;
            let write_gate = Arc::new(WriteGate::new());
            write_gate.record_accepted_count(3);
            write_gate.record_last_error(
                "authoritative write via `node-a` failed: increment fixture row timed out after 10ms"
                    .to_string(),
            );
            let runner = WriteConvergenceInvariantRunner::new(
                None,
                Duration::from_millis(10),
                Duration::from_millis(150),
                Arc::clone(&write_gate),
                ClusterMember::ALL
                    .into_iter()
                    .map(|member| routing_target_for_dsn(member, fixture.dsn.as_str()))
                    .collect::<Result<Vec<_>, _>>()?,
                Arc::new(AtomicBool::new(false)),
                None,
            );

            runner.ensure_healthy(ClusterMember::ALL.as_slice())?;
            Ok(())
        }
        .await;

        fixture.handle.shutdown().await?;
        run_result
    }

    #[test]
    fn current_thread_cleanup_waits_for_detached_worker_shutdown(
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let write_gate = Arc::new(WriteGate::new());
        let (write_started_tx, write_started_rx) = sync_channel(1);
        let (cleanup_started_tx, cleanup_started_rx) = sync_channel(1);
        let (cleanup_done_tx, cleanup_done_rx) = sync_channel(1);
        let (release_write_tx, release_write_rx) = oneshot::channel();
        let committed_count = Arc::new(AtomicU64::new(0));
        let stop_requested = Arc::new(AtomicBool::new(false));
        let worker = build_blocked_write_worker(
            Arc::clone(&write_gate),
            Arc::clone(&stop_requested),
            Arc::clone(&committed_count),
            write_started_tx,
            release_write_rx,
        )?;
        let runner = WriteConvergenceInvariantRunner::new(
            None,
            Duration::from_millis(10),
            Duration::from_millis(10),
            Arc::clone(&write_gate),
            vec![sample_routing_target(ClusterMember::NodeA)?],
            stop_requested,
            Some(worker),
        );

        write_started_rx
            .recv_timeout(Duration::from_secs(1))
            .map_err(|_| IoError::other("timed out waiting for detached worker to start"))?;

        let ensure_thread = thread::spawn(move || {
            let send_started = cleanup_started_tx.send(());
            let result = send_started
                .map_err(|_| IoError::other("cleanup start signal receiver dropped"))
                .and_then(|()| {
                    Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .map_err(|err| {
                            IoError::other(format!("build cleanup runtime failed: {err}"))
                        })
                })
                .and_then(|runtime| {
                    runtime
                        .block_on(async { runner.ensure_running() })
                        .map_err(|err| IoError::other(err.to_string()))
                });
            let _ = cleanup_done_tx.send(result);
        });

        cleanup_started_rx
            .recv_timeout(Duration::from_secs(1))
            .map_err(|_| IoError::other("timed out waiting for cleanup to start"))?;
        thread::sleep(Duration::from_millis(50));
        release_write_tx
            .send(())
            .map_err(|_| IoError::other("detached worker release receiver dropped"))?;
        cleanup_done_rx
            .recv_timeout(Duration::from_secs(2))
            .map_err(|_| IoError::other("current-thread cleanup did not finish"))??;
        ensure_thread
            .join()
            .map_err(|_| IoError::other("cleanup test thread panicked"))?;
        assert_eq!(committed_count.load(Ordering::SeqCst), 1);
        Ok(())
    }

    #[test]
    fn ensure_running_does_not_probe_unreachable_members_during_cleanup(
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let write_gate = Arc::new(WriteGate::new());
        write_gate.record_accepted_count(2);
        let runner = WriteConvergenceInvariantRunner::new(
            None,
            Duration::from_millis(10),
            Duration::from_millis(10),
            Arc::clone(&write_gate),
            vec![
                routing_target_for_dsn(
                    ClusterMember::NodeA,
                    "host=127.0.0.1 port=1 user=postgres dbname=postgres sslmode=disable",
                )?,
                routing_target_for_dsn(
                    ClusterMember::NodeB,
                    "host=127.0.0.1 port=1 user=postgres dbname=postgres sslmode=disable",
                )?,
            ],
            Arc::new(AtomicBool::new(false)),
            None,
        );

        assert!(runner.ensure_running().is_ok());
        Ok(())
    }

    #[test]
    fn selected_routing_targets_follow_online_member_subset(
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let runner = WriteConvergenceInvariantRunner::new(
            None,
            Duration::from_millis(10),
            Duration::from_millis(10),
            Arc::new(WriteGate::new()),
            ClusterMember::ALL
                .into_iter()
                .map(sample_routing_target)
                .collect::<Result<Vec<_>, _>>()?,
            Arc::new(AtomicBool::new(false)),
            None,
        );

        let selected =
            runner.selected_routing_targets(&[ClusterMember::NodeA, ClusterMember::NodeC])?;

        assert_eq!(
            selected
                .iter()
                .map(|target| target.member)
                .collect::<Vec<_>>(),
            vec![ClusterMember::NodeA, ClusterMember::NodeC],
        );
        Ok(())
    }

    fn routing_target_for_dsn(
        member: ClusterMember,
        dsn: &str,
    ) -> Result<PostgresRoutingTarget, Box<dyn Error + Send + Sync>> {
        Ok(PostgresRoutingTarget {
            member,
            conninfo: dsn.parse()?,
        })
    }

    fn sample_routing_target(member: ClusterMember) -> Result<PostgresRoutingTarget, IoError> {
        Ok(PostgresRoutingTarget {
            member,
            conninfo: "host=127.0.0.1 port=5432 user=postgres dbname=postgres sslmode=disable"
                .parse::<PgConnInfo>()
                .map_err(|err| IoError::other(err.to_string()))?,
        })
    }

    struct RealPostgresFixture {
        _guard: NamespaceGuard,
        dsn: String,
        handle: PgHandle,
    }

    impl RealPostgresFixture {
        async fn spawn(test_name: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
            let postgres_bin = require_pg16_bin_for_real_tests("postgres")?;
            let initdb_bin = require_pg16_bin_for_real_tests("initdb")?;
            let guard = NamespaceGuard::new(test_name)?;
            let namespace = guard.namespace()?;
            let data_dir = prepare_pgdata_dir(namespace, "node-a")?;
            let socket_dir = namespace.child_dir("run/node-a");
            let log_dir = namespace.child_dir("logs/node-a");
            fs::create_dir_all(&socket_dir)?;
            fs::create_dir_all(&log_dir)?;

            let reservation = allocate_ports(1)?;
            let port = reservation.as_slice()[0];
            drop(reservation);

            let handle = spawn_pg16(PgInstanceSpec {
                postgres_bin,
                initdb_bin,
                data_dir,
                socket_dir,
                log_dir,
                port,
                startup_timeout: Duration::from_secs(25),
            })
            .await?;

            let dsn =
                format!("host=127.0.0.1 port={port} user=postgres dbname=postgres sslmode=disable");
            wait_for_postgres_ready(dsn.as_str(), Duration::from_secs(20)).await?;

            Ok(Self {
                _guard: guard,
                dsn,
                handle,
            })
        }
    }

    struct SessionHandle {
        client: Arc<Client>,
        connection_task: ConnectionTask,
    }

    impl Drop for SessionHandle {
        fn drop(&mut self) {
            self.connection_task.abort();
        }
    }

    async fn connect_client(
        dsn: &str,
    ) -> Result<(Arc<Client>, ConnectionTask), Box<dyn Error + Send + Sync>> {
        let (client, connection) = tokio_postgres::connect(dsn, NoTls).await?;
        Ok((Arc::new(client), tokio::spawn(connection)))
    }

    async fn connect_session(
        dsn: &str,
        read_only: bool,
    ) -> Result<SessionHandle, Box<dyn Error + Send + Sync>> {
        let (client, connection_task) = connect_client(dsn).await?;
        if read_only {
            client
                .simple_query("SET default_transaction_read_only = on")
                .await?;
        }
        Ok(SessionHandle {
            client,
            connection_task,
        })
    }

    fn build_authoritative_write_worker(
        dsn: &str,
        write_gate: Arc<WriteGate>,
        stop_requested: Arc<AtomicBool>,
        poll_interval: Duration,
    ) -> Result<ThreadJoinHandle<Result<(), String>>, Box<dyn Error + Send + Sync>> {
        let dsn = dsn.to_string();
        let task = thread::Builder::new()
            .name("write-convergence-test-authoritative".to_string())
            .spawn(move || {
                Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|err| format!("build authoritative test write runtime failed: {err}"))?
                    .block_on(async move {
                        loop {
                            if stop_requested.load(Ordering::SeqCst) {
                                return Ok(());
                            }
                            let write_permit = match write_gate.try_start_write() {
                                Some(write_permit) => write_permit,
                                None => return Ok(()),
                            };
                            let write_result = match connect_session(dsn.as_str(), false).await {
                                Ok(session) => {
                                    let result = perform_authoritative_write(
                                        session.client.as_ref(),
                                        write_gate.as_ref(),
                                        poll_interval,
                                    )
                                    .await
                                    .map_err(|err| err.to_string());
                                    drop(session);
                                    result
                                }
                                Err(err) => Err(err.to_string()),
                            };
                            drop(write_permit);
                            match write_result {
                                Ok(()) => write_gate.clear_last_error(),
                                Err(err) => write_gate.record_last_error(err),
                            }
                            if stop_requested.load(Ordering::SeqCst) {
                                return Ok(());
                            }
                            tokio::time::sleep(poll_interval).await;
                        }
                    })
            })?;
        Ok(task)
    }

    fn build_blocked_write_worker(
        write_gate: Arc<WriteGate>,
        stop_requested: Arc<AtomicBool>,
        committed_count: Arc<AtomicU64>,
        write_started_tx: SyncSender<()>,
        release_write_rx: oneshot::Receiver<()>,
    ) -> Result<ThreadJoinHandle<Result<(), String>>, Box<dyn Error + Send + Sync>> {
        let task = thread::Builder::new()
            .name("write-convergence-test-blocked".to_string())
            .spawn(move || {
                Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|err| format!("build blocked test runtime failed: {err}"))?
                    .block_on(async move {
                        let write_permit = write_gate.try_start_write().ok_or_else(|| {
                            "test worker gate was already closed before write start".to_string()
                        })?;
                        write_started_tx
                            .send(())
                            .map_err(|_| "test worker write-start receiver dropped".to_string())?;
                        release_write_rx
                            .await
                            .map_err(|_| "test worker release signal sender dropped".to_string())?;
                        committed_count.fetch_add(1, Ordering::SeqCst);
                        drop(write_permit);
                        loop {
                            if stop_requested.load(Ordering::SeqCst) {
                                return Ok(());
                            }
                            tokio::time::sleep(Duration::from_millis(10)).await;
                        }
                    })
            })?;
        Ok(task)
    }

    async fn initialize_fixture_row(
        client: &Client,
        count: i64,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        client.batch_execute(CREATE_FIXTURE_TABLE_SQL).await?;
        client
            .execute(
                "
INSERT INTO public.write_convergence_invariant (id, written_count)
VALUES ($1, $2)
ON CONFLICT (id) DO UPDATE
SET written_count = EXCLUDED.written_count
",
                &[&FIXTURE_ROW_ID, &count],
            )
            .await?;
        Ok(())
    }

    async fn initialize_fixture_row_via_dsn(
        dsn: &str,
        count: i64,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let session = connect_session(dsn, false).await?;
        let result = initialize_fixture_row(session.client.as_ref(), count).await;
        drop(session);
        result
    }

    async fn wait_for_row_count_at_least(
        client: &Client,
        minimum: u64,
        timeout: Duration,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let deadline = Instant::now() + timeout;
        loop {
            let observed = read_count(client, Duration::from_millis(250)).await?;
            if observed >= minimum {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(IoError::other(format!(
                    "timed out waiting for row count >= {minimum}; observed {observed}"
                ))
                .into());
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    async fn wait_for_postgres_ready(
        dsn: &str,
        timeout: Duration,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let deadline = Instant::now() + timeout;
        loop {
            match tokio_postgres::connect(dsn, NoTls).await {
                Ok((client, connection)) => {
                    let connection_task = tokio::spawn(connection);
                    client.simple_query("SELECT 1").await?;
                    drop(client);
                    connection_task.await??;
                    return Ok(());
                }
                Err(err) => {
                    if Instant::now() >= deadline {
                        return Err(Box::new(err));
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }
}
