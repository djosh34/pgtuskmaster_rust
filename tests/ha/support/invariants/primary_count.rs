use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
    time::Duration,
};

use pgtuskmaster_rust::pginfo::state::PgInfoState;
use tokio::{
    runtime::{Builder, Handle},
    sync::RwLock,
    task::JoinHandle,
    time::Instant,
};

use crate::support::{
    error::{HarnessError, Result},
    observer::pgtm::PgtmObserver,
    topology::ClusterMember,
};

type MemberPrimaryObservation = std::result::Result<(ClusterMember, bool), String>;
type ObserveAllMembersFuture = Pin<Box<dyn Future<Output = [MemberPrimaryObservation; 3]> + Send>>;
type ObserveAllMembers = Arc<dyn Fn() -> ObserveAllMembersFuture + Send + Sync>;

pub struct PrimaryCountInvariantRunner {
    poll_interval: Duration,
    health_deadline: Duration,
    num_primaries: Arc<AtomicI32>,
    fatal_error: Arc<RwLock<Option<String>>>,
    task: JoinHandle<()>,
}

impl PrimaryCountInvariantRunner {
    pub async fn start(
        observer: PgtmObserver,
        poll_interval: Duration,
        health_deadline: Duration,
    ) -> Result<Self> {
        let observe_all: ObserveAllMembers = Arc::new(move || {
            let observer = observer.clone();
            Box::pin(async move {
                let (node_a, node_b, node_c) = tokio::join!(
                    observe_member_primary(observer.clone(), ClusterMember::NodeA),
                    observe_member_primary(observer.clone(), ClusterMember::NodeB),
                    observe_member_primary(observer, ClusterMember::NodeC),
                );
                [node_a, node_b, node_c]
            })
        });
        Ok(Self::start_with_observe_all(
            poll_interval,
            health_deadline,
            observe_all,
        ))
    }
}

impl std::fmt::Debug for PrimaryCountInvariantRunner {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PrimaryCountInvariantRunner")
            .field("poll_interval", &self.poll_interval)
            .field("health_deadline", &self.health_deadline)
            .field("num_primaries", &self.num_primaries.load(Ordering::SeqCst))
            .finish()
    }
}

