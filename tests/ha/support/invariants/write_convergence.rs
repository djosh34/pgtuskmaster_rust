use std::time::Duration;
use tokio::{
    runtime::Runtime,
    sync::{mpsc, oneshot},
    task::JoinError,
    time::{self, timeout},
};
use tokio_postgres::{Client, NoTls};

use crate::support::{
    error::HarnessError,
    observer::pgtm::{PostgresRoutingTarget, PgtmObserver},
    topology::ClusterMember,
};

const SELECT_ONE_SQL: &str = "SELECT 1";

#[derive(Debug)]
pub struct WriteConvergenceInvariantRunner {
    request_sender:
        mpsc::UnboundedSender<oneshot::Sender<Result<bool, WriteConvergenceInvariantError>>>,
    write_deadline: Duration,
    runtime: Runtime,
}

#[derive(Debug, thiserror::Error)]
pub enum WriteConvergenceInvariantError {
    #[error("failed to initialize tokio runtime for write-convergence invariant: {source}")]
    RuntimeInit { source: std::io::Error },

    #[error("write-convergence invariant request could not be dispatched")]
    HealthRequestDispatchFailed,

    #[error("write-convergence invariants request timed out after {deadline:?}")]
    HealthCheckTimedOut { deadline: Duration },

    #[error("write-convergence invariants response channel was closed")]
    HealthRequestChannelClosed,

    #[error("failed to collect write-convergence probe target for `{member}`: {source}")]
    ProbeTarget {
        member: ClusterMember,
        #[source]
        source: Box<HarnessError>,
    },

    #[error("write-convergence invariant check failed")]
    HealthRequestUnhealthy,
    

    #[error("write-convergence probe for {member} failed to connect to `{dsn}`: {source}")]
    ProbeConnectFailed {
        member: ClusterMember,
        dsn: String,
        #[source]
        source: tokio_postgres::Error,
    },

    #[error("write-convergence invariant expected {expected} probes, got {actual}")]
    ProbeCountMismatch {
        expected: usize,
        actual: usize,
    },

    #[error("write-convergence probe for `{member}` failed while executing SELECT 1: {source}")]
    ProbeQueryFailed {
        member: ClusterMember,
        #[source]
        source: tokio_postgres::Error,
    },

    #[error("write-convergence probe task failed while joining: {context}: {source}")]
    TaskJoinFailed {
        context: &'static str,
        #[source]
        source: JoinError,
    },
}

struct Probe {
    member: ClusterMember,
    client: Client,
}

impl WriteConvergenceInvariantRunner {
    pub fn start(
        observer: PgtmObserver,
        poll_interval: Duration,
        write_deadline: Duration,
    ) -> Result<Self, WriteConvergenceInvariantError> {
        let runtime = Runtime::new().map_err(|source| WriteConvergenceInvariantError::RuntimeInit {
            source,
        })?;

        let (request_sender, mut request_receiver) =
            mpsc::unbounded_channel::<oneshot::Sender<Result<bool, WriteConvergenceInvariantError>>>();

        let invariant_task = runtime.spawn(async move {
            let mut probes: Vec<Probe> = Vec::new();
            let mut interval = time::interval(poll_interval);
            let mut current_health = false;

            loop {
                tokio::select! {
                    maybe_sender = request_receiver.recv() => match maybe_sender {
                        Some(response_sender) => {
                            let _ = response_sender.send(Ok(current_health));
                        }
                        None => break,
                    },
                    _ = interval.tick() => {
                        current_health = poll_probes(&observer, &mut probes).await.unwrap_or_else(|err| {
                                eprintln!("{err}");
                                false
                            });
                    },
                }
            }
        });
        drop(invariant_task);

        Ok(Self {
            request_sender,
            write_deadline,
            runtime,
        })
    }

