use std::{
    fs,
    io::Cursor,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use rustls::{
    self,
    pki_types::{CertificateDer, PrivateKeyDer},
    RootCertStore,
};
use tokio::{
    runtime::{Builder, Handle, RuntimeFlavor},
    sync::RwLock,
    task::JoinHandle,
    time::Instant,
};
use tokio_postgres::{Client, NoTls};
use tokio_postgres_rustls::MakeRustlsConnect;

use pgtuskmaster_rust::pginfo::conninfo::{
    conninfo_entries, conninfo_value, render_conninfo_value,
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
    written_count: Arc<AtomicU64>,
    members: Vec<MemberWorker>,
}

#[derive(Debug, thiserror::Error)]
pub enum WriteConvergenceInvariantError {
    #[error("write-convergence invariant failed: {0}")]
    Failed(String),
}

struct MemberWorker {
    member: ClusterMember,
    routing_target: PostgresRoutingTarget,
    task: JoinHandle<()>,
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

        let pause_write = Arc::new(RwLock::new(()));
        let written_count = Arc::new(AtomicU64::new(0));
        let members = routing_targets
            .into_iter()
            .map(|routing_target| {
                spawn_member_worker(
                    routing_target,
                    Arc::clone(&pause_write),
                    Arc::clone(&written_count),
                    poll_interval,
                )
            })
            .collect::<Vec<_>>();
        Ok(Self::new(
            Some(observer),
            poll_interval,
            write_deadline,
            written_count,
            members,
        ))
    }

    pub fn ensure_healthy(&self) -> Result<(), WriteConvergenceInvariantError> {
        self.members.iter().for_each(|member| member.task.abort());
        let write_deadline = self.write_deadline;
        let poll_interval = self.poll_interval;
        let written_count = Arc::clone(&self.written_count);
        let members = member_observation_targets(self.members.as_slice(), self.observer.clone());
        let future = async move {
            let expected_count = written_count.load(Ordering::SeqCst);
            if expected_count == 0 {
                return Ok(());
            }
            wait_for_convergence(
                members.as_slice(),
                expected_count,
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
        self.members.iter().for_each(|member| member.task.abort());
    }
}

impl std::fmt::Debug for MemberWorker {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MemberWorker")
            .field("member", &self.member)
            .finish()
    }
}

impl WriteConvergenceInvariantRunner {
    fn new(
        observer: Option<PgtmObserver>,
        poll_interval: Duration,
        write_deadline: Duration,
        written_count: Arc<AtomicU64>,
        members: Vec<MemberWorker>,
    ) -> Self {
        Self {
            observer,
            poll_interval,
            write_deadline,
            written_count,
            members,
        }
    }
}

struct MemberObservationTarget {
    member: ClusterMember,
    observer: Option<PgtmObserver>,
    routing_target: PostgresRoutingTarget,
}

async fn connect_member(
    routing_target: &PostgresRoutingTarget,
    connect_timeout: Duration,
) -> Result<(Arc<Client>, ConnectionTask), String> {
    let connect_dsn =
        connectable_dsn(routing_target.dsn.as_str()).map_err(|err| err.to_string())?;
    if dsn_uses_tls_files(routing_target.dsn.as_str())? {
        let tls =
            build_tls_connector(routing_target.dsn.as_str()).map_err(|err| err.to_string())?;
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
    pause_write: Arc<RwLock<()>>,
    written_count: Arc<AtomicU64>,
    poll_interval: Duration,
) -> MemberWorker {
    let member = routing_target.member;
    let task = tokio::spawn(run_member_worker(
        routing_target.clone(),
        pause_write,
        written_count,
        poll_interval,
    ));
    MemberWorker {
        member,
        routing_target,
        task,
    }
}

async fn run_member_worker(
    routing_target: PostgresRoutingTarget,
    pause_write: Arc<RwLock<()>>,
    written_count: Arc<AtomicU64>,
    poll_interval: Duration,
) {
    loop {
        match connect_member(&routing_target, poll_interval).await {
            Ok((client, connection_task)) => {
                maintain_connected_member(
                    client,
                    connection_task,
                    Arc::clone(&pause_write),
                    Arc::clone(&written_count),
                    poll_interval,
                    poll_interval,
                )
                .await;
            }
            Err(err) => {
                let _ = err;
                tokio::time::sleep(poll_interval).await;
            }
        }
    }
}

async fn maintain_connected_member(
    client: Arc<Client>,
    mut connection_task: ConnectionTask,
    pause_write: Arc<RwLock<()>>,
    written_count: Arc<AtomicU64>,
    poll_interval: Duration,
    query_timeout: Duration,
) {
    loop {
        tokio::select! {
            _ = tokio::time::sleep(poll_interval) => {
                let _pause_guard = pause_write.read().await;
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
                    Ok(Ok(1)) => {
                        written_count.fetch_add(1, Ordering::SeqCst);
                    }
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
            }
            connection_result = &mut connection_task => {
                let _ = connection_result;
                return;
            }
        }
    }
}

fn member_observation_targets(
    members: &[MemberWorker],
    observer: Option<PgtmObserver>,
) -> Vec<MemberObservationTarget> {
    members
        .iter()
        .map(|member| MemberObservationTarget {
            member: member.member,
            observer: observer.clone(),
            routing_target: member.routing_target.clone(),
        })
        .collect()
}

async fn read_monitored_member_counts(
    members: &[MemberObservationTarget],
    query_timeout: Duration,
) -> Vec<MemberCountObservation> {
    futures::future::join_all(
        members
            .iter()
            .map(|member| read_member_count(member, query_timeout)),
    )
    .await
}

async fn read_member_count(
    member: &MemberObservationTarget,
    query_timeout: Duration,
) -> MemberCountObservation {
    read_member_count_via_fresh_connection(member, query_timeout, None).await
}

async fn read_member_count_via_fresh_connection(
    member: &MemberObservationTarget,
    connect_timeout: Duration,
    previous_error: Option<String>,
) -> MemberCountObservation {
    let routing_target = match resolve_observation_routing_target(member) {
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
    member: &MemberObservationTarget,
) -> Result<PostgresRoutingTarget, String> {
    member.observer.as_ref().map_or_else(
        || Ok(member.routing_target.clone()),
        |observer| {
            observer
                .postgres_routing_target(member.member)
                .map_err(|err| err.to_string())
        },
    )
}

async fn wait_for_convergence(
    members: &[MemberObservationTarget],
    expected_count: u64,
    poll_interval: Duration,
    write_deadline: Duration,
) -> Result<(), WriteConvergenceInvariantError> {
    let deadline = Instant::now() + write_deadline;
    loop {
        let observations = read_monitored_member_counts(members, poll_interval).await;
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

fn build_tls_connector(dsn: &str) -> Result<MakeRustlsConnect, WriteConvergenceInvariantError> {
    let root_cert_path = required_conninfo_value(dsn, "sslrootcert")?;
    let client_cert_path = required_conninfo_value(dsn, "sslcert")?;
    let client_key_path = required_conninfo_value(dsn, "sslkey")?;

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

fn required_conninfo_value(dsn: &str, key: &str) -> Result<String, WriteConvergenceInvariantError> {
    conninfo_value(dsn, key)
        .map_err(WriteConvergenceInvariantError::Failed)?
        .ok_or_else(|| {
            WriteConvergenceInvariantError::Failed(format!("dsn did not contain `{key}`: {dsn}"))
        })
}

fn dsn_uses_tls_files(dsn: &str) -> Result<bool, String> {
    ["sslrootcert", "sslcert", "sslkey"]
        .into_iter()
        .try_fold(false, |uses_tls_files, key| {
            conninfo_value(dsn, key).map(|value| uses_tls_files || value.is_some())
        })
}

fn connectable_dsn(dsn: &str) -> Result<String, WriteConvergenceInvariantError> {
    conninfo_entries(dsn)
        .map_err(WriteConvergenceInvariantError::Failed)
        .map(|entries| {
            entries
                .into_iter()
                .filter_map(|(key, value)| match key.as_str() {
                    "sslrootcert" | "sslcert" | "sslkey" => None,
                    "sslmode" if matches!(value.as_str(), "verify-ca" | "verify-full") => {
                        Some((key, "require".to_string()))
                    }
                    _ => Some((key, value)),
                })
                .map(|(key, value)| format!("{key}={}", render_conninfo_value(value.as_str())))
                .collect::<Vec<_>>()
                .join(" ")
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

#[cfg(test)]
mod tests {
    use super::{
        connectable_dsn, convergence_failure, maintain_connected_member,
        observations_match_expected, read_count, required_conninfo_value, ConnectionTask,
        MemberCountObservation, MemberWorker, WriteConvergenceInvariantError,
        WriteConvergenceInvariantRunner, CREATE_FIXTURE_TABLE_SQL, FIXTURE_ROW_ID,
    };
    use crate::support::{observer::pgtm::PostgresRoutingTarget, topology::ClusterMember};
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
            Arc,
        },
        time::Duration,
    };
    use tokio::{sync::RwLock, time::Instant};
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

    #[test]
    fn required_conninfo_value_accepts_tls_paths_beyond_first_pair(
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let dsn = concat!(
            "host=node-a ",
            "hostaddr=127.0.0.1 ",
            "port=5432 ",
            "user=postgres ",
            "dbname=postgres ",
            "sslmode=verify-full ",
            "sslrootcert='/tmp/ca bundle.pem' ",
            "sslcert=/tmp/client.crt ",
            "sslkey=/tmp/client.key"
        );

        assert_eq!(
            required_conninfo_value(dsn, "sslrootcert")?,
            "/tmp/ca bundle.pem".to_string()
        );
        assert_eq!(
            required_conninfo_value(dsn, "sslcert")?,
            "/tmp/client.crt".to_string()
        );
        assert_eq!(
            required_conninfo_value(dsn, "sslkey")?,
            "/tmp/client.key".to_string()
        );
        Ok(())
    }

    #[test]
    fn connectable_dsn_strips_tls_path_fields_but_preserves_general_conninfo(
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let dsn = concat!(
            "host=node-a ",
            "hostaddr=127.0.0.1 ",
            "port=5432 ",
            "user=postgres ",
            "dbname='postgres db' ",
            "sslmode=verify-full ",
            "sslrootcert='/tmp/ca bundle.pem' ",
            "sslcert=/tmp/client.crt ",
            "sslkey=/tmp/client.key"
        );

        let connect_dsn = connectable_dsn(dsn)?;

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
            let pause_write = Arc::new(RwLock::new(()));
            let written_count = Arc::new(AtomicU64::new(0));
            initialize_fixture_row_via_dsn(fixture.dsn.as_str(), 0).await?;
            let primary_probe = connect_session(fixture.dsn.as_str(), false).await?;
            let replica_probe = connect_session(fixture.dsn.as_str(), true).await?;
            let runner = WriteConvergenceInvariantRunner::new(
                None,
                Duration::from_millis(10),
                Duration::from_millis(250),
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
            let primary_count =
                read_count(primary_probe.client.as_ref(), Duration::from_millis(250)).await?;
            let replica_count =
                read_count(replica_probe.client.as_ref(), Duration::from_millis(250)).await?;
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
        let mut fixture_a = RealPostgresFixture::spawn("write-convergence-standalone-a").await?;
        let mut fixture_b = RealPostgresFixture::spawn("write-convergence-standalone-b").await?;
        let run_result = async {
            let pause_write = Arc::new(RwLock::new(()));
            let written_count = Arc::new(AtomicU64::new(0));
            initialize_fixture_row_via_dsn(fixture_a.dsn.as_str(), 0).await?;
            initialize_fixture_row_via_dsn(fixture_b.dsn.as_str(), 0).await?;
            let runner = WriteConvergenceInvariantRunner::new(
                None,
                Duration::from_millis(10),
                Duration::from_millis(150),
                Arc::clone(&written_count),
                vec![
                    build_member_writer(
                        ClusterMember::NodeA,
                        fixture_a.dsn.as_str(),
                        true,
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
                        fixture_b.dsn.as_str(),
                        true,
                        Arc::clone(&pause_write),
                        Arc::clone(&written_count),
                        Duration::from_millis(10),
                    )
                    .await?,
                ],
            );

            wait_for_counter_at_least(written_count.as_ref(), 1, Duration::from_secs(5)).await?;
            let result = runner.ensure_healthy();
            drop(runner);
            match result {
                Ok(()) => Err(IoError::other("expected ensure_healthy to fail").into()),
                Err(WriteConvergenceInvariantError::Failed(message)) => {
                    assert!(message.contains("converge to"));
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
            let pause_write = Arc::new(RwLock::new(()));
            let written_count = Arc::new(AtomicU64::new(0));
            initialize_fixture_row_via_dsn(fixture.dsn.as_str(), 0).await?;
            let runner = WriteConvergenceInvariantRunner::new(
                None,
                Duration::from_millis(10),
                Duration::from_millis(150),
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

            tokio::time::sleep(Duration::from_millis(50)).await;
            runner.ensure_healthy()?;
            drop(runner);
            Ok(())
        }
        .await;

        fixture.handle.shutdown().await?;
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
        pause_write: Arc<RwLock<()>>,
        written_count: Arc<AtomicU64>,
        poll_interval: Duration,
    ) -> Result<MemberWorker, Box<dyn Error + Send + Sync>> {
        let (client, connection_task) = connect_client(dsn).await?;
        if read_only {
            client
                .simple_query("SET default_transaction_read_only = on")
                .await?;
        }
        let task = tokio::spawn(maintain_connected_member(
            Arc::clone(&client),
            connection_task,
            pause_write,
            written_count,
            poll_interval,
            poll_interval,
        ));
        Ok(MemberWorker {
            member,
            routing_target: PostgresRoutingTarget {
                member,
                dsn: dsn.to_string(),
            },
            task,
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
