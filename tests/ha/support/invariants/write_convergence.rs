use std::{
    fs,
    io::Cursor,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use rustls::{
    self,
    pki_types::{CertificateDer, PrivateKeyDer},
    RootCertStore,
};
use tokio::{
    runtime::{Builder, Handle},
    sync::RwLock,
    task::JoinHandle,
    time::Instant,
};
use tokio_postgres::Client;
use tokio_postgres_rustls::MakeRustlsConnect;

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
const RESET_FIXTURE_ROW_SQL: &str = "
INSERT INTO public.write_convergence_invariant (id, written_count)
VALUES ($1, 0)
ON CONFLICT (id) DO UPDATE
SET written_count = EXCLUDED.written_count";
const INCREMENT_FIXTURE_ROW_SQL: &str = "
UPDATE public.write_convergence_invariant
SET written_count = written_count + 1
WHERE id = $1";
const SELECT_FIXTURE_ROW_SQL: &str = "
SELECT written_count
FROM public.write_convergence_invariant
WHERE id = $1";

pub struct WriteConvergenceInvariantRunner {
    poll_interval: Duration,
    write_deadline: Duration,
    pause_write: Arc<RwLock<()>>,
    written_count: Arc<AtomicU64>,
    health_failure: Arc<Mutex<Option<String>>>,
    members: Vec<MemberWriter>,
    monitor_task: JoinHandle<()>,
}

#[derive(Debug, thiserror::Error)]
pub enum WriteConvergenceInvariantError {
    #[error("write-convergence invariant failed: {0}")]
    Failed(String),
}

struct MemberWriter {
    member: ClusterMember,
    client: Arc<Client>,
    fatal_error: Arc<Mutex<Option<String>>>,
    connection_task: JoinHandle<()>,
    writer_task: JoinHandle<()>,
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

        let connected_members = connect_all_members(routing_targets.as_slice()).await?;
        initialize_fixture(connected_members.as_slice(), poll_interval, write_deadline).await?;

        let pause_write = Arc::new(RwLock::new(()));
        let written_count = Arc::new(AtomicU64::new(0));
        let members = connected_members
            .into_iter()
            .map(|connected_member| {
                let pause_write = Arc::clone(&pause_write);
                let written_count = Arc::clone(&written_count);
                let writer_client = connected_member.client.clone();
                let writer_task = tokio::spawn(async move {
                    writer_loop(writer_client, pause_write, written_count, poll_interval).await;
                });
                MemberWriter {
                    member: connected_member.member,
                    client: connected_member.client,
                    fatal_error: connected_member.fatal_error,
                    connection_task: connected_member.connection_task,
                    writer_task,
                }
            })
            .collect::<Vec<_>>();
        Ok(Self::new(
            poll_interval,
            write_deadline,
            pause_write,
            written_count,
            members,
        ))
    }

    pub fn ensure_healthy(&self) -> Result<(), WriteConvergenceInvariantError> {
        self.ensure_tasks_running()?;
        self.block_on(async {
            let _pause_guard = self.pause_write.write().await;
            self.evaluate_health_check().await
        })?
    }
}

impl std::fmt::Debug for WriteConvergenceInvariantRunner {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WriteConvergenceInvariantRunner")
            .field("poll_interval", &self.poll_interval)
            .field("write_deadline", &self.write_deadline)
            .field("members", &self.members)
            .finish()
    }
}

impl Drop for WriteConvergenceInvariantRunner {
    fn drop(&mut self) {
        self.monitor_task.abort();
        self.members.iter().for_each(|member| {
            member.writer_task.abort();
            member.connection_task.abort();
        });
    }
}

impl std::fmt::Debug for MemberWriter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MemberWriter")
            .field("member", &self.member)
            .finish()
    }
}

