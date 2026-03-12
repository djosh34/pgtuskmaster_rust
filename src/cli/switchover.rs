use crate::{
    api::{DcsTrustResponse, MemberRoleResponse, ReadinessResponse, SqlStatusResponse},
    cli::{
        client::CliApiClient,
        config::OperatorContext,
        error::CliError,
        output,
        status::{
            authority_primary_member, build_sampled_cluster_snapshot, observed_role,
            PeerObservation, SampledClusterSnapshot,
        },
    },
};

pub(crate) async fn run_clear(context: &OperatorContext, json: bool) -> Result<String, CliError> {
    let client = CliApiClient::from_config(context.api_client.clone())?;
    let response = client.delete_switchover().await?;
    output::render_accepted_output(&response, json)
}

pub(crate) async fn run_request(
    context: &OperatorContext,
    json: bool,
    switchover_to: Option<String>,
) -> Result<String, CliError> {
    let snapshot = build_sampled_cluster_snapshot(context, false).await?;
    validate_switchover_request(&snapshot, switchover_to.as_deref())?;

    let client = CliApiClient::from_config(context.api_client.clone())?;
    let response = client.post_switchover(switchover_to).await?;
    output::render_accepted_output(&response, json)
}

fn validate_switchover_request(
    snapshot: &SampledClusterSnapshot,
    switchover_to: Option<&str>,
) -> Result<(), CliError> {
    validate_seed_authority(snapshot)?;

    let Some(target_member_id) = switchover_to else {
        return Ok(());
    };

    let target_observation = snapshot
        .observations
        .get(target_member_id)
        .ok_or_else(|| {
            CliError::Resolution(format!(
                "cannot target member `{target_member_id}` for switchover: no sampled observation was recorded"
            ))
        })?;

    let target_sample = match target_observation {
        PeerObservation {
            sampled: Ok(sampled),
            ..
        } => sampled,
        PeerObservation {
            sampled: Err(message),
            ..
        } => {
            return Err(CliError::Resolution(format!(
                "cannot target member `{target_member_id}` for switchover: sampled cluster state could not reach it: {message}"
            )));
        }
    };

    let target_member = target_sample
        .state
        .members
        .iter()
        .find(|member| member.member_id == target_member_id)
        .ok_or_else(|| {
            CliError::Resolution(format!(
                "cannot target member `{target_member_id}` for switchover: sampled target state did not include its own member record"
            ))
        })?;

    if target_sample.state.dcs_trust != DcsTrustResponse::FullQuorum {
        return Err(CliError::Resolution(format!(
            "cannot target member `{target_member_id}` for switchover: it does not report full quorum trust"
        )));
    }

    if observed_role(target_member, &target_sample.state) != "replica"
        || target_member.role != MemberRoleResponse::Replica
        || target_member.sql != SqlStatusResponse::Healthy
        || target_member.readiness != ReadinessResponse::Ready
    {
        return Err(CliError::Resolution(format!(
            "cannot target member `{target_member_id}` for switchover: it is not a healthy replica in sampled cluster state"
        )));
    }

    Ok(())
}

