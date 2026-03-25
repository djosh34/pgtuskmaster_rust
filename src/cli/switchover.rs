use std::net::SocketAddr;

use crate::{
    api::{authoritative_primary_member, NodeState},
    cli::{
        client::{CliApiClient, CliApiClientConfig},
        config::OperatorContext,
        error::CliError,
        render::render_accepted_response,
        status::fetch_seed_state,
    },
    dcs::DcsMemberState,
    state::MemberId,
};

pub(crate) async fn run_clear(context: &OperatorContext, json: bool) -> Result<String, CliError> {
    let client = CliApiClient::from_config(context.api_client.clone())?;
    let response = client.delete_switchover().await?;
    render_accepted_response(&response, json)
}

pub(crate) async fn run_request(
    context: &OperatorContext,
    json: bool,
    switchover_to: Option<String>,
) -> Result<String, CliError> {
    let (state, _queried_via) = fetch_seed_state(context).await?;
    let client_config =
        authoritative_primary_client_config(context, &state, switchover_to.as_deref())?;

    let client = CliApiClient::from_config(client_config)?;
    let response = client.post_switchover(switchover_to).await?;
    render_accepted_response(&response, json)
}

fn authoritative_primary_client_config(
    context: &OperatorContext,
    state: &NodeState,
    switchover_to: Option<&str>,
) -> Result<CliApiClientConfig, CliError> {
    let (primary_id, primary_member) = resolve_switchover_request(state, switchover_to)?;
    if primary_id == &state.identity.member_id {
        return Ok(context.api_client.clone());
    }
    let operator_api = primary_member.operator_api_target().ok_or_else(|| {
        CliError::Resolution(format!(
            "authoritative primary `{}` does not advertise an operator API route",
            primary_id.as_str()
        ))
    })?;
    let base_url = operator_api.to_url().map_err(|err| {
        CliError::Resolution(format!(
            "authoritative primary `{}` advertises an invalid operator API route: {err}",
            primary_id.as_str()
        ))
    })?;
    let resolve_to = remap_resolve_to(context.api_client.resolve_to, &base_url)?;

    Ok(CliApiClientConfig {
        base_url,
        resolve_to,
        ..context.api_client.clone()
    })
}

fn resolve_switchover_request<'a>(
    state: &'a NodeState,
    switchover_to: Option<&str>,
) -> Result<(&'a MemberId, &'a DcsMemberState), CliError> {
    if !state.dcs.is_quorum() {
        return Err(CliError::Resolution(format!(
            "cannot request switchover via `{}`: seed node does not currently report coordinated DCS state",
            state.identity.member_id.0
        )));
    }

    let primary_id = authoritative_primary_member(state).ok_or_else(|| {
        CliError::Resolution(
            "seed state does not currently expose an authoritative primary".to_string(),
        )
    })?;
    let primary_member = state.dcs.member(primary_id).ok_or_else(|| {
        CliError::Resolution(format!(
            "authoritative primary `{}` is not present in the DCS member slots",
            primary_id.as_str()
        ))
    })?;
    validate_switchover_target(state, primary_id, switchover_to)?;

    Ok((primary_id, primary_member))
}

fn validate_switchover_target(
    state: &NodeState,
    primary_id: &MemberId,
    switchover_to: Option<&str>,
) -> Result<(), CliError> {
    let Some(target_member_id) = switchover_to else {
        return Ok(());
    };

    if primary_id.as_str() == target_member_id {
        return Err(CliError::Resolution(format!(
            "cannot target member `{target_member_id}` for switchover: it is already the leader"
        )));
    }

    let target_member = state
        .dcs
        .member(&MemberId(target_member_id.to_string()))
        .ok_or_else(|| {
            CliError::Resolution(format!(
                "cannot target member `{target_member_id}` for switchover: it is not present in the seed node DCS member slots"
            ))
        })?;

    if !target_member.postgres().is_ready_replica() {
        return Err(CliError::Resolution(format!(
            "cannot target member `{target_member_id}` for switchover: it is not a ready replica in the seed node DCS view"
        )));
    }

    Ok(())
}

