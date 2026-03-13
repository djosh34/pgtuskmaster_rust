use std::{
    collections::{BTreeMap, BTreeSet},
    hash::{DefaultHasher, Hash, Hasher},
    time::{Duration, Instant},
};

use cucumber::{given, then, when};

use crate::support::{
    error::{HarnessError, Result},
    faults::{BlockerKind, TrafficPath, ALL_CLUSTER_MEMBERS},
    observer::pgtm::{ClusterStatusView, ConnectionTarget},
    world::{HaWorld, HarnessShared},
};

const PRIMARY_CRASH_REJOIN_PROOF_ALIAS: &str = "primary_crash_rejoin_proof";

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
    world.remember_alias(alias.as_str(), primary.clone());
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
                member_a.clone(),
                "choose_two_replicas",
            )?;
            record_alias(
                world,
                alias_b.as_str(),
                member_b.clone(),
                "choose_two_replicas",
            )
        }
        _ => Err(HarnessError::message(format!(
            "expected exactly two non-primary nodes, observed {}",
            replicas.join(", ")
        ))),
    }
}

#[given(regex = r#"^I record the remaining replica as "([^"]+)"$"#)]
async fn i_record_the_remaining_replica_as(world: &mut HaWorld, alias: String) -> Result<()> {
    let status =
        wait_for_status_snapshot(world, format!("record_remaining_replica.{alias}").as_str())
            .await?;
    let primary = single_primary(&status)?;
    let used_members = world
        .scenario
        .aliases
        .values()
        .cloned()
        .collect::<BTreeSet<_>>();
    let member_id = replica_members(&status)
        .into_iter()
        .find(|member_id| member_id != &primary && !used_members.contains(member_id))
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
        harness.kill_node(primary_member.as_str())?;
    }
    world.remember_alias("killed_node", primary_member.clone());
    world.add_stopped_node(primary_member.as_str());
    Ok(())
}

#[when(regex = r#"^I kill the node named "([^"]+)"$"#)]
async fn i_kill_the_node_named(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    {
        let harness = world.harness()?;
        harness.kill_node(member_id.as_str())?;
    }
    world.add_stopped_node(member_id.as_str());
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
            harness.kill_node(member_id.as_str())?;
        }
        world.add_stopped_node(member_id.as_str());
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
        harness.start_node(killed_node.as_str())?;
    }
    world.remove_stopped_node(killed_node.as_str());
    Ok(())
}

#[when(regex = r#"^I restart the node named "([^"]+)"$"#)]
async fn i_restart_the_node_named(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    {
        let harness = world.harness()?;
        harness.start_node(member_id.as_str())?;
    }
    world.remove_stopped_node(member_id.as_str());
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
            harness.start_node(member_id.as_str())?;
        }
        world.remove_stopped_node(member_id.as_str());
    }
    Ok(())
}

