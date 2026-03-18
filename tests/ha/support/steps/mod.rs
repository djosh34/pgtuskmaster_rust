use std::{
    collections::BTreeSet,
    time::{Duration, Instant},
};

use cucumber::{given, then, when};
use pgtuskmaster_rust::{
    api::NodeState,
    dcs::{ClusterMemberView, MemberPostgresView},
    ha::types::{AuthorityProjection, PublicationState},
    pginfo::state::Readiness,
    state::ObservedWalPosition,
};

use crate::support::{
    error::{HarnessError, Result},
    faults::{BlockerKind, TrafficPath},
    givens::HaGivenId,
    observer::{pgtm::ClusterStateObservation, sql},
    topology::ClusterMember,
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
    let primary = wait_for_authoritative_single_primary(
        world,
        format!("wait.stable_primary.{alias}").as_str(),
        PollKind::Startup,
        online_expected_count(world),
        None,
        None,
    )
    .await?;
    world.record_member_alias(alias.as_str(), primary, "wait.stable_primary")?;
    world.record_member_alias("current_primary", primary, "wait.current_primary")?;
    Ok(())
}

#[then("cluster becomes healthy")]
async fn cluster_becomes_healthy(world: &mut HaWorld) -> Result<()> {
    let primary = wait_for_authoritative_single_primary(
        world,
        "outcome.cluster_healthy",
        PollKind::Recovery,
        online_expected_count(world),
        None,
        None,
    )
    .await?;
    world.remember_member_alias("current_primary", primary);
    Ok(())
}

#[then("cluster becomes unhealthy")]
async fn cluster_becomes_unhealthy(world: &mut HaWorld) -> Result<()> {
    wait_for_no_operator_primary(world).await
}

#[given(regex = r#"^I choose one non-primary node as "([^"]+)"$"#)]
async fn i_choose_one_non_primary_node_as(world: &mut HaWorld, alias: String) -> Result<()> {
    let replicas =
        wait_for_minimum_replicas(world, format!("choose_one_replica.{alias}").as_str(), 1).await?;
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
        2,
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
    let member_id = world.resolve_member_reference(member_ref.as_str())?;
    world.harness()?.kill_node(member_id)?;
    world.add_stopped_node(member_id);
    Ok(())
}

#[when(regex = r#"^I kill the nodes named "([^"]+)" and "([^"]+)"$"#)]
async fn i_kill_the_nodes_named(
    world: &mut HaWorld,
    member_ref_a: String,
    member_ref_b: String,
) -> Result<()> {
    for member_ref in [member_ref_a.as_str(), member_ref_b.as_str()] {
        let member_id = world.resolve_member_reference(member_ref)?;
        world.harness()?.kill_node(member_id)?;
        world.add_stopped_node(member_id);
    }
    Ok(())
}

#[when("I kill all database nodes")]
async fn i_kill_all_database_nodes(world: &mut HaWorld) -> Result<()> {
    for member_id in all_cluster_members() {
        world.harness()?.kill_node(member_id)?;
        world.add_stopped_node(member_id);
    }
    Ok(())
}

#[when(regex = r#"^I restart the node named "([^"]+)"$"#)]
async fn i_restart_the_node_named(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = world.resolve_member_reference(member_ref.as_str())?;
    world.harness()?.start_node(member_id)?;
    world.remove_stopped_node(member_id);
    Ok(())
}

#[when(regex = r#"^I start only the fixed nodes "([^"]+)" and "([^"]+)"$"#)]
async fn i_start_only_the_fixed_nodes(
    world: &mut HaWorld,
    member_ref_a: String,
    member_ref_b: String,
) -> Result<()> {
    for member_ref in [member_ref_a.as_str(), member_ref_b.as_str()] {
        let member_id = world.resolve_member_reference(member_ref)?;
        world.harness()?.start_node(member_id)?;
        world.remove_stopped_node(member_id);
    }
    Ok(())
}

#[when("I request a planned switchover")]
async fn i_request_a_planned_switchover(world: &mut HaWorld) -> Result<()> {
    let seed_member = world.require_member_alias("current_primary")?;
    let target_member = wait_for_planned_switchover_precondition(world, seed_member).await?;
    let harness = world.harness()?;
    let response = harness
        .observer()
        .switchover_request_via_member(seed_member, Some(target_member))?;
    harness.record_note(
        "switchover.request",
        format!("target={target_member} response={response}"),
    )?;
    Ok(())
}

