use std::{
    collections::{BTreeMap, BTreeSet},
    hash::{DefaultHasher, Hash, Hasher},
    time::{Duration, Instant},
};

use cucumber::{given, then, when};
use pgtuskmaster_rust::{
    api::NodeState,
    dcs::{DcsMemberPostgresView, DcsMemberView, DcsTrust},
    ha::types::{AuthorityProjection, PublicationState, TargetRole},
    pginfo::state::Readiness,
};

use crate::support::{
    error::{HarnessError, Result},
    faults::{BlockerKind, TrafficPath},
    observer::pgtm::ConnectionTarget,
    topology::ClusterMember,
    world::{HaWorld, HarnessShared, MemberSet},
};

#[given(regex = r#"^the "([^"]+)" harness is running$"#)]
async fn the_harness_is_running(world: &mut HaWorld, given_name: String) -> Result<()> {
    let harness = HarnessShared::initialize(given_name.as_str()).await?;
    harness.record_note("scenario", format!("started given `{given_name}`"))?;
    world.set_harness(harness);
    Ok(())
}

#[given("the cluster reaches one stable primary")]
#[then("the cluster reaches one stable primary")]
async fn the_cluster_reaches_one_stable_primary(world: &mut HaWorld) -> Result<()> {
    let primary = wait_for_authoritative_single_primary(
        world,
        "legacy.stable_primary",
        PollKind::Startup,
        all_cluster_members().len(),
        None,
        None,
    )
    .await?;
    world.remember_alias("initial_primary", primary);
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
        all_cluster_members().len(),
        None,
        None,
    )
    .await?;
    world.remember_alias(alias.as_str(), primary);
    world.remember_alias("current_primary", primary);
    Ok(())
}

#[given(regex = r#"^I choose one non-primary node as "([^"]+)"$"#)]
async fn i_choose_one_non_primary_node_as(world: &mut HaWorld, alias: String) -> Result<()> {
    let replicas =
        wait_for_minimum_replicas(world, format!("choose_one_replica.{alias}").as_str(), 1).await?;
    let member_id = replicas
        .first()
        .cloned()
        .ok_or_else(|| HarnessError::message("cluster has no non-primary node to choose"))?;
    record_alias(world, alias.as_str(), member_id, "choose_one_replica")
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
            record_alias(
                world,
                alias_a.as_str(),
                *member_a,
                "choose_two_replicas",
            )?;
            record_alias(
                world,
                alias_b.as_str(),
                *member_b,
                "choose_two_replicas",
            )
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

#[given(regex = r#"^I record the remaining replica as "([^"]+)"$"#)]
async fn i_record_the_remaining_replica_as(world: &mut HaWorld, alias: String) -> Result<()> {
    let replicas = wait_for_replicas(
        world,
        format!("record_remaining_replica.{alias}").as_str(),
        2,
    )
    .await?;
    let used_members = world
        .scenario
        .aliases
        .members_by_alias
        .values()
        .cloned()
        .collect::<BTreeSet<_>>();
    let member_id = replicas
        .into_iter()
        .find(|member_id| !used_members.contains(member_id))
        .ok_or_else(|| HarnessError::message("could not find an unrecorded remaining replica"))?;
    record_alias(world, alias.as_str(), member_id, "record_remaining_replica")
}

#[given("I create a proof table for this feature")]
async fn i_create_a_proof_table_for_this_feature(world: &mut HaWorld) -> Result<()> {
    let _ = ensure_proof_table(world)?;
    Ok(())
}

#[given(regex = r#"^I insert proof row "([^"]+)" through "([^"]+)"$"#)]
#[when(regex = r#"^I insert proof row "([^"]+)" through "([^"]+)"$"#)]
#[then(regex = r#"^I insert proof row "([^"]+)" through "([^"]+)"$"#)]
async fn i_insert_proof_row_through(
    world: &mut HaWorld,
    row_value: String,
    member_ref: String,
) -> Result<()> {
    insert_proof_row(world, row_value.as_str(), member_ref.as_str()).await
}

#[then(regex = r#"^the (\d+) online nodes contain exactly the recorded proof rows$"#)]
async fn the_online_nodes_contain_exactly_the_recorded_proof_rows(
    world: &mut HaWorld,
    expected_online: String,
) -> Result<()> {
    let expected_online = parse_count(expected_online.as_str())?;
    wait_for_recorded_proof_rows(world, expected_online).await
}

#[when("the current primary container crashes")]
async fn the_current_primary_container_crashes(world: &mut HaWorld) -> Result<()> {
    let status = current_status(world)?;
    let primary_member = single_primary(&status)?;
    {
        let harness = world.harness()?;
        harness.kill_node(primary_member)?;
    }
    world.remember_alias("killed_node", primary_member);
    world.add_stopped_node(primary_member);
    Ok(())
}

#[when(regex = r#"^I kill the node named "([^"]+)"$"#)]
async fn i_kill_the_node_named(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    {
        let harness = world.harness()?;
        harness.kill_node(member_id)?;
    }
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
        let member_id = resolve_member_reference(world, member_ref)?;
        {
            let harness = world.harness()?;
            harness.kill_node(member_id)?;
        }
        world.add_stopped_node(member_id);
    }
    Ok(())
}

#[when("I kill all database nodes")]
async fn i_kill_all_database_nodes(world: &mut HaWorld) -> Result<()> {
    for member_id in all_cluster_members() {
        {
            let harness = world.harness()?;
            harness.kill_node(member_id)?;
        }
        world.add_stopped_node(member_id);
    }
    Ok(())
}

#[when("I start the killed node container again")]
async fn i_start_the_killed_node_container_again(world: &mut HaWorld) -> Result<()> {
    let killed_node = world.require_alias("killed_node")?;
    {
        let harness = world.harness()?;
        harness.start_node(killed_node)?;
    }
    world.remove_stopped_node(killed_node);
    Ok(())
}

#[when(regex = r#"^I restart the node named "([^"]+)"$"#)]
async fn i_restart_the_node_named(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    {
        let harness = world.harness()?;
        harness.start_node(member_id)?;
    }
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
        let member_id = resolve_member_reference(world, member_ref)?;
        {
            let harness = world.harness()?;
            harness.start_node(member_id)?;
        }
        world.remove_stopped_node(member_id);
    }
    Ok(())
}