impl WriteConvergenceInvariantRunner {
    fn new(
        poll_interval: Duration,
        write_deadline: Duration,
        pause_write: Arc<RwLock<()>>,
        written_count: Arc<AtomicU64>,
        members: Vec<MemberWriter>,
    ) -> Self {
        let health_failure = Arc::new(Mutex::new(None));
        let monitor_task = tokio::spawn(monitor_loop(
            poll_interval,
            write_deadline,
            Arc::clone(&pause_write),
            Arc::clone(&written_count),
            members
                .iter()
                .map(|member| MonitoredMember {
                    member: member.member,
                    client: Arc::clone(&member.client),
                })
                .collect::<Vec<_>>(),
            Arc::clone(&health_failure),
        ));
        Self {
            poll_interval,
            write_deadline,
            pause_write,
            written_count,
            health_failure,
            members,
            monitor_task,
        }
    }

    fn ensure_tasks_running(&self) -> Result<(), WriteConvergenceInvariantError> {
        self.health_failure()?;
        if self.monitor_task.is_finished() {
            return Err(WriteConvergenceInvariantError::Failed(
                "health monitor task stopped".to_string(),
            ));
        }
        self.members.iter().try_for_each(|member| {
            if member.connection_task.is_finished() {
                return Err(WriteConvergenceInvariantError::Failed(format!(
                    "connection task for `{}` stopped",
                    member.member
                )));
            }
            if member.writer_task.is_finished() {
                return Err(WriteConvergenceInvariantError::Failed(format!(
                    "writer task for `{}` stopped",
                    member.member
                )));
            }
            member
                .fatal_error
                .lock()
                .map_err(|_| {
                    WriteConvergenceInvariantError::Failed(format!(
                        "fatal error mutex for `{}` was poisoned",
                        member.member
                    ))
                })?
                .as_ref()
                .map_or(Ok(()), |message| {
                    Err(WriteConvergenceInvariantError::Failed(format!(
                        "connection for `{}` failed: {message}",
                        member.member
                    )))
                })
        })
    }

    fn health_failure(&self) -> Result<(), WriteConvergenceInvariantError> {
        self.health_failure
            .lock()
            .map_err(|_| {
                WriteConvergenceInvariantError::Failed(
                    "health failure mutex was poisoned".to_string(),
                )
            })?
            .as_ref()
            .map_or(Ok(()), |message| {
                Err(WriteConvergenceInvariantError::Failed(message.clone()))
            })
    }

    fn block_on<T>(
        &self,
        future: impl std::future::Future<Output = T>,
    ) -> Result<T, WriteConvergenceInvariantError> {
        match Handle::try_current() {
            Ok(handle) => Ok(tokio::task::block_in_place(|| handle.block_on(future))),
            Err(_) => Builder::new_current_thread()
                .enable_all()
                .build()
                .map(|runtime| runtime.block_on(future))
                .map_err(|err| {
                    WriteConvergenceInvariantError::Failed(format!(
                        "build runtime for write convergence invariant failed: {err}"
                    ))
                }),
        }
    }

    async fn evaluate_health_check(&self) -> Result<(), WriteConvergenceInvariantError> {
        let expected_count = self.written_count.load(Ordering::SeqCst);
        if expected_count == 0 {
            return Err(WriteConvergenceInvariantError::Failed(format!(
                "no successful writes observed on `{FIXTURE_TABLE}` row `{FIXTURE_ROW_ID}` before {:?}",
                self.write_deadline
            )));
        }
        let observations = read_member_counts(self.members.as_slice()).await;
        if observations_match_expected(observations.as_slice(), expected_count) {
            Ok(())
        } else {
            Err(convergence_failure(
                expected_count,
                observations.as_slice(),
                self.write_deadline,
            ))
        }
    }
}

struct ConnectedMember {
    member: ClusterMember,
    client: Arc<Client>,
    fatal_error: Arc<Mutex<Option<String>>>,
    connection_task: JoinHandle<()>,
}

struct MonitoredMember {
    member: ClusterMember,
    client: Arc<Client>,
}

