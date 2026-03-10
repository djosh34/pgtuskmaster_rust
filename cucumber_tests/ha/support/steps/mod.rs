use std::time::{Duration, Instant};

use cucumber::{given, then, when};

use crate::support::{
    error::{HarnessError, Result},
    observer::pgtm::ClusterStatusView,
    world::{HarnessShared, HaWorld},
};

const PROOF_TABLE: &str = "ha_cucumber_proof";

#[given(regex = r#"^the "([^"]+)" harness is running$"#)]
async fn the_harness_is_running(world: &mut HaWorld, given_name: String) -> Result<()> {
    let harness = HarnessShared::initialize(given_name.as_str()).await?;
    world.set_harness(harness);
    Ok(())
}

#[given("the cluster reaches one stable primary")]
#[then("the cluster reaches one stable primary")]
async fn the_cluster_reaches_one_stable_primary(world: &mut HaWorld) -> Result<()> {
    let harness = world.harness()?;
    poll_for_status(harness, PollKind::Startup, |status| {
        require_sampled_members(status, 3)?;
        single_primary(status).map(|_| ())
    })
    .await
}

#[when("the current primary container crashes")]
async fn the_current_primary_container_crashes(world: &mut HaWorld) -> Result<()> {
    let primary_member = {
        let harness = world.harness()?;
        let status = harness.observer().status()?;
        single_primary(&status)?
    };
    {
        let harness = world.harness()?;
        let container_id = harness.service_container_id(primary_member.as_str())?;
        harness.docker.kill_container(container_id.as_str())?;
    }
    world.killed_node = Some(primary_member);
    Ok(())
}

#[then("after the configured HA lease deadline a different node becomes the only primary")]
async fn a_different_node_becomes_the_only_primary(world: &mut HaWorld) -> Result<()> {
    let killed_node = world
        .killed_node
        .clone()
        .ok_or_else(|| HarnessError::message("no killed node was recorded"))?;
    let new_primary = {
        let harness = world.harness()?;
        poll_for_status(harness, PollKind::Failover, |status| {
            require_sampled_members(status, 2)?;
            let primary = single_primary(status)?;
            if primary == killed_node {
                return Err(HarnessError::message(format!(
                    "primary did not change after crash; `{primary}` is still primary"
                )));
            }
            Ok(primary)
        })
        .await?
    };
    world.new_primary = Some(new_primary);
    Ok(())
}

#[then("I can write a proof row through the new primary")]
async fn i_can_write_a_proof_row_through_the_new_primary(world: &mut HaWorld) -> Result<()> {
    let expected_primary = world
        .new_primary
        .clone()
        .ok_or_else(|| HarnessError::message("new primary was not recorded"))?;
    let proof_token = {
        let harness = world.harness()?;
        let connection = harness.observer().primary_tls_json()?;
        let primary = connection
            .targets
            .first()
            .ok_or_else(|| HarnessError::message("pgtm primary --json --tls returned zero targets"))?;
        if primary.member_id != expected_primary {
            return Err(HarnessError::message(format!(
                "pgtm primary resolved `{}` but expected `{expected_primary}`",
                primary.member_id
            )));
        }

        let proof_token = format!("proof-{}", harness.run_id);
        let sql = harness.sql();
        let _ = sql.execute(
            primary.dsn.as_str(),
            format!(
                "CREATE TABLE IF NOT EXISTS {PROOF_TABLE} (token TEXT PRIMARY KEY);"
            )
            .as_str(),
        )?;
        let _ = sql.execute(
            primary.dsn.as_str(),
            format!(
                "INSERT INTO {PROOF_TABLE} (token) VALUES ('{proof_token}') ON CONFLICT (token) DO NOTHING;"
            )
            .as_str(),
        )?;
        proof_token
    };
    world.proof_token = Some(proof_token);
    Ok(())
}