#[when(regex = r#"^I request a targeted switchover to "([^"]+)"$"#)]
async fn i_request_a_targeted_switchover_to(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = world.resolve_member_reference(member_ref.as_str())?;
    let seed_member = world.require_member_alias("current_primary")?;
    let harness = world.harness()?;
    let response = harness
        .observer()
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
    let member_id = world.resolve_member_reference(member_ref.as_str())?;
    let seed_member = world.require_member_alias("current_primary")?;
    wait_for_targeted_switchover_rejection_precondition(world, seed_member, member_id).await?;
    let request_result = world
        .harness()?
        .observer()
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
    let member_id = world.resolve_member_reference(member_ref.as_str())?;
    let observed = wait_for_authoritative_single_primary(
        world,
        format!("outcome.primary.{member_id}").as_str(),
        PollKind::Recovery,
        online_expected_count(world),
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
    world.harness()?.stop_all_dcs_services()
}

#[when("I stop a DCS quorum majority")]
async fn i_stop_a_dcs_quorum_majority(world: &mut HaWorld) -> Result<()> {
    world.harness()?.stop_dcs_quorum_majority()
}

#[when("I start the DCS service")]
async fn i_start_the_dcs_service(world: &mut HaWorld) -> Result<()> {
    world.harness()?.start_all_dcs_services()
}

#[when("I restore DCS quorum")]
async fn i_restore_dcs_quorum(world: &mut HaWorld) -> Result<()> {
    world.harness()?.start_dcs_quorum_majority()
}

#[when(regex = r#"^I stop the local DCS service for node named "([^"]+)"$"#)]
async fn i_stop_the_local_dcs_service_for_node_named(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = world.resolve_member_reference(member_ref.as_str())?;
    world.harness()?.stop_member_local_dcs(member_id)
}

#[when(regex = r#"^I start the local DCS service for node named "([^"]+)"$"#)]
async fn i_start_the_local_dcs_service_for_node_named(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = world.resolve_member_reference(member_ref.as_str())?;
    world.harness()?.start_member_local_dcs(member_id)
}

#[when(regex = r#"^I fully isolate the node named "([^"]+)" from the cluster$"#)]
async fn i_fully_isolate_the_node_named_from_the_cluster(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = world.resolve_member_reference(member_ref.as_str())?;
    let harness = world.harness()?;
    for path in [TrafficPath::Dcs, TrafficPath::Api, TrafficPath::Postgres] {
        harness.isolate_member_from_all_peers_on_path(member_id, path)?;
    }
    harness.cut_member_off_from_dcs(member_id)?;
    harness.isolate_member_from_observer_on_api(member_id)?;
    world.mark_observer_unreachable(member_id);
    Ok(())
}

#[when(regex = r#"^I cut the node named "([^"]+)" off from DCS$"#)]
async fn i_cut_the_node_named_off_from_dcs(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = world.resolve_member_reference(member_ref.as_str())?;
    world.harness()?.cut_member_off_from_dcs(member_id)
}

#[when(regex = r#"^I isolate the node named "([^"]+)" from observer API access$"#)]
async fn i_isolate_the_node_named_from_observer_api_access(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = world.resolve_member_reference(member_ref.as_str())?;
    world
        .harness()?
        .isolate_member_from_observer_on_api(member_id)?;
    world.mark_observer_unreachable(member_id);
    Ok(())
}

#[when(regex = r#"^I heal network faults on the node named "([^"]+)"$"#)]
async fn i_heal_network_faults_on_the_node_named(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = world.resolve_member_reference(member_ref.as_str())?;
    world.harness()?.heal_member_network_faults(member_id)?;
    world.clear_observer_unreachable(member_id);
    Ok(())
}

#[when("I heal all network faults")]
async fn i_heal_all_network_faults(world: &mut HaWorld) -> Result<()> {
    world.harness()?.clear_all_network_faults()?;
    world.clear_observer_unreachable_members();
    Ok(())
}

#[given(regex = r#"^I enable the "([^"]+)" blocker on the node named "([^"]+)"$"#)]
#[when(regex = r#"^I enable the "([^"]+)" blocker on the node named "([^"]+)"$"#)]
async fn i_enable_the_blocker_on_the_node_named(
    world: &mut HaWorld,
    blocker_name: String,
    member_ref: String,
) -> Result<()> {
    let member_id = world.resolve_member_reference(member_ref.as_str())?;
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
    let member_id = world.resolve_member_reference(member_ref.as_str())?;
    let blocker = parse_blocker_kind(blocker_name.as_str())?;
    world.harness()?.set_blocker(member_id, blocker, false)
}

#[when(regex = r#"^I wipe the data directory on the node named "([^"]+)"$"#)]
async fn i_wipe_the_data_directory_on_the_node_named(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = world.resolve_member_reference(member_ref.as_str())?;
    world.harness()?.wipe_member_data_dir(member_id)
}

async fn wait_for_replicas(
    world: &mut HaWorld,
    phase: &str,
    expected_replicas: usize,
) -> Result<Vec<ClusterMember>> {
    let expected_members = online_member_ids(world);
    poll_for_cluster_observation(world, phase, PollKind::Startup, |observation| {
        let primary = compatible_primary_from_observation(
            observation,
            expected_members.as_slice(),
            expected_members.len(),
            None,
        )?;
        let primary_state = require_observed_member_state(observation, primary)?;
        let replicas = replica_members(primary_state);
        if replicas.len() == expected_replicas {
            Ok(replicas)
        } else {
            Err(HarnessError::message(format!(
                "expected {expected_replicas} visible replicas, observed {}",
                replicas.len()
            )))
        }
    })
    .await
}

async fn wait_for_minimum_replicas(
    world: &mut HaWorld,
    phase: &str,
    minimum_replicas: usize,
) -> Result<Vec<ClusterMember>> {
    let expected_members = online_member_ids(world);
    poll_for_cluster_observation(world, phase, PollKind::Startup, |observation| {
        let primary = compatible_primary_from_observation(
            observation,
            expected_members.as_slice(),
            expected_members.len(),
            None,
        )?;
        let primary_state = require_observed_member_state(observation, primary)?;
        let replicas = replica_members(primary_state);
        if replicas.len() >= minimum_replicas {
            Ok(replicas)
        } else {
            Err(HarnessError::message(format!(
                "expected at least {minimum_replicas} visible replicas, observed {}",
                replicas.len()
            )))
        }
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
) -> Result<ClusterMember> {
    let relevant_members = online_member_ids(world);
    let deadline = {
        let harness = world.harness()?;
        Instant::now() + kind.deadline(harness)
    };
    let poll_interval = {
        let harness = world.harness()?;
        harness.timeouts.poll_interval
    };
    let mut last_error = None;

    while Instant::now() < deadline {
        let attempt: Result<ClusterMember> = (|| {
            let observation = {
                let harness = world.harness()?;
                let observation = harness.observer().observe_states()?;
                record_cluster_observation(harness, phase, &observation)?;
                observation
            };
            let primary = compatible_primary_from_observation(
                &observation,
                relevant_members.as_slice(),
                expected_online,
                exact_primary,
            )?;
            let harness = world.harness()?;
            let primary_target = harness.observer().postgres_routing_target(primary)?;
            probe_writable_primary(harness, primary_target.dsn.as_str())?;
            Ok(primary)
        })();
        match attempt {
            Ok(primary) => {
                return Ok(primary);
            }
            Err(err) => last_error = Some(err.to_string()),
        }
        let terminal_error = {
            let harness = world.harness()?;
            terminal_container_failure(harness, &world.stopped_members, kind)?
        };
        if let Some(terminal_error) = terminal_error {
            return Err(HarnessError::message(format!(
                "{}\nterminal container failure detected: {terminal_error}",
                last_error.unwrap_or_else(|| "authoritative primary polling failed".to_string())
            )));
        }
        tokio::time::sleep(poll_interval).await;
    }

    Err(HarnessError::message(format!(
        "{} deadline expired while waiting for a stable authoritative primary; last observed error: {}",
        kind.label(),
        last_error.unwrap_or_else(|| "no authoritative primary verification attempt ran".to_string())
    )))
}

async fn wait_for_no_operator_primary(world: &mut HaWorld) -> Result<()> {
    let relevant_members = online_member_ids(world);
    let expected_online = relevant_members.len();
    let deadline = {
        let harness = world.harness()?;
        Instant::now() + harness.timeouts.failover_deadline
    };
    let poll_interval = {
        let harness = world.harness()?;
        harness.timeouts.poll_interval
    };
    let mut last_error = None;

    while Instant::now() < deadline {
        let attempt: Result<()> = (|| {
            let harness = world.harness()?;
            let observation = harness.observer().observe_states()?;
            record_cluster_observation(harness, "primary.none", &observation)?;
            match compatible_primary_from_observation(
                &observation,
                relevant_members.as_slice(),
                expected_online,
                None,
            ) {
                Ok(primary) => {
                    let primary_target = harness.observer().postgres_routing_target(primary)?;
                    match probe_writable_primary(harness, primary_target.dsn.as_str()) {
                        Ok(()) => Err(HarnessError::message(format!(
                            "cluster still exposes a compatible writable primary `{primary}`"
                        ))),
                        Err(_) => Ok(()),
                    }
                }
                Err(_) => Ok(()),
            }
        })();
        match attempt {
            Ok(()) => return Ok(()),
            Err(err) => last_error = Some(err.to_string()),
        }
        tokio::time::sleep(poll_interval).await;
    }

    Err(HarnessError::message(format!(
        "timed out waiting for pgtm to expose no operator-visible primary; last observed error: {}",
        last_error.unwrap_or_else(|| "no no-primary verification attempt ran".to_string())
    )))
}

async fn wait_for_targeted_switchover_rejection_precondition(
    world: &mut HaWorld,
    seed_member: ClusterMember,
    target_member: ClusterMember,
) -> Result<()> {
    let deadline = {
        let harness = world.harness()?;
        Instant::now() + harness.timeouts.failover_deadline
    };
    let poll_interval = {
        let harness = world.harness()?;
        harness.timeouts.poll_interval
    };
    let mut last_error = None;

    while Instant::now() < deadline {
        let attempt: Result<()> = (|| {
            let harness = world.harness()?;
            let status = harness.observer().state_via_member(seed_member)?;
            harness.record_status_snapshot("switchover.rejected.precondition", &status)?;
            match status.dcs.member(&target_member.member_id()) {
                None => Ok(()),
                Some(member) if !member_slot_is_api_switchover_eligible(member) => Ok(()),
                Some(member) => Err(HarnessError::message(format!(
                    "target `{target_member}` is still promotion-eligible via `{seed_member}` with postgres state {:?}",
                    member.postgres()
                ))),
            }
        })();
        match attempt {
            Ok(()) => return Ok(()),
            Err(err) => last_error = Some(err.to_string()),
        }
        tokio::time::sleep(poll_interval).await;
    }

    Err(HarnessError::message(format!(
        "timed out waiting for `{target_member}` to become an ineligible targeted switchover target; last observed error: {}",
        last_error.unwrap_or_else(|| "no ineligibility check ran".to_string())
    )))
}

async fn wait_for_planned_switchover_precondition(
    world: &mut HaWorld,
    seed_member: ClusterMember,
) -> Result<ClusterMember> {
    let deadline = {
        let harness = world.harness()?;
        Instant::now() + harness.timeouts.failover_deadline
    };
    let poll_interval = {
        let harness = world.harness()?;
        harness.timeouts.poll_interval
    };
    let mut last_error = None;

    while Instant::now() < deadline {
        let attempt: Result<ClusterMember> = (|| {
            let harness = world.harness()?;
            let status = harness.observer().state_via_member(seed_member)?;
            harness.record_status_snapshot("switchover.request.precondition", &status)?;
            select_planned_switchover_target(&status, seed_member).ok_or_else(|| {
                HarnessError::message(format!(
                    "no eligible planned switchover target is currently visible via `{seed_member}`"
                ))
            })
        })();
        match attempt {
            Ok(target_member) => return Ok(target_member),
            Err(err) => last_error = Some(err.to_string()),
        }
        tokio::time::sleep(poll_interval).await;
    }

    Err(HarnessError::message(format!(
        "timed out waiting for an eligible planned switchover target via `{seed_member}`; last observed error: {}",
        last_error.unwrap_or_else(|| "no planned switchover precondition check ran".to_string())
    )))
}

fn select_planned_switchover_target(
    status: &NodeState,
    seed_member: ClusterMember,
) -> Option<ClusterMember> {
    status
        .dcs
        .members()
        .filter_map(|(member_id, member_view)| {
            ClusterMember::parse(member_id.0.as_str())
                .ok()
                .filter(|member| *member != seed_member)
                .filter(|_| member_slot_is_api_switchover_eligible(member_view))
                .map(|member| (member, planned_switchover_target_rank(member_view)))
        })
        .max_by(|(left_member, left_rank), (right_member, right_rank)| {
            left_rank
                .cmp(right_rank)
                .then_with(|| right_member.cmp(left_member))
        })
        .map(|(member, _)| member)
}

fn planned_switchover_target_rank(member: &ClusterMemberView) -> (u8, u64, u64) {
    match member.postgres() {
        MemberPostgresView::Replica {
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
        MemberPostgresView::Unknown { common } => (
            0,
            common.timeline.map_or(0, |timeline| u64::from(timeline.0)),
            0,
        ),
        MemberPostgresView::Primary { .. } => (0, 0, 0),
    }
}

fn member_slot_is_api_switchover_eligible(member: &ClusterMemberView) -> bool {
    match member.postgres() {
        MemberPostgresView::Primary { .. } => false,
        MemberPostgresView::Unknown { common } | MemberPostgresView::Replica { common, .. } => {
            common.readiness == Readiness::Ready
        }
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
        Instant::now() + kind.deadline(harness)
    };
    let poll_interval = {
        let harness = world.harness()?;
        harness.timeouts.poll_interval
    };
    let mut last_error = None;

    while Instant::now() < deadline {
        let observation_result = {
            let harness = world.harness()?;
            harness.observer().observe_states()
        };
        match observation_result {
            Ok(observation) => {
                {
                    let harness = world.harness()?;
                    record_cluster_observation(harness, phase, &observation)?;
                }
                match check(&observation) {
                    Ok(value) => return Ok(value),
                    Err(err) => last_error = Some(err.to_string()),
                }
            }
            Err(err) => last_error = Some(err.to_string()),
        }
        let terminal_error = {
            let harness = world.harness()?;
            terminal_container_failure(harness, &world.stopped_members, kind)?
        };
        if let Some(terminal_error) = terminal_error {
            return Err(HarnessError::message(format!(
                "{}\nterminal container failure detected: {terminal_error}",
                last_error.unwrap_or_else(|| "status polling failed".to_string())
            )));
        }
        tokio::time::sleep(poll_interval).await;
    }

    Err(HarnessError::message(format!(
        "{} deadline expired; last observed error: {}",
        kind.label(),
        last_error.unwrap_or_else(|| "no status observed".to_string())
    )))
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

fn compatible_primary_from_observation(
    observation: &ClusterStateObservation,
    relevant_members: &[ClusterMember],
    expected_online: usize,
    exact_primary: Option<ClusterMember>,
) -> Result<ClusterMember> {
    let states = relevant_member_states(observation, relevant_members)?;
    let primary = compatible_authoritative_primary(states.as_slice())?;
    let primary_state = require_observed_member_state(observation, primary)?;
    match authoritative_primary(primary_state) {
        Some(observed_primary) if observed_primary == primary => {}
        Some(observed_primary) => {
            return Err(HarnessError::message(format!(
                "primary member `{primary}` self-reported `{observed_primary}` instead"
            )));
        }
        None => {
            return Err(HarnessError::message(format!(
                "primary member `{primary}` did not self-report authoritative primary; authority={}",
                format_authority(primary_state),
            )));
        }
    }
    require_visible_members(primary_state, expected_online)?;
    if let Some(expected_primary) = exact_primary {
        if primary != expected_primary {
            return Err(HarnessError::message(format!(
                "expected `{expected_primary}` to be primary, observed `{primary}`"
            )));
        }
    }
    Ok(primary)
}

fn relevant_member_states<'a>(
    observation: &'a ClusterStateObservation,
    relevant_members: &[ClusterMember],
) -> Result<Vec<(ClusterMember, &'a NodeState)>> {
    relevant_members
        .iter()
        .copied()
        .map(|member| {
            require_observed_member_state(observation, member).map(|state| (member, state))
        })
        .collect::<Result<Vec<_>>>()
}

fn require_observed_member_state(
    observation: &ClusterStateObservation,
    member: ClusterMember,
) -> Result<&NodeState> {
    observation
        .get(&member)
        .ok_or_else(|| {
            HarnessError::message(format!(
                "cluster observation did not include member `{member}`"
            ))
        })?
        .as_ref()
        .ok()
        .ok_or_else(|| HarnessError::message(member_observation_failure(observation, member)))
}

fn member_observation_failure(
    observation: &ClusterStateObservation,
    member: ClusterMember,
) -> String {
    observation
        .get(&member)
        .and_then(|observation| observation.as_ref().err().map(String::as_str))
        .map(|failure| {
            format!(
                "expected `pgtm status --json` via `{member}` to succeed, but it failed: {failure}"
            )
        })
        .unwrap_or_else(|| format!("expected `pgtm status --json` via `{member}` to succeed"))
}

fn compatible_authoritative_primary(
    states: &[(ClusterMember, &NodeState)],
) -> Result<ClusterMember> {
    let observed_primaries = states
        .iter()
        .filter_map(|(_member, state)| authoritative_primary(state))
        .collect::<BTreeSet<_>>();
    match observed_primaries.len() {
        0 => Err(HarnessError::message(format!(
            "cluster has no compatible authoritative primary; observations={}",
            format_observed_authorities(states),
        ))),
        1 => observed_primaries.into_iter().next().ok_or_else(|| {
            HarnessError::message("authoritative primary set disappeared unexpectedly")
        }),
        _ => Err(HarnessError::message(format!(
            "cluster reports conflicting authoritative primaries; observations={}",
            format_observed_authorities(states),
        ))),
    }
}

fn format_observed_authorities(states: &[(ClusterMember, &NodeState)]) -> String {
    states
        .iter()
        .map(|(member, state)| format!("{member}={}", format_authority(state)))
        .collect::<Vec<_>>()
        .join("; ")
}

fn replica_members(status: &NodeState) -> Vec<ClusterMember> {
    status
        .dcs
        .members()
        .filter(|(_member_id, member)| {
            matches!(member.postgres(), MemberPostgresView::Replica { .. })
        })
        .filter_map(|(member_id, _member)| ClusterMember::parse(member_id.0.as_str()).ok())
        .collect::<Vec<_>>()
}

fn require_visible_members(status: &NodeState, expected: usize) -> Result<()> {
    let visible = status.dcs.member_count();
    if visible >= expected {
        Ok(())
    } else {
        Err(HarnessError::message(format!(
            "expected at least {expected} visible members, observed {visible}; warnings={}",
            format_warnings(status)
        )))
    }
}

fn format_warnings(status: &NodeState) -> String {
    let mut warnings = Vec::new();
    if !status.dcs.is_quorum() {
        warnings.push("dcs_mode=not_trusted".to_string());
    }
    if !matches!(
        status.ha.publication,
        PublicationState::Projected(AuthorityProjection::Primary(_))
    ) {
        warnings.push(format!("authority={}", format_authority(status)));
    }
    if status.dcs.member_count() == 0 {
        warnings.push("no_members".to_string());
    }
    if warnings.is_empty() {
        "none".to_string()
    } else {
        warnings.join("; ")
    }
}

fn format_authority(status: &NodeState) -> String {
    match &status.ha.publication {
        PublicationState::Projected(AuthorityProjection::Primary(epoch)) => {
            format!("primary({})", epoch.holder.0)
        }
        PublicationState::Projected(AuthorityProjection::NoPrimary(reason)) => {
            format!("no_primary({reason:?})").to_lowercase()
        }
        PublicationState::Unknown => "unknown".to_string(),
    }
}

fn authoritative_primary(status: &NodeState) -> Option<ClusterMember> {
    match &status.ha.publication {
        PublicationState::Projected(AuthorityProjection::Primary(epoch)) => {
            ClusterMember::parse(epoch.holder.0.as_str()).ok()
        }
        PublicationState::Unknown
        | PublicationState::Projected(AuthorityProjection::NoPrimary(_)) => None,
    }
}

fn probe_writable_primary(harness: &HarnessShared, dsn: &str) -> Result<()> {
    let probe_sql =
        "CREATE TEMP TABLE pgtm_writable_primary_probe ON COMMIT DROP AS SELECT 'probe'::text AS token;";
    let _ = sql::execute(
        harness.materialized_dir(),
        dsn,
        probe_sql,
        harness.timeouts.poll_interval,
    )?;
    Ok(())
}

fn terminal_container_failure(
    harness: &HarnessShared,
    expected_offline: &BTreeSet<ClusterMember>,
    kind: PollKind,
) -> Result<Option<String>> {
    let cluster_members = all_cluster_members();
    let services = match kind {
        PollKind::Startup | PollKind::Recovery => cluster_members.as_slice(),
    };

    let mut failures = Vec::new();
    for service in services {
        if expected_offline.contains(service) {
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

fn online_expected_count(world: &HaWorld) -> usize {
    online_member_ids(world).len()
}

fn online_member_ids(world: &HaWorld) -> Vec<ClusterMember> {
    all_cluster_members()
        .iter()
        .filter(|member| {
            !world.stopped_members.contains(*member)
                && !world.observer_unreachable_members.contains(*member)
        })
        .copied()
        .collect::<Vec<_>>()
}

fn all_cluster_members() -> [ClusterMember; 3] {
    ClusterMember::ALL
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
