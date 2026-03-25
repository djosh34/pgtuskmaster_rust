use std::{
    collections::BTreeSet,
    time::{Duration, Instant},
};

use cucumber::{given, then, when};
use pgtuskmaster_rust::{
    dcs::DcsMemberState,
    pginfo::state::{PgInfoState, Readiness},
    state::ObservedWalPosition,
};

use crate::support::{
    error::{HarnessError, Result},
    faults::{BlockerKind, TrafficPath},
    givens::HaGivenId,
    invariants::probe_routing_target_connectivity,
    observer::{pgtm::ClusterStateObservation, sql},
    topology::{ClusterMember, DcsService},
    world::{HaWorld, HarnessShared},
};

#[given(regex = r#"^the "([^"]+)" harness is running$"#)]
async fn the_harness_is_running(world: &mut HaWorld, given_name: String) -> Result<()> {
    let given = HaGivenId::parse(given_name.as_str())?;
    let harness = HarnessShared::initialize(given, world.scenario_name()?).await?;
    harness.record_note("scenario", format!("started given `{given_name}`"))?;
    world.set_harness(harness);
    Ok(())
}

#[given(regex = r#"^I wait for exactly one stable primary as "([^"]+)"$"#)]
#[then(regex = r#"^I wait for exactly one stable primary as "([^"]+)"$"#)]
async fn i_wait_for_exactly_one_stable_primary_as(
    world: &mut HaWorld,
    alias: String,
) -> Result<()> {
    let expected_replicas = world.online_expected_count().saturating_sub(1);
    let (primary, _) = wait_for_authoritative_single_primary(
        world,
        format!("wait.stable_primary.{alias}").as_str(),
        PollKind::Startup,
        world.online_expected_count(),
        None,
        None,
    )
    .await?;
    if expected_replicas > 0 {
        wait_for_replicas(
            world,
            format!("wait.stable_primary_replicas.{alias}").as_str(),
            |replicas| {
                if replicas.len() == expected_replicas {
                    Ok(())
                } else {
                    Err(HarnessError::message(format!(
                        "expected {expected_replicas} visible replicas, observed {}",
                        replicas.len()
                    )))
                }
            },
        )
        .await?;
    }
    world.record_member_alias(alias.as_str(), primary, "wait.stable_primary")?;
    world.record_member_alias("current_primary", primary, "wait.current_primary")?;
    Ok(())
}

#[then("cluster becomes healthy")]
async fn cluster_becomes_healthy(world: &mut HaWorld) -> Result<()> {
    let (primary, healthy_members) = wait_for_authoritative_single_primary(
        world,
        "outcome.cluster_healthy",
        PollKind::Recovery,
        world.online_expected_count(),
        None,
        None,
    )
    .await?;
    world.remember_member_alias("current_primary", primary);
    world
        .harness()?
        .ensure_accepted_writes_healthy(healthy_members.as_slice())?;
    Ok(())
}

#[then("cluster becomes unhealthy")]
async fn cluster_becomes_unhealthy(world: &mut HaWorld) -> Result<()> {
    wait_for_no_operator_primary(world).await
}

#[given(regex = r#"^I choose one non-primary node as "([^"]+)"$"#)]
async fn i_choose_one_non_primary_node_as(world: &mut HaWorld, alias: String) -> Result<()> {
    let replicas = wait_for_replicas(
        world,
        format!("choose_one_replica.{alias}").as_str(),
        |replicas| {
            if replicas.is_empty() {
                Err(HarnessError::message(
                    "expected at least 1 visible replica, observed 0",
                ))
            } else {
                Ok(())
            }
        },
    )
    .await?;
    let member_id = replicas
        .first()
        .copied()
        .ok_or_else(|| HarnessError::message("cluster has no non-primary node to choose"))?;
    world.record_member_alias(alias.as_str(), member_id, "choose_one_replica")
}

