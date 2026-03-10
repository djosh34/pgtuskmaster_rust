use serde::Serialize;

use crate::{
    api::HaClusterMemberResponse,
    cli::{
        args::ConnectionOptions,
        client::CliTlsConfig,
        config::OperatorContext,
        error::CliError,
        output,
        status::{
            build_sampled_cluster_snapshot, local_role_from_state, ClusterWarning,
            SampledClusterSnapshot,
        },
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionCommandKind {
    Primary,
    Replicas,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConnectionTarget {
    pub member_id: String,
    pub postgres_host: String,
    pub postgres_port: u16,
    pub dsn: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ConnectionView {
    pub cluster_name: String,
    pub scope: String,
    pub kind: ConnectionCommandKind,
    pub tls: bool,
    pub sampled_member_count: usize,
    pub discovered_member_count: usize,
    pub warnings: Vec<ClusterWarning>,
    pub targets: Vec<ConnectionTarget>,
}

pub(crate) async fn run_primary(
    context: &OperatorContext,
    options: ConnectionOptions,
) -> Result<String, CliError> {
    let snapshot = build_sampled_cluster_snapshot(context, false).await?;
    let view = resolve_primary_view(&snapshot, &context.postgres_client_tls, options.tls)?;
    output::render_connection_view(&view, options.json)
}

pub(crate) async fn run_replicas(
    context: &OperatorContext,
    options: ConnectionOptions,
) -> Result<String, CliError> {
    let snapshot = build_sampled_cluster_snapshot(context, false).await?;
    let view = resolve_replicas_view(&snapshot, &context.postgres_client_tls, options.tls)?;
    output::render_connection_view(&view, options.json)
}

pub(crate) fn resolve_primary_view(
    snapshot: &SampledClusterSnapshot,
    tls: &CliTlsConfig,
    emit_tls: bool,
) -> Result<ConnectionView, CliError> {
    let blocking_warnings = blocking_primary_warnings(snapshot);
    if !blocking_warnings.is_empty() {
        return Err(CliError::Resolution(format!(
            "cannot resolve primary from sampled cluster state: {}",
            join_warning_messages(blocking_warnings)
        )));
    }

    let primary_members = find_sampled_members_by_role(snapshot, "primary");
    match primary_members.as_slice() {
        [] => Err(CliError::Resolution(
            "no sampled primary member was observed".to_string(),
        )),
        [member] => Ok(build_connection_view(
            snapshot,
            tls,
            emit_tls,
            ConnectionCommandKind::Primary,
            vec![build_connection_target(member, tls, emit_tls)?],
        )),
        members => Err(CliError::Resolution(format!(
            "multiple sampled primaries were observed: {}",
            members
                .iter()
                .map(|member| member.member_id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

pub(crate) fn resolve_replicas_view(
    snapshot: &SampledClusterSnapshot,
    tls: &CliTlsConfig,
    emit_tls: bool,
) -> Result<ConnectionView, CliError> {
    let blocking_warnings = blocking_replica_warnings(snapshot);
    if !blocking_warnings.is_empty() {
        return Err(CliError::Resolution(format!(
            "cannot resolve replicas from sampled cluster state: {}",
            join_warning_messages(blocking_warnings)
        )));
    }

    let replica_targets = find_sampled_members_by_role(snapshot, "replica")
        .into_iter()
        .map(|member| build_connection_target(member, tls, emit_tls))
        .collect::<Result<Vec<_>, _>>()?;

    if replica_targets.is_empty() {
        return Err(CliError::Resolution(
            "no sampled replica members were observed".to_string(),
        ));
    }

    Ok(build_connection_view(
        snapshot,
        tls,
        emit_tls,
        ConnectionCommandKind::Replicas,
        replica_targets,
    ))
}

fn build_connection_view(
    snapshot: &SampledClusterSnapshot,
    _tls: &CliTlsConfig,
    emit_tls: bool,
    kind: ConnectionCommandKind,
    targets: Vec<ConnectionTarget>,
) -> ConnectionView {
    ConnectionView {
        cluster_name: snapshot.seed_state.cluster_name.clone(),
        scope: snapshot.seed_state.scope.clone(),
        kind,
        tls: emit_tls,
        sampled_member_count: snapshot.sampled_member_count(),
        discovered_member_count: snapshot.discovered_member_count(),
        warnings: snapshot.warnings.clone(),
        targets,
    }
}

fn blocking_primary_warnings(snapshot: &SampledClusterSnapshot) -> Vec<&ClusterWarning> {
    snapshot
        .warnings
        .iter()
        .filter(|warning| {
            matches!(
                warning.code.as_str(),
                "missing_api_url"
                    | "unreachable_node"
                    | "missing_observation"
                    | "insufficient_sampling"
                    | "leader_mismatch"
                    | "membership_mismatch"
                    | "multi_primary"
            )
        })
        .collect()
}

fn blocking_replica_warnings(snapshot: &SampledClusterSnapshot) -> Vec<&ClusterWarning> {
    snapshot
        .warnings
        .iter()
        .filter(|warning| {
            matches!(
                warning.code.as_str(),
                "leader_mismatch" | "membership_mismatch" | "multi_primary"
            )
        })
        .collect()
}

fn join_warning_messages(warnings: Vec<&ClusterWarning>) -> String {
    warnings
        .iter()
        .map(|warning| warning.message.as_str())
        .collect::<Vec<_>>()
        .join("; ")
}

fn find_sampled_members_by_role<'a>(
    snapshot: &'a SampledClusterSnapshot,
    role: &str,
) -> Vec<&'a HaClusterMemberResponse> {
    let mut members = snapshot
        .discovered_members
        .iter()
        .filter(|member| {
            snapshot
                .observations
                .get(&member.member_id)
                .and_then(|observation| observation.sampled.as_ref().ok())
                .is_some_and(|sampled| local_role_from_state(&sampled.state) == role)
        })
        .collect::<Vec<_>>();
    members.sort_by_key(|member| {
        (
            member.member_id != snapshot.queried_via.member_id,
            member.member_id.as_str(),
        )
    });
    members
}

fn build_connection_target(
    member: &HaClusterMemberResponse,
    tls: &CliTlsConfig,
    emit_tls: bool,
) -> Result<ConnectionTarget, CliError> {
    let postgres_host = member.postgres_host.trim();
    if postgres_host.is_empty() || member.postgres_port == 0 {
        return Err(CliError::Resolution(format!(
            "member {} does not advertise PostgreSQL host/port",
            member.member_id
        )));
    }

    let dsn = render_connection_dsn(postgres_host, member.postgres_port, tls, emit_tls)?;
    Ok(ConnectionTarget {
        member_id: member.member_id.clone(),
        postgres_host: postgres_host.to_string(),
        postgres_port: member.postgres_port,
        dsn,
    })
}

fn render_connection_dsn(
    postgres_host: &str,
    postgres_port: u16,
    tls: &CliTlsConfig,
    emit_tls: bool,
) -> Result<String, CliError> {
    let mut fields = vec![
        ("host", postgres_host.to_string()),
        ("port", postgres_port.to_string()),
        ("user", "postgres".to_string()),
        ("dbname", "postgres".to_string()),
    ];
    if emit_tls {
        fields.push(("sslmode", "verify-full".to_string()));
        maybe_push_tls_path_field(
            &mut fields,
            "sslrootcert",
            "pgtm postgres client CA certificate",
            tls.ca_cert_pem.as_ref(),
            tls.ca_cert_path.as_ref(),
        )?;
        maybe_push_tls_path_field(
            &mut fields,
            "sslcert",
            "pgtm postgres client certificate",
            tls.client_cert_pem.as_ref(),
            tls.client_cert_path.as_ref(),
        )?;
        maybe_push_tls_path_field(
            &mut fields,
            "sslkey",
            "pgtm postgres client key",
            tls.client_key_pem.as_ref(),
            tls.client_key_path.as_ref(),
        )?;
    }

    Ok(fields
        .iter()
        .map(|(key, value)| format!("{key}={}", escape_libpq_value(value.as_str())))
        .collect::<Vec<_>>()
        .join(" "))
}

fn maybe_push_tls_path_field(
    fields: &mut Vec<(&'static str, String)>,
    dsn_key: &'static str,
    field_label: &'static str,
    pem: Option<&Vec<u8>>,
    path: Option<&std::path::PathBuf>,
) -> Result<(), CliError> {
    match (pem, path) {
        (Some(_), Some(path)) => {
            fields.push((dsn_key, path.to_string_lossy().into_owned()));
            Ok(())
        }
        (None, Some(path)) => {
            fields.push((dsn_key, path.to_string_lossy().into_owned()));
            Ok(())
        }
        (Some(_), None) => Err(CliError::Resolution(format!(
            "`--tls` cannot render {field_label} because the effective config is not path-backed"
        ))),
        (None, None) => Ok(()),
    }
}

fn escape_libpq_value(value: &str) -> String {
    let requires_quotes = value.is_empty()
        || value
            .chars()
            .any(|ch| ch.is_whitespace() || ch == '\'' || ch == '\\');
    if !requires_quotes {
        return value.to_string();
    }

    let escaped = value.chars().fold(String::new(), |mut acc, ch| {
        match ch {
            '\'' | '\\' => {
                acc.push('\\');
                acc.push(ch);
            }
            _ => acc.push(ch),
        }
        acc
    });
    format!("'{escaped}'")
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::PathBuf};

    use crate::{
        api::{
            DcsTrustResponse, DesiredNodeStateResponse, HaClusterMemberResponse, HaStateResponse,
            MemberRoleResponse, PrimaryPlanResponse, ReadinessResponse, ReplicaPlanResponse,
            SqlStatusResponse,
        },
        cli::{
            client::CliTlsConfig,
            status::{ClusterWarning, QueryOrigin, SampledClusterSnapshot},
        },
    };

    use super::{escape_libpq_value, resolve_primary_view, resolve_replicas_view};

    fn sample_member(member_id: &str, api_url: Option<&str>) -> HaClusterMemberResponse {
        HaClusterMemberResponse {
            member_id: member_id.to_string(),
            postgres_host: format!("{member_id}.db.example.com"),
            postgres_port: 5432,
            api_url: api_url.map(ToString::to_string),
            role: MemberRoleResponse::Replica,
            sql: SqlStatusResponse::Healthy,
            readiness: ReadinessResponse::Ready,
            timeline: Some(7),
            write_lsn: None,
            replay_lsn: Some(5),
            updated_at_ms: 1,
            pg_version: 1,
        }
    }

    fn sample_state(
        self_member_id: &str,
        desired_state: DesiredNodeStateResponse,
        leader: Option<&str>,
        members: Vec<HaClusterMemberResponse>,
    ) -> HaStateResponse {
        HaStateResponse {
            cluster_name: "cluster-a".to_string(),
            scope: "scope-a".to_string(),
            self_member_id: self_member_id.to_string(),
            leader: leader.map(ToString::to_string),
            switchover_pending: false,
            switchover_to: None,
            member_count: members.len(),
            members,
            dcs_trust: DcsTrustResponse::FreshQuorum,
            cluster_mode: match leader {
                Some(value) => crate::api::ClusterModeResponse::InitializedLeaderPresent {
                    leader: value.to_string(),
                },
                None => crate::api::ClusterModeResponse::InitializedNoLeaderFreshQuorum,
            },
            desired_state,
            ha_tick: 1,
            snapshot_sequence: 10,
        }
    }

    fn sample_snapshot(
        seed_state: HaStateResponse,
        discovered_members: Vec<HaClusterMemberResponse>,
        observations: BTreeMap<String, crate::cli::status::PeerObservation>,
        warnings: Vec<ClusterWarning>,
    ) -> SampledClusterSnapshot {
        SampledClusterSnapshot {
            seed_state,
            discovered_members,
            queried_via: QueryOrigin {
                member_id: "node-a".to_string(),
                api_url: "http://node-a:8080".to_string(),
            },
            observations,
            warnings,
        }
    }

    fn tls_paths() -> CliTlsConfig {
        CliTlsConfig {
            ca_cert_path: Some(PathBuf::from("/etc/pgtm/postgres ca.pem")),
            client_cert_path: Some(PathBuf::from("/etc/pgtm/postgres.crt")),
            client_key_path: Some(PathBuf::from("/run/secrets/postgres.key")),
            ca_cert_pem: Some(b"ca".to_vec()),
            client_cert_pem: Some(b"cert".to_vec()),
            client_key_pem: Some(b"key".to_vec()),
        }
    }

    #[test]
    fn primary_resolution_renders_single_dsn() -> Result<(), String> {
        let members = vec![
            sample_member("node-a", Some("http://node-a:8080")),
            sample_member("node-b", Some("http://node-b:8080")),
        ];
        let seed_state = sample_state(
            "node-a",
            DesiredNodeStateResponse::Primary {
                plan: PrimaryPlanResponse::KeepLeader,
            },
            Some("node-a"),
            members.clone(),
        );
        let replica_state = sample_state(
            "node-b",
            DesiredNodeStateResponse::Replica {
                plan: ReplicaPlanResponse::DirectFollow {
                    leader_member_id: "node-a".to_string(),
                },
            },
            Some("node-a"),
            members.clone(),
        );
        let observations = BTreeMap::from([
            (
                "node-a".to_string(),
                crate::cli::status::PeerObservation {
                    member_id: "node-a".to_string(),
                    sampled: Ok(crate::cli::status::SampledNodeState {
                        state: seed_state.clone(),
                        debug: None,
                    }),
                },
            ),
            (
                "node-b".to_string(),
                crate::cli::status::PeerObservation {
                    member_id: "node-b".to_string(),
                    sampled: Ok(crate::cli::status::SampledNodeState {
                        state: replica_state,
                        debug: None,
                    }),
                },
            ),
        ]);
        let snapshot = sample_snapshot(seed_state, members, observations, Vec::new());

        let view = resolve_primary_view(&snapshot, &CliTlsConfig::default(), false)
            .map_err(|err| err.to_string())?;

        if view.targets.len() != 1 {
            return Err("expected exactly one primary target".to_string());
        }
        if view.targets[0].dsn
            != "host=node-a.db.example.com port=5432 user=postgres dbname=postgres"
        {
            return Err(format!("unexpected dsn {}", view.targets[0].dsn));
        }
        Ok(())
    }

    #[test]
    fn primary_resolution_fails_when_sampling_is_incomplete() {
        let members = vec![
            sample_member("node-a", Some("http://node-a:8080")),
            sample_member("node-b", Some("http://node-b:8080")),
        ];
        let seed_state = sample_state(
            "node-a",
            DesiredNodeStateResponse::Primary {
                plan: PrimaryPlanResponse::KeepLeader,
            },
            Some("node-a"),
            members.clone(),
        );
        let observations = BTreeMap::from([
            (
                "node-a".to_string(),
                crate::cli::status::PeerObservation {
                    member_id: "node-a".to_string(),
                    sampled: Ok(crate::cli::status::SampledNodeState {
                        state: seed_state.clone(),
                        debug: None,
                    }),
                },
            ),
            (
                "node-b".to_string(),
                crate::cli::status::PeerObservation {
                    member_id: "node-b".to_string(),
                    sampled: Err("timed out".to_string()),
                },
            ),
        ]);
        let warnings = vec![
            ClusterWarning {
                code: "unreachable_node".to_string(),
                message: "node node-b could not be sampled: timed out".to_string(),
            },
            ClusterWarning {
                code: "insufficient_sampling".to_string(),
                message: "sampled 1/2 discovered members".to_string(),
            },
        ];
        let snapshot = sample_snapshot(seed_state, members, observations, warnings);

        let result = resolve_primary_view(&snapshot, &CliTlsConfig::default(), false);
        assert!(
            result.is_err(),
            "primary should fail on incomplete sampling"
        );
    }

    #[test]
    fn replicas_resolution_allows_partial_sampling() -> Result<(), String> {
        let members = vec![
            sample_member("node-a", Some("http://node-a:8080")),
            sample_member("node-b", Some("http://node-b:8080")),
            sample_member("node-c", Some("http://node-c:8080")),
        ];
        let seed_state = sample_state(
            "node-a",
            DesiredNodeStateResponse::Primary {
                plan: PrimaryPlanResponse::KeepLeader,
            },
            Some("node-a"),
            members.clone(),
        );
        let replica_state = sample_state(
            "node-b",
            DesiredNodeStateResponse::Replica {
                plan: ReplicaPlanResponse::DirectFollow {
                    leader_member_id: "node-a".to_string(),
                },
            },
            Some("node-a"),
            members.clone(),
        );
        let observations = BTreeMap::from([
            (
                "node-a".to_string(),
                crate::cli::status::PeerObservation {
                    member_id: "node-a".to_string(),
                    sampled: Ok(crate::cli::status::SampledNodeState {
                        state: seed_state.clone(),
                        debug: None,
                    }),
                },
            ),
            (
                "node-b".to_string(),
                crate::cli::status::PeerObservation {
                    member_id: "node-b".to_string(),
                    sampled: Ok(crate::cli::status::SampledNodeState {
                        state: replica_state,
                        debug: None,
                    }),
                },
            ),
            (
                "node-c".to_string(),
                crate::cli::status::PeerObservation {
                    member_id: "node-c".to_string(),
                    sampled: Err("timed out".to_string()),
                },
            ),
        ]);
        let warnings = vec![
            ClusterWarning {
                code: "unreachable_node".to_string(),
                message: "node node-c could not be sampled: timed out".to_string(),
            },
            ClusterWarning {
                code: "insufficient_sampling".to_string(),
                message: "sampled 2/3 discovered members".to_string(),
            },
        ];
        let snapshot = sample_snapshot(seed_state, members, observations, warnings);

        let view = resolve_replicas_view(&snapshot, &CliTlsConfig::default(), false)
            .map_err(|err| err.to_string())?;
        if view.targets.len() != 1 {
            return Err("expected one reachable replica".to_string());
        }
        if view.targets[0].member_id != "node-b" {
            return Err(format!(
                "expected node-b replica target, got {}",
                view.targets[0].member_id
            ));
        }
        Ok(())
    }

    #[test]
    fn tls_rendering_requires_path_backed_material() {
        let members = vec![sample_member("node-a", Some("http://node-a:8080"))];
        let seed_state = sample_state(
            "node-a",
            DesiredNodeStateResponse::Primary {
                plan: PrimaryPlanResponse::KeepLeader,
            },
            Some("node-a"),
            members.clone(),
        );
        let observations = BTreeMap::from([(
            "node-a".to_string(),
            crate::cli::status::PeerObservation {
                member_id: "node-a".to_string(),
                sampled: Ok(crate::cli::status::SampledNodeState {
                    state: seed_state.clone(),
                    debug: None,
                }),
            },
        )]);
        let snapshot = sample_snapshot(seed_state, members, observations, Vec::new());
        let tls = CliTlsConfig {
            ca_cert_pem: Some(b"inline".to_vec()),
            ..CliTlsConfig::default()
        };

        let result = resolve_primary_view(&snapshot, &tls, true);
        assert!(
            result.is_err(),
            "inline-only TLS material should not be rendered into DSN output"
        );
    }

    #[test]
    fn tls_paths_are_quoted_for_libpq_keyword_values() -> Result<(), String> {
        let members = vec![sample_member("node-a", Some("http://node-a:8080"))];
        let seed_state = sample_state(
            "node-a",
            DesiredNodeStateResponse::Primary {
                plan: PrimaryPlanResponse::KeepLeader,
            },
            Some("node-a"),
            members.clone(),
        );
        let observations = BTreeMap::from([(
            "node-a".to_string(),
            crate::cli::status::PeerObservation {
                member_id: "node-a".to_string(),
                sampled: Ok(crate::cli::status::SampledNodeState {
                    state: seed_state.clone(),
                    debug: None,
                }),
            },
        )]);
        let snapshot = sample_snapshot(seed_state, members, observations, Vec::new());

        let view =
            resolve_primary_view(&snapshot, &tls_paths(), true).map_err(|err| err.to_string())?;
        if !view.targets[0]
            .dsn
            .contains("sslrootcert='/etc/pgtm/postgres ca.pem'")
        {
            return Err(format!("unexpected tls dsn {}", view.targets[0].dsn));
        }
        Ok(())
    }

    #[test]
    fn libpq_escaping_quotes_special_values() {
        assert_eq!(escape_libpq_value("simple"), "simple");
        assert_eq!(escape_libpq_value("/tmp/a b"), "'/tmp/a b'");
        assert_eq!(escape_libpq_value("a'b"), "'a\\'b'");
        assert_eq!(escape_libpq_value(r"a\b"), r"'a\\b'");
    }
}