#[when("I start the killed node container again")]
async fn i_start_the_killed_node_container_again(world: &mut HaWorld) -> Result<()> {
    let killed_node = world
        .killed_node
        .clone()
        .ok_or_else(|| HarnessError::message("no killed node was recorded"))?;
    let harness = world.harness()?;
    let container_id = harness.service_container_id(killed_node.as_str())?;
    harness.docker.start_container(container_id.as_str())?;
    Ok(())
}

#[then("after the configured recovery deadline the restarted node rejoins as a replica")]
async fn the_restarted_node_rejoins_as_a_replica(world: &mut HaWorld) -> Result<()> {
    let killed_node = world
        .killed_node
        .clone()
        .ok_or_else(|| HarnessError::message("no killed node was recorded"))?;
    let harness = world.harness()?;
    let _ = poll_for_status(harness, PollKind::Recovery, |status| {
        require_sampled_members(status, 3)?;
        let primary = single_primary(status)?;
        let restarted_node = status
            .nodes
            .iter()
            .find(|node| node.member_id == killed_node)
            .ok_or_else(|| {
                HarnessError::message(format!(
                    "restarted node `{killed_node}` is not present in cluster status"
                ))
            })?;
        if !restarted_node.sampled {
            return Err(HarnessError::message(format!(
                "restarted node `{killed_node}` is not sampled yet"
            )));
        }
        if restarted_node.role != "replica" {
            return Err(HarnessError::message(format!(
                "restarted node `{killed_node}` role is `{}` instead of `replica`",
                restarted_node.role
            )));
        }
        if restarted_node.member_id == primary {
            return Err(HarnessError::message(format!(
                "restarted node `{killed_node}` became primary instead of rejoining as replica"
            )));
        }
        Ok(())
    })
    .await?;
    Ok(())
}

#[then("the proof row is visible from the restarted node")]
async fn the_proof_row_is_visible_from_the_restarted_node(world: &mut HaWorld) -> Result<()> {
    let killed_node = world
        .killed_node
        .clone()
        .ok_or_else(|| HarnessError::message("no killed node was recorded"))?;
    let proof_token = world
        .proof_token
        .clone()
        .ok_or_else(|| HarnessError::message("proof token was not recorded"))?;
    let harness = world.harness()?;
    let deadline = Instant::now() + harness.timeouts.recovery_deadline;
    let query = format!("SELECT count(*) FROM {PROOF_TABLE} WHERE token = '{proof_token}';");
    let mut last_error = None;

    while Instant::now() < deadline {
        let attempt = harness
            .observer()
            .replicas_tls_json()
            .and_then(|replicas| {
                replicas
                    .targets
                    .into_iter()
                    .find(|target| target.member_id == killed_node)
                    .ok_or_else(|| {
                        HarnessError::message(format!(
                            "pgtm replicas did not include restarted node `{killed_node}`"
                        ))
                    })
            })
            .and_then(|restarted| harness.sql().execute(restarted.dsn.as_str(), query.as_str()))
            .and_then(|result| {
                if result.trim() == "1" {
                    Ok(())
                } else {
                    Err(HarnessError::message(format!(
                        "expected proof row count 1 from restarted replica `{killed_node}`, got `{}`",
                        result.trim()
                    )))
                }
            });

        match attempt {
            Ok(()) => return Ok(()),
            Err(err) => {
                last_error = Some(err.to_string());
            }
        }
        tokio::time::sleep(harness.timeouts.poll_interval).await;
    }

    Err(HarnessError::message(format!(
        "timed out waiting for proof row on restarted replica `{killed_node}`; last observed error: {}",
        last_error.unwrap_or_else(|| "no proof visibility attempt ran".to_string())
    )))
}

#[then("the cluster still has exactly one primary")]
async fn the_cluster_still_has_exactly_one_primary(world: &mut HaWorld) -> Result<()> {
    let harness = world.harness()?;
    let status = harness.observer().status()?;
    require_sampled_members(&status, 3)?;
    let _ = single_primary(&status)?;
    Ok(())
}