async fn connect_all_members(
    routing_targets: &[PostgresRoutingTarget],
) -> Result<Vec<ConnectedMember>, WriteConvergenceInvariantError> {
    let mut connected_members = Vec::with_capacity(routing_targets.len());
    for routing_target in routing_targets {
        let tls = build_tls_connector(routing_target.dsn.as_str())?;
        let (client, connection) = tokio_postgres::connect(routing_target.dsn.as_str(), tls)
            .await
            .map_err(|err| {
                WriteConvergenceInvariantError::Failed(format!(
                    "connect to `{}` failed: {err}",
                    routing_target.member
                ))
            })?;
        let fatal_error = Arc::new(Mutex::new(None));
        let fatal_error_for_task = Arc::clone(&fatal_error);
        let member = routing_target.member;
        let connection_task = tokio::spawn(async move {
            if let Err(err) = connection.await {
                if let Ok(mut slot) = fatal_error_for_task.lock() {
                    *slot = Some(err.to_string());
                }
            }
        });
        connected_members.push(ConnectedMember {
            member,
            client: Arc::new(client),
            fatal_error,
            connection_task,
        });
    }
    Ok(connected_members)
}

async fn initialize_fixture(
    members: &[ConnectedMember],
    poll_interval: Duration,
    write_deadline: Duration,
) -> Result<(), WriteConvergenceInvariantError> {
    let setup_errors = members
        .iter()
        .map(|member| async {
            member
                .client
                .batch_execute(CREATE_FIXTURE_TABLE_SQL)
                .await
                .map_err(|err| format!("`{}`: {err}", member.member))?;
            member
                .client
                .execute(RESET_FIXTURE_ROW_SQL, &[&FIXTURE_ROW_ID])
                .await
                .map(|_| ())
                .map_err(|err| format!("`{}`: {err}", member.member))
        })
        .collect::<Vec<_>>();
    let mut write_setup_errors = Vec::new();
    for setup_result in futures::future::join_all(setup_errors).await {
        if let Err(err) = setup_result {
            write_setup_errors.push(err);
        } else {
            return wait_for_fixture_visibility(members, poll_interval, write_deadline).await;
        }
    }
    Err(WriteConvergenceInvariantError::Failed(format!(
        "failed to initialize `{FIXTURE_TABLE}` on any member: {}",
        write_setup_errors.join("; ")
    )))
}

async fn wait_for_fixture_visibility(
    members: &[ConnectedMember],
    poll_interval: Duration,
    write_deadline: Duration,
) -> Result<(), WriteConvergenceInvariantError> {
    let deadline = Instant::now() + write_deadline;
    loop {
        let observations = futures::future::join_all(members.iter().map(|member| async {
            match read_count(&member.client).await {
                Ok(count) => MemberCountObservation::Observed {
                    member: member.member,
                    count,
                },
                Err(err) => MemberCountObservation::Failed {
                    member: member.member,
                    message: err.to_string(),
                },
            }
        }))
        .await;
        if observations_match_expected(observations.as_slice(), 0) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(WriteConvergenceInvariantError::Failed(format!(
                "fixture row `{FIXTURE_ROW_ID}` did not become visible on all members before {:?}; observed: {}",
                write_deadline,
                render_observations(observations.as_slice()),
            )));
        }
        tokio::time::sleep(poll_interval).await;
    }
}

async fn writer_loop(
    client: Arc<Client>,
    pause_write: Arc<RwLock<()>>,
    written_count: Arc<AtomicU64>,
    poll_interval: Duration,
) {
    loop {
        {
            let _pause_guard = pause_write.read().await;
            if matches!(
                client
                    .execute(INCREMENT_FIXTURE_ROW_SQL, &[&FIXTURE_ROW_ID])
                    .await,
                Ok(1)
            ) {
                written_count.fetch_add(1, Ordering::SeqCst);
            }
        }
        tokio::time::sleep(poll_interval).await;
    }
}