#[when("I request a planned switchover")]
async fn i_request_a_planned_switchover(world: &mut HaWorld) -> Result<()> {
    world.clear_primary_history();
    let seed_member = world.require_alias("current_primary")?;
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
    world.clear_primary_history();
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    let seed_member = world.require_alias("current_primary")?;
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

#[given("I record the current pgtm primary and replicas views")]
#[then("I record the current pgtm primary and replicas views")]
async fn i_record_the_current_pgtm_primary_and_replicas_views(world: &mut HaWorld) -> Result<()> {
    let harness = world.harness()?;
    let primary = harness.observer().primary_tls_json()?;
    let replicas = harness.observer().replicas_tls_json()?;
    harness.record_note(
        "pgtm.views.before_action",
        serde_json::to_string(&serde_json::json!({
            "primary": primary,
            "replicas": replicas,
        }))
        .map_err(|source| HarnessError::Json {
            context: "serializing pgtm primary/replicas snapshot".to_string(),
            source,
        })?,
    )?;
    Ok(())
}

#[then("after the configured HA lease deadline a different node becomes the only primary")]
async fn after_the_configured_ha_lease_deadline_a_different_node_becomes_the_only_primary(
    world: &mut HaWorld,
) -> Result<()> {
    let killed_node = world.require_alias("killed_node")?;
    let new_primary = wait_for_single_primary(
        world,
        "legacy.failover.new_primary",
        PollKind::Failover,
        online_expected_count(world),
        None,
        Some(killed_node),
    )
    .await?;
    world.remember_alias("new_primary", new_primary);
    Ok(())
}

#[then(regex = r#"^the primary named "([^"]+)" remains the only primary$"#)]
async fn the_primary_named_remains_the_only_primary(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    let observed = wait_for_single_primary(
        world,
        format!("primary.same.{member_id}").as_str(),
        PollKind::Failover,
        online_expected_count(world),
        Some(member_id),
        None,
    )
    .await?;
    let _ = wait_for_primary_resolution_for_member(
        world,
        format!("primary.same.resolution.{member_id}").as_str(),
        PollKind::Failover,
        Some(member_id),
    )
    .await?;
    if observed != member_id {
        return Err(HarnessError::message(format!(
            "expected `{member_id}` to remain primary, observed `{observed}`"
        )));
    }
    Ok(())
}

#[then(regex = r#"^there is no operator-visible primary across (\d+) online node[s]?$"#)]
async fn there_is_no_operator_visible_primary_across_online_nodes(
    world: &mut HaWorld,
    expected_online: String,
) -> Result<()> {
    let expected_online = parse_count(expected_online.as_str())?;
    wait_for_no_operator_primary(world, expected_online).await
}

#[then("the lone online node is not treated as a writable primary")]
async fn the_lone_online_node_is_not_treated_as_a_writable_primary(
    world: &mut HaWorld,
) -> Result<()> {
    let online_members = online_member_ids(world);
    if online_members.len() != 1 {
        return Err(HarnessError::message(format!(
            "expected exactly one online member, observed {:?}",
            online_members
        )));
    }
    wait_for_members_to_reject_proof_writes(world, online_members.as_slice()).await
}

#[then(regex = r#"^exactly one primary exists across (\d+) running node(?:s)? as "([^"]+)"$"#)]
async fn exactly_one_primary_exists_across_running_nodes_as(
    world: &mut HaWorld,
    expected_online: String,
    alias: String,
) -> Result<()> {
    let expected_online = parse_count(expected_online.as_str())?;
    let intended_online = online_member_ids(world);
    let primary = poll_for_status(
        world,
        format!("primary.across.{expected_online}.{alias}").as_str(),
        PollKind::Recovery,
        |status| {
            require_visible_members(status, expected_online)?;
            let primary = single_primary(status)?;
            if intended_online.iter().any(|member_id| member_id == &primary) {
                Ok(primary)
            } else {
                Err(HarnessError::message(format!(
                    "expected operator-visible primary within {:?}, observed `{primary}`",
                    intended_online
                )))
            }
        },
    )
    .await?;
    let _ = wait_for_primary_resolution_for_member(
        world,
        format!("primary.across.primary.{expected_online}.{alias}").as_str(),
        PollKind::Recovery,
        Some(primary),
    )
    .await?;
    world.remember_alias(alias.as_str(), primary);
    world.remember_alias("current_primary", primary);
    Ok(())
}

#[then(regex = r#"^I wait for a different stable primary than "([^"]+)" as "([^"]+)"$"#)]
async fn i_wait_for_a_different_stable_primary_than_as(
    world: &mut HaWorld,
    previous_ref: String,
    alias: String,
) -> Result<()> {
    let previous_member = resolve_member_reference(world, previous_ref.as_str())?;
    let primary = wait_for_single_primary(
        world,
        format!("primary.changed.{alias}").as_str(),
        PollKind::Failover,
        online_expected_count(world),
        None,
        Some(previous_member),
    )
    .await?;
    let _ = wait_for_primary_resolution_for_member(
        world,
        format!("primary.changed.primary.{alias}").as_str(),
        PollKind::Failover,
        Some(primary),
    )
    .await?;
    world.remember_alias(alias.as_str(), primary);
    world.remember_alias("current_primary", primary);
    Ok(())
}

#[then(regex = r#"^I wait for the primary named "([^"]+)" to become the only primary$"#)]
async fn i_wait_for_the_primary_named_to_become_the_only_primary(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    let observed = wait_for_single_primary(
        world,
        format!("primary.targeted.{member_id}").as_str(),
        PollKind::Failover,
        online_expected_count(world),
        Some(member_id),
        None,
    )
    .await?;
    let _ = wait_for_primary_resolution_for_member(
        world,
        format!("primary.targeted.primary.{member_id}").as_str(),
        PollKind::Failover,
        Some(member_id),
    )
    .await?;
    if observed != member_id {
        return Err(HarnessError::message(format!(
            "expected `{member_id}` to become primary, observed `{observed}`"
        )));
    }
    Ok(())
}

#[then("the remaining online non-primary node is a replica")]
async fn the_remaining_online_non_primary_node_is_a_replica(world: &mut HaWorld) -> Result<()> {
    let intended_online = online_member_ids(world);
    let expected_online = online_expected_count(world);
    poll_for_status(
        world,
        "remaining.online.replica",
        PollKind::Failover,
        |status| {
            require_visible_members(status, expected_online)?;
            let primary = single_primary(status)?;
            let replicas = replica_members(status)
                .into_iter()
                .filter(|member_id| {
                    member_id != &primary
                        && intended_online
                            .iter()
                            .any(|expected_member_id| expected_member_id == member_id)
                })
                .count();
            if replicas == 1 {
                Ok(())
            } else {
                Err(HarnessError::message(format!(
                    "expected one remaining replica, observed {replicas}"
                )))
            }
        },
    )
    .await
}

#[then("the cluster is degraded but operational across 2 running nodes")]
async fn the_cluster_is_degraded_but_operational_across_two_running_nodes(
    world: &mut HaWorld,
) -> Result<()> {
    let intended_online = online_member_ids(world);
    poll_for_status(
        world,
        "cluster.degraded_two_node",
        PollKind::Recovery,
        |status| {
            require_visible_members(status, 2)?;
            let primary = single_primary(status)?;
            if !intended_online.iter().any(|member_id| member_id == &primary) {
                return Err(HarnessError::message(format!(
                    "expected operator-visible primary within {:?}, observed `{primary}`",
                    intended_online
                )));
            }
            let non_primary_members = operator_visible_member_ids(status)
                .into_iter()
                .filter(|member_id| {
                    member_id != &primary
                        && intended_online.iter().any(|expected| expected == member_id)
                })
                .collect::<Vec<_>>();
            if non_primary_members.len() == 1 {
                Ok(())
            } else {
                Err(HarnessError::message(format!(
                "expected exactly one non-primary in degraded two-node state, observed {}",
                non_primary_members
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            )))
            }
        },
    )
    .await
}

#[then("after the configured recovery deadline the restarted node rejoins as a replica")]
#[when("after the configured recovery deadline the restarted node rejoins as a replica")]
async fn after_the_configured_recovery_deadline_the_restarted_node_rejoins_as_a_replica(
    world: &mut HaWorld,
) -> Result<()> {
    wait_for_member_to_rejoin_as_replica(world, "killed_node").await
}

#[then(regex = r#"^the node named "([^"]+)" rejoins as a replica$"#)]
#[when(regex = r#"^the node named "([^"]+)" rejoins as a replica$"#)]
async fn the_node_named_rejoins_as_a_replica(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    wait_for_member_to_rejoin_as_replica(world, member_ref.as_str()).await
}

#[then(regex = r#"^the node named "([^"]+)" remains online as a replica$"#)]
#[when(regex = r#"^the node named "([^"]+)" remains online as a replica$"#)]
async fn the_node_named_remains_online_as_a_replica(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    wait_for_member_to_rejoin_as_replica(world, member_ref.as_str()).await
}

#[then(regex = r#"^the node named "([^"]+)" remains offline$"#)]
async fn the_node_named_remains_offline(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    let harness = world.harness()?;
    let container_id = harness.service_container_id(member_id.into())?;
    let state = harness
        .docker
        .container_state_status(container_id.as_str())?;
    if matches!(state.as_str(), "exited" | "dead") {
        Ok(())
    } else {
        Err(HarnessError::message(format!(
            "expected `{member_id}` to stay offline, observed docker state `{state}`"
        )))
    }
}

#[then("the proof row is visible from the restarted node")]
async fn the_proof_row_is_visible_from_the_restarted_node(world: &mut HaWorld) -> Result<()> {
    let member_id = world.require_alias("killed_node")?;
    let expected_rows = world
        .scenario
        .workload
        .proof
        .recorded_rows
        .iter()
        .map(|row| row.as_str().to_string())
        .collect::<Vec<_>>();
    wait_for_member_rows(world, member_id.as_str(), &expected_rows).await
}

#[then("the cluster still has exactly one primary")]
async fn the_cluster_still_has_exactly_one_primary(world: &mut HaWorld) -> Result<()> {
    let status = current_status(world)?;
    require_visible_members(&status, online_expected_count(world))?;
    let _ = single_primary(&status)?;
    Ok(())
}

#[then(regex = r#"^pgtm primary points to "([^"]+)"$"#)]
async fn pgtm_primary_points_to(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    let harness = world.harness()?;
    let primary = harness.observer().primary_tls_json()?;
    match primary.targets.as_slice() {
        [target] if target.member_id == member_id.service_name() => Ok(()),
        [target] => Err(HarnessError::message(format!(
            "pgtm primary pointed to `{}` instead of `{member_id}`",
            target.member_id
        ))),
        [] => Err(HarnessError::message("pgtm primary returned zero targets")),
        _ => Err(HarnessError::message(format!(
            "pgtm primary returned multiple targets: {}",
            primary
                .targets
                .iter()
                .map(|target| target.member_id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

#[then(regex = r#"^pgtm replicas list every cluster member except "([^"]+)"$"#)]
async fn pgtm_replicas_list_every_cluster_member_except(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let excluded_member = resolve_member_reference(world, member_ref.as_str())?;
    let expected = all_cluster_members()
        .iter()
        .filter(|member_id| **member_id != excluded_member)
        .map(|member_id| (*member_id).to_string())
        .collect::<BTreeSet<_>>();
    wait_for_pgtm_replicas(world, expected).await
}

#[then(regex = r#"^the primary history never included "([^"]+)"$"#)]
async fn the_primary_history_never_included(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    if world
        .scenario
        .invariants
        .observed_authoritative_primaries
        .iter()
        .any(|observed| observed == &member_id)
    {
        let history = world
            .scenario
            .invariants
            .observed_authoritative_primaries
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        return Err(HarnessError::message(format!(
            "primary history unexpectedly included `{member_id}`: {history}",
        )));
    }
    Ok(())
}

async fn insert_proof_row(world: &mut HaWorld, row_value: &str, member_ref: &str) -> Result<()> {
    let table_name = ensure_proof_table(world)?;
    let member_id = resolve_member_reference(world, member_ref)?;
    let target = sql_target_for_member(world.harness()?, member_id)?;
    let create_sql = format!("CREATE TABLE IF NOT EXISTS {table_name} (token TEXT PRIMARY KEY);");
    let insert_sql = format!(
        "INSERT INTO {table_name} (token) VALUES ('{}') ON CONFLICT (token) DO NOTHING;",
        sql_quote_literal(row_value)
    );
    let harness = world.harness()?;
    if let Err(err) = harness
        .sql()
        .execute(target.dsn.as_str(), insert_sql.as_str())
    {
        if !err.to_string().contains("does not exist") {
            return Err(err);
        }
        let _ = harness
            .sql()
            .execute(target.dsn.as_str(), create_sql.as_str())?;
        let _ = harness
            .sql()
            .execute(target.dsn.as_str(), insert_sql.as_str())?;
    }
    harness.record_note(
        "sql.insert_proof_row",
        format!("member={member_id} row={row_value}"),
    )?;
    if !world
        .scenario
        .workload
        .proof
        .recorded_rows
        .iter()
        .any(|existing| existing.as_str() == row_value)
    {
        world
            .scenario
            .workload
            .proof
            .recorded_rows
            .push(row_value.into());
    }
    if world.scenario.transition.stopped_members.is_empty()
        && world
            .scenario
            .transition
            .observation_scope
            .observer_unreachable_members
            .is_empty()
        && world.scenario.transition.wedged_members.is_empty()
        && world
            .scenario
            .workload
            .proof
            .convergence_blocked_members
            .is_empty()
    {
        let expected_online = online_expected_count(world);
        return wait_for_recorded_proof_rows(world, expected_online).await;
    }
    Ok(())
}

fn ensure_proof_table(world: &mut HaWorld) -> Result<String> {
    if let Some(table_name) = world.scenario.workload.proof.table.as_ref() {
        return Ok(table_name.as_str().to_string());
    }

    let harness = world.harness()?;
    let table_name = proof_table_name(harness);
    let create_sql = format!("CREATE TABLE IF NOT EXISTS {table_name} (token TEXT PRIMARY KEY);");
    let primary = current_primary_target(harness)?;
    let _ = harness
        .sql()
        .execute(primary.dsn.as_str(), create_sql.as_str())?;
    harness.record_note("sql.create_proof_table", format!("table={table_name}"))?;
    world.scenario.workload.proof.table = Some(table_name.clone().into());
    Ok(table_name)
}

async fn wait_for_recorded_proof_rows(world: &mut HaWorld, expected_online: usize) -> Result<()> {
    let table_name = world
        .scenario
        .workload
        .proof
        .table
        .as_ref()
        .map(|table| table.as_str().to_string())
        .ok_or_else(|| HarnessError::message("proof table was not created"))?;
    let expected_rows = world
        .scenario
        .workload
        .proof
        .recorded_rows
        .iter()
        .map(|row| row.as_str().to_string())
        .collect::<Vec<_>>();
    let deadline = {
        let harness = world.harness()?;
        Instant::now() + harness.timeouts.recovery_deadline
    };
    let poll_interval = {
        let harness = world.harness()?;
        harness.timeouts.poll_interval
    };
    let mut last_error = None;

    while Instant::now() < deadline {
        let attempt = {
            let targets = direct_online_connection_targets(world)?;
            if targets.len() < expected_online {
                Err(HarnessError::message(format!(
                    "expected at least {expected_online} online connection targets, observed {}",
                    targets.len()
                )))
            } else {
                let harness = world.harness()?;
                verify_rows_on_targets(
                    harness,
                    targets.as_slice(),
                    table_name.as_str(),
                    &expected_rows,
                )
            }
        };
        match attempt {
            Ok(()) => return Ok(()),
            Err(err) => last_error = Some(err.to_string()),
        }
        tokio::time::sleep(poll_interval).await;
    }

    Err(HarnessError::message(format!(
        "timed out waiting for exact proof-row convergence on {expected_online} nodes; last observed error: {}",
        last_error.unwrap_or_else(|| "no proof-row verification attempt ran".to_string())
    )))
}

async fn wait_for_member_rows(
    world: &mut HaWorld,
    member_ref: &str,
    expected_rows: &[String],
) -> Result<()> {
    let table_name = world
        .scenario
        .workload
        .proof
        .table
        .as_ref()
        .map(|table| table.as_str().to_string())
        .ok_or_else(|| HarnessError::message("proof table was not created"))?;
    let member_id = resolve_member_reference(world, member_ref)?;
    let deadline = {
        let harness = world.harness()?;
        Instant::now() + harness.timeouts.recovery_deadline
    };
    let poll_interval = {
        let harness = world.harness()?;
        harness.timeouts.poll_interval
    };
    let mut last_error = None;

    while Instant::now() < deadline {
        let attempt = {
            let harness = world.harness()?;
            fetch_rows_for_member(harness, table_name.as_str(), member_id).and_then(
                |observed_rows| {
                    assert_exact_rows(member_id.service_name(), &observed_rows, expected_rows)
                },
            )
        };
        match attempt {
            Ok(()) => return Ok(()),
            Err(err) => last_error = Some(err.to_string()),
        }
        tokio::time::sleep(poll_interval).await;
    }

    Err(HarnessError::message(format!(
        "timed out waiting for proof rows on `{member_id}`; last observed error: {}",
        last_error.unwrap_or_else(|| "no row verification attempt ran".to_string())
    )))
}

async fn wait_for_single_primary(
    world: &mut HaWorld,
    phase: &str,
    kind: PollKind,
    expected_online: usize,
    exact_primary: Option<ClusterMember>,
    different_from: Option<ClusterMember>,
) -> Result<ClusterMember> {
    let expected_primary = exact_primary;
    let previous_primary = different_from;
    poll_for_status(world, phase, kind, |status| {
        require_visible_members(status, expected_online)?;
        let primary = single_primary(status)?;
        if let Some(expected_primary) = expected_primary.as_ref() {
            if primary != *expected_primary {
                return Err(HarnessError::message(format!(
                    "expected `{expected_primary}` to be primary, observed `{primary}`"
                )));
            }
        }
        if let Some(previous_primary) = previous_primary.as_ref() {
            if primary == *previous_primary {
                return Err(HarnessError::message(format!(
                    "expected a different primary than `{previous_primary}`, observed `{primary}`"
                )));
            }
        }
        Ok(primary)
    })
    .await
}

async fn wait_for_authoritative_single_primary(
    world: &mut HaWorld,
    phase: &str,
    kind: PollKind,
    expected_online: usize,
    exact_primary: Option<ClusterMember>,
    different_from: Option<ClusterMember>,
) -> Result<ClusterMember> {
    let expected_primary = exact_primary;
    let previous_primary = different_from;
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
            let status = {
                let harness = world.harness()?;
                let status = harness.observer().state()?;
                harness.record_status_snapshot(phase, &status)?;
                status
            };
            require_visible_members(&status, expected_online)?;
            let primary = single_primary(&status)?;
            if let Some(expected_primary) = expected_primary.as_ref() {
                if primary != *expected_primary {
                    Err(HarnessError::message(format!(
                        "expected `{expected_primary}` to be primary, observed `{primary}`"
                    )))?;
                }
            }
            if let Some(previous_primary) = previous_primary.as_ref() {
                if primary == *previous_primary {
                    Err(HarnessError::message(format!(
                        "expected a different primary than `{previous_primary}`, observed `{primary}`"
                    )))?;
                }
            }
            let target = {
                let harness = world.harness()?;
                let target = current_primary_target(harness)?;
                let _ = harness.sql().execute(target.dsn.as_str(), "SELECT 1;")?;
                target
            };
            if target.member_id != primary.service_name() {
                Err(HarnessError::message(format!(
                    "DCS-reported primary was `{primary}`, but authoritative pgtm primary resolved to `{}`",
                    target.member_id
                )))?;
            }
            Ok(primary)
        })();
        match attempt {
            Ok(primary) => return Ok(primary),
            Err(err) => last_error = Some(err.to_string()),
        }
        let terminal_error = {
            let harness = world.harness()?;
            terminal_container_failure(harness, &world.scenario.transition.stopped_members, kind)?
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

async fn wait_for_no_operator_primary(world: &mut HaWorld, expected_online: usize) -> Result<()> {
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
            for member_id in online_member_ids(world) {
                let status = harness.observer().state_via_member(member_id)?;
                let snapshot_label = format!("primary.none.{member_id}");
                harness.record_status_snapshot(snapshot_label.as_str(), &status)?;
                require_visible_members(&status, expected_online)?;
                require_no_authoritative_primary(&status)?;
            }
            Ok(())
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

async fn wait_for_member_to_rejoin_as_replica(world: &mut HaWorld, member_ref: &str) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref)?;
    let expected_online = online_expected_count(world);
    let deadline = {
        let harness = world.harness()?;
        Instant::now() + harness.timeouts.recovery_deadline
    };
    let poll_interval = {
        let harness = world.harness()?;
        harness.timeouts.poll_interval
    };
    let mut last_error = None;

    while Instant::now() < deadline {
        let attempt = {
            let harness = world.harness()?;
            assert_member_is_replica_via_member(harness, member_id, expected_online)
        };
        match attempt {
            Ok(()) => return Ok(()),
            Err(err) => last_error = Some(err.to_string()),
        }
        tokio::time::sleep(poll_interval).await;
    }

    Err(HarnessError::message(format!(
        "timed out waiting for `{member_id}` to report and behave as a replica; last observed error: {}",
        last_error.unwrap_or_else(|| "no replica verification attempt ran".to_string())
    )))
}

async fn wait_for_members_to_reject_proof_writes(
    world: &mut HaWorld,
    members: &[ClusterMember],
) -> Result<()> {
    let table_name = ensure_proof_table(world)?;
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
            for member in members {
                let probe_value = format!("non-writable-probe-{member}");
                let target = sql_target_for_member(harness, *member)?;
                let probe_sql = format!(
                    "BEGIN; INSERT INTO {table_name} (token) VALUES ('{}'); ROLLBACK;",
                    sql_quote_literal(probe_value.as_str())
                );
                if let Ok(output) = harness.sql().execute(target.dsn.as_str(), probe_sql.as_str())
                {
                    Err(HarnessError::message(format!(
                        "member `{member}` unexpectedly accepted a write probe while no primary should be writable: {output}"
                    )))?;
                }
            }
            Ok(())
        })();
        match attempt {
            Ok(()) => return Ok(()),
            Err(err) => last_error = Some(err.to_string()),
        }
        tokio::time::sleep(poll_interval).await;
    }

    Err(HarnessError::message(format!(
        "timed out waiting for members {:?} to reject proof writes; last observed error: {}",
        members,
        last_error.unwrap_or_else(|| "no write-rejection verification attempt ran".to_string())
    )))
}

async fn wait_for_pgtm_replicas(world: &mut HaWorld, expected: BTreeSet<String>) -> Result<()> {
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
            let replicas = harness.observer().replicas_tls_json()?;
            let observed = replicas
                .targets
                .iter()
                .map(|target| target.member_id.clone())
                .collect::<BTreeSet<_>>();
            if observed == expected {
                Ok(())
            } else {
                Err(HarnessError::message(format!(
                    "expected pgtm replicas {:?}, observed {:?}",
                    expected, observed
                )))
            }
        })();
        match attempt {
            Ok(()) => return Ok(()),
            Err(err) => last_error = Some(err.to_string()),
        }
        tokio::time::sleep(poll_interval).await;
    }

    Err(HarnessError::message(format!(
        "timed out waiting for pgtm replicas {:?}; last observed error: {}",
        expected,
        last_error.unwrap_or_else(|| "no replicas verification attempt ran".to_string())
    )))
}

