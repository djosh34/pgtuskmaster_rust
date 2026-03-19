use std::{
    collections::BTreeSet,
    fs,
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
    runtime::{Builder, Handle, RuntimeFlavor},
    task::JoinHandle,
    time::Instant,
};
use tokio_postgres::{Client, NoTls};
use tokio_postgres_rustls::MakeRustlsConnect;

use pgtuskmaster_rust::pginfo::{
    conninfo::PgClientTls,
    state::{PgConnInfo, PgSslMode},
};

use crate::support::{
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
WHERE id = $1";
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
    members: Mutex<Vec<MemberWorker>>,
}

#[derive(Debug, thiserror::Error)]
pub enum WriteConvergenceInvariantError {
    #[error("write-convergence invariant failed: {0}")]
    Failed(String),
}

struct MemberWorker {
    routing_target: PostgresRoutingTarget,
    stop_requested: Arc<AtomicBool>,
    thread: Option<ThreadJoinHandle<Result<(), String>>>,
}

#[derive(Default)]
struct WriteGate {
    state: Mutex<WriteGateState>,
    drained: Condvar,
}

#[derive(Default)]
struct WriteGateState {
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
        let members = routing_targets
            .iter()
            .cloned()
            .map(|routing_target| {
                spawn_member_worker(routing_target, Arc::clone(&write_gate), poll_interval)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::new(
            Some(observer),
            poll_interval,
            write_deadline,
            write_gate,
            routing_targets,
            members,
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
        let future = async move {
            wait_for_convergence(
                members.as_slice(),
                observer.as_ref(),
                poll_interval,
                write_deadline,
            )
            .await
        };

        match Handle::try_current() {
            Ok(handle) if handle.runtime_flavor() == RuntimeFlavor::MultiThread => {
                tokio::task::block_in_place(|| handle.block_on(future))
            }
            Ok(_) | Err(_) => thread::spawn(move || {
                Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|err| {
                        WriteConvergenceInvariantError::Failed(format!(
                            "build runtime for write convergence invariant failed: {err}"
                        ))
                    })?
                    .block_on(future)
            })
            .join()
            .map_err(|_| {
                WriteConvergenceInvariantError::Failed(
                    "write convergence invariant health check thread panicked".to_string(),
                )
            })?,
        }
    }

    pub fn ensure_running(&self) -> Result<(), WriteConvergenceInvariantError> {
        self.write_gate.close_and_drain();
        self.stop_members()
    }
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
        if let Err(err) = self.stop_members() {
            assert!(
                thread::panicking(),
                "write convergence invariant cleanup failed: {err}"
            );
        }
    }
}

impl std::fmt::Debug for MemberWorker {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MemberWorker")
            .field("member", &self.routing_target.member)
            .finish()
    }
}

impl WriteConvergenceInvariantRunner {
    fn new(
        observer: Option<PgtmObserver>,
        poll_interval: Duration,
        write_deadline: Duration,
        write_gate: Arc<WriteGate>,
        routing_targets: Vec<PostgresRoutingTarget>,
        members: Vec<MemberWorker>,
    ) -> Self {
        Self {
            observer,
            poll_interval,
            write_deadline,
            write_gate,
            routing_targets,
            members: Mutex::new(members),
        }
    }