async fn monitor_loop(
    poll_interval: Duration,
    write_deadline: Duration,
    pause_write: Arc<RwLock<()>>,
    written_count: Arc<AtomicU64>,
    members: Vec<MonitoredMember>,
    health_failure: Arc<Mutex<Option<String>>>,
) {
    let first_success_deadline = Instant::now() + write_deadline;
    loop {
        if written_count.load(Ordering::SeqCst) == 0 {
            if Instant::now() >= first_success_deadline {
                store_health_failure(
                    health_failure.as_ref(),
                    format!(
                        "no successful writes observed on `{FIXTURE_TABLE}` row `{FIXTURE_ROW_ID}` before {:?}",
                        write_deadline
                    ),
                );
                return;
            }
            tokio::time::sleep(poll_interval).await;
            continue;
        }

        {
            let _pause_guard = pause_write.write().await;
            if let Err(err) = wait_for_convergence(
                members.as_slice(),
                written_count.load(Ordering::SeqCst),
                poll_interval,
                write_deadline,
            )
            .await
            {
                store_health_failure(health_failure.as_ref(), err.to_string());
                return;
            }
        }
        tokio::time::sleep(poll_interval).await;
    }
}

async fn read_member_counts(members: &[MemberWriter]) -> Vec<MemberCountObservation> {
    futures::future::join_all(members.iter().map(|member| async {
        match read_count(&member.client).await {
            Ok(count) => MemberCountObservation::Observed {
                member: member.member,
                count,
            },
            Err(err) => MemberCountObservation::Failed {
                member: member.member,
                message: err.to_string(),
            },
        }
    }))
    .await
}

async fn read_monitored_member_counts(members: &[MonitoredMember]) -> Vec<MemberCountObservation> {
    futures::future::join_all(members.iter().map(|member| async {
        match read_count(&member.client).await {
            Ok(count) => MemberCountObservation::Observed {
                member: member.member,
                count,
            },
            Err(err) => MemberCountObservation::Failed {
                member: member.member,
                message: err.to_string(),
            },
        }
    }))
    .await
}

async fn wait_for_convergence(
    members: &[MonitoredMember],
    expected_count: u64,
    poll_interval: Duration,
    write_deadline: Duration,
) -> Result<(), WriteConvergenceInvariantError> {
    let deadline = Instant::now() + write_deadline;
    loop {
        let observations = read_monitored_member_counts(members).await;
        if observations_match_expected(observations.as_slice(), expected_count) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(convergence_failure(
                expected_count,
                observations.as_slice(),
                write_deadline,
            ));
        }
        tokio::time::sleep(poll_interval).await;
    }
}