async fn poll_for_status<T, F>(
    harness: &HarnessShared,
    kind: PollKind,
    mut check: F,
) -> Result<T>
where
    F: FnMut(&ClusterStatusView) -> Result<T>,
{
    let deadline = Instant::now() + kind.deadline(harness);
    let mut last_error = None;
    while Instant::now() < deadline {
        let attempt: Result<T> = match harness.observer().status() {
            Ok(status) => check(&status),
            Err(err) => Err(err),
        };
        match attempt {
            Ok(value) => return Ok(value),
            Err(err) => {
                let err_text = err.to_string();
                if let Some(terminal_error) = terminal_container_failure(harness, kind)? {
                    return Err(HarnessError::message(format!(
                        "{err_text}\nterminal container failure detected: {terminal_error}"
                    )));
                }
                last_error = Some(err_text);
            }
        }
        tokio::time::sleep(harness.timeouts.poll_interval).await;
    }

    Err(HarnessError::message(format!(
        "{} deadline expired; last observed error: {}",
        kind.label(),
        last_error.unwrap_or_else(|| "no status observed".to_string())
    )))
}

fn single_primary(status: &ClusterStatusView) -> Result<String> {
    let primaries = status
        .nodes
        .iter()
        .filter(|node| node.sampled && node.role == "primary")
        .map(|node| node.member_id.clone())
        .collect::<Vec<_>>();
    match primaries.as_slice() {
        [primary] => Ok(primary.clone()),
        [] => Err(HarnessError::message(format!(
            "cluster has no sampled primary; queried via {} {} and warnings={}",
            status.queried_via.member_id,
            status.queried_via.api_url,
            format_warnings(status)
        ))),
        _ => Err(HarnessError::message(format!(
            "cluster has multiple primaries: {}",
            primaries.join(", ")
        ))),
    }
}

fn require_sampled_members(status: &ClusterStatusView, expected: usize) -> Result<()> {
    let sampled = status.sampled_member_count;
    if sampled == expected {
        return Ok(());
    }

    Err(HarnessError::message(format!(
        "expected {expected} sampled members, observed {sampled} sampled out of {} discovered; warnings={}",
        status.discovered_member_count,
        format_warnings(status)
    )))
}

fn format_warnings(status: &ClusterStatusView) -> String {
    if status.warnings.is_empty() {
        return "none".to_string();
    }
    status
        .warnings
        .iter()
        .map(|warning| format!("{}={}", warning.code, warning.message))
        .collect::<Vec<_>>()
        .join("; ")
}

fn terminal_container_failure(
    harness: &HarnessShared,
    kind: PollKind,
) -> Result<Option<String>> {
    let services = match kind {
        PollKind::Startup | PollKind::Recovery => ["node-a", "node-b", "node-c"].as_slice(),
        PollKind::Failover => return Ok(None),
    };

    let mut failures = Vec::new();
    for service in services {
        let container_id = match harness.service_container_id(service) {
            Ok(container_id) => container_id,
            Err(err) => {
                failures.push(format!("{service}=container-resolution-failed({err})"));
                continue;
            }
        };
        let state = harness.docker.container_state_status(container_id.as_str())?;
        if matches!(state.as_str(), "exited" | "dead") {
            failures.push(format!("{service}={state}"));
        }
    }

    if failures.is_empty() {
        Ok(None)
    } else {
        Ok(Some(failures.join(", ")))
    }
}

#[derive(Clone, Copy, Debug)]
enum PollKind {
    Startup,
    Failover,
    Recovery,
}

impl PollKind {
    fn deadline(self, harness: &HarnessShared) -> Duration {
        match self {
            Self::Startup => harness.timeouts.startup_deadline,
            Self::Failover => harness.timeouts.failover_deadline,
            Self::Recovery => harness.timeouts.recovery_deadline,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Startup => "startup",
            Self::Failover => "failover",
            Self::Recovery => "recovery",
        }
    }
}