    fn stop_members(&self) -> Result<(), WriteConvergenceInvariantError> {
        let members = {
            let mut members = self.members.lock().map_err(|_| {
                WriteConvergenceInvariantError::Failed(
                    "write convergence member-worker mutex was poisoned".to_string(),
                )
            })?;
            members.drain(..).collect::<Vec<_>>()
        };
        members.iter().for_each(MemberWorker::request_stop);
        members.into_iter().try_for_each(MemberWorker::join)
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

impl MemberWorker {
    fn request_stop(&self) {
        self.stop_requested.store(true, Ordering::SeqCst);
    }

    fn join(mut self) -> Result<(), WriteConvergenceInvariantError> {
        let thread = match self.thread.take() {
            Some(thread) => thread,
            None => return Ok(()),
        };
        thread
            .join()
            .map_err(|_| {
                WriteConvergenceInvariantError::Failed(format!(
                    "write worker thread for `{}` panicked",
                    self.routing_target.member
                ))
            })?
            .map_err(WriteConvergenceInvariantError::Failed)
    }
}

async fn connect_member(
    routing_target: &PostgresRoutingTarget,
    connect_timeout: Duration,
) -> Result<(Arc<Client>, ConnectionTask), String> {
    let connect_dsn = connectable_conninfo(&routing_target.conninfo).to_string();
    if conninfo_uses_tls_files(&routing_target.conninfo) {
        let tls = build_tls_connector(&routing_target.conninfo).map_err(|err| err.to_string())?;
        let (client, connection) = tokio::time::timeout(
            connect_timeout,
            tokio_postgres::connect(connect_dsn.as_str(), tls),
        )
        .await
        .map_err(|_| {
            format!(
                "connect to `{}` timed out after {:?}",
                routing_target.member, connect_timeout
            )
        })?
        .map_err(|err| format!("connect to `{}` failed: {err}", routing_target.member))?;
        let client: Arc<Client> = Arc::new(client);
        let connection_task = tokio::spawn(connection);
        match tokio::time::timeout(connect_timeout, client.simple_query("SELECT 1")).await {
            Ok(Ok(_)) => {}
            Ok(Err(err)) => {
                connection_task.abort();
                return Err(format!(
                    "connect to `{}` failed: {err}",
                    routing_target.member
                ));
            }
            Err(_) => {
                connection_task.abort();
                return Err(format!(
                    "connect to `{}` probe timed out after {:?}",
                    routing_target.member, connect_timeout
                ));
            }
        }
        return Ok((client, connection_task));
    }

    let (client, connection) = tokio::time::timeout(
        connect_timeout,
        tokio_postgres::connect(connect_dsn.as_str(), NoTls),
    )
    .await
    .map_err(|_| {
        format!(
            "connect to `{}` timed out after {:?}",
            routing_target.member, connect_timeout
        )
    })?
    .map_err(|err| format!("connect to `{}` failed: {err}", routing_target.member))?;
    let client: Arc<Client> = Arc::new(client);
    let connection_task = tokio::spawn(connection);
    match tokio::time::timeout(connect_timeout, client.simple_query("SELECT 1")).await {
        Ok(Ok(_)) => {}
        Ok(Err(err)) => {
            connection_task.abort();
            return Err(format!(
                "connect to `{}` failed: {err}",
                routing_target.member
            ));
        }
        Err(_) => {
            connection_task.abort();
            return Err(format!(
                "connect to `{}` probe timed out after {:?}",
                routing_target.member, connect_timeout
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

fn write_attempt_requires_reconnect(err: &tokio_postgres::Error) -> bool {
    err.as_db_error().is_none()
}

fn spawn_member_worker(
    routing_target: PostgresRoutingTarget,
    write_gate: Arc<WriteGate>,
    poll_interval: Duration,
) -> Result<MemberWorker, WriteConvergenceInvariantError> {
    let stop_requested = Arc::new(AtomicBool::new(false));
    let thread_routing_target = routing_target.clone();
    let thread_stop_requested = Arc::clone(&stop_requested);
    let thread = thread::Builder::new()
        .name(format!(
            "write-convergence-{}",
            thread_routing_target.member
        ))
        .spawn(move || {
            Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|err| {
                    format!(
                        "build runtime for write worker `{}` failed: {err}",
                        thread_routing_target.member
                    )
                })?
                .block_on(run_member_worker(
                    thread_routing_target,
                    write_gate,
                    thread_stop_requested,
                    poll_interval,
                ))
        })
        .map_err(|err| {
            WriteConvergenceInvariantError::Failed(format!(
                "spawn write worker thread for `{}` failed: {err}",
                routing_target.member
            ))
        })?;
    Ok(MemberWorker {
        routing_target,
        stop_requested,
        thread: Some(thread),
    })
}

async fn run_member_worker(
    routing_target: PostgresRoutingTarget,
    write_gate: Arc<WriteGate>,
    stop_requested: Arc<AtomicBool>,
    poll_interval: Duration,
) -> Result<(), String> {
    loop {
        if stop_requested.load(Ordering::SeqCst) {
            return Ok(());
        }
        match connect_member(&routing_target, poll_interval).await {
            Ok((client, connection_task)) => {
                maintain_connected_member(
                    client,
                    connection_task,
                    Arc::clone(&write_gate),
                    Arc::clone(&stop_requested),
                    poll_interval,
                    poll_interval,
                )
                .await;
            }
            Err(err) => {
                // THIS IS ONE OF THE ONLY VALID EXCEPTION TO THE "don't drop err via let _ = err rule"
                // That is because we explicitly we want that a failed connection just get retried when scenarios progress
                // DO NOT REMOVE THIS COMMENT DURING REFACTORS!
                let _ = err;
                if stop_requested.load(Ordering::SeqCst) {
                    return Ok(());
                }
                tokio::time::sleep(poll_interval).await;
            }
        }
    }
}

async fn maintain_connected_member(
    client: Arc<Client>,
    mut connection_task: ConnectionTask,
    write_gate: Arc<WriteGate>,
    stop_requested: Arc<AtomicBool>,
    poll_interval: Duration,
    query_timeout: Duration,
) {
    loop {
        if stop_requested.load(Ordering::SeqCst) {
            connection_task.abort();
            return;
        }
        tokio::select! {
            _ = tokio::time::sleep(poll_interval) => {
                let write_permit = match write_gate.try_start_write() {
                    Some(write_permit) => write_permit,
                    None => {
                        connection_task.abort();
                        return;
                    }
                };
                match tokio::time::timeout(query_timeout, apply_fixture_row_setup(client.as_ref())).await {
                    Ok(Ok(())) => {}
                    Ok(Err(err)) if write_attempt_requires_reconnect(&err) => {
                        connection_task.abort();
                        return;
                    }
                    Ok(Err(_)) => {
                        continue;
                    }
                    Err(_) => {
                        connection_task.abort();
                        return;
                    }
                }
                match tokio::time::timeout(
                    query_timeout,
                    client.execute(INCREMENT_FIXTURE_ROW_SQL, &[&FIXTURE_ROW_ID]),
                )
                .await
                {
                    Ok(Ok(1)) => {}
                    Ok(Err(err)) if write_attempt_requires_reconnect(&err) => {
                        connection_task.abort();
                        return;
                    }
                    Ok(Ok(_)) | Ok(Err(_)) => {}
                    Err(_) => {
                        connection_task.abort();
                        return;
                    }
                }
                drop(write_permit);
            }
            connection_result = &mut connection_task => {
                let _ = connection_result;
                return;
            }
        }
    }
}

async fn read_monitored_member_counts(
    members: &[PostgresRoutingTarget],
    observer: Option<&PgtmObserver>,
    query_timeout: Duration,
) -> Vec<MemberCountObservation> {
    futures::future::join_all(
        members
            .iter()
            .map(|member| read_member_count(member, observer, query_timeout)),
    )
    .await
}

async fn read_member_count(
    member: &PostgresRoutingTarget,
    observer: Option<&PgtmObserver>,
    query_timeout: Duration,
) -> MemberCountObservation {
    read_member_count_via_fresh_connection(member, observer, query_timeout, None).await
}

async fn read_member_count_via_fresh_connection(
    member: &PostgresRoutingTarget,
    observer: Option<&PgtmObserver>,
    connect_timeout: Duration,
    previous_error: Option<String>,
) -> MemberCountObservation {
    let routing_target = match resolve_observation_routing_target(member, observer) {
        Ok(routing_target) => routing_target,
        Err(err) => {
            return MemberCountObservation::Failed {
                member: member.member,
                message: previous_error.map_or(err.clone(), |previous| {
                    format!(
                        "existing observation failed: {previous}; refresh routing failed: {err}"
                    )
                }),
            };
        }
    };
    match connect_member(&routing_target, connect_timeout).await {
        Ok((client, connection_task)) => {
            let count_result = read_count(client.as_ref(), connect_timeout).await;
            connection_task.abort();
            match count_result {
                Ok(count) => MemberCountObservation::Observed {
                    member: member.member,
                    count,
                },
                Err(err) => MemberCountObservation::Failed {
                    member: member.member,
                    message: previous_error.map_or_else(
                        || err.to_string(),
                        |previous| format!(
                            "existing observation failed: {previous}; fresh reconnect read failed: {err}"
                        ),
                    ),
                },
            }
        }
        Err(err) => MemberCountObservation::Failed {
            member: member.member,
            message: previous_error.map_or_else(
                || err.clone(),
                |previous| {
                    format!(
                        "existing observation failed: {previous}; fresh reconnect failed: {err}"
                    )
                },
            ),
        },
    }
}

fn resolve_observation_routing_target(
    member: &PostgresRoutingTarget,
    observer: Option<&PgtmObserver>,
) -> Result<PostgresRoutingTarget, String> {
    observer.map_or_else(
        || Ok(member.clone()),
        |observer| {
            observer
                .postgres_routing_target(member.member)
                .map_err(|err| err.to_string())
        },
    )
}

async fn wait_for_convergence(
    members: &[PostgresRoutingTarget],
    observer: Option<&PgtmObserver>,
    poll_interval: Duration,
    write_deadline: Duration,
) -> Result<(), WriteConvergenceInvariantError> {
    let deadline = Instant::now() + write_deadline;
    loop {
        let observations = read_monitored_member_counts(members, observer, poll_interval).await;
        if observations_are_converged(observations.as_slice()) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(convergence_failure(observations.as_slice(), write_deadline));
        }
        tokio::time::sleep(poll_interval).await;
    }
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

fn observations_are_converged(observations: &[MemberCountObservation]) -> bool {
    let mut shared_count = None;
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
) -> WriteConvergenceInvariantError {
    WriteConvergenceInvariantError::Failed(format!(
        "expected selected members to converge on `{FIXTURE_TABLE}` row `{FIXTURE_ROW_ID}` before {:?}; observed: {}",
        write_deadline,
        render_observations(observations),
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        connectable_conninfo, convergence_failure, maintain_connected_member,
        observations_are_converged, read_count, required_tls_path, ConnectionTask,
        MemberCountObservation, MemberWorker, WriteConvergenceInvariantError,
        WriteConvergenceInvariantRunner, WriteGate, CREATE_FIXTURE_TABLE_SQL, FIXTURE_ROW_ID,
    };
    use crate::support::{observer::pgtm::PostgresRoutingTarget, topology::ClusterMember};
    use pgtuskmaster_rust::{
        pginfo::{
            conninfo::PgClientTls,
            state::{PgConnInfo, PgSslMode},
        },
        state::PgEndpoint,
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
            endpoint: PgEndpoint::tcp("node-a".to_string(), 5432)?,
            hostaddr: Some(std::net::Ipv4Addr::LOCALHOST.into()),
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

        assert!(!observations_are_converged(observations.as_slice()));

        let err = convergence_failure(observations.as_slice(), Duration::from_millis(10));

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
            let members = vec![
                build_member_writer(
                    ClusterMember::NodeA,
                    fixture.dsn.as_str(),
                    false,
                    Arc::clone(&write_gate),
                    Duration::from_millis(10),
                )
                .await?,
                build_member_writer(
                    ClusterMember::NodeB,
                    fixture.dsn.as_str(),
                    true,
                    Arc::clone(&write_gate),
                    Duration::from_millis(10),
                )
                .await?,
                build_member_writer(
                    ClusterMember::NodeC,
                    fixture.dsn.as_str(),
                    true,
                    Arc::clone(&write_gate),
                    Duration::from_millis(10),
                )
                .await?,
            ];
            let routing_targets = members
                .iter()
                .map(|member| member.routing_target.clone())
                .collect::<Vec<_>>();
            let runner = WriteConvergenceInvariantRunner::new(
                None,
                Duration::from_millis(10),
                Duration::from_millis(250),
                Arc::clone(&write_gate),
                routing_targets,
                members,
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
            let members = vec![
                build_member_writer(
                    ClusterMember::NodeA,
                    fixture_a.dsn.as_str(),
                    true,
                    Arc::clone(&write_gate),
                    Duration::from_millis(10),
                )
                .await?,
                build_member_writer(
                    ClusterMember::NodeB,
                    fixture_b.dsn.as_str(),
                    false,
                    Arc::clone(&write_gate),
                    Duration::from_millis(10),
                )
                .await?,
                build_member_writer(
                    ClusterMember::NodeC,
                    fixture_b.dsn.as_str(),
                    true,
                    Arc::clone(&write_gate),
                    Duration::from_millis(10),
                )
                .await?,
            ];
            let routing_targets = members
                .iter()
                .map(|member| member.routing_target.clone())
                .collect::<Vec<_>>();
            let writable_probe = connect_session(fixture_b.dsn.as_str(), false).await?;
            let runner = WriteConvergenceInvariantRunner::new(
                None,
                Duration::from_millis(10),
                Duration::from_millis(150),
                Arc::clone(&write_gate),
                routing_targets,
                members,
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
    async fn three_read_only_members_with_zero_shared_writes_still_count_as_converged(
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut fixture = RealPostgresFixture::spawn("write-convergence-zero-writes").await?;
        let run_result = async {
            let write_gate = Arc::new(WriteGate::new());
            initialize_fixture_row_via_dsn(fixture.dsn.as_str(), 0).await?;
            let members = vec![
                build_member_writer(
                    ClusterMember::NodeA,
                    fixture.dsn.as_str(),
                    true,
                    Arc::clone(&write_gate),
                    Duration::from_millis(10),
                )
                .await?,
                build_member_writer(
                    ClusterMember::NodeB,
                    fixture.dsn.as_str(),
                    true,
                    Arc::clone(&write_gate),
                    Duration::from_millis(10),
                )
                .await?,
                build_member_writer(
                    ClusterMember::NodeC,
                    fixture.dsn.as_str(),
                    true,
                    Arc::clone(&write_gate),
                    Duration::from_millis(10),
                )
                .await?,
            ];
            let routing_targets = members
                .iter()
                .map(|member| member.routing_target.clone())
                .collect::<Vec<_>>();
            let runner = WriteConvergenceInvariantRunner::new(
                None,
                Duration::from_millis(10),
                Duration::from_millis(150),
                Arc::clone(&write_gate),
                routing_targets,
                members,
            );

            tokio::time::sleep(Duration::from_millis(50)).await;
            runner.ensure_healthy(ClusterMember::ALL.as_slice())?;
            drop(runner);
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
        let worker = build_blocked_member_worker(
            ClusterMember::NodeA,
            Arc::clone(&write_gate),
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
            vec![worker],
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
            Vec::new(),
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

    async fn build_member_writer(
        member: ClusterMember,
        dsn: &str,
        read_only: bool,
        write_gate: Arc<WriteGate>,
        poll_interval: Duration,
    ) -> Result<MemberWorker, Box<dyn Error + Send + Sync>> {
        let (client, connection_task) = connect_client(dsn).await?;
        if read_only {
            client
                .simple_query("SET default_transaction_read_only = on")
                .await?;
        }
        let stop_requested = Arc::new(AtomicBool::new(false));
        let thread_stop_requested = Arc::clone(&stop_requested);
        let task = thread::Builder::new()
            .name(format!("write-convergence-test-{member}"))
            .spawn(move || {
                Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|err| {
                        format!("build test write runtime for `{member}` failed: {err}")
                    })?
                    .block_on(maintain_connected_member(
                        Arc::clone(&client),
                        connection_task,
                        write_gate,
                        thread_stop_requested,
                        poll_interval,
                        poll_interval,
                    ));
                Ok(())
            })?;
        Ok(MemberWorker {
            routing_target: PostgresRoutingTarget {
                member,
                conninfo: dsn.parse()?,
            },
            stop_requested,
            thread: Some(task),
        })
    }

    fn build_blocked_member_worker(
        member: ClusterMember,
        write_gate: Arc<WriteGate>,
        committed_count: Arc<AtomicU64>,
        write_started_tx: SyncSender<()>,
        release_write_rx: oneshot::Receiver<()>,
    ) -> Result<MemberWorker, Box<dyn Error + Send + Sync>> {
        let routing_target = PostgresRoutingTarget {
            member,
            conninfo: "host=127.0.0.1 port=5432 user=postgres dbname=postgres sslmode=disable"
                .parse()?,
        };
        let stop_requested = Arc::new(AtomicBool::new(false));
        let thread_stop_requested = Arc::clone(&stop_requested);
        let thread_member = member;
        let task = thread::Builder::new()
            .name(format!("write-convergence-test-blocked-{member}"))
            .spawn(move || {
                Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|err| {
                        format!("build blocked test runtime for `{thread_member}` failed: {err}")
                    })?
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
                            if thread_stop_requested.load(Ordering::SeqCst) {
                                return Ok(());
                            }
                            tokio::time::sleep(Duration::from_millis(10)).await;
                        }
                    })
            })?;
        Ok(MemberWorker {
            routing_target,
            stop_requested,
            thread: Some(task),
        })
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