async fn read_count(client: &Client) -> Result<u64, WriteConvergenceInvariantError> {
    let row = client
        .query_opt(SELECT_FIXTURE_ROW_SQL, &[&FIXTURE_ROW_ID])
        .await
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

fn build_tls_connector(dsn: &str) -> Result<MakeRustlsConnect, WriteConvergenceInvariantError> {
    let root_cert_path = dsn_parameter(dsn, "sslrootcert")?;
    let client_cert_path = dsn_parameter(dsn, "sslcert")?;
    let client_key_path = dsn_parameter(dsn, "sslkey")?;

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

fn dsn_parameter(dsn: &str, key: &str) -> Result<String, WriteConvergenceInvariantError> {
    dsn.split(' ')
        .find_map(|segment| segment.split_once('='))
        .and_then(|(segment_key, value)| (segment_key == key).then(|| value.to_string()))
        .ok_or_else(|| {
            WriteConvergenceInvariantError::Failed(format!("dsn did not contain `{key}`: {dsn}"))
        })
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
    fn matches_expected(&self, expected: u64) -> bool {
        match self {
            Self::Observed { count, .. } => *count == expected,
            Self::Failed { .. } => false,
        }
    }

    fn render(&self) -> String {
        match self {
            Self::Observed { member, count } => format!("`{member}`={count}"),
            Self::Failed { member, message } => format!("`{member}` error={message}"),
        }
    }
}

fn observations_match_expected(
    observations: &[MemberCountObservation],
    expected_count: u64,
) -> bool {
    observations
        .iter()
        .all(|observation| observation.matches_expected(expected_count))
}

fn render_observations(observations: &[MemberCountObservation]) -> String {
    observations
        .iter()
        .map(MemberCountObservation::render)
        .collect::<Vec<_>>()
        .join(", ")
}

fn convergence_failure(
    expected_count: u64,
    observations: &[MemberCountObservation],
    write_deadline: Duration,
) -> WriteConvergenceInvariantError {
    WriteConvergenceInvariantError::Failed(format!(
        "expected all members to converge to `{expected_count}` on `{FIXTURE_TABLE}` row `{FIXTURE_ROW_ID}` before {:?}; observed: {}",
        write_deadline,
        render_observations(observations),
    ))
}

fn store_health_failure(failure: &Mutex<Option<String>>, message: String) {
    if let Ok(mut slot) = failure.lock() {
        *slot = Some(message);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        convergence_failure, observations_match_expected, read_count, writer_loop,
        MemberCountObservation, MemberWriter, WriteConvergenceInvariantError,
        WriteConvergenceInvariantRunner, CREATE_FIXTURE_TABLE_SQL, FIXTURE_ROW_ID,
    };
    use crate::support::topology::ClusterMember;
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
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc, Mutex,
        },
        time::Duration,
    };
    use tokio::{sync::RwLock, task::JoinHandle, time::Instant};
    use tokio_postgres::{Client, NoTls};

    #[test]
    fn convergence_failure_reports_dual_primary_style_divergence() {
        let observations = vec![
            MemberCountObservation::Observed {
                member: ClusterMember::NodeA,
                count: 1,
            },
            MemberCountObservation::Observed {
                member: ClusterMember::NodeB,
                count: 1,
            },
            MemberCountObservation::Observed {
                member: ClusterMember::NodeC,
                count: 1,
            },
        ];

        assert!(!observations_match_expected(observations.as_slice(), 2));

        let err = convergence_failure(2, observations.as_slice(), Duration::from_millis(10));

        match err {
            WriteConvergenceInvariantError::Failed(message) => {
                assert!(message.contains("converge to `2`"));
                assert!(message.contains("`node-a`=1"));
                assert!(message.contains("`node-b`=1"));
                assert!(message.contains("`node-c`=1"));
            }
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn one_primary_and_two_replicas_are_determined_healthy(
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut fixture = RealPostgresFixture::spawn("write-convergence-loop").await?;
        let run_result = async {
            let pause_write = Arc::new(RwLock::new(()));
            let written_count = Arc::new(AtomicU64::new(0));
            initialize_fixture_row_via_dsn(fixture.dsn.as_str(), 0).await?;
            let primary_probe = connect_session(fixture.dsn.as_str(), false).await?;
            let replica_probe = connect_session(fixture.dsn.as_str(), true).await?;
            let runner = WriteConvergenceInvariantRunner::new(
                Duration::from_millis(10),
                Duration::from_millis(250),
                Arc::clone(&pause_write),
                Arc::clone(&written_count),
                vec![
                    build_member_writer(
                        ClusterMember::NodeA,
                        fixture.dsn.as_str(),
                        false,
                        Arc::clone(&pause_write),
                        Arc::clone(&written_count),
                        Duration::from_millis(10),
                    )
                    .await?,
                    build_member_writer(
                        ClusterMember::NodeB,
                        fixture.dsn.as_str(),
                        true,
                        Arc::clone(&pause_write),
                        Arc::clone(&written_count),
                        Duration::from_millis(10),
                    )
                    .await?,
                    build_member_writer(
                        ClusterMember::NodeC,
                        fixture.dsn.as_str(),
                        true,
                        Arc::clone(&pause_write),
                        Arc::clone(&written_count),
                        Duration::from_millis(10),
                    )
                    .await?,
                ],
            );

            wait_for_counter_at_least(written_count.as_ref(), 3, Duration::from_secs(5)).await?;
            wait_for_row_count_at_least(primary_probe.client.as_ref(), 3, Duration::from_secs(5))
                .await?;
            wait_for_row_count_at_least(replica_probe.client.as_ref(), 3, Duration::from_secs(5))
                .await?;

            runner.ensure_healthy()?;
            let primary_count = read_count(primary_probe.client.as_ref()).await?;
            let replica_count = read_count(replica_probe.client.as_ref()).await?;
            let shared = written_count.load(Ordering::SeqCst);
            drop(runner);
            drop(primary_probe);
            drop(replica_probe);
            assert_eq!(primary_count, shared);
            assert_eq!(replica_count, shared);
            Ok(())
        }
        .await;

        fixture.handle.shutdown().await?;
        run_result
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn three_replicas_are_not_determined_healthy() -> Result<(), Box<dyn Error + Send + Sync>>
    {
        let mut fixture = RealPostgresFixture::spawn("write-convergence-three-replicas").await?;
        let run_result = async {
            let pause_write = Arc::new(RwLock::new(()));
            let written_count = Arc::new(AtomicU64::new(0));
            initialize_fixture_row_via_dsn(fixture.dsn.as_str(), 0).await?;
            let runner = WriteConvergenceInvariantRunner::new(
                Duration::from_millis(10),
                Duration::from_millis(150),
                Arc::clone(&pause_write),
                Arc::clone(&written_count),
                vec![
                    build_member_writer(
                        ClusterMember::NodeA,
                        fixture.dsn.as_str(),
                        true,
                        Arc::clone(&pause_write),
                        Arc::clone(&written_count),
                        Duration::from_millis(10),
                    )
                    .await?,
                    build_member_writer(
                        ClusterMember::NodeB,
                        fixture.dsn.as_str(),
                        true,
                        Arc::clone(&pause_write),
                        Arc::clone(&written_count),
                        Duration::from_millis(10),
                    )
                    .await?,
                    build_member_writer(
                        ClusterMember::NodeC,
                        fixture.dsn.as_str(),
                        true,
                        Arc::clone(&pause_write),
                        Arc::clone(&written_count),
                        Duration::from_millis(10),
                    )
                    .await?,
                ],
            );

            let background_message =
                wait_for_background_failure(&runner, Duration::from_secs(2)).await?;
            let result = runner.ensure_healthy();
            drop(runner);

            assert!(background_message.contains("no successful writes observed"));
            match result {
                Ok(()) => Err(IoError::other("expected ensure_healthy to fail").into()),
                Err(WriteConvergenceInvariantError::Failed(message)) => {
                    assert!(message.contains("no successful writes observed"));
                    Ok(())
                }
            }
        }
        .await;

        fixture.handle.shutdown().await?;
        run_result
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn two_primaries_and_one_replica_fail_in_background_without_ensure_healthy(
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut fixture_a = RealPostgresFixture::spawn("write-convergence-primary-a").await?;
        let mut fixture_b = RealPostgresFixture::spawn("write-convergence-primary-b").await?;
        let run_result = async {
            let pause_write = Arc::new(RwLock::new(()));
            let written_count = Arc::new(AtomicU64::new(0));
            initialize_fixture_row_via_dsn(fixture_a.dsn.as_str(), 0).await?;
            initialize_fixture_row_via_dsn(fixture_b.dsn.as_str(), 0).await?;
            let runner = WriteConvergenceInvariantRunner::new(
                Duration::from_millis(10),
                Duration::from_millis(150),
                Arc::clone(&pause_write),
                Arc::clone(&written_count),
                vec![
                    build_member_writer(
                        ClusterMember::NodeA,
                        fixture_a.dsn.as_str(),
                        false,
                        Arc::clone(&pause_write),
                        Arc::clone(&written_count),
                        Duration::from_millis(10),
                    )
                    .await?,
                    build_member_writer(
                        ClusterMember::NodeB,
                        fixture_b.dsn.as_str(),
                        false,
                        Arc::clone(&pause_write),
                        Arc::clone(&written_count),
                        Duration::from_millis(10),
                    )
                    .await?,
                    build_member_writer(
                        ClusterMember::NodeC,
                        fixture_a.dsn.as_str(),
                        true,
                        Arc::clone(&pause_write),
                        Arc::clone(&written_count),
                        Duration::from_millis(10),
                    )
                    .await?,
                ],
            );

            let background_message =
                wait_for_background_failure(&runner, Duration::from_secs(2)).await?;
            drop(runner);

            assert!(background_message.contains("converge to"));
            assert!(background_message.contains("`node-a`="));
            assert!(background_message.contains("`node-b`="));
            assert!(background_message.contains("`node-c`="));
            Ok(())
        }
        .await;

        fixture_a.handle.shutdown().await?;
        fixture_b.handle.shutdown().await?;
        run_result
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

            let dsn = format!("host=127.0.0.1 port={port} user=postgres dbname=postgres");
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
        connection_task: JoinHandle<()>,
    }

    impl Drop for SessionHandle {
        fn drop(&mut self) {
            self.connection_task.abort();
        }
    }

    async fn connect_client(
        dsn: &str,
    ) -> Result<
        (Arc<Client>, Arc<Mutex<Option<String>>>, JoinHandle<()>),
        Box<dyn Error + Send + Sync>,
    > {
        let (client, connection) = tokio_postgres::connect(dsn, NoTls).await?;
        let fatal_error = Arc::new(Mutex::new(None));
        let fatal_error_for_task = Arc::clone(&fatal_error);
        let connection_task = tokio::spawn(async move {
            if let Err(err) = connection.await {
                if let Ok(mut slot) = fatal_error_for_task.lock() {
                    *slot = Some(err.to_string());
                }
            }
        });
        Ok((Arc::new(client), fatal_error, connection_task))
    }

    async fn connect_session(
        dsn: &str,
        read_only: bool,
    ) -> Result<SessionHandle, Box<dyn Error + Send + Sync>> {
        let (client, _fatal_error, connection_task) = connect_client(dsn).await?;
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
        pause_write: Arc<RwLock<()>>,
        written_count: Arc<AtomicU64>,
        poll_interval: Duration,
    ) -> Result<MemberWriter, Box<dyn Error + Send + Sync>> {
        let (client, fatal_error, connection_task) = connect_client(dsn).await?;
        if read_only {
            client
                .simple_query("SET default_transaction_read_only = on")
                .await?;
        }
        let writer_task = tokio::spawn(writer_loop(
            Arc::clone(&client),
            pause_write,
            written_count,
            poll_interval,
        ));
        Ok(MemberWriter {
            member,
            client,
            fatal_error,
            connection_task,
            writer_task,
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

    async fn wait_for_counter_at_least(
        counter: &AtomicU64,
        minimum: u64,
        timeout: Duration,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let deadline = Instant::now() + timeout;
        loop {
            let observed = counter.load(Ordering::SeqCst);
            if observed >= minimum {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(IoError::other(format!(
                    "timed out waiting for shared counter >= {minimum}; observed {observed}"
                ))
                .into());
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    async fn wait_for_row_count_at_least(
        client: &Client,
        minimum: u64,
        timeout: Duration,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let deadline = Instant::now() + timeout;
        loop {
            let observed = read_count(client).await?;
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

    async fn wait_for_background_failure(
        runner: &WriteConvergenceInvariantRunner,
        timeout: Duration,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let deadline = Instant::now() + timeout;
        loop {
            match runner.health_failure() {
                Ok(()) => {}
                Err(WriteConvergenceInvariantError::Failed(message)) => return Ok(message),
            }
            if Instant::now() >= deadline {
                return Err(IoError::other("timed out waiting for background failure").into());
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