#[given(regex = r#"^I choose the two non-primary nodes as "([^"]+)" and "([^"]+)"$"#)]
async fn i_choose_the_two_non_primary_nodes_as(
    world: &mut HaWorld,
    alias_a: String,
    alias_b: String,
) -> Result<()> {
    let replicas = wait_for_replicas(
        world,
        format!("choose_two_replicas.{alias_a}.{alias_b}").as_str(),
        |replicas| {
            if replicas.len() == 2 {
                Ok(())
            } else {
                Err(HarnessError::message(format!(
                    "expected 2 visible replicas, observed {}",
                    replicas.len()
                )))
            }
        },
    )
    .await?;
    match replicas.as_slice() {
        [member_a, member_b] => {
            world.record_member_alias(alias_a.as_str(), *member_a, "choose_two_replicas")?;
            world.record_member_alias(alias_b.as_str(), *member_b, "choose_two_replicas")
        }
        _ => Err(HarnessError::message(format!(
            "expected exactly two non-primary nodes, observed {}",
            replicas
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

#[when(regex = r#"^I kill the node named "([^"]+)"$"#)]
async fn i_kill_the_node_named(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = resolve_member(world, member_ref.as_str())?;
    world.kill_nodes([member_id])
}

#[when(regex = r#"^I kill the nodes named "([^"]+)" and "([^"]+)"$"#)]
async fn i_kill_the_nodes_named(
    world: &mut HaWorld,
    member_ref_a: String,
    member_ref_b: String,
) -> Result<()> {
    world.kill_nodes([
        resolve_member(world, member_ref_a.as_str())?,
        resolve_member(world, member_ref_b.as_str())?,
    ])
}

#[when("I kill all database nodes")]
async fn i_kill_all_database_nodes(world: &mut HaWorld) -> Result<()> {
    world.kill_nodes(ClusterMember::ALL)
}

#[when(regex = r#"^I restart the node named "([^"]+)"$"#)]
async fn i_restart_the_node_named(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = resolve_member(world, member_ref.as_str())?;
    world.start_nodes([member_id])
}

#[when(regex = r#"^I start only the fixed nodes "([^"]+)" and "([^"]+)"$"#)]
async fn i_start_only_the_fixed_nodes(
    world: &mut HaWorld,
    member_ref_a: String,
    member_ref_b: String,
) -> Result<()> {
    world.start_nodes([
        resolve_member(world, member_ref_a.as_str())?,
        resolve_member(world, member_ref_b.as_str())?,
    ])
}

#[when("I request a planned switchover")]
async fn i_request_a_planned_switchover(world: &mut HaWorld) -> Result<()> {
    let seed_member = world.require_member_alias("current_primary")?;
    let deadline = world.harness()?.timeouts.failover_deadline;
    let target_member = poll_until(
        world,
        deadline,
        None,
        "no planned switchover precondition check ran",
        |world| {
            let harness = world.harness()?;
            let status = harness.observer.state_via_member(seed_member)?;
            harness.record_status_snapshot("switchover.request.precondition", &status)?;
            status
                .dcs
                .members()
                .filter_map(|(member_id, member_view)| {
                    ClusterMember::parse(member_id.0.as_str())
                        .ok()
                        .filter(|member| *member != seed_member)
                        .filter(|_| member_slot_is_api_switchover_eligible(member_view))
                        .map(|member| {
                            let rank = match member_view.postgres() {
                                PgInfoState::Replica {
                                    common,
                                    replay_lsn,
                                    follow_lsn,
                                    ..
                                } => Some(ObservedWalPosition {
                                    timeline: common.timeline,
                                    lsn: *replay_lsn,
                                })
                                .or_else(|| {
                                    follow_lsn.map(|lsn| ObservedWalPosition {
                                        timeline: common.timeline,
                                        lsn,
                                    })
                                })
                                .map(|wal| {
                                    (
                                        1,
                                        wal.timeline.map_or(0, |timeline| u64::from(timeline.0)),
                                        wal.lsn.0,
                                    )
                                })
                                .unwrap_or((0, 0, 0)),
                                PgInfoState::Unknown { common } => (
                                    0,
                                    common.timeline.map_or(0, |timeline| u64::from(timeline.0)),
                                    0,
                                ),
                                PgInfoState::Primary { .. } => (0, 0, 0),
                            };
                            (member, rank)
                        })
                })
                .max_by(|(left_member, left_rank), (right_member, right_rank)| {
                    left_rank
                        .cmp(right_rank)
                        .then_with(|| right_member.cmp(left_member))
                })
                .map(|(member, _)| member)
                .ok_or_else(|| {
                    HarnessError::message(format!(
                        "no eligible planned switchover target is currently visible via `{seed_member}`"
                    ))
                })
        },
        |last_error| {
            HarnessError::message(format!(
                "timed out waiting for an eligible planned switchover target via `{seed_member}`; last observed error: {last_error}"
            ))
        },
    )
    .await?;
    let harness = world.harness()?;
    let response = harness
        .observer
        .switchover_request_via_member(seed_member, Some(target_member))?;
    harness.record_note(
        "switchover.request",
        format!("target={target_member} response={response}"),
    )?;
    Ok(())
}

#[when(regex = r#"^I request a targeted switchover to "([^"]+)"$"#)]
async fn i_request_a_targeted_switchover_to(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = resolve_member(world, member_ref.as_str())?;
    let seed_member = world.require_member_alias("current_primary")?;
    let deadline = world.harness()?.timeouts.failover_deadline;
    poll_until(
        world,
        deadline,
        None,
        "no targeted switchover precondition check ran",
        |world| {
            let harness = world.harness()?;
            let status = harness.observer.state_via_member(seed_member)?;
            harness.record_status_snapshot("switchover.request.targeted_precondition", &status)?;
            match status.dcs.member(&member_id.member_id()) {
                Some(member) if member_slot_is_api_switchover_eligible(member) => Ok(()),
                Some(member) => Err(HarnessError::message(format!(
                    "target `{member_id}` is not yet promotion-eligible via `{seed_member}` with postgres state {:?}",
                    member.postgres()
                ))),
                None => Err(HarnessError::message(format!(
                    "target `{member_id}` is not visible via `{seed_member}` yet"
                ))),
            }
        },
        |last_error| {
            HarnessError::message(format!(
                "timed out waiting for `{member_id}` to become an eligible targeted switchover target via `{seed_member}`; last observed error: {last_error}"
            ))
        },
    )
    .await?;
    let harness = world.harness()?;
    let response = harness
        .observer
        .switchover_request_via_member(seed_member, Some(member_id))?;
    harness.record_note(
        "switchover.request.targeted",
        format!("target={member_id} response={response}"),
    )?;
    Ok(())
}

#[when(
    regex = r#"^I attempt a targeted switchover to "([^"]+)" and capture the operator-visible error$"#
)]
async fn i_attempt_a_targeted_switchover_to_and_capture_the_operator_visible_error(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member(world, member_ref.as_str())?;
    let seed_member = world.require_member_alias("current_primary")?;
    let deadline = world.harness()?.timeouts.failover_deadline;
    poll_until(
        world,
        deadline,
        None,
        "no ineligibility check ran",
        |world| {
            let harness = world.harness()?;
            let status = harness.observer.state_via_member(seed_member)?;
            harness.record_status_snapshot("switchover.rejected.precondition", &status)?;
            match status.dcs.member(&member_id.member_id()) {
                None => Ok(()),
                Some(member) if !member_slot_is_api_switchover_eligible(member) => Ok(()),
                Some(member) => Err(HarnessError::message(format!(
                    "target `{member_id}` is still promotion-eligible via `{seed_member}` with postgres state {:?}",
                    member.postgres()
                ))),
            }
        },
        |last_error| {
            HarnessError::message(format!(
                "timed out waiting for `{member_id}` to become an ineligible targeted switchover target; last observed error: {last_error}"
            ))
        },
    )
    .await?;
    let request_result = world
        .harness()?
        .observer
        .switchover_request_via_member(seed_member, Some(member_id));
    match request_result {
        Ok(output) => Err(HarnessError::message(format!(
            "expected targeted switchover to `{member_id}` to be rejected, but it succeeded: {output}"
        ))),
        Err(err) => {
            let rendered = err.to_string();
            world.harness()?.record_note(
                "switchover.request.targeted_rejected",
                format!("target={member_id} error={rendered}"),
            )?;
            Ok(())
        }
    }
}

#[then(regex = r#"^"([^"]+)" becomes primary$"#)]
async fn quoted_member_becomes_primary(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = resolve_member(world, member_ref.as_str())?;
    let (observed, _) = wait_for_authoritative_single_primary(
        world,
        format!("outcome.primary.{member_id}").as_str(),
        PollKind::Recovery,
        world.online_expected_count(),
        Some(member_id),
        None,
    )
    .await?;
    if observed != member_id {
        return Err(HarnessError::message(format!(
            "expected `{member_id}` to become primary, observed `{observed}`"
        )));
    }
    world.remember_member_alias("current_primary", observed);
    Ok(())
}

#[when("I stop the DCS service")]
async fn i_stop_the_dcs_service(world: &mut HaWorld) -> Result<()> {
    let harness = world.harness()?;
    set_dcs_services(world, harness.given.dcs_services(), false)
}

#[when("I stop a DCS quorum majority")]
async fn i_stop_a_dcs_quorum_majority(world: &mut HaWorld) -> Result<()> {
    let harness = world.harness()?;
    set_dcs_services(world, harness.given.quorum_majority_dcs_services(), false)
}

#[when("I start the DCS service")]
async fn i_start_the_dcs_service(world: &mut HaWorld) -> Result<()> {
    let harness = world.harness()?;
    set_dcs_services(world, harness.given.dcs_services(), true)
}

#[when("I restore DCS quorum")]
async fn i_restore_dcs_quorum(world: &mut HaWorld) -> Result<()> {
    let harness = world.harness()?;
    set_dcs_services(world, harness.given.quorum_majority_dcs_services(), true)
}

#[when(regex = r#"^I stop the local DCS service for node named "([^"]+)"$"#)]
async fn i_stop_the_local_dcs_service_for_node_named(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let harness = world.harness()?;
    let service = harness
        .given
        .local_dcs_service_for(resolve_member(world, member_ref.as_str())?);
    set_dcs_services(world, &[service], false)
}

#[when(regex = r#"^I start the local DCS service for node named "([^"]+)"$"#)]
async fn i_start_the_local_dcs_service_for_node_named(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let harness = world.harness()?;
    let service = harness
        .given
        .local_dcs_service_for(resolve_member(world, member_ref.as_str())?);
    set_dcs_services(world, &[service], true)
}

#[when(regex = r#"^I fully isolate the node named "([^"]+)" from the cluster$"#)]
async fn i_fully_isolate_the_node_named_from_the_cluster(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member(world, member_ref.as_str())?;
    world.fully_isolate_node_from_cluster(member_id)
}

#[when(regex = r#"^I cut the node named "([^"]+)" off from DCS$"#)]
async fn i_cut_the_node_named_off_from_dcs(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = resolve_member(world, member_ref.as_str())?;
    let harness = world.harness()?;
    harness.block_member_path_to_host(
        member_id,
        TrafficPath::Dcs,
        harness
            .given
            .local_dcs_service_for(member_id)
            .service_name(),
    )
}

#[when(regex = r#"^I isolate the node named "([^"]+)" from observer API access$"#)]
async fn i_isolate_the_node_named_from_observer_api_access(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member(world, member_ref.as_str())?;
    world.isolate_node_from_observer_api_access(member_id)
}

#[when(regex = r#"^I heal network faults on the node named "([^"]+)"$"#)]
async fn i_heal_network_faults_on_the_node_named(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member(world, member_ref.as_str())?;
    world.heal_node_network_faults(member_id)
}

#[when("I heal all network faults")]
async fn i_heal_all_network_faults(world: &mut HaWorld) -> Result<()> {
    world.heal_all_network_faults()
}

#[given(regex = r#"^I enable the "([^"]+)" blocker on the node named "([^"]+)"$"#)]
#[when(regex = r#"^I enable the "([^"]+)" blocker on the node named "([^"]+)"$"#)]
async fn i_enable_the_blocker_on_the_node_named(
    world: &mut HaWorld,
    blocker_name: String,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member(world, member_ref.as_str())?;
    let blocker = parse_blocker_kind(blocker_name.as_str())?;
    world.harness()?.set_blocker(member_id, blocker, true)
}

#[given(regex = r#"^I disable the "([^"]+)" blocker on the node named "([^"]+)"$"#)]
#[when(regex = r#"^I disable the "([^"]+)" blocker on the node named "([^"]+)"$"#)]
async fn i_disable_the_blocker_on_the_node_named(
    world: &mut HaWorld,
    blocker_name: String,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member(world, member_ref.as_str())?;
    let blocker = parse_blocker_kind(blocker_name.as_str())?;
    world.harness()?.set_blocker(member_id, blocker, false)
}

#[when(regex = r#"^I wipe the data directory on the node named "([^"]+)"$"#)]
async fn i_wipe_the_data_directory_on_the_node_named(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member(world, member_ref.as_str())?;
    world.harness()?.wipe_member_data_dir(member_id)
}

async fn wait_for_replicas<F>(
    world: &mut HaWorld,
    phase: &str,
    check: F,
) -> Result<Vec<ClusterMember>>
where
    F: Fn(&[ClusterMember]) -> Result<()>,
{
    let expected_members = world.online_member_ids();
    poll_for_cluster_observation(world, phase, PollKind::Startup, |observation| {
        let primary = observation.compatible_primary(
            expected_members.as_slice(),
            expected_members.len(),
            None,
        )?;
        let replicas = observation.replica_members(primary)?;
        check(replicas.as_slice())?;
        Ok(replicas)
    })
    .await
}

async fn wait_for_authoritative_single_primary(
    world: &mut HaWorld,
    phase: &str,
    kind: PollKind,
    expected_online: usize,
    exact_primary: Option<ClusterMember>,
    _different_from: Option<ClusterMember>,
) -> Result<(ClusterMember, Vec<ClusterMember>)> {
    let relevant_members = world.online_member_ids();
    let deadline = {
        let harness = world.harness()?;
        kind.deadline(harness)
    };

    poll_until(
        world,
        deadline,
        Some("authoritative primary polling failed"),
        "no authoritative primary verification attempt ran",
        |world| {
            let observation = {
                let harness = world.harness()?;
                let observation = harness.observer.observe_states()?;
                record_cluster_observation(harness, phase, &observation)?;
                observation
            };
            let primary = observation.compatible_primary(
                relevant_members.as_slice(),
                expected_online,
                exact_primary,
            )?;
            let harness = world.harness()?;
            let primary_target = harness.observer.postgres_routing_target(primary)?;
            let primary_dsn = primary_target.conninfo.to_string();
            probe_writable_primary(harness, primary_dsn.as_str())?;
            let healthy_members =
                observed_healthy_members_from_poll(harness, &observation, primary)?;
            Ok((primary, healthy_members))
        },
        |last_error| {
            HarnessError::message(format!(
                "{} deadline expired while waiting for a stable authoritative primary; last observed error: {last_error}",
                kind.label(),
            ))
        },
    )
    .await
}

async fn wait_for_no_operator_primary(world: &mut HaWorld) -> Result<()> {
    let relevant_members = world.online_member_ids();
    let expected_online = relevant_members.len();
    let deadline = world.harness()?.timeouts.failover_deadline;

    poll_until(
        world,
        deadline,
        None,
        "no no-primary verification attempt ran",
        |world| {
            let harness = world.harness()?;
            let observation = harness.observer.observe_states()?;
            record_cluster_observation(harness, "primary.none", &observation)?;
            match observation.compatible_primary(relevant_members.as_slice(), expected_online, None)
            {
                Ok(primary) => {
                    let primary_target = harness.observer.postgres_routing_target(primary)?;
                    let primary_dsn = primary_target.conninfo.to_string();
                    match probe_writable_primary(harness, primary_dsn.as_str()) {
                        Ok(()) => Err(HarnessError::message(format!(
                            "cluster still exposes a compatible writable primary `{primary}`"
                        ))),
                        Err(_) => Ok(()),
                    }
                }
                Err(_) => Ok(()),
            }
        },
        |last_error| {
            HarnessError::message(format!(
                "timed out waiting for pgtm to expose no operator-visible primary; last observed error: {last_error}"
            ))
        },
    )
    .await
}

fn member_slot_is_api_switchover_eligible(member: &DcsMemberState) -> bool {
    match member.postgres() {
        PgInfoState::Primary { .. } => false,
        PgInfoState::Unknown { common } | PgInfoState::Replica { common, .. } => {
            common.readiness == Readiness::Ready
        }
    }
}

fn resolve_member(world: &HaWorld, member_ref: &str) -> Result<ClusterMember> {
    world.resolve_member_reference(member_ref)
}

fn set_dcs_services(world: &HaWorld, services: &[DcsService], started: bool) -> Result<()> {
    let harness = world.harness()?;
    if started {
        harness.start_dcs_services(services.iter().copied())
    } else {
        harness.stop_dcs_services(services.iter().copied())
    }
}

async fn poll_for_cluster_observation<T, F>(
    world: &mut HaWorld,
    phase: &str,
    kind: PollKind,
    mut check: F,
) -> Result<T>
where
    F: FnMut(&ClusterStateObservation) -> Result<T>,
{
    let deadline = {
        let harness = world.harness()?;
        kind.deadline(harness)
    };

    poll_until(
        world,
        deadline,
        Some("status polling failed"),
        "no status observed",
        |world| {
            let observation = {
                let harness = world.harness()?;
                let observation = harness.observer.observe_states()?;
                record_cluster_observation(harness, phase, &observation)?;
                observation
            };
            check(&observation)
        },
        |last_error| {
            HarnessError::message(format!(
                "{} deadline expired; last observed error: {last_error}",
                kind.label(),
            ))
        },
    )
    .await
}

async fn poll_until<T, F, G>(
    world: &mut HaWorld,
    deadline_window: Duration,
    terminal_failure_fallback: Option<&str>,
    no_attempt_fallback: &str,
    mut attempt: F,
    timeout_error: G,
) -> Result<T>
where
    F: FnMut(&mut HaWorld) -> Result<T>,
    G: FnOnce(String) -> HarnessError,
{
    let deadline = Instant::now() + deadline_window;
    let poll_interval = world.harness()?.timeouts.poll_interval;
    let mut last_error = None;

    while Instant::now() < deadline {
        match attempt(world) {
            Ok(value) => return Ok(value),
            Err(err) => last_error = Some(err.to_string()),
        }
        if let Some(fallback) = terminal_failure_fallback {
            let terminal_error = {
                let harness = world.harness()?;
                terminal_container_failure(harness, &world.stopped_members)?
            };
            if let Some(terminal_error) = terminal_error {
                return Err(HarnessError::message(format!(
                    "{}\nterminal container failure detected: {terminal_error}",
                    last_error
                        .as_ref()
                        .cloned()
                        .unwrap_or_else(|| fallback.to_string())
                )));
            }
        }
        tokio::time::sleep(poll_interval).await;
    }

    Err(timeout_error(
        last_error.unwrap_or_else(|| no_attempt_fallback.to_string()),
    ))
}

fn record_cluster_observation(
    harness: &HarnessShared,
    phase: &str,
    observation: &ClusterStateObservation,
) -> Result<()> {
    observation
        .iter()
        .try_for_each(|(member, outcome)| match outcome {
            Ok(state) => harness.record_status_snapshot(
                format!("{phase}.{}", member.service_name()).as_str(),
                state,
            ),
            Err(message) => harness.record_note(
                format!("{phase}.{}", member.service_name()).as_str(),
                format!("status_failed={message}"),
            ),
        })
}

fn observed_healthy_members_from_poll(
    harness: &HarnessShared,
    observation: &ClusterStateObservation,
    primary: ClusterMember,
) -> Result<Vec<ClusterMember>> {
    observed_healthy_members(primary, observation.replica_members(primary)?, |member| {
        if member == primary {
            return Ok(true);
        }

        match probe_member_postgres(harness, member) {
            Ok(()) => Ok(true),
            Err(err) => {
                harness.record_note(
                    "outcome.cluster_healthy.member_skipped",
                    format!("member={member} reason={err}"),
                )?;
                Ok(false)
            }
        }
    })
}

fn observed_healthy_members(
    primary: ClusterMember,
    replicas: impl IntoIterator<Item = ClusterMember>,
    mut is_probeable: impl FnMut(ClusterMember) -> Result<bool>,
) -> Result<Vec<ClusterMember>> {
    std::iter::once(primary)
        .chain(replicas)
        .try_fold(Vec::new(), |mut members, member| {
            if is_probeable(member)? {
                members.push(member);
            }
            Ok(members)
        })
}

fn probe_writable_primary(harness: &HarnessShared, dsn: &str) -> Result<()> {
    let probe_sql =
        "CREATE TEMP TABLE pgtm_writable_primary_probe ON COMMIT DROP AS SELECT 'probe'::text AS token;";
    probe_postgres(harness, dsn, probe_sql)
}

fn probe_member_postgres(harness: &HarnessShared, member: ClusterMember) -> Result<()> {
    let target = harness.observer.postgres_routing_target(member)?;
    probe_routing_target_connectivity(&target, harness.timeouts.poll_interval)
        .map_err(HarnessError::from)
}

fn probe_postgres(harness: &HarnessShared, dsn: &str, probe_sql: &str) -> Result<()> {
    let _ = sql::execute(
        harness.materialized_dir.as_path(),
        dsn,
        probe_sql,
        harness.timeouts.poll_interval,
    )?;
    Ok(())
}

fn terminal_container_failure(
    harness: &HarnessShared,
    expected_offline: &BTreeSet<ClusterMember>,
) -> Result<Option<String>> {
    let mut failures = Vec::new();
    for service in ClusterMember::ALL {
        if expected_offline.contains(&service) {
            continue;
        }
        let container_id = match harness.service_container_id(service.service_name()) {
            Ok(container_id) => container_id,
            Err(err) => {
                failures.push(format!("{service}=container-resolution-failed({err})"));
                continue;
            }
        };
        let state = harness
            .docker
            .container_state_status(container_id.as_str())?;
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
    Recovery,
}

impl PollKind {
    fn deadline(self, harness: &HarnessShared) -> Duration {
        match self {
            Self::Startup => harness.timeouts.startup_deadline,
            Self::Recovery => harness.timeouts.recovery_deadline,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Startup => "startup",
            Self::Recovery => "recovery",
        }
    }
}

fn parse_blocker_kind(raw_value: &str) -> Result<BlockerKind> {
    match raw_value {
        "pg_basebackup" => Ok(BlockerKind::PgBasebackup),
        "pg_rewind" => Ok(BlockerKind::PgRewind),
        "postgres_start" => Ok(BlockerKind::PostgresStart),
        _ => Err(HarnessError::message(format!(
            "unsupported blocker `{raw_value}`"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{observed_healthy_members, ClusterMember};

    #[test]
    fn observed_healthy_members_skip_ready_replicas_without_fresh_probe() -> Result<(), String> {
        let probeable_members = BTreeSet::from([ClusterMember::NodeA, ClusterMember::NodeB]);

        let selected = observed_healthy_members(
            ClusterMember::NodeA,
            [ClusterMember::NodeB, ClusterMember::NodeC],
            |member| Ok(probeable_members.contains(&member)),
        )
        .map_err(|err| err.to_string())?;

        assert_eq!(selected, vec![ClusterMember::NodeA, ClusterMember::NodeB],);
        Ok(())
    }
}
