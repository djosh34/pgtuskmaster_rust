use std::time::Duration;

use tokio::{
    runtime::Runtime,
    sync::{mpsc, oneshot},
    time::{self, timeout},
};
use tokio_postgres::{Client, NoTls};

use crate::support::{
    error::{HarnessError, Result},
    observer::pgtm::{PostgresRoutingTarget, PgtmObserver},
    topology::ClusterMember,
};

const SELECT_ONE_SQL: &str = "SELECT 1";

pub struct WriteConvergenceInvariantRunner {
    request_sender: mpsc::UnboundedSender<oneshot::Sender<bool>>,
    write_deadline: Duration,
    runtime: Runtime,
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
    ) -> Result<Self> {
        let runtime = Runtime::new().map_err(|source| {
            HarnessError::message(format!(
                "failed to initialize tokio runtime for write convergence invariant: {source}"
            ))
        })?;

        let (request_sender, mut request_receiver) =
            mpsc::unbounded_channel::<oneshot::Sender<bool>>();

        let _ = runtime.spawn(async move {
            let mut probes: Vec<Probe> = Vec::new();
            let mut interval = time::interval(poll_interval);
            let mut current_health = false;

            loop {
                tokio::select! {
                    maybe_sender = request_receiver.recv() => match maybe_sender {
                        Some(response_sender) => {
                            let _ = response_sender.send(current_health);
                        }
                        None => break,
                    },
                    _ = interval.tick() => {
                        current_health = poll_probes(&observer, &mut probes).await;
                    },
                }
            }
        });

        Ok(Self {
            request_sender,
            write_deadline,
            runtime,
        })
    }

    pub fn ensure_healthy(&self) -> bool {
        let (response_sender, response_receiver) = oneshot::channel::<bool>();

        if self.request_sender.send(response_sender).is_err() {
            eprintln!("write-convergence invariant request could not be dispatched");
            return false;
        }

        self
            .runtime
            .block_on(timeout(self.write_deadline, response_receiver))
            .ok()
            .and_then(|result| result.ok())
            .unwrap_or(false)
    }
}

async fn poll_probes(observer: &PgtmObserver, probes: &mut Vec<Probe>) -> bool {
    if probes.is_empty() {
        return initialize_probes(observer, probes).await;
    }

    if query_all(probes).await.is_ok() {
        return true;
    }

    eprintln!("write_convergence invariant query failed, reconnecting");
    probes.clear();
    initialize_probes(observer, probes).await
}

async fn initialize_probes(observer: &PgtmObserver, probes: &mut Vec<Probe>) -> bool {
    let mut next_probes = Vec::with_capacity(ClusterMember::ALL.len());

    for member in ClusterMember::ALL {
        let target = match observer.postgres_routing_target(member) {
            Ok(target) => target,
            Err(err) => {
                eprintln!("write_convergence invariant could not collect probe target: {err}");
                return false;
            }
        };

        match connect_probe(target).await {
            Ok(probe) => next_probes.push(probe),
            Err(err) => {
                eprintln!("write_convergence invariant failed to create probe: {err}");
                return false;
            }
        }
    }

    if next_probes.len() != ClusterMember::ALL.len() {
        eprintln!(
            "write_convergence invariant expected {} probes, got {}",
            ClusterMember::ALL.len(),
            next_probes.len()
        );
        return false;
    }

    *probes = next_probes;
    query_all(probes).await.is_ok()
}

async fn connect_probe(target: PostgresRoutingTarget) -> Result<Probe> {
    let (client, connection) = tokio_postgres::connect(&target.dsn, NoTls)
        .await
        .map_err(|source| {
            HarnessError::message(format!(
                "write-convergence probe for {} failed to connect to `{}`: {source}",
                target.member, target.dsn
            ))
        })?;

    let _ = tokio::spawn(async move {
        if let Err(source) = connection.await {
            eprintln!("write-convergence probe connection task failed: {source}");
        }
    });

    Ok(Probe {
        member: target.member,
        client,
    })
}

async fn query_all(probes: &[Probe]) -> Result<()> {
    for probe in probes {
        probe
            .client
            .query_one(SELECT_ONE_SQL, &[])
            .await
            .map(|_| ())
            .map_err(|source| {
                HarnessError::message(format!(
                    "write-convergence probe for {} failed while executing SELECT 1: {}",
                    probe.member, source
                ))
            })?;
    }
    Ok(())
}