    pub fn ensure_healthy(&self) -> Result<(), WriteConvergenceInvariantError> {
        let (response_sender, response_receiver) =
            oneshot::channel::<Result<bool, WriteConvergenceInvariantError>>();

        self.request_sender
            .send(response_sender)
            .map_err(|_| WriteConvergenceInvariantError::HealthRequestDispatchFailed)?;

        self.runtime
            .block_on(timeout(self.write_deadline, response_receiver))
            .map_err(|_| WriteConvergenceInvariantError::HealthCheckTimedOut {
                deadline: self.write_deadline,
            })
            .and_then(|probe_health| {
                probe_health.map_err(|_| WriteConvergenceInvariantError::HealthRequestChannelClosed)?
            })
            .and_then(|is_healthy| {
                if is_healthy {
                    Ok(())
                } else {
                    Err(WriteConvergenceInvariantError::HealthRequestUnhealthy)
                }
            })
    }
}

async fn poll_probes(
    observer: &PgtmObserver,
    probes: &mut Vec<Probe>,
) -> Result<bool, WriteConvergenceInvariantError> {
    if probes.is_empty() {
        return initialize_probes(observer, probes).await;
    }

    match query_all(std::mem::take(probes)).await {
        Ok(next_probes) => {
            *probes = next_probes;
            Ok(true)
        }
        Err(err) => {
            eprintln!("{err}");
            eprintln!("write_convergence invariant query failed, reconnecting");
            probes.clear();
            initialize_probes(observer, probes).await
        }
    }
}

async fn initialize_probes(
    observer: &PgtmObserver,
    probes: &mut Vec<Probe>,
) -> Result<bool, WriteConvergenceInvariantError> {
    let mut connect_tasks = tokio::task::JoinSet::new();

    for member in ClusterMember::ALL {
        let target = observer
            .postgres_routing_target(member)
            .map_err(|source| WriteConvergenceInvariantError::ProbeTarget {
                member,
                source: Box::new(source),
            })?;
        connect_tasks.spawn(async move { connect_probe(target).await });
    }

    let next_probes = collect_joinset(
        connect_tasks,
        "write_convergence probe connection task failed",
    )
    .await?;

    if next_probes.len() != ClusterMember::ALL.len() {
        return Err(WriteConvergenceInvariantError::ProbeCountMismatch {
            expected: ClusterMember::ALL.len(),
            actual: next_probes.len(),
        });
    }

    query_all(next_probes).await.map(|next_probes| {
        *probes = next_probes;
        true
    })
}

async fn connect_probe(target: PostgresRoutingTarget) -> Result<Probe, WriteConvergenceInvariantError> {
    let dsn = target.dsn.to_string();
    let (client, connection) = tokio_postgres::connect(&dsn, NoTls)
        .await
        .map_err(|source| WriteConvergenceInvariantError::ProbeConnectFailed {
            member: target.member,
            dsn,
            source,
        })?;

    let connection_task = tokio::spawn(async move {
        if let Err(source) = connection.await {
            eprintln!("write-convergence probe connection task failed: {source}");
        }
    });
    drop(connection_task);

    Ok(Probe {
        member: target.member,
        client,
    })
}

async fn query_all(
    probes: Vec<Probe>,
) -> Result<Vec<Probe>, WriteConvergenceInvariantError> {
    let mut query_tasks = tokio::task::JoinSet::new();

    for probe in probes {
        query_tasks.spawn(async move {
            let member = probe.member;

            probe
                .client
                .query_one(SELECT_ONE_SQL, &[])
                .await
                .map(|_| probe)
                .map_err(|source| WriteConvergenceInvariantError::ProbeQueryFailed {
                    member,
                    source,
                })
        });
    }

    collect_joinset(
        query_tasks,
        "write-convergence probe task failed while executing SELECT 1",
    )
    .await
}

async fn collect_joinset<T>(
    mut tasks: tokio::task::JoinSet<Result<T, WriteConvergenceInvariantError>>,
    join_error_prefix: &'static str,
) -> Result<Vec<T>, WriteConvergenceInvariantError>
where
    T: 'static,
{
    let mut collected = Vec::new();

    while let Some(task_result) = tasks.join_next().await {
        let value = task_result.map_err(|source| WriteConvergenceInvariantError::TaskJoinFailed {
            context: join_error_prefix,
            source,
        })?;

        collected.push(value?);
    }

    Ok(collected)
}