async fn wait_for_replicas(
    world: &mut HaWorld,
    phase: &str,
    expected_replicas: usize,
) -> Result<Vec<ClusterMember>> {
    let expected_online = online_expected_count(world);
    poll_for_status(world, phase, PollKind::Startup, |status| {
        require_visible_members(status, expected_online)?;
        let replicas = replica_members(status);
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
    let expected_online = online_expected_count(world);
    poll_for_status(world, phase, PollKind::Startup, |status| {
        require_visible_members(status, expected_online)?;
        let replicas = replica_members(status);
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

async fn wait_for_primary_resolution_for_member(
    world: &mut HaWorld,
    phase: &str,
    kind: PollKind,
    expected_member_id: Option<ClusterMember>,
) -> Result<ConnectionTarget> {
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
        let attempt: Result<ConnectionTarget> = (|| {
            let target = {
                let harness = world.harness()?;
                let status = harness.observer().state()?;
                harness.record_status_snapshot(phase, &status)?;
                let target = current_primary_target(harness)?;
                if let Some(expected_member_id) = expected_member_id {
                    if target.member_id != expected_member_id.service_name() {
                        Err(HarnessError::message(format!(
                            "pgtm primary resolved to `{}` instead of expected `{expected_member_id}`",
                            target.member_id
                        )))?;
                    }
                }
                let _ = harness.sql().execute(target.dsn.as_str(), "SELECT 1;")?;
                target
            };
            let primary_member = ClusterMember::parse(target.member_id.as_str())?;
            world.record_primary_observation(primary_member);
            Ok(target)
        })();
        match attempt {
            Ok(target) => return Ok(target),
            Err(err) => last_error = Some(err.to_string()),
        }
        tokio::time::sleep(poll_interval).await;
    }

    Err(HarnessError::message(format!(
        "{} deadline expired while waiting for an authoritative pgtm primary target; last observed error: {}",
        kind.label(),
        last_error.unwrap_or_else(|| "no primary-resolution attempt ran".to_string())
    )))
}

async fn poll_for_status<T, F>(
    world: &mut HaWorld,
    phase: &str,
    kind: PollKind,
    mut check: F,
) -> Result<T>
where
    F: FnMut(&NodeState) -> Result<T>,
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
        let status_result = {
            let harness = world.harness()?;
            harness.observer().state()
        };
        match status_result {
            Ok(status) => {
                {
                    let harness = world.harness()?;
                    harness.record_status_snapshot(phase, &status)?;
                }
                match check(&status) {
                    Ok(value) => return Ok(value),
                    Err(err) => last_error = Some(err.to_string()),
                }
            }
            Err(err) => last_error = Some(err.to_string()),
        }
        let terminal_error = {
            let harness = world.harness()?;
            terminal_container_failure(harness, &world.scenario.transition.stopped_members, kind)?
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

fn current_status(world: &HaWorld) -> Result<NodeState> {
    let harness = world.harness()?;
    let status = harness.observer().state()?;
    harness.record_status_snapshot("status.instant", &status)?;
    Ok(status)
}

fn current_primary_target(harness: &HarnessShared) -> Result<ConnectionTarget> {
    let primary = harness.observer().primary_tls_json()?;
    match primary.targets.as_slice() {
        [target] => Ok(target.clone()),
        [] => Err(HarnessError::message("pgtm primary returned zero targets")),
        _ => Err(HarnessError::message(format!(
            "pgtm primary returned multiple targets: {}",
            primary
                .targets
                .iter()
                .map(|target| target.member_id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

fn current_connection_targets(harness: &HarnessShared) -> Result<Vec<ConnectionTarget>> {
    let primary = harness.observer().primary_tls_json()?;
    let replicas = harness.observer().replicas_tls_json()?;
    let mut by_member = BTreeMap::new();
    for target in primary
        .targets
        .into_iter()
        .chain(replicas.targets.into_iter())
    {
        by_member.insert(target.member_id.clone(), target);
    }
    Ok(by_member.into_values().collect::<Vec<_>>())
}

fn direct_connection_target(member_id: ClusterMember) -> ConnectionTarget {
    ConnectionTarget {
        member_id: member_id.to_string(),
        postgres_host: member_id.to_string(),
        postgres_port: 5432,
        dsn: format!(
            "host={member_id} port=5432 user=postgres dbname=postgres sslmode=verify-full sslrootcert=/etc/pgtuskmaster/tls/ca.crt sslcert=/etc/pgtuskmaster/tls/observer.crt sslkey=/etc/pgtuskmaster/tls/observer.key"
        ),
    }
}

fn direct_online_connection_targets(world: &HaWorld) -> Result<Vec<ConnectionTarget>> {
    Ok(online_member_ids(world)
        .into_iter()
        .map(direct_connection_target)
        .collect::<Vec<_>>())
}

fn pgtm_connection_target_for_member(
    harness: &HarnessShared,
    member_id: ClusterMember,
) -> Result<ConnectionTarget> {
    if let Ok(primary_target) = current_primary_target(harness) {
        if primary_target.member_id == member_id.service_name() {
            return Ok(primary_target);
        }
    }

    current_connection_targets(harness)?
        .into_iter()
        .find(|target| target.member_id == member_id.service_name())
        .ok_or_else(|| {
            HarnessError::message(format!(
                "member `{member_id}` is not currently reachable through pgtm connection helpers"
            ))
        })
}

fn sql_target_for_member(
    harness: &HarnessShared,
    member_id: ClusterMember,
) -> Result<ConnectionTarget> {
    match pgtm_connection_target_for_member(harness, member_id) {
        Ok(target) => Ok(target),
        Err(_) => Ok(direct_connection_target(member_id)),
    }
}

fn verify_rows_on_targets(
    harness: &HarnessShared,
    targets: &[ConnectionTarget],
    table_name: &str,
    expected_rows: &[String],
) -> Result<()> {
    for target in targets {
        let observed_rows = fetch_rows_via_target(harness, table_name, target)?;
        assert_exact_rows(target.member_id.as_str(), &observed_rows, expected_rows)?;
    }
    Ok(())
}

fn fetch_rows_for_member(
    harness: &HarnessShared,
    table_name: &str,
    member_id: ClusterMember,
) -> Result<Vec<String>> {
    let target = sql_target_for_member(harness, member_id)?;
    fetch_rows_via_target(harness, table_name, &target)
}

fn fetch_rows_via_target(
    harness: &HarnessShared,
    table_name: &str,
    target: &ConnectionTarget,
) -> Result<Vec<String>> {
    let query =
        format!("SELECT COALESCE(string_agg(token, E'\\n' ORDER BY token), '') FROM {table_name};");
    let output = harness.sql().execute(target.dsn.as_str(), query.as_str())?;
    Ok(output
        .trim()
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>())
}

fn assert_exact_rows(
    member_label: &str,
    observed_rows: &[String],
    expected_rows: &[String],
) -> Result<()> {
    let mut canonical_observed = observed_rows.to_vec();
    canonical_observed.sort();
    let mut canonical_expected = expected_rows.to_vec();
    canonical_expected.sort();
    if canonical_observed == canonical_expected {
        Ok(())
    } else {
        Err(HarnessError::message(format!(
            "member `{member_label}` rows {:?} did not match expected {:?}",
            canonical_observed, canonical_expected
        )))
    }
}

fn record_alias(
    world: &mut HaWorld,
    alias: &str,
    member: ClusterMember,
    phase: &str,
) -> Result<()> {
    {
        let harness = world.harness()?;
        harness.record_note(phase, format!("alias `{alias}` -> `{member}`"))?;
    }
    world.remember_alias(alias, member);
    Ok(())
}

fn resolve_member_reference(world: &HaWorld, member_ref: &str) -> Result<ClusterMember> {
    world.require_alias(member_ref).or_else(|_| ClusterMember::parse(member_ref))
}

fn single_primary(status: &NodeState) -> Result<ClusterMember> {
    match authoritative_primary(status) {
        Some(primary) => Ok(primary),
        None => Err(HarnessError::message(format!(
            "cluster has no authoritative primary; authority={} warnings={}",
            format_authority(status),
            format_warnings(status),
        ))),
    }
}

fn replica_members(status: &NodeState) -> Vec<ClusterMember> {
    status
        .dcs
        .members
        .iter()
        .filter(|(_member_id, member)| matches!(&member.postgres, DcsMemberPostgresView::Replica(_)))
        .filter_map(|(member_id, _member)| ClusterMember::parse(member_id.0.as_str()).ok())
        .collect::<Vec<_>>()
}

fn operator_visible_member_ids(status: &NodeState) -> Vec<ClusterMember> {
    status
        .dcs
        .members
        .keys()
        .filter_map(|member_id| ClusterMember::parse(member_id.0.as_str()).ok())
        .collect::<Vec<_>>()
}

fn assert_member_is_replica_via_member(
    harness: &HarnessShared,
    member: ClusterMember,
    expected_online: usize,
) -> Result<()> {
    let status = harness.observer().state_via_member(member)?;
    let snapshot_label = format!("status.replica.{member}");
    harness.record_status_snapshot(snapshot_label.as_str(), &status)?;
    require_visible_members(&status, expected_online)?;
    let primary = single_primary(&status)?;
    let member_status = status
        .dcs
        .members
        .get(&member.member_id())
        .ok_or_else(|| {
            HarnessError::message(format!("member `{member}` is not present in status"))
        })?;
    if member == primary {
        return Err(HarnessError::message(format!(
            "member `{member}` is still the primary instead of a replica"
        )));
    }
    match &member_status.postgres {
        DcsMemberPostgresView::Replica(_) => Ok(()),
        DcsMemberPostgresView::Unknown(_) => {
            Err(HarnessError::message(format!(
                "member `{member}` role remained `unknown`; HA role assertions must come directly from NodeState"
            )))
        }
        DcsMemberPostgresView::Primary(_) => Err(HarnessError::message(format!(
            "member `{member}` role is `primary` instead of `replica`"
        ))),
    }
}

fn require_visible_members(status: &NodeState, expected: usize) -> Result<()> {
    let visible = status.dcs.members.len();
    if visible >= expected {
        return Ok(());
    }

    Err(HarnessError::message(format!(
        "expected at least {expected} visible members, observed {visible}; warnings={}",
        format_warnings(status)
    )))
}

fn require_no_authoritative_primary(status: &NodeState) -> Result<()> {
    match &status.ha.publication {
        PublicationState::Projected(AuthorityProjection::NoPrimary(_)) => Ok(()),
        PublicationState::Projected(AuthorityProjection::Primary(epoch)) => {
            Err(HarnessError::message(format!(
            "expected no authoritative primary, but `{}` was still published",
            epoch.holder.0
        )))
        }
        PublicationState::Unknown => Err(HarnessError::message(
            "expected an explicit no-primary authority result, but authority remained `unknown`",
        )),
    }
}

fn format_warnings(status: &NodeState) -> String {
    let mut warnings = Vec::new();
    if status.dcs.trust != DcsTrust::FullQuorum {
        warnings.push(format!("dcs_trust={:?}", status.dcs.trust).to_lowercase());
    }
    if !matches!(
        status.ha.publication,
        PublicationState::Projected(AuthorityProjection::Primary(_))
    ) {
        warnings.push(format!("authority={}", format_authority(status)));
    }
    if status.dcs.members.is_empty() {
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

fn self_is_fail_safe(status: &NodeState, member: ClusterMember) -> bool {
    status.self_member_id == member.service_name() && matches!(status.ha.role, TargetRole::FailSafe(_))
}

fn terminal_container_failure(
    harness: &HarnessShared,
    expected_offline: &MemberSet,
    kind: PollKind,
) -> Result<Option<String>> {
    let cluster_members = all_cluster_members();
    let services = match kind {
        PollKind::Startup | PollKind::Recovery => cluster_members.as_slice(),
        PollKind::Failover => return Ok(None),
    };

    let mut failures = Vec::new();
    for service in services {
        if expected_offline.contains(*service) {
            continue;
        }
        let container_id = match harness.service_container_id((*service).into()) {
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

fn parse_count(raw_value: &str) -> Result<usize> {
    raw_value.parse::<usize>().map_err(|err| {
        HarnessError::message(format!("failed to parse `{raw_value}` as usize: {err}"))
    })
}

fn proof_table_name(harness: &HarnessShared) -> String {
    const MAX_SQL_IDENTIFIER_BYTES: usize = 63;
    const PROOF_TABLE_PREFIX: &str = "ha_cucumber_proof_";

    let suffix =
        sanitize_sql_identifier(format!("{}_{}", harness.feature_name, harness.run_id).as_str());
    let candidate = format!("{PROOF_TABLE_PREFIX}{suffix}");
    if candidate.len() <= MAX_SQL_IDENTIFIER_BYTES {
        return candidate;
    }

    let mut hasher = DefaultHasher::new();
    candidate.hash(&mut hasher);
    let digest = format!("{:016x}", hasher.finish());
    let keep = MAX_SQL_IDENTIFIER_BYTES
        .saturating_sub(digest.len())
        .saturating_sub(1);
    let truncated = &candidate[..keep];
    format!("{truncated}_{digest}")
}

fn sanitize_sql_identifier(raw_value: &str) -> String {
    let mut sanitized = raw_value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    if sanitized
        .chars()
        .next()
        .map(|character| character.is_ascii_digit())
        .unwrap_or(true)
    {
        sanitized.insert(0, 't');
        sanitized.insert(1, '_');
    }
    sanitized
}

fn sql_quote_literal(raw_value: &str) -> String {
    raw_value.replace('\'', "''")
}

fn online_expected_count(world: &HaWorld) -> usize {
    all_cluster_members().len()
        - world.scenario.transition.stopped_members.len()
        - world.scenario.transition.wedged_members.len()
        - world
            .scenario
            .transition
            .observation_scope
            .observer_unreachable_members
            .len()
}

fn online_member_ids(world: &HaWorld) -> Vec<ClusterMember> {
    all_cluster_members()
        .iter()
        .filter(|member| {
            !world.scenario.transition.stopped_members.contains(**member)
                && !world.scenario.transition.wedged_members.contains(**member)
                && !world
                    .scenario
                    .transition
                    .observation_scope
                    .observer_unreachable_members
                    .contains(**member)
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

#[then("I can write a proof row through the new primary")]
async fn i_can_write_a_proof_row_through_the_new_primary(world: &mut HaWorld) -> Result<()> {
    let row_value = {
        let harness = world.harness()?;
        format!("proof-{}", harness.run_id)
    };
    insert_proof_row(world, row_value.as_str(), "new_primary").await
}

#[given("I create one workload table for this feature")]
async fn i_create_one_workload_table_for_this_feature(world: &mut HaWorld) -> Result<()> {
    let _ = ensure_proof_table(world)?;
    Ok(())
}

#[when("I start a bounded concurrent write workload and record commit outcomes")]
async fn i_start_a_bounded_concurrent_write_workload_and_record_commit_outcomes(
    world: &mut HaWorld,
) -> Result<()> {
    if world.scenario.workload.active.is_some() {
        return Err(HarnessError::message(
            "a workload is already active for this scenario",
        ));
    }
    let table_name = ensure_proof_table(world)?;
    let workload = {
        let harness = world.harness()?;
        harness.record_note("sql.workload.start", format!("table={table_name}"))?;
        crate::support::workload::SqlWorkloadHandle::start(
            harness.feature_name.as_str(),
            table_name.as_str(),
            harness.observer(),
            harness.sql(),
        )
    };
    let deadline = {
        let harness = world.harness()?;
        Instant::now() + harness.timeouts.startup_deadline
    };
    let poll_interval = {
        let harness = world.harness()?;
        harness.timeouts.poll_interval
    };
    while Instant::now() < deadline {
        if workload.committed_count_so_far()? > 0 {
            world.scenario.workload.active = Some(workload);
            return Ok(());
        }
        tokio::time::sleep(poll_interval).await;
    }
    world.scenario.workload.active = Some(workload);
    Err(HarnessError::message(
        "workload did not record an initial committed row before the deadline",
    ))
}

#[when("I stop the workload and verify it committed at least one row")]
async fn i_stop_the_workload_and_verify_it_committed_at_least_one_row(
    world: &mut HaWorld,
) -> Result<()> {
    let workload = world
        .scenario
        .workload
        .active
        .take()
        .ok_or_else(|| HarnessError::message("no active workload was running"))?;
    let summary = workload.stop()?;
    if summary.committed_count() == 0 {
        return Err(HarnessError::message(
            "workload did not commit any rows before it stopped",
        ));
    }

    for token in summary.committed_tokens() {
        if !world
            .scenario
            .workload
            .proof
            .recorded_rows
            .iter()
            .any(|existing| existing.as_str() == token)
        {
            world
                .scenario
                .workload
                .proof
                .recorded_rows
                .push(token.clone().into());
        }
    }

    {
        let harness = world.harness()?;
        let artifact = serde_json::to_value(&summary).map_err(|source| HarnessError::Json {
            context: "serializing workload summary".to_string(),
            source,
        })?;
        harness.write_artifact_json("workload-summary.json", &artifact)?;
        harness.record_note(
            "sql.workload.stop",
            format!("committed_rows={}", summary.committed_count()),
        )?;
    }
    world.scenario.workload.last_summary = Some(summary);
    Ok(())
}

#[then("there is no dual-primary evidence during the transition window")]
async fn there_is_no_dual_primary_evidence_during_the_transition_window(
    world: &mut HaWorld,
) -> Result<()> {
    world.harness()?.assert_no_dual_primary_evidence()
}

#[then("there is no dual-primary evidence and no split-brain write evidence during the transition window")]
async fn there_is_no_dual_primary_evidence_and_no_split_brain_write_evidence_during_the_transition_window(
    world: &mut HaWorld,
) -> Result<()> {
    world.harness()?.assert_no_dual_primary_evidence()?;
    let summary = world
        .scenario
        .workload
        .last_summary
        .as_ref()
        .ok_or_else(|| HarnessError::message("no workload summary was recorded"))?;
    let committed_tokens = summary.committed_tokens();
    let distinct = committed_tokens
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .len();
    if distinct == committed_tokens.len() {
        Ok(())
    } else {
        Err(HarnessError::message(
            "workload recorded duplicate committed tokens, which is split-brain evidence",
        ))
    }
}

#[when(regex = r#"^I wedge the node named "([^"]+)"$"#)]
async fn i_wedge_the_node_named(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    world.harness()?.wedge_member_postgres(member_id)?;
    world.add_wedged_node(member_id);
    Ok(())
}

#[when(regex = r#"^I unwedge the node named "([^"]+)"$"#)]
async fn i_unwedge_the_node_named(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    world
        .harness()?
        .unwedge_member_postgres(member_id)?;
    world.remove_wedged_node(member_id);
    Ok(())
}

#[when("I stop the DCS service")]
#[when("I stop a DCS quorum majority")]
async fn i_stop_the_dcs_service(world: &mut HaWorld) -> Result<()> {
    world.harness()?.stop_service(crate::support::faults::ETCD_SERVICE)
}

#[when("I start the DCS service")]
#[when("I restore DCS quorum")]
async fn i_start_the_dcs_service(world: &mut HaWorld) -> Result<()> {
    world.harness()?.start_service(crate::support::faults::ETCD_SERVICE)
}

#[given("I start tracking primary history")]
#[when("I start tracking primary history")]
#[then("I start tracking primary history")]
async fn i_start_tracking_primary_history(world: &mut HaWorld) -> Result<()> {
    world.clear_primary_history();
    world
        .harness()?
        .record_note("primary_history.reset", "cleared observed primary history")
}

#[when(regex = r#"^I isolate the node named "([^"]+)" from all peers on the "([^"]+)" path$"#)]
async fn i_isolate_the_node_named_from_all_peers_on_the_path(
    world: &mut HaWorld,
    member_ref: String,
    path_name: String,
) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    let path = parse_traffic_path(path_name.as_str())?;
    world
        .harness()?
        .isolate_member_from_all_peers_on_path(member_id, path)?;
    if path == TrafficPath::Postgres {
        world.add_proof_convergence_blocker(member_id);
    }
    Ok(())
}

#[when(regex = r#"^I isolate the nodes named "([^"]+)" and "([^"]+)" on the "([^"]+)" path$"#)]
async fn i_isolate_the_nodes_named_and_on_the_path(
    world: &mut HaWorld,
    member_ref_a: String,
    member_ref_b: String,
    path_name: String,
) -> Result<()> {
    let member_a = resolve_member_reference(world, member_ref_a.as_str())?;
    let member_b = resolve_member_reference(world, member_ref_b.as_str())?;
    let path = parse_traffic_path(path_name.as_str())?;
    world.harness()?.isolate_member_from_peer_on_path(
        member_a,
        member_b,
        path,
    )?;
    if path == TrafficPath::Postgres {
        world.add_proof_convergence_blocker(member_a);
        world.add_proof_convergence_blocker(member_b);
    }
    Ok(())
}

#[when(regex = r#"^I fully isolate the node named "([^"]+)" from the cluster$"#)]
async fn i_fully_isolate_the_node_named_from_the_cluster(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
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
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    world.harness()?.cut_member_off_from_dcs(member_id)
}

#[when(regex = r#"^I isolate the node named "([^"]+)" from observer API access$"#)]
async fn i_isolate_the_node_named_from_observer_api_access(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
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
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    world
        .harness()?
        .heal_member_network_faults(member_id)?;
    world.clear_observer_unreachable(member_id);
    world.remove_proof_convergence_blocker(member_id);
    Ok(())
}

#[when("I heal all network faults")]
async fn i_heal_all_network_faults(world: &mut HaWorld) -> Result<()> {
    world.harness()?.clear_all_network_faults()?;
    world.clear_observer_unreachable_members();
    world.clear_proof_convergence_blockers();
    Ok(())
}

#[given(regex = r#"^I enable the "([^"]+)" blocker on the node named "([^"]+)"$"#)]
#[when(regex = r#"^I enable the "([^"]+)" blocker on the node named "([^"]+)"$"#)]
#[then(regex = r#"^I enable the "([^"]+)" blocker on the node named "([^"]+)"$"#)]
async fn i_enable_the_blocker_on_the_node_named(
    world: &mut HaWorld,
    blocker_name: String,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    let blocker = parse_blocker_kind(blocker_name.as_str())?;
    world
        .harness()?
        .set_blocker(member_id, blocker, true)?;
    if blocker == BlockerKind::PgBasebackup {
        world.add_proof_convergence_blocker(member_id);
    }
    Ok(())
}

#[given(regex = r#"^I disable the "([^"]+)" blocker on the node named "([^"]+)"$"#)]
#[when(regex = r#"^I disable the "([^"]+)" blocker on the node named "([^"]+)"$"#)]
#[then(regex = r#"^I disable the "([^"]+)" blocker on the node named "([^"]+)"$"#)]
async fn i_disable_the_blocker_on_the_node_named(
    world: &mut HaWorld,
    blocker_name: String,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    let blocker = parse_blocker_kind(blocker_name.as_str())?;
    world
        .harness()?
        .set_blocker(member_id, blocker, false)?;
    if blocker == BlockerKind::PgBasebackup {
        world.remove_proof_convergence_blocker(member_id);
    }
    Ok(())
}

#[when(regex = r#"^I wipe the data directory on the node named "([^"]+)"$"#)]
async fn i_wipe_the_data_directory_on_the_node_named(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    world.harness()?.wipe_member_data_dir(member_id)
}

#[when(regex = r#"^I start the node named "([^"]+)" but keep it marked unavailable$"#)]
async fn i_start_the_node_named_but_keep_it_marked_unavailable(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    world.harness()?.start_node(member_id)?;
    world.add_stopped_node(member_id);
    Ok(())
}

#[when(
    regex = r#"^I attempt a targeted switchover to "([^"]+)" and capture the operator-visible error$"#
)]
async fn i_attempt_a_targeted_switchover_to_and_capture_the_operator_visible_error(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    world.clear_primary_history();
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    let seed_member = world.require_alias("current_primary")?;
    wait_for_targeted_switchover_rejection_precondition(world, seed_member, member_id).await?;
    let request_result = {
        let harness = world.harness()?;
        harness
            .observer()
            .switchover_request_via_member(seed_member, Some(member_id))
    };
    match request_result {
        Ok(output) => {
            world.scenario.command.last_output = Some(output.clone());
            Err(HarnessError::message(format!(
                "expected targeted switchover to `{member_id}` to be rejected, but it succeeded: {output}"
            )))
        }
        Err(err) => {
            let rendered = err.to_string();
            world.scenario.command.last_output = Some(rendered.clone());
            {
                let harness = world.harness()?;
                harness.record_note(
                    "switchover.request.targeted_rejected",
                    format!("target={member_id} error={rendered}"),
                )?;
            }
            Ok(())
        }
    }
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
            let maybe_target = status.dcs.members.get(&target_member.member_id());
            match maybe_target {
                None => Ok(()),
                Some(member) if !member_slot_is_api_switchover_eligible(member) => Ok(()),
                Some(member) => Err(HarnessError::message(format!(
                    "target `{target_member}` is still promotion-eligible via `{seed_member}` with postgres state {:?}",
                    member.postgres
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
        .members
        .iter()
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

fn planned_switchover_target_rank(member: &DcsMemberView) -> (u8, u64, u64) {
    match &member.postgres {
        DcsMemberPostgresView::Replica(observation) => observation
            .replay_wal
            .as_ref()
            .or(observation.follow_wal.as_ref())
            .map(|wal| {
                (
                    1,
                    wal.timeline.map_or(0, |timeline| u64::from(timeline.0)),
                    wal.lsn.0,
                )
            })
            .unwrap_or((0, 0, 0)),
        DcsMemberPostgresView::Unknown(observation) => (
            0,
            observation.timeline.map_or(0, |timeline| u64::from(timeline.0)),
            0,
        ),
        DcsMemberPostgresView::Primary(_) => (0, 0, 0),
    }
}

fn member_slot_is_api_switchover_eligible(member: &DcsMemberView) -> bool {
    match &member.postgres {
        DcsMemberPostgresView::Primary(_) => false,
        DcsMemberPostgresView::Unknown(observation) => observation.readiness == Readiness::Ready,
        DcsMemberPostgresView::Replica(observation) => observation.readiness == Readiness::Ready,
    }
}

#[then("the last operator-visible error is recorded")]
async fn the_last_operator_visible_error_is_recorded(world: &mut HaWorld) -> Result<()> {
    match world.scenario.command.last_output.as_ref() {
        Some(message) if !message.trim().is_empty() => Ok(()),
        _ => Err(HarnessError::message(
            "no operator-visible error was recorded for the last action",
        )),
    }
}

#[then(regex = r#"^direct API observation to "([^"]+)" fails$"#)]
async fn direct_api_observation_to_fails(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    match world
        .harness()?
        .observer()
        .state_via_member(member_id)
    {
        Ok(status) => Err(HarnessError::message(format!(
            "direct API observation to `{member_id}` unexpectedly succeeded via self_member_id `{}`",
            status.self_member_id
        ))),
        Err(_) => Ok(()),
    }
}

#[then(regex = r#"^the node named "([^"]+)" is not queryable through pgtm connection helpers$"#)]
#[then(regex = r#"^the node named "([^"]+)" is not queryable$"#)]
async fn the_node_named_is_not_queryable_through_pgtm_connection_helpers(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    match pgtm_connection_target_for_member(world.harness()?, member_id) {
        Ok(target) => match world
            .harness()?
            .sql()
            .execute(target.dsn.as_str(), "SELECT 1;")
        {
            Ok(_) => Err(HarnessError::message(format!(
                "member `{member_id}` was still queryable via DSN `{}`",
                target.dsn
            ))),
            Err(_) => Ok(()),
        },
        Err(_) => Ok(()),
    }
}

#[then(regex = r#"^the node named "([^"]+)" logs contain "([^"]+)"$"#)]
async fn the_node_named_logs_contain(
    world: &mut HaWorld,
    member_ref: String,
    expected_text: String,
) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    let logs = world.harness()?.service_logs(member_id.into())?;
    if logs.contains(expected_text.as_str()) {
        Ok(())
    } else {
        Err(HarnessError::message(format!(
            "logs for `{member_id}` did not contain `{expected_text}`"
        )))
    }
}

#[then(regex = r#"^the node named "([^"]+)" is not primary in the current status$"#)]
async fn the_node_named_is_not_primary_in_the_current_status(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    let status = current_status(world)?;
    match single_primary(&status) {
        Ok(primary) if primary == member_id => Err(HarnessError::message(format!(
            "member `{member_id}` was still the current primary"
        ))),
        Ok(_) | Err(_) => Ok(()),
    }
}

#[then(regex = r#"^the aliases "([^"]+)", "([^"]+)", and "([^"]+)" are distinct$"#)]
async fn the_aliases_and_are_distinct(
    world: &mut HaWorld,
    alias_a: String,
    alias_b: String,
    alias_c: String,
) -> Result<()> {
    let member_a = world.require_alias(alias_a.as_str())?;
    let member_b = world.require_alias(alias_b.as_str())?;
    let member_c = world.require_alias(alias_c.as_str())?;
    let distinct = [member_a, member_b, member_c]
        .into_iter()
        .collect::<BTreeSet<_>>();
    if distinct.len() == 3 {
        Ok(())
    } else {
        Err(HarnessError::message(format!(
            "expected aliases `{alias_a}`, `{alias_b}`, and `{alias_c}` to be distinct, observed `{member_a}`, `{member_b}`, `{member_c}`"
        )))
    }
}

#[then(regex = r#"^the node named "([^"]+)" emitted blocker evidence for "([^"]+)"$"#)]
async fn the_node_named_emitted_blocker_evidence_for(
    world: &mut HaWorld,
    member_ref: String,
    blocker_name: String,
) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    let blocker_kind = parse_blocker_kind(blocker_name.as_str())?;
    let expected_snippet = match blocker_kind {
        BlockerKind::PgBasebackup => "pg_basebackup wrapper",
        BlockerKind::PgRewind => "pg_rewind wrapper",
        BlockerKind::PostgresStart => "postgres wrapper",
    };
    let deadline = {
        let harness = world.harness()?;
        Instant::now() + harness.timeouts.failover_deadline
    };
    let poll_interval = {
        let harness = world.harness()?;
        harness.timeouts.poll_interval
    };

    while Instant::now() < deadline {
        let logs = world.harness()?.docker.compose_logs(
            world.harness()?.compose_file.as_path(),
            world.harness()?.compose_project.as_str(),
        )?;
        if logs.contains(expected_snippet) {
            return Ok(());
        }
        tokio::time::sleep(poll_interval).await;
    }

    Err(HarnessError::message(format!(
        "compose logs did not contain blocker evidence `{expected_snippet}` for `{member_id}`"
    )))
}

#[then("every running node reports fail_safe in debug output")]
async fn every_running_node_reports_fail_safe_in_debug_output(world: &mut HaWorld) -> Result<()> {
    let deadline = {
        let harness = world.harness()?;
        Instant::now() + harness.timeouts.failover_deadline
    };
    let poll_interval = {
        let harness = world.harness()?;
        harness.timeouts.poll_interval
    };
    let stopped = world.scenario.transition.stopped_members.clone();
    let mut last_error = None;

    while Instant::now() < deadline {
        let attempt: Result<()> = (|| {
            for member_id in all_cluster_members() {
                if stopped.contains(member_id) {
                    continue;
                }
                let status = world
                    .harness()?
                    .observer()
                    .state_via_member(member_id)?;
                if !self_is_fail_safe(&status, member_id) {
                    Err(HarnessError::message(format!(
                        "member `{member_id}` did not report fail_safe (self_member_id={} authority={} warnings={})",
                        status.self_member_id,
                        format_authority(&status),
                        format_warnings(&status)
                    )))?;
                }
            }
            Ok(())
        })();
        match attempt {
            Ok(()) => return Ok(()),
            Err(err) => last_error = Some(err.to_string()),
        }
        tokio::time::sleep(poll_interval).await;
    }

    Err(HarnessError::message(format!(
        "timed out waiting for every running node to report fail_safe; last observed error: {}",
        last_error.unwrap_or_else(|| "no fail-safe verification attempt ran".to_string())
    )))
}

#[then(regex = r#"^the node named "([^"]+)" enters fail-safe or loses primary authority safely$"#)]
async fn the_node_named_enters_fail_safe_or_loses_primary_authority_safely(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
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
            let member_status = world
                .harness()?
                .observer()
                .state_via_member(member_id)?;
            if self_is_fail_safe(&member_status, member_id) {
                return Ok(());
            }
            let status = current_status(world)?;
            match authoritative_primary(&status) {
                Some(primary) if primary == member_id => Err(HarnessError::message(format!(
                    "member `{member_id}` still held primary authority as `{primary}`"
                ))),
                Some(_) | None => Ok(()),
            }
        })();
        match attempt {
            Ok(()) => return Ok(()),
            Err(err) => last_error = Some(err.to_string()),
        }
        tokio::time::sleep(poll_interval).await;
    }

    Err(HarnessError::message(format!(
        "timed out waiting for `{member_id}` to enter fail_safe or lose primary authority; last observed error: {}",
        last_error.unwrap_or_else(|| "no fail-safe-or-no-primary verification attempt ran".to_string())
    )))
}

#[then("the recorded workload evidence establishes a fencing cutoff with no later commits")]
async fn the_recorded_workload_evidence_establishes_a_fencing_cutoff_with_no_later_commits(
    world: &mut HaWorld,
) -> Result<()> {
    let summary = world
        .scenario
        .workload
        .last_summary
        .as_ref()
        .ok_or_else(|| HarnessError::message("no workload summary was recorded"))?;
    let first_rejection = summary
        .events
        .iter()
        .find_map(|event| match event.outcome {
            crate::support::workload::WorkloadOutcome::Rejected { .. } => Some(event.started_at_ms),
            crate::support::workload::WorkloadOutcome::Committed => None,
        })
        .ok_or_else(|| {
            HarnessError::message(
                "workload never recorded a rejected write, so no fencing cutoff was observed",
            )
        })?;
    let late_commit = summary.events.iter().find(|event| {
        matches!(
            event.outcome,
            crate::support::workload::WorkloadOutcome::Committed
        ) && event.finished_at_ms > first_rejection
    });
    match late_commit {
        Some(event) => Err(HarnessError::message(format!(
            "workload committed `{}` after the first rejection cutoff at {}",
            event.token, first_rejection
        ))),
        None => Ok(()),
    }
}

#[then(
    regex = r#"^the nodes named "([^"]+)" and "([^"]+)" do not yet contain proof row "([^"]+)"$"#
)]
async fn the_nodes_named_and_do_not_yet_contain_proof_row(
    world: &mut HaWorld,
    member_ref_a: String,
    member_ref_b: String,
    row_value: String,
) -> Result<()> {
    for member_ref in [member_ref_a.as_str(), member_ref_b.as_str()] {
        assert_member_does_not_have_row(world, member_ref, row_value.as_str())?;
    }
    Ok(())
}

#[then(regex = r#"^the node named "([^"]+)" does not yet contain proof row "([^"]+)"$"#)]
async fn the_node_named_does_not_yet_contain_proof_row(
    world: &mut HaWorld,
    member_ref: String,
    row_value: String,
) -> Result<()> {
    assert_member_does_not_have_row(world, member_ref.as_str(), row_value.as_str())
}

#[given(regex = r#"^I record marker "([^"]+)"$"#)]
#[when(regex = r#"^I record marker "([^"]+)"$"#)]
async fn i_record_marker(world: &mut HaWorld, marker_name: String) -> Result<()> {
    let timestamp_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|err| HarnessError::message(format!("system clock error: {err}")))?;
    world.record_marker(marker_name.as_str(), timestamp_ms);
    Ok(())
}

#[then(regex = r#"^the node named "([^"]+)" never becomes primary after marker "([^"]+)"$"#)]
async fn the_node_named_never_becomes_primary_after_marker(
    world: &mut HaWorld,
    member_ref: String,
    marker_name: String,
) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    let marker = world.marker(marker_name.as_str())?;
    world
        .harness()?
        .assert_member_never_primary_since(member_id, marker)
}

#[then(regex = r#"^the aliases "([^"]+)", "([^"]+)", and "([^"]+)" are all distinct$"#)]
async fn the_aliases_and_are_all_distinct(
    world: &mut HaWorld,
    alias_a: String,
    alias_b: String,
    alias_c: String,
) -> Result<()> {
    let values = [
        world.require_alias(alias_a.as_str())?,
        world.require_alias(alias_b.as_str())?,
        world.require_alias(alias_c.as_str())?,
    ];
    let distinct = values.iter().cloned().collect::<BTreeSet<_>>();
    if distinct.len() == values.len() {
        Ok(())
    } else {
        Err(HarnessError::message(format!(
            "expected aliases to be distinct, observed {:?}",
            values
        )))
    }
}

fn assert_member_does_not_have_row(
    world: &mut HaWorld,
    member_ref: &str,
    row_value: &str,
) -> Result<()> {
    let table_name = world
        .scenario
        .workload
        .proof
        .table
        .as_ref()
        .map(|table| table.as_str().to_string())
        .ok_or_else(|| HarnessError::message("proof table was not created"))?;
    let member_id = resolve_member_reference(world, member_ref)?;
    let observed = fetch_rows_for_member(world.harness()?, table_name.as_str(), member_id)?;
    if observed.iter().any(|value| value == row_value) {
        Err(HarnessError::message(format!(
            "member `{member_id}` already contained row `{row_value}`"
        )))
    } else {
        Ok(())
    }
}

fn parse_traffic_path(raw_value: &str) -> Result<TrafficPath> {
    match raw_value {
        "dcs" | "etcd" => Ok(TrafficPath::Dcs),
        "api" => Ok(TrafficPath::Api),
        "postgres" | "replication" => Ok(TrafficPath::Postgres),
        _ => Err(HarnessError::message(format!(
            "unknown traffic path `{raw_value}`"
        ))),
    }
}

fn parse_blocker_kind(raw_value: &str) -> Result<BlockerKind> {
    match raw_value {
        "pg_basebackup" => Ok(BlockerKind::PgBasebackup),
        "pg_rewind" => Ok(BlockerKind::PgRewind),
        "postgres_start" | "startup" | "rejoin" => Ok(BlockerKind::PostgresStart),
        _ => Err(HarnessError::message(format!(
            "unknown blocker kind `{raw_value}`"
        ))),
    }
}