fn remap_resolve_to(
    resolve_to: Option<SocketAddr>,
    base_url: &reqwest::Url,
) -> Result<Option<SocketAddr>, CliError> {
    resolve_to
        .map(|resolve_to| {
            let port = base_url.port_or_known_default().ok_or_else(|| {
                CliError::RequestBuild(format!(
                    "resolved authoritative API URL `{base_url}` did not include a usable port"
                ))
            })?;
            Ok(SocketAddr::new(resolve_to.ip(), port))
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        io::{Read, Write},
        net::TcpListener,
        sync::mpsc,
        time::Duration,
    };

    use reqwest::Url;

    use super::run_request;
    use crate::{
        api::NodeState,
        cli::{
            client::{CliApiClientConfig, CliAuthConfig},
            config::OperatorContext,
        },
        dcs::{DcsMemberState, DcsSnapshot},
        ha::{
            state::HaState,
            types::{AuthorityProjection, HaDecision, HaObservation, PublicationState},
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus},
        process::state::ProcessState,
        state::{
            ApiRoute, ClusterName, LeaseEpoch, MemberId, NodeIdentity, PgRoute, ScopeName,
            SwitchoverState, TimelineId, UnixMillis, WalLsn, WorkerStatus,
        },
    };

    #[tokio::test(flavor = "current_thread")]
    async fn request_posts_to_authoritative_primary_advertised_api_route() -> Result<(), String> {
        let (primary_addr, primary_rx) = spawn_request_server(vec![
            "HTTP/1.1 202 Accepted\r\ncontent-type: application/json\r\ncontent-length: 17\r\n\r\n{\"accepted\":true}".to_string(),
        ])?;
        let state = sample_seed_state(format!("http://127.0.0.1:{}/", primary_addr.port()))?;
        let state_response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
            serde_json::to_string(&state)
                .map_err(|err| format!("serialize state failed: {err}"))?
                .len(),
            serde_json::to_string(&state)
                .map_err(|err| format!("serialize state failed: {err}"))?
        );
        let (seed_addr, seed_rx) = spawn_request_server(vec![state_response])?;

        let context = OperatorContext {
            api_client: CliApiClientConfig {
                base_url: Url::parse(format!("http://127.0.0.1:{}/", seed_addr.port()).as_str())
                    .map_err(|err| format!("build seed url failed: {err}"))?,
                timeout_ms: 5_000,
                auth: CliAuthConfig {
                    read_token: Some("read-token".to_string()),
                    admin_token: Some("admin-token".to_string()),
                },
                tls: None,
                resolve_to: None,
            },
            postgres_client_tls: None,
            api_auth_enabled: true,
        };

        run_request(&context, false, Some("node-b".to_string()))
            .await
            .map_err(|err| err.to_string())?;

        let seed_request = seed_rx
            .recv_timeout(Duration::from_secs(2))
            .map_err(|err| format!("failed to receive seed request: {err}"))?;
        let seed_request_lower = seed_request.to_lowercase();
        if !seed_request.starts_with("GET /state HTTP/1.1") {
            return Err(format!("unexpected seed request: {seed_request}"));
        }
        if !seed_request_lower.contains("authorization: bearer read-token") {
            return Err(format!(
                "seed request did not use read token: {seed_request}"
            ));
        }

        let primary_request = primary_rx
            .recv_timeout(Duration::from_secs(2))
            .map_err(|err| format!("failed to receive primary request: {err}"))?;
        let primary_request_lower = primary_request.to_lowercase();
        if !primary_request.starts_with("POST /switchover HTTP/1.1") {
            return Err(format!("unexpected primary request: {primary_request}"));
        }
        if !primary_request_lower.contains("authorization: bearer admin-token") {
            return Err(format!(
                "primary request did not use admin token: {primary_request}"
            ));
        }
        if !primary_request_lower
            .contains(format!("host: 127.0.0.1:{}", primary_addr.port()).as_str())
        {
            return Err(format!(
                "primary request did not target the authoritative primary route: {primary_request}"
            ));
        }
        if !primary_request.contains("node-b") {
            return Err(format!(
                "primary request did not contain the targeted switchover member: {primary_request}"
            ));
        }

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn request_reuses_seed_route_when_seed_is_authoritative_primary() -> Result<(), String> {
        let state = sample_seed_primary_state()?;
        let state_body = serde_json::to_string(&state)
            .map_err(|err| format!("serialize state failed: {err}"))?;
        let state_response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
            state_body.len(),
            state_body
        );
        let (seed_addr, seed_rx) = spawn_request_server(vec![
            state_response,
            "HTTP/1.1 202 Accepted\r\ncontent-type: application/json\r\ncontent-length: 17\r\n\r\n{\"accepted\":true}".to_string(),
        ])?;

        let context = OperatorContext {
            api_client: CliApiClientConfig {
                base_url: Url::parse(format!("http://seed.invalid:{}/", seed_addr.port()).as_str())
                    .map_err(|err| format!("build seed url failed: {err}"))?,
                timeout_ms: 5_000,
                auth: CliAuthConfig {
                    read_token: Some("read-token".to_string()),
                    admin_token: Some("admin-token".to_string()),
                },
                tls: None,
                resolve_to: Some(std::net::SocketAddr::from((
                    [127, 0, 0, 1],
                    seed_addr.port(),
                ))),
            },
            postgres_client_tls: None,
            api_auth_enabled: true,
        };

        run_request(&context, false, Some("node-b".to_string()))
            .await
            .map_err(|err| err.to_string())?;

        let state_request = seed_rx
            .recv_timeout(Duration::from_secs(2))
            .map_err(|err| format!("failed to receive state request: {err}"))?;
        let state_request_lower = state_request.to_lowercase();
        if !state_request.starts_with("GET /state HTTP/1.1") {
            return Err(format!("unexpected state request: {state_request}"));
        }
        if !state_request_lower.contains("authorization: bearer read-token") {
            return Err(format!(
                "state request did not use read token: {state_request}"
            ));
        }

        let seed_request = seed_rx
            .recv_timeout(Duration::from_secs(2))
            .map_err(|err| format!("failed to receive seed request: {err}"))?;
        let seed_request_lower = seed_request.to_lowercase();
        if !seed_request.starts_with("POST /switchover HTTP/1.1") {
            return Err(format!("unexpected seed request: {seed_request}"));
        }
        if !seed_request_lower.contains("authorization: bearer admin-token") {
            return Err(format!(
                "seed request did not use admin token: {seed_request}"
            ));
        }
        if !seed_request_lower.contains(format!("host: seed.invalid:{}", seed_addr.port()).as_str())
        {
            return Err(format!(
                "seed request did not reuse the configured seed route: {seed_request}"
            ));
        }

        Ok(())
    }

    fn sample_seed_state(primary_api_url: String) -> Result<NodeState, String> {
        let primary_id = MemberId("node-c".to_string());
        let target_id = MemberId("node-b".to_string());
        let seed_id = MemberId("node-a".to_string());
        Ok(NodeState {
            identity: NodeIdentity {
                cluster_name: ClusterName("cluster-a".to_string()),
                scope: ScopeName("scope-a".to_string()),
                member_id: seed_id.clone(),
            },
            pg: replica_state(),
            process: ProcessState::starting(),
            dcs: DcsSnapshot::quorum(
                Some(LeaseEpoch {
                    holder: primary_id.clone(),
                    generation: 7,
                }),
                SwitchoverState::None,
                BTreeMap::from([
                    (
                        seed_id,
                        DcsMemberState {
                            cluster_postgres: PgRoute::tcp("node-a".to_string(), 5432)?,
                            operator_postgres: None,
                            operator_api: None,
                            postgres: replica_state(),
                        },
                    ),
                    (
                        target_id.clone(),
                        DcsMemberState {
                            cluster_postgres: PgRoute::tcp("node-b".to_string(), 5432)?,
                            operator_postgres: None,
                            operator_api: None,
                            postgres: replica_state(),
                        },
                    ),
                    (
                        primary_id.clone(),
                        DcsMemberState {
                            cluster_postgres: PgRoute::tcp("node-c".to_string(), 5432)?,
                            operator_postgres: None,
                            operator_api: Some(ApiRoute::parse(primary_api_url)?),
                            postgres: primary_state(),
                        },
                    ),
                ]),
            ),
            ha: HaState {
                worker: WorkerStatus::Running,
                tick: 0,
                managed_roles_reconciled: true,
                publication: PublicationState::Projected(AuthorityProjection::Primary(
                    LeaseEpoch {
                        holder: primary_id,
                        generation: 7,
                    },
                )),
                decision: HaDecision::initial(),
                observation: HaObservation::initial(),
                clear_switchover: false,
                steps: Vec::new(),
            },
        })
    }

    fn sample_seed_primary_state() -> Result<NodeState, String> {
        let primary_id = MemberId("node-a".to_string());
        let target_id = MemberId("node-b".to_string());
        let other_replica_id = MemberId("node-c".to_string());
        Ok(NodeState {
            identity: NodeIdentity {
                cluster_name: ClusterName("cluster-a".to_string()),
                scope: ScopeName("scope-a".to_string()),
                member_id: primary_id.clone(),
            },
            pg: primary_state(),
            process: ProcessState::starting(),
            dcs: DcsSnapshot::quorum(
                Some(LeaseEpoch {
                    holder: primary_id.clone(),
                    generation: 7,
                }),
                SwitchoverState::None,
                BTreeMap::from([
                    (
                        primary_id.clone(),
                        DcsMemberState {
                            cluster_postgres: PgRoute::tcp("node-a".to_string(), 5432)?,
                            operator_postgres: None,
                            operator_api: None,
                            postgres: primary_state(),
                        },
                    ),
                    (
                        target_id.clone(),
                        DcsMemberState {
                            cluster_postgres: PgRoute::tcp("node-b".to_string(), 5432)?,
                            operator_postgres: None,
                            operator_api: None,
                            postgres: replica_state(),
                        },
                    ),
                    (
                        other_replica_id,
                        DcsMemberState {
                            cluster_postgres: PgRoute::tcp("node-c".to_string(), 5432)?,
                            operator_postgres: None,
                            operator_api: None,
                            postgres: replica_state(),
                        },
                    ),
                ]),
            ),
            ha: HaState {
                worker: WorkerStatus::Running,
                tick: 0,
                managed_roles_reconciled: true,
                publication: PublicationState::Projected(AuthorityProjection::Primary(
                    LeaseEpoch {
                        holder: primary_id,
                        generation: 7,
                    },
                )),
                decision: HaDecision::initial(),
                observation: HaObservation::initial(),
                clear_switchover: false,
                steps: Vec::new(),
            },
        })
    }

    fn primary_state() -> PgInfoState {
        PgInfoState::Primary {
            common: sample_common(Readiness::Ready, false),
            wal_lsn: WalLsn(42),
            slots: Vec::new(),
        }
    }

    fn replica_state() -> PgInfoState {
        PgInfoState::Replica {
            common: sample_common(Readiness::Ready, true),
            replay_lsn: WalLsn(41),
            follow_lsn: Some(WalLsn(42)),
            upstream: Some(crate::pginfo::state::UpstreamInfo {
                member_id: MemberId("node-c".to_string()),
            }),
        }
    }

    fn sample_common(readiness: Readiness, hot_standby: bool) -> PgInfoCommon {
        PgInfoCommon {
            worker: WorkerStatus::Running,
            sql: SqlStatus::Healthy,
            readiness,
            timeline: Some(TimelineId(7)),
            system_identifier: None,
            pg_config: PgConfig {
                port: Some(5432),
                hot_standby: Some(hot_standby),
                primary_conninfo: None,
                primary_slot_name: None,
                extra: BTreeMap::new(),
            },
            last_refresh_at: Some(UnixMillis(123)),
        }
    }

    fn spawn_request_server(
        responses: Vec<String>,
    ) -> Result<(std::net::SocketAddr, mpsc::Receiver<String>), String> {
        let listener =
            TcpListener::bind("127.0.0.1:0").map_err(|err| format!("bind failed: {err}"))?;
        let addr = listener
            .local_addr()
            .map_err(|err| format!("local_addr failed: {err}"))?;
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = (|| -> Result<(), String> {
                for response in responses {
                    let (mut stream, _) = listener
                        .accept()
                        .map_err(|err| format!("accept failed: {err}"))?;
                    let mut buf = [0_u8; 8192];
                    let bytes = stream
                        .read(&mut buf)
                        .map_err(|err| format!("read failed: {err}"))?;
                    let request = String::from_utf8(buf[..bytes].to_vec())
                        .map_err(|err| format!("request utf8 decode failed: {err}"))?;
                    stream
                        .write_all(response.as_bytes())
                        .map_err(|err| format!("write failed: {err}"))?;
                    tx.send(request)
                        .map_err(|err| format!("send request failed: {err}"))?;
                }
                Ok(())
            })();
            if let Err(err) = result {
                let _ = tx.send(format!("server-error: {err}"));
            }
        });
        Ok((addr, rx))
    }
}