fn validate_seed_authority(snapshot: &SampledClusterSnapshot) -> Result<(), CliError> {
    if snapshot.seed_state.dcs_trust != DcsTrustResponse::FullQuorum {
        return Err(CliError::Resolution(format!(
            "cannot request switchover via `{}`: sampled seed node does not have full quorum trust",
            snapshot.seed_state.self_member_id
        )));
    }

    if authority_primary_member(&snapshot.seed_state).as_deref()
        != Some(snapshot.seed_state.self_member_id.as_str())
    {
        return Err(CliError::Resolution(format!(
            "cannot request switchover via `{}`: sampled seed node is not the authoritative primary",
            snapshot.seed_state.self_member_id
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        api::{
            DcsTrustResponse, HaAuthorityResponse, HaClusterMemberResponse, HaStateResponse,
            LeaseEpochResponse, MemberRoleResponse, ReadinessResponse, SqlStatusResponse,
            TargetRoleResponse,
        },
        cli::{
            error::CliError,
            status::{PeerObservation, QueryOrigin, SampledClusterSnapshot, SampledNodeState},
            switchover::validate_switchover_request,
        },
    };

    fn sample_member(member_id: &str) -> HaClusterMemberResponse {
        HaClusterMemberResponse {
            member_id: member_id.to_string(),
            postgres_host: member_id.to_string(),
            postgres_port: 5432,
            api_url: Some(format!("https://{member_id}:8443")),
            role: if member_id == "node-a" {
                MemberRoleResponse::Primary
            } else {
                MemberRoleResponse::Replica
            },
            sql: SqlStatusResponse::Healthy,
            readiness: ReadinessResponse::Ready,
            timeline: Some(if member_id == "node-a" { 2 } else { 1 }),
            write_lsn: Some(20),
            replay_lsn: Some(10),
            updated_at_ms: 1,
            pg_version: 1,
        }
    }

    fn sample_state(
        self_member_id: &str,
        trust: DcsTrustResponse,
        authority: HaAuthorityResponse,
        members: Vec<HaClusterMemberResponse>,
    ) -> HaStateResponse {
        HaStateResponse {
            cluster_name: "cluster-a".to_string(),
            scope: "scope-a".to_string(),
            self_member_id: self_member_id.to_string(),
            leader: authority_primary_member_for_test(&authority),
            switchover_pending: false,
            switchover_to: None,
            member_count: members.len(),
            members,
            dcs_trust: trust,
            authority,
            fence_cutoff: None,
            ha_role: TargetRoleResponse::Leader {
                epoch: LeaseEpochResponse {
                    holder: self_member_id.to_string(),
                    generation: 42,
                },
            },
            ha_tick: 7,
            planned_actions: Vec::new(),
            snapshot_sequence: 1,
        }
    }

    fn authority_primary_for(member_id: &str) -> HaAuthorityResponse {
        HaAuthorityResponse::Primary {
            member_id: member_id.to_string(),
            epoch: LeaseEpochResponse {
                holder: member_id.to_string(),
                generation: 42,
            },
        }
    }

    fn authority_primary_member_for_test(authority: &HaAuthorityResponse) -> Option<String> {
        match authority {
            HaAuthorityResponse::Primary { member_id, .. } => Some(member_id.clone()),
            HaAuthorityResponse::NoPrimary { .. } | HaAuthorityResponse::Unknown => None,
        }
    }

    fn sampled_snapshot(
        seed_state: HaStateResponse,
        observations: Vec<PeerObservation>,
    ) -> SampledClusterSnapshot {
        SampledClusterSnapshot {
            queried_via: QueryOrigin {
                member_id: seed_state.self_member_id.clone(),
                api_url: format!("https://{}:8443", seed_state.self_member_id),
            },
            discovered_members: seed_state.members.clone(),
            warnings: Vec::new(),
            observations: observations
                .into_iter()
                .map(|observation| (observation.member_id.clone(), observation))
                .collect::<BTreeMap<_, _>>(),
            seed_state,
        }
    }

    #[test]
    fn targeted_request_rejects_unreachable_target() {
        let members = vec![sample_member("node-a"), sample_member("node-b")];
        let seed_state = sample_state(
            "node-a",
            DcsTrustResponse::FullQuorum,
            authority_primary_for("node-a"),
            members,
        );
        let snapshot = sampled_snapshot(
            seed_state.clone(),
            vec![
                PeerObservation {
                    member_id: "node-a".to_string(),
                    sampled: Ok(SampledNodeState {
                        state: seed_state,
                        debug: None,
                    }),
                },
                PeerObservation {
                    member_id: "node-b".to_string(),
                    sampled: Err("transport error".to_string()),
                },
            ],
        );

        let result = validate_switchover_request(&snapshot, Some("node-b"));
        match result {
            Err(CliError::Resolution(message)) => {
                assert!(message.contains("could not reach"));
            }
            other => panic!("expected resolution error, got {other:?}"),
        }
    }

    #[test]
    fn targeted_request_accepts_healthy_replica() {
        let members = vec![sample_member("node-a"), sample_member("node-b")];
        let seed_state = sample_state(
            "node-a",
            DcsTrustResponse::FullQuorum,
            authority_primary_for("node-a"),
            members.clone(),
        );
        let target_state = sample_state(
            "node-b",
            DcsTrustResponse::FullQuorum,
            authority_primary_for("node-a"),
            members,
        );
        let snapshot = sampled_snapshot(
            seed_state.clone(),
            vec![
                PeerObservation {
                    member_id: "node-a".to_string(),
                    sampled: Ok(SampledNodeState {
                        state: seed_state,
                        debug: None,
                    }),
                },
                PeerObservation {
                    member_id: "node-b".to_string(),
                    sampled: Ok(SampledNodeState {
                        state: target_state,
                        debug: None,
                    }),
                },
            ],
        );

        let result = validate_switchover_request(&snapshot, Some("node-b"));
        assert!(result.is_ok());
    }

    #[test]
    fn request_rejects_non_authoritative_seed() {
        let members = vec![sample_member("node-a"), sample_member("node-b")];
        let seed_state = sample_state(
            "node-a",
            DcsTrustResponse::FullQuorum,
            HaAuthorityResponse::NoPrimary {
                reason: crate::api::NoPrimaryReasonResponse::Recovering,
            },
            members,
        );
        let snapshot = sampled_snapshot(
            seed_state.clone(),
            vec![PeerObservation {
                member_id: "node-a".to_string(),
                sampled: Ok(SampledNodeState {
                    state: seed_state,
                    debug: None,
                }),
            }],
        );

        let result = validate_switchover_request(&snapshot, None);
        match result {
            Err(CliError::Resolution(message)) => {
                assert!(message.contains("not the authoritative primary"));
            }
            other => panic!("expected resolution error, got {other:?}"),
        }
    }
}
