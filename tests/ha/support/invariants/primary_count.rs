use std::{
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
        let num_primaries = Arc::new(AtomicI32::new(0));
        let fatal_error = Arc::new(RwLock::new(None));
        let task_num_primaries = Arc::clone(&num_primaries);
        let task_fatal_error = Arc::clone(&fatal_error);
        let task = tokio::spawn(async move {
            loop {
                let (node_a, node_b, node_c) = tokio::join!(
                    observe_member_primary(observer.clone(), ClusterMember::NodeA),
                    observe_member_primary(observer.clone(), ClusterMember::NodeB),
                    observe_member_primary(observer.clone(), ClusterMember::NodeC),
                );
                let observations = [node_a, node_b, node_c];
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

        Ok(Self {
            poll_interval,
            health_deadline,
            num_primaries,
            fatal_error,
            task,
        })
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
) -> std::result::Result<(ClusterMember, bool), String> {
    tokio::task::spawn_blocking(move || {
        let is_primary = observer
            .state_via_member(member)
            .map(|state| matches!(state.pg, PgInfoState::Primary { .. }))
            .unwrap_or(false);
        (member, is_primary)
    })
    .await
    .map_err(|err| format!("join self-primary observation for `{member}` failed: {err}"))
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
    use super::validate_primary_count;

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
}