impl Drop for PrimaryCountInvariantRunner {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl PrimaryCountInvariantRunner {
    fn start_with_observe_all(
        poll_interval: Duration,
        health_deadline: Duration,
        observe_all: ObserveAllMembers,
    ) -> Self {
        let num_primaries = Arc::new(AtomicI32::new(0));
        let fatal_error = Arc::new(RwLock::new(None));
        let task_num_primaries = Arc::clone(&num_primaries);
        let task_fatal_error = Arc::clone(&fatal_error);
        let task = tokio::spawn(async move {
            loop {
                let observations = observe_all().await;
                if let Some(message) = observations
                    .iter()
                    .find_map(|outcome| outcome.as_ref().err().cloned())
                {
                    *task_fatal_error.write().await = Some(message);
                    return;
                }

                let primary_members = observations
                    .into_iter()
                    .filter_map(|outcome| match outcome {
                        Ok((member, true)) => Some(member),
                        Ok((_, false)) => None,
                        Err(_) => None,
                    })
                    .collect::<Vec<_>>();
                let num_primaries = primary_members.len() as i32;
                task_num_primaries.store(num_primaries, Ordering::SeqCst);

                if num_primaries > 1 {
                    let members = primary_members
                        .into_iter()
                        .map(|member| format!("`{member}`"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    *task_fatal_error.write().await = Some(format!(
                        "observed `{num_primaries}` self-reported primaries at once: {members}"
                    ));
                    return;
                }

                tokio::time::sleep(poll_interval).await;
            }
        });

        Self {
            poll_interval,
            health_deadline,
            num_primaries,
            fatal_error,
            task,
        }
    }

    pub fn ensure_healthy(&self) -> Result<()> {
        self.block_on(async {
            let deadline = Instant::now() + self.health_deadline;
            loop {
                self.ensure_task_running().await?;

                let num_primaries = self.num_primaries.load(Ordering::SeqCst);
                match validate_primary_count(num_primaries) {
                    Ok(true) => return Ok(()),
                    Ok(false) => {}
                    Err(message) => return Err(HarnessError::message(message)),
                }

                if Instant::now() >= deadline {
                    return Err(HarnessError::message(format!(
                        "timed out waiting for self-reported primary count to become `1` before {:?}; last observed count was `{num_primaries}`",
                        self.health_deadline
                    )));
                }

                tokio::time::sleep(self.poll_interval).await;
            }
        })?
    }
}

impl PrimaryCountInvariantRunner {
    async fn ensure_task_running(&self) -> Result<()> {
        if let Some(message) = self.fatal_error.read().await.clone() {
            return Err(HarnessError::message(message));
        }

        if self.task.is_finished() {
            return Err(HarnessError::message(
                "primary-count runner stopped unexpectedly",
            ));
        }

        Ok(())
    }

    fn block_on<T>(&self, future: impl std::future::Future<Output = T>) -> Result<T> {
        match Handle::try_current() {
            Ok(handle) => Ok(tokio::task::block_in_place(|| handle.block_on(future))),
            Err(_) => Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|err| {
                    HarnessError::message(format!(
                        "build runtime for primary-count invariant failed: {err}"
                    ))
                })
                .map(|runtime| runtime.block_on(future)),
        }
    }
}

async fn observe_member_primary(
    observer: PgtmObserver,
    member: ClusterMember,
) -> MemberPrimaryObservation {
    tokio::task::spawn_blocking(move || {
        observer
            .state_via_member(member)
            .map(|state| (member, matches!(state.pg, PgInfoState::Primary { .. })))
            .or(Ok((member, false)))
    })
    .await
    .map_err(|err| format!("join self-primary observation for `{member}` failed: {err}"))?
}

fn validate_primary_count(num_primaries: i32) -> std::result::Result<bool, String> {
    match num_primaries {
        1 => Ok(true),
        i32::MIN..=0 => Ok(false),
        _ => Err(format!(
            "observed `{num_primaries}` self-reported primaries at once"
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        time::Duration,
    };

    use super::{
        validate_primary_count, MemberPrimaryObservation, ObserveAllMembers,
        PrimaryCountInvariantRunner,
    };
    use crate::support::topology::ClusterMember;

    fn observed_poll(states: [bool; 3]) -> [MemberPrimaryObservation; 3] {
        [
            Ok((ClusterMember::NodeA, states[0])),
            Ok((ClusterMember::NodeB, states[1])),
            Ok((ClusterMember::NodeC, states[2])),
        ]
    }

    fn unavailable_poll() -> [MemberPrimaryObservation; 3] {
        [
            Ok((ClusterMember::NodeA, false)),
            Ok((ClusterMember::NodeB, false)),
            Ok((ClusterMember::NodeC, false)),
        ]
    }

    fn scripted_observer(polls: Vec<[MemberPrimaryObservation; 3]>) -> ObserveAllMembers {
        let polls = Arc::new(polls);
        let next_poll = Arc::new(AtomicUsize::new(0));
        Arc::new(move || {
            let polls = Arc::clone(&polls);
            let next_poll = Arc::clone(&next_poll);
            Box::pin(async move {
                let index = next_poll
                    .fetch_add(1, Ordering::SeqCst)
                    .min(polls.len().saturating_sub(1));
                polls[index].clone()
            })
        })
    }

    #[test]
    fn zero_primaries_keeps_waiting() {
        assert_eq!(validate_primary_count(0), Ok(false));
    }

    #[test]
    fn one_primary_is_healthy() {
        assert_eq!(validate_primary_count(1), Ok(true));
    }

    #[test]
    fn multiple_primaries_fail_immediately() {
        assert_eq!(
            validate_primary_count(2),
            Err("observed `2` self-reported primaries at once".to_string())
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn healthy_when_exactly_one_primary_is_reported() {
        let runner = PrimaryCountInvariantRunner::start_with_observe_all(
            Duration::from_millis(5),
            Duration::from_millis(50),
            scripted_observer(vec![observed_poll([false, true, false])]),
        );

        assert!(runner.ensure_healthy().is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn healthy_after_waiting_for_primary_count_to_reach_one() {
        let runner = PrimaryCountInvariantRunner::start_with_observe_all(
            Duration::from_millis(5),
            Duration::from_millis(50),
            scripted_observer(vec![
                observed_poll([false, false, false]),
                observed_poll([false, true, false]),
            ]),
        );

        assert!(runner.ensure_healthy().is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn times_out_when_primary_count_never_reaches_one() {
        let runner = PrimaryCountInvariantRunner::start_with_observe_all(
            Duration::from_millis(5),
            Duration::from_millis(20),
            scripted_observer(vec![observed_poll([false, false, false])]),
        );

        let err = match runner.ensure_healthy() {
            Ok(()) => "expected timeout error, but runner was healthy".to_string(),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("timed out waiting for self-reported primary count to become `1`"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn split_brain_fails_even_if_health_is_checked_later() {
        let runner = PrimaryCountInvariantRunner::start_with_observe_all(
            Duration::from_millis(5),
            Duration::from_millis(50),
            scripted_observer(vec![
                observed_poll([false, true, false]),
                observed_poll([true, true, false]),
            ]),
        );

        tokio::time::sleep(Duration::from_millis(20)).await;

        assert!(runner.task.is_finished());
        let err = match runner.ensure_healthy() {
            Ok(()) => "expected split-brain error, but runner was healthy".to_string(),
            Err(err) => err.to_string(),
        };
        assert!(
            err.contains("observed `2` self-reported primaries at once: `node-a`, `node-b`"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn probe_failure_counts_as_not_primary() {
        let runner = PrimaryCountInvariantRunner::start_with_observe_all(
            Duration::from_millis(5),
            Duration::from_millis(50),
            scripted_observer(vec![
                unavailable_poll(),
                observed_poll([false, true, false]),
            ]),
        );

        assert!(runner.ensure_healthy().is_ok());
    }
}