#[when("I request a planned switchover")]
async fn i_request_a_planned_switchover(world: &mut HaWorld) -> Result<()> {
    world.clear_primary_history();
    let seed_member = world.require_alias("current_primary")?;
    let harness = world.harness()?;
    let response = harness
        .observer()
        .switchover_request_via_member(seed_member.as_str(), None)?;
    harness.record_note("switchover.request", response)?;
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
        .switchover_request_via_member(seed_member.as_str(), Some(member_id.as_str()))?;
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
        Some(killed_node.as_str()),
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
        Some(member_id.as_str()),
        None,
    )
    .await?;
    let _ = wait_for_primary_resolution_for_member(
        world,
        format!("primary.same.resolution.{member_id}").as_str(),
        PollKind::Failover,
        Some(member_id.as_str()),
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
    wait_for_no_operator_primary(world, 1).await
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
            let relevant_nodes = status
                .nodes
                .iter()
                .filter(|node| {
                    intended_online
                        .iter()
                        .any(|member_id| member_id == &node.member_id)
                        && node.sampled
                })
                .collect::<Vec<_>>();
            let primaries = relevant_nodes
                .iter()
                .filter(|node| node.role == "primary")
                .map(|node| node.member_id.clone())
                .collect::<Vec<_>>();
            match primaries.as_slice() {
                [primary] => Ok(primary.clone()),
                [] => Err(HarnessError::message(format!(
                    "expected one sampled primary across the intended online nodes, observed none; sampled_relevant={} warnings={}",
                    relevant_nodes
                        .iter()
                        .map(|node| format!("{}:{}", node.member_id, node.role))
                        .collect::<Vec<_>>()
                        .join(", "),
                    format_warnings(status),
                ))),
                _ => Err(HarnessError::message(format!(
                    "expected one sampled primary across the intended online nodes, observed {}",
                    primaries.join(", ")
                ))),
            }
        },
    )
    .await?;
    let _ = wait_for_primary_resolution_for_member(
        world,
        format!("primary.across.primary.{expected_online}.{alias}").as_str(),
        PollKind::Recovery,
        Some(primary.as_str()),
    )
    .await?;
    world.remember_alias(alias.as_str(), primary.clone());
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
        Some(previous_member.as_str()),
    )
    .await?;
    let _ = wait_for_primary_resolution_for_member(
        world,
        format!("primary.changed.primary.{alias}").as_str(),
        PollKind::Failover,
        Some(primary.as_str()),
    )
    .await?;
    world.remember_alias(alias.as_str(), primary.clone());
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
        Some(member_id.as_str()),
        None,
    )
    .await?;
    let _ = wait_for_primary_resolution_for_member(
        world,
        format!("primary.targeted.primary.{member_id}").as_str(),
        PollKind::Failover,
        Some(member_id.as_str()),
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
    let status = current_status(world)?;
    require_sampled_members(&status, online_expected_count(world))?;
    let primary = single_primary(&status)?;
    let replicas = status
        .nodes
        .iter()
        .filter(|node| node.sampled && node.member_id != primary && node.role == "replica")
        .count();
    if replicas == 1 {
        Ok(())
    } else {
        Err(HarnessError::message(format!(
            "expected one remaining sampled replica, observed {replicas}"
        )))
    }
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
            require_sampled_members(status, 2)?;
            let relevant_nodes = status
                .nodes
                .iter()
                .filter(|node| {
                    intended_online
                        .iter()
                        .any(|member_id| member_id == &node.member_id)
                })
                .collect::<Vec<_>>();
            let primaries = relevant_nodes
                .iter()
                .filter(|node| node.sampled && node.role == "primary")
                .map(|node| node.member_id.as_str())
                .collect::<Vec<_>>();
            let primary = match primaries.as_slice() {
                [member_id] => *member_id,
                [] => Err(HarnessError::message(
                    "expected one sampled primary across the intended online nodes, observed none",
                ))?,
                _ => Err(HarnessError::message(format!(
                    "expected one sampled primary across the intended online nodes, observed {}",
                    primaries.join(", ")
                )))?,
            };
            let non_primary_sampled = status
                .nodes
                .iter()
                .filter(|node| {
                    node.sampled
                        && node.member_id != primary
                        && intended_online
                            .iter()
                            .any(|member_id| member_id == &node.member_id)
                })
                .map(|node| node.member_id.as_str())
                .collect::<Vec<_>>();
            if non_primary_sampled.len() == 1 {
                Ok(())
            } else {
                Err(HarnessError::message(format!(
                "expected exactly one sampled non-primary in degraded two-node state, observed {}",
                non_primary_sampled.join(", ")
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
    let container_id = harness.service_container_id(member_id.as_str())?;
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
    let expected_rows = world.scenario.proof_rows.clone();
    wait_for_member_rows(world, member_id.as_str(), &expected_rows).await
}

#[then("the cluster still has exactly one primary")]
async fn the_cluster_still_has_exactly_one_primary(world: &mut HaWorld) -> Result<()> {
    let status = current_status(world)?;
    require_sampled_members(&status, online_expected_count(world))?;
    let _ = single_primary(&status)?;
    Ok(())
}

#[then(regex = r#"^pgtm primary points to "([^"]+)"$"#)]
async fn pgtm_primary_points_to(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    let harness = world.harness()?;
    let primary = harness.observer().primary_tls_json()?;
    match primary.targets.as_slice() {
        [target] if target.member_id == member_id => Ok(()),
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
        .filter(|member_id| **member_id != excluded_member.as_str())
        .map(|member_id| (*member_id).to_string())
        .collect::<BTreeSet<_>>();
    wait_for_pgtm_replicas(world, expected).await
}

#[then(regex = r#"^the primary history never included "([^"]+)"$"#)]
async fn the_primary_history_never_included(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    if world
        .scenario
        .observed_primaries
        .iter()
        .any(|observed| observed == &member_id)
    {
        return Err(HarnessError::message(format!(
            "primary history unexpectedly included `{member_id}`: {}",
            world.scenario.observed_primaries.join(", ")
        )));
    }
    Ok(())
}

async fn insert_proof_row(world: &mut HaWorld, row_value: &str, member_ref: &str) -> Result<()> {
    let table_name = ensure_proof_table(world)?;
    let member_id = resolve_member_reference(world, member_ref)?;
    let target = sql_target_for_member(world.harness()?, member_id.as_str())?;
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
        .proof_rows
        .iter()
        .any(|existing| existing == row_value)
    {
        world.scenario.proof_rows.push(row_value.to_string());
    }
    if world.scenario.stopped_nodes.is_empty()
        && world.scenario.unsampled_nodes.is_empty()
        && world.scenario.wedged_nodes.is_empty()
        && world.scenario.proof_convergence_blocked_nodes.is_empty()
    {
        let expected_online = online_expected_count(world);
        return wait_for_recorded_proof_rows(world, expected_online).await;
    }
    Ok(())
}

fn ensure_proof_table(world: &mut HaWorld) -> Result<String> {
    if let Some(table_name) = world.scenario.proof_table.clone() {
        return Ok(table_name);
    }

    let harness = world.harness()?;
    let table_name = proof_table_name(harness);
    let create_sql = format!("CREATE TABLE IF NOT EXISTS {table_name} (token TEXT PRIMARY KEY);");
    let primary = current_primary_target(harness)?;
    let _ = harness
        .sql()
        .execute(primary.dsn.as_str(), create_sql.as_str())?;
    harness.record_note("sql.create_proof_table", format!("table={table_name}"))?;
    world.scenario.proof_table = Some(table_name.clone());
    Ok(table_name)
}

async fn wait_for_recorded_proof_rows(world: &mut HaWorld, expected_online: usize) -> Result<()> {
    let table_name = world
        .scenario
        .proof_table
        .clone()
        .ok_or_else(|| HarnessError::message("proof table was not created"))?;
    let expected_rows = world.scenario.proof_rows.clone();
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
        .proof_table
        .clone()
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
            fetch_rows_for_member(harness, table_name.as_str(), member_id.as_str()).and_then(
                |observed_rows| assert_exact_rows(&member_id, &observed_rows, expected_rows),
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
    exact_primary: Option<&str>,
    different_from: Option<&str>,
) -> Result<String> {
    let expected_primary = exact_primary.map(str::to_string);
    let previous_primary = different_from.map(str::to_string);
    poll_for_status(world, phase, kind, |status| {
        require_sampled_members(status, expected_online)?;
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
    exact_primary: Option<&str>,
    different_from: Option<&str>,
) -> Result<String> {
    let expected_primary = exact_primary.map(str::to_string);
    let previous_primary = different_from.map(str::to_string);
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
        let attempt: Result<String> = (|| {
            let status = {
                let harness = world.harness()?;
                let status = harness.observer().status()?;
                harness.record_status_snapshot(phase, &status)?;
                status
            };
            require_sampled_members(&status, expected_online)?;
            let primary = single_primary(&status)?;
            world.record_primary_observation(primary.as_str());
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
            if target.member_id != primary {
                Err(HarnessError::message(format!(
                    "sampled cluster primary was `{primary}`, but authoritative pgtm primary resolved to `{}`",
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
            terminal_container_failure(harness, &world.scenario.stopped_nodes, kind)?
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
                let status = harness.observer().status_via_member(member_id.as_str())?;
                let snapshot_label = format!("primary.none.{member_id}");
                harness.record_status_snapshot(snapshot_label.as_str(), &status)?;
                require_sampled_members(&status, expected_online)?;
                if let Ok(primary) = harness
                    .observer()
                    .primary_tls_json_via_member(member_id.as_str())
                {
                    Err(HarnessError::message(format!(
                        "expected pgtm primary via `{member_id}` to fail, but it returned targets: {}",
                        primary
                            .targets
                            .iter()
                            .map(|target| target.member_id.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
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
            assert_member_is_replica_via_member(harness, member_id.as_str(), expected_online)
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
) -> Result<Vec<String>> {
    let expected_online = online_expected_count(world);
    poll_for_status(world, phase, PollKind::Startup, |status| {
        require_sampled_members(status, expected_online)?;
        let replicas = replica_members(status);
        if replicas.len() == expected_replicas {
            Ok(replicas)
        } else {
            Err(HarnessError::message(format!(
                "expected {expected_replicas} sampled replicas, observed {}",
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
) -> Result<Vec<String>> {
    let expected_online = online_expected_count(world);
    poll_for_status(world, phase, PollKind::Startup, |status| {
        require_sampled_members(status, expected_online)?;
        let replicas = replica_members(status);
        if replicas.len() >= minimum_replicas {
            Ok(replicas)
        } else {
            Err(HarnessError::message(format!(
                "expected at least {minimum_replicas} sampled replicas, observed {}",
                replicas.len()
            )))
        }
    })
    .await
}

async fn wait_for_status_snapshot(world: &mut HaWorld, phase: &str) -> Result<ClusterStatusView> {
    let expected_online = online_expected_count(world);
    poll_for_status(world, phase, PollKind::Startup, |status| {
        require_sampled_members(status, expected_online)?;
        Ok(status.clone())
    })
    .await
}

async fn wait_for_primary_resolution_for_member(
    world: &mut HaWorld,
    phase: &str,
    kind: PollKind,
    expected_member_id: Option<&str>,
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
            let harness = world.harness()?;
            let status = harness.observer().status()?;
            harness.record_status_snapshot(phase, &status)?;
            let target = current_primary_target(harness)?;
            if let Some(expected_member_id) = expected_member_id {
                if target.member_id != expected_member_id {
                    Err(HarnessError::message(format!(
                        "pgtm primary resolved to `{}` instead of expected `{expected_member_id}`",
                        target.member_id
                    )))?;
                }
            }
            let _ = harness.sql().execute(target.dsn.as_str(), "SELECT 1;")?;
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
    F: FnMut(&ClusterStatusView) -> Result<T>,
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
            harness.observer().status()
        };
        match status_result {
            Ok(status) => {
                if let Ok(primary) = single_primary(&status) {
                    world.record_primary_observation(primary.as_str());
                }
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
            terminal_container_failure(harness, &world.scenario.stopped_nodes, kind)?
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

fn current_status(world: &HaWorld) -> Result<ClusterStatusView> {
    let harness = world.harness()?;
    let status = harness.observer().status()?;
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

fn direct_connection_target(member_id: &str) -> ConnectionTarget {
    ConnectionTarget {
        member_id: member_id.to_string(),
        dsn: format!(
            "host={member_id} port=5432 user=postgres dbname=postgres sslmode=verify-full sslrootcert=/etc/pgtuskmaster/tls/ca.crt sslcert=/etc/pgtuskmaster/tls/observer.crt sslkey=/etc/pgtuskmaster/tls/observer.key"
        ),
    }
}

fn direct_online_connection_targets(world: &HaWorld) -> Result<Vec<ConnectionTarget>> {
    Ok(online_member_ids(world)
        .into_iter()
        .map(|member_id| direct_connection_target(member_id.as_str()))
        .collect::<Vec<_>>())
}

fn pgtm_connection_target_for_member(
    harness: &HarnessShared,
    member_id: &str,
) -> Result<ConnectionTarget> {
    if let Ok(primary_target) = current_primary_target(harness) {
        if primary_target.member_id == member_id {
            return Ok(primary_target);
        }
    }

    current_connection_targets(harness)?
        .into_iter()
        .find(|target| target.member_id == member_id)
        .ok_or_else(|| {
            HarnessError::message(format!(
                "member `{member_id}` is not currently reachable through pgtm connection helpers"
            ))
        })
}

fn sql_target_for_member(harness: &HarnessShared, member_id: &str) -> Result<ConnectionTarget> {
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
    member_id: &str,
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

fn record_alias(world: &mut HaWorld, alias: &str, member_id: String, phase: &str) -> Result<()> {
    {
        let harness = world.harness()?;
        harness.record_note(phase, format!("alias `{alias}` -> `{member_id}`"))?;
    }
    world.remember_alias(alias, member_id);
    Ok(())
}

fn resolve_member_reference(world: &HaWorld, member_ref: &str) -> Result<String> {
    match world.scenario.aliases.get(member_ref) {
        Some(member_id) => Ok(member_id.clone()),
        None if all_cluster_members().contains(&member_ref) => Ok(member_ref.to_string()),
        None => Err(HarnessError::message(format!(
            "member reference `{member_ref}` is neither a recorded alias nor a known node"
        ))),
    }
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

fn replica_members(status: &ClusterStatusView) -> Vec<String> {
    status
        .nodes
        .iter()
        .filter(|node| node.sampled && node.role == "replica")
        .map(|node| node.member_id.clone())
        .collect::<Vec<_>>()
}

fn assert_member_is_replica_via_member(
    harness: &HarnessShared,
    member_id: &str,
    expected_online: usize,
) -> Result<()> {
    let status = harness.observer().status_via_member(member_id)?;
    let snapshot_label = format!("status.replica.{member_id}");
    harness.record_status_snapshot(snapshot_label.as_str(), &status)?;
    require_sampled_members(&status, expected_online)?;
    let primary = single_primary(&status)?;
    let member = status
        .nodes
        .iter()
        .find(|node| node.member_id == member_id)
        .ok_or_else(|| {
            HarnessError::message(format!("member `{member_id}` is not present in status"))
        })?;
    if !member.sampled {
        return Err(HarnessError::message(format!(
            "member `{member_id}` is present but not sampled"
        )));
    }
    if member.member_id == primary {
        return Err(HarnessError::message(format!(
            "member `{member_id}` is still the primary instead of a replica"
        )));
    }
    if member.role == "replica" {
        return Ok(());
    }
    if member.role == "unknown" {
        let target = sql_target_for_member(harness, member_id)?;
        let recovery = harness
            .sql()
            .execute(target.dsn.as_str(), "SELECT pg_is_in_recovery();")?;
        if target.member_id == member_id && recovery.trim() == "t" {
            return Ok(());
        }
    }
    Err(HarnessError::message(format!(
        "member `{member_id}` role is `{}` instead of `replica`",
        member.role
    )))
}

fn require_sampled_members(status: &ClusterStatusView, expected: usize) -> Result<()> {
    let sampled = status.sampled_member_count;
    if sampled >= expected {
        return Ok(());
    }

    Err(HarnessError::message(format!(
        "expected at least {expected} sampled members, observed {sampled} sampled out of {} discovered; warnings={}",
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
    expected_offline: &BTreeSet<String>,
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
        let container_id = match harness.service_container_id(service) {
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
        - world.scenario.stopped_nodes.len()
        - world.scenario.unsampled_nodes.len()
}

fn online_member_ids(world: &HaWorld) -> Vec<String> {
    all_cluster_members()
        .iter()
        .filter(|member_id| {
            !world.scenario.stopped_nodes.contains(**member_id)
                && !world.scenario.unsampled_nodes.contains(**member_id)
        })
        .map(|member_id| (*member_id).to_string())
        .collect::<Vec<_>>()
}

fn all_cluster_members() -> [&'static str; 3] {
    ["node-a", "node-b", "node-c"]
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
    world.remember_alias(PRIMARY_CRASH_REJOIN_PROOF_ALIAS, row_value.clone());
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
    if world.scenario.active_workload.is_some() {
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
            world.scenario.active_workload = Some(workload);
            return Ok(());
        }
        tokio::time::sleep(poll_interval).await;
    }
    world.scenario.active_workload = Some(workload);
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
        .active_workload
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
            .proof_rows
            .iter()
            .any(|existing| existing == &token)
        {
            world.scenario.proof_rows.push(token);
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
    world.scenario.last_workload_summary = Some(summary);
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
        .last_workload_summary
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
    world.harness()?.wedge_member_postgres(member_id.as_str())?;
    world.add_wedged_node(member_id.as_str());
    Ok(())
}

#[when(regex = r#"^I unwedge the node named "([^"]+)"$"#)]
async fn i_unwedge_the_node_named(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    world
        .harness()?
        .unwedge_member_postgres(member_id.as_str())?;
    world.remove_wedged_node(member_id.as_str());
    Ok(())
}

#[when("I stop the DCS service")]
#[when("I stop a DCS quorum majority")]
async fn i_stop_the_dcs_service(world: &mut HaWorld) -> Result<()> {
    world.harness()?.stop_service("etcd")
}

#[when("I start the DCS service")]
#[when("I restore DCS quorum")]
async fn i_start_the_dcs_service(world: &mut HaWorld) -> Result<()> {
    world.harness()?.start_service("etcd")
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
        .isolate_member_from_all_peers_on_path(member_id.as_str(), path)?;
    if path == TrafficPath::Postgres {
        world.add_proof_convergence_blocker(member_id.as_str());
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
        member_a.as_str(),
        member_b.as_str(),
        path,
    )?;
    if path == TrafficPath::Postgres {
        world.add_proof_convergence_blocker(member_a.as_str());
        world.add_proof_convergence_blocker(member_b.as_str());
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
        harness.isolate_member_from_all_peers_on_path(member_id.as_str(), path)?;
    }
    harness.cut_member_off_from_dcs(member_id.as_str())?;
    harness.isolate_member_from_observer_on_api(member_id.as_str())?;
    world.add_unsampled_node(member_id.as_str());
    Ok(())
}

#[when(regex = r#"^I cut the node named "([^"]+)" off from DCS$"#)]
async fn i_cut_the_node_named_off_from_dcs(world: &mut HaWorld, member_ref: String) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    world.harness()?.cut_member_off_from_dcs(member_id.as_str())
}

#[when(regex = r#"^I isolate the node named "([^"]+)" from observer API access$"#)]
async fn i_isolate_the_node_named_from_observer_api_access(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    world
        .harness()?
        .isolate_member_from_observer_on_api(member_id.as_str())?;
    world.add_unsampled_node(member_id.as_str());
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
        .heal_member_network_faults(member_id.as_str())?;
    world.remove_unsampled_node(member_id.as_str());
    world.remove_proof_convergence_blocker(member_id.as_str());
    Ok(())
}

#[when("I heal all network faults")]
async fn i_heal_all_network_faults(world: &mut HaWorld) -> Result<()> {
    world.harness()?.clear_all_network_faults()?;
    world.clear_unsampled_nodes();
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
        .set_blocker(member_id.as_str(), blocker, true)?;
    if blocker == BlockerKind::PgBasebackup {
        world.add_proof_convergence_blocker(member_id.as_str());
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
        .set_blocker(member_id.as_str(), blocker, false)?;
    if blocker == BlockerKind::PgBasebackup {
        world.remove_proof_convergence_blocker(member_id.as_str());
    }
    Ok(())
}

#[when(regex = r#"^I wipe the data directory on the node named "([^"]+)"$"#)]
async fn i_wipe_the_data_directory_on_the_node_named(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    world.harness()?.wipe_member_data_dir(member_id.as_str())
}

#[when(regex = r#"^I start the node named "([^"]+)" but keep it marked unavailable$"#)]
async fn i_start_the_node_named_but_keep_it_marked_unavailable(
    world: &mut HaWorld,
    member_ref: String,
) -> Result<()> {
    let member_id = resolve_member_reference(world, member_ref.as_str())?;
    world.harness()?.start_node(member_id.as_str())?;
    world.add_stopped_node(member_id.as_str());
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
    let request_result = {
        let harness = world.harness()?;
        harness
            .observer()
            .switchover_request_via_member(seed_member.as_str(), Some(member_id.as_str()))
    };
    match request_result {
        Ok(output) => {
            world.scenario.last_command_output = Some(output.clone());
            Err(HarnessError::message(format!(
                "expected targeted switchover to `{member_id}` to be rejected, but it succeeded: {output}"
            )))
        }
        Err(err) => {
            let rendered = err.to_string();
            world.scenario.last_command_output = Some(rendered.clone());
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

#[then("the last operator-visible error is recorded")]
async fn the_last_operator_visible_error_is_recorded(world: &mut HaWorld) -> Result<()> {
    match world.scenario.last_command_output.as_ref() {
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
        .status_via_member(member_id.as_str())
    {
        Ok(status) => Err(HarnessError::message(format!(
            "direct API observation to `{member_id}` unexpectedly succeeded via {}",
            status.queried_via.api_url
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
    match pgtm_connection_target_for_member(world.harness()?, member_id.as_str()) {
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
    let logs = world.harness()?.service_logs(member_id.as_str())?;
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
    let distinct = [member_a.clone(), member_b.clone(), member_c.clone()]
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
    let stopped = world.scenario.stopped_nodes.clone();
    let mut last_error = None;

    while Instant::now() < deadline {
        let attempt: Result<()> = (|| {
            for member_id in ALL_CLUSTER_MEMBERS {
                if stopped.contains(member_id) {
                    continue;
                }
                let debug = world
                    .harness()?
                    .observer()
                    .debug_verbose_via_member(member_id)?;
                let rendered =
                    serde_json::to_string(&debug).map_err(|source| HarnessError::Json {
                        context: format!("serializing debug verbose for `{member_id}`"),
                        source,
                    })?;
                if !rendered.contains("fail_safe") {
                    let ha_phase = debug
                        .get("ha")
                        .and_then(|value| value.get("phase"))
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("missing");
                    let ha_decision = debug
                        .get("ha")
                        .and_then(|value| value.get("decision"))
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("missing");
                    let dcs_trust = debug
                        .get("dcs")
                        .and_then(|value| value.get("trust"))
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("missing");
                    Err(HarnessError::message(format!(
                        "member `{member_id}` debug output did not contain fail_safe (ha.phase={ha_phase}, ha.decision={ha_decision}, dcs.trust={dcs_trust})"
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
            let debug = world
                .harness()?
                .observer()
                .debug_verbose_via_member(member_id.as_str())?;
            let rendered = serde_json::to_string(&debug).map_err(|source| HarnessError::Json {
                context: format!("serializing debug verbose for `{member_id}`"),
                source,
            })?;
            if rendered.contains("fail_safe") {
                return Ok(());
            }
            let status = current_status(world)?;
            match single_primary(&status) {
                Ok(primary) if primary != member_id => Ok(()),
                Ok(primary) => Err(HarnessError::message(format!(
                    "member `{member_id}` still held primary authority as `{primary}`"
                ))),
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
        .last_workload_summary
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
        .assert_member_never_primary_since(member_id.as_str(), marker)
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
        .proof_table
        .clone()
        .ok_or_else(|| HarnessError::message("proof table was not created"))?;
    let member_id = resolve_member_reference(world, member_ref)?;
    let observed =
        fetch_rows_for_member(world.harness()?, table_name.as_str(), member_id.as_str())?;
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
