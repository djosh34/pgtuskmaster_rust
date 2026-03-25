use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    net::{Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
};

use pgtuskmaster_rust::{
    api::{authoritative_primary_member, AcceptedResponse, NodeState},
    ha::types::{AuthorityProjection, PublicationState},
    pginfo::{
        conninfo::PgClientTls,
        state::{PgConnInfo, PgInfoState, PgSslMode, Readiness},
    },
};
use serde::de::DeserializeOwned;

use crate::support::{
    config::harness_settings,
    docker::cli::DockerCli,
    error::{HarnessError, Result},
    process,
    topology::ClusterMember,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostgresRoutingTarget {
    pub member: ClusterMember,
    pub conninfo: PgConnInfo,
}

type ObservedStateResult = std::result::Result<NodeState, String>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClusterStateObservation(BTreeMap<ClusterMember, ObservedStateResult>);

impl ClusterStateObservation {
    pub fn iter(&self) -> impl Iterator<Item = (&ClusterMember, &ObservedStateResult)> {
        self.0.iter()
    }

    pub fn into_values(self) -> impl Iterator<Item = ObservedStateResult> {
        self.0.into_values()
    }

    pub fn compatible_primary(
        &self,
        relevant_members: &[ClusterMember],
        expected_online: usize,
        exact_primary: Option<ClusterMember>,
    ) -> Result<ClusterMember> {
        let states = relevant_members
            .iter()
            .copied()
            .map(|member| {
                self.require_observed_member_state(member)
                    .map(|state| (member, state))
            })
            .collect::<Result<Vec<_>>>()?;
        let observed_primaries = states
            .iter()
            .filter_map(|(_member, state)| authoritative_primary(state))
            .collect::<BTreeSet<_>>();
        let primary = match observed_primaries.len() {
            0 => Err(HarnessError::message(format!(
                "cluster has no compatible authoritative primary; observations={}",
                format_observed_authorities(states.as_slice()),
            ))),
            1 => observed_primaries.into_iter().next().ok_or_else(|| {
                HarnessError::message("authoritative primary set disappeared unexpectedly")
            }),
            _ => Err(HarnessError::message(format!(
                "cluster reports conflicting authoritative primaries; observations={}",
                format_observed_authorities(states.as_slice()),
            ))),
        }?;
        let primary_state = self.require_observed_member_state(primary)?;
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

    pub fn require_observed_member_state(&self, member: ClusterMember) -> Result<&NodeState> {
        self.0
            .get(&member)
            .ok_or_else(|| {
                HarnessError::message(format!(
                    "cluster observation did not include member `{member}`"
                ))
            })?
            .as_ref()
            .map_err(|failure| {
                HarnessError::message(format!(
                    "expected `pgtm status --json` via `{member}` to succeed, but it failed: {failure}"
                ))
            })
    }

    pub fn replica_members(&self, primary: ClusterMember) -> Result<Vec<ClusterMember>> {
        self.require_observed_member_state(primary).map(|status| {
            status
                .dcs
                .members()
                .filter(|(_member_id, member)| {
                    matches!(
                        member.postgres(),
                        PgInfoState::Replica { common, .. }
                            if common.readiness == Readiness::Ready
                    )
                })
                .filter_map(|(member_id, _member)| ClusterMember::parse(member_id.0.as_str()).ok())
                .collect::<Vec<_>>()
        })
    }
}

impl FromIterator<(ClusterMember, ObservedStateResult)> for ClusterStateObservation {
    fn from_iter<T: IntoIterator<Item = (ClusterMember, ObservedStateResult)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[derive(Clone, Debug)]
pub struct PgtmObserver {
    docker: DockerCli,
    compose_file: PathBuf,
    compose_project: String,
    materialized_dir: PathBuf,
}

impl PgtmObserver {
    pub fn new(
        docker: DockerCli,
        compose_file: PathBuf,
        compose_project: String,
        materialized_dir: PathBuf,
    ) -> Self {
        Self {
            docker,
            compose_file,
            compose_project,
            materialized_dir,
        }
    }

    pub fn observe_states(&self) -> Result<ClusterStateObservation> {
        ClusterMember::ALL
            .into_iter()
            .map(|member| Ok((member, self.observe_state_via_member(member))))
            .collect()
    }

    pub fn state_via_member(&self, member: ClusterMember) -> Result<NodeState> {
        self.observe_state_via_member(member).map_err(|message| {
            HarnessError::message(format!("pgtm status via `{member}` failed: {message}"))
        })
    }

    pub fn postgres_routing_target(&self, member: ClusterMember) -> Result<PostgresRoutingTarget> {
        let published_port = self.member_published_port(member, "5432/tcp")?;
        let ca_cert_path = self.materialized_dir.join("configs/tls/ca.crt");
        let observer_cert_path = self.materialized_dir.join("configs/tls/observer.crt");
        let observer_key_path = self.materialized_dir.join("configs/tls/observer.key");
        Ok(PostgresRoutingTarget {
            member,
            conninfo: host_postgres_conninfo(
                member,
                published_port,
                ca_cert_path.as_path(),
                observer_cert_path.as_path(),
                observer_key_path.as_path(),
            )?,
        })
    }

    pub fn switchover_request_via_member(
        &self,
        member: ClusterMember,
        target: Option<ClusterMember>,
    ) -> Result<String> {
        let runtime_config = self.materialize_host_observer_config(member)?;
        let request_args = target
            .into_iter()
            .flat_map(|target_member| {
                [
                    "--switchover-to".to_string(),
                    target_member.service_name().to_string(),
                ]
            })
            .collect::<Vec<_>>();
        let output = self.run_command_via_member::<AcceptedResponse>(
            member,
            runtime_config.as_path(),
            ["switchover".to_string(), "request".to_string()]
                .into_iter()
                .chain(request_args)
                .collect::<Vec<_>>(),
            "pgtm switchover request",
            "switchover response",
        )?;
        match output {
            Ok(output) => serde_json::to_string(&output).map_err(|source| {
                HarnessError::message(format!("serializing switchover response failed: {source}"))
            }),
            Err(message) => Err(HarnessError::message(format!(
                "pgtm switchover request via `{member}` failed: {message}"
            ))),
        }
    }

    fn observe_state_via_member(
        &self,
        member: ClusterMember,
    ) -> std::result::Result<NodeState, String> {
        let runtime_config = self
            .materialize_host_observer_config(member)
            .map_err(|err| err.to_string())?;
        self.run_command_via_member::<NodeState>(
            member,
            runtime_config.as_path(),
            vec!["status".to_string()],
            "pgtm status",
            "node state",
        )
        .map_err(|err| err.to_string())?
    }

    fn run_command_via_member<T>(
        &self,
        member: ClusterMember,
        runtime_config: &Path,
        command_args: Vec<String>,
        context_label: &str,
        payload_label: &str,
    ) -> Result<std::result::Result<T, String>>
    where
        T: DeserializeOwned,
    {
        let env_candidate = std::env::var_os("CARGO_BIN_EXE_pgtm")
            .map(PathBuf::from)
            .filter(|path| path.exists());
        let executable = match env_candidate {
            Some(path) => path,
            None => harness_settings()?.pgtm_executable().to_path_buf(),
        };
        process::ensure_absolute_executable(executable.as_path())?;
        let args = [
            "--config".to_string(),
            runtime_config.display().to_string(),
            "--json".to_string(),
        ]
        .into_iter()
        .chain(command_args)
        .collect::<Vec<_>>();
        let context = format!("{context_label} via `{member}`");
        let output = process::run(
            executable.as_path(),
            None,
            context.clone(),
            args.as_slice(),
            [("PATH", "")],
        );
        match output {
            Ok(stdout) => {
                let parsed = serde_json::from_str::<T>(stdout.as_str()).map_err(|source| {
                    HarnessError::Json {
                        context: context.clone(),
                        source,
                    }
                })?;
                Ok(Ok(parsed))
            }
            Err(HarnessError::CommandFailed {
                executable,
                context,
                status,
                stdout,
                stderr,
            }) => Ok(Err(format!(
                "command `{}` failed while {context}: status={status}\nexpected payload: {payload_label}\nstdout:\n{stdout}\nstderr:\n{stderr}",
                executable.display(),
            ))),
            Err(err) => Err(err),
        }
    }

    fn member_published_port(&self, member: ClusterMember, port: &str) -> Result<u16> {
        let container_id = self.docker.compose_container_id(
            self.compose_file.as_path(),
            self.compose_project.as_str(),
            member.service_name(),
        )?;
        self.docker.published_host_port(container_id.as_str(), port)
    }

    fn materialize_host_observer_config(&self, member: ClusterMember) -> Result<PathBuf> {
        let published_api_port = self.member_published_port(member, "8443/tcp")?;
        let config_path = self
            .materialized_dir
            .join("configs/observer")
            .join(format!("{}-pgtm.toml", member.service_name()));
        let ca_cert_path = self.materialized_dir.join("configs/tls/ca.crt");
        let read_token_path = self.materialized_dir.join("secrets/api-read-token");
        let admin_token_path = self.materialized_dir.join("secrets/api-admin-token");
        let observer_cert_path = self.materialized_dir.join("configs/tls/observer.crt");
        let observer_key_path = self.materialized_dir.join("configs/tls/observer.key");
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|source| HarnessError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let config = build_host_observer_config(
            member,
            SocketAddr::from(([127, 0, 0, 1], published_api_port)),
            ca_cert_path.as_path(),
            read_token_path.as_path(),
            admin_token_path.as_path(),
            observer_cert_path.as_path(),
            observer_key_path.as_path(),
        )?;
        pgtuskmaster_test_support::config_v2::load_operator_config_contents(config.as_str())
            .map(|_| ())
            .map_err(|source| {
                HarnessError::message(format!(
                    "rendered observer config for `{member}` failed validation: {source}"
                ))
            })?;
        fs::write(config_path.as_path(), config).map_err(|source| HarnessError::Io {
            path: config_path.clone(),
            source,
        })?;
        Ok(config_path)
    }
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

fn format_observed_authorities(states: &[(ClusterMember, &NodeState)]) -> String {
    states
        .iter()
        .map(|(member, state)| format!("{member}={}", format_authority(state)))
        .collect::<Vec<_>>()
        .join("; ")
}

fn format_warnings(status: &NodeState) -> String {
    let mut warnings = Vec::new();
    if !status.dcs.is_quorum() {
        warnings.push("dcs_mode=not_trusted".to_string());
    }
    if authoritative_primary_member(status).is_none() {
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
    authoritative_primary_member(status)
        .and_then(|member_id| ClusterMember::parse(member_id.as_str()).ok())
}

fn host_postgres_conninfo(
    member: ClusterMember,
    port: u16,
    ca_cert_path: &Path,
    observer_cert_path: &Path,
    observer_key_path: &Path,
) -> Result<PgConnInfo> {
    Ok(PgConnInfo {
        route: pgtuskmaster_rust::state::PgRoute::tcp_hostaddr(
            member.service_name().to_string(),
            port,
            Some(Ipv4Addr::LOCALHOST.into()),
        )
        .map_err(HarnessError::message)?,
        user: "postgres".to_string(),
        dbname: "postgres".to_string(),
        application_name: None,
        connect_timeout_s: None,
        options: None,
        tls: PgClientTls {
            mode: PgSslMode::VerifyFull,
            root_cert: Some(ca_cert_path.to_path_buf()),
            client_cert: Some(observer_cert_path.to_path_buf()),
            client_key: Some(observer_key_path.to_path_buf()),
        },
    })
}

fn build_host_observer_config(
    member: ClusterMember,
    resolve_to: SocketAddr,
    ca_cert_path: &Path,
    read_token_path: &Path,
    admin_token_path: &Path,
    observer_cert_path: &Path,
    observer_key_path: &Path,
) -> Result<String> {
    Ok(format!(
        r#"[api]
base_url = {base_url}
expected_transport = "https"
resolve_to = {resolve_to}

[api.auth]
type = "role_tokens"
read_token = {read_token_path}
admin_token = {admin_token_path}

[api.tls]
ca_cert = {ca_cert_path}
identity = {{ cert = {observer_cert_path}, key = {observer_key_path} }}

[postgres.tls]
ca_cert = {ca_cert_path}
identity = {{ cert = {observer_cert_path}, key = {observer_key_path} }}
"#,
        base_url = toml_string(
            format!("https://{}:{}", member.service_name(), resolve_to.port()).as_str()
        ),
        resolve_to = toml_string(resolve_to.to_string().as_str()),
        read_token_path = toml_path_source(read_token_path),
        admin_token_path = toml_path_source(admin_token_path),
        ca_cert_path = toml_path_source(ca_cert_path),
        observer_cert_path = toml_path_source(observer_cert_path),
        observer_key_path = toml_path_source(observer_key_path),
    ))
}

fn toml_string(value: &str) -> String {
    toml::Value::String(value.to_string()).to_string()
}

fn toml_path_source(path: &Path) -> String {
    format!(
        "{{ path = {} }}",
        toml_string(path.display().to_string().as_str())
    )
}

#[cfg(test)]
mod tests {
    use super::{build_host_observer_config, host_postgres_conninfo, ClusterStateObservation};
    use crate::support::topology::ClusterMember;
    use pgtuskmaster_rust::{
        dcs::{DcsMemberState, DcsSnapshot},
        ha::{
            state::HaState,
            types::{
                AuthorityProjection, CandidateState, HaDecision, HaMode, HaObservation,
                LocalDataState, PublicationState,
            },
        },
        pginfo::state::{PgConfig, PgInfoCommon, PgInfoState, Readiness, SqlStatus, UpstreamInfo},
        process::state::ProcessState,
        state::{
            ClusterName, LeaseEpoch, MemberId, NodeIdentity, PgRoute, ScopeName, SwitchoverState,
            TimelineId, UnixMillis, WalLsn, WorkerStatus,
        },
    };
    use std::{collections::BTreeMap, fs, net::SocketAddr, path::Path};

    use crate::support::{error::HarnessError, files::with_temporary_directory};

    #[test]
    fn host_postgres_conninfo_renders_tls_paths() -> Result<(), String> {
        let dsn = host_postgres_conninfo(
            ClusterMember::NodeA,
            5432,
            Path::new("/tmp/ca bundle.pem"),
            Path::new("/tmp/observer cert.pem"),
            Path::new("/tmp/observer key.pem"),
        )
        .map_err(|err| err.to_string())?
        .to_string();

        assert!(dsn.contains("sslrootcert='/tmp/ca bundle.pem'"));
        assert!(dsn.contains("sslcert='/tmp/observer cert.pem'"));
        assert!(dsn.contains("sslkey='/tmp/observer key.pem'"));
        assert!(dsn.contains("hostaddr=127.0.0.1"));
        Ok(())
    }

    #[test]
    fn host_observer_config_round_trips_through_toml() -> Result<(), String> {
        with_temporary_directory("pgtm-observer", "round-trip", |dir| {
            let [ca_path, read_token_path, admin_token_path, observer_cert_path, observer_key_path] =
                [
                    "ca bundle.pem",
                    "read token",
                    "admin token",
                    "observer cert.pem",
                    "observer key.pem",
                ]
                .map(|name| dir.join(name));
            for path in [
                &ca_path,
                &read_token_path,
                &admin_token_path,
                &observer_cert_path,
                &observer_key_path,
            ] {
                fs::write(path, "placeholder").map_err(|source| HarnessError::Io {
                    path: path.to_path_buf(),
                    source,
                })?;
            }
            let rendered = build_host_observer_config(
                ClusterMember::NodeB,
                SocketAddr::from(([127, 0, 0, 1], 18443)),
                ca_path.as_path(),
                read_token_path.as_path(),
                admin_token_path.as_path(),
                observer_cert_path.as_path(),
                observer_key_path.as_path(),
            )?;
            pgtuskmaster_test_support::config_v2::load_operator_config_contents(rendered.as_str())
                .map(|_| ())
                .map_err(|source| {
                    HarnessError::message(format!("observer config should parse as TOML: {source}"))
                })?;
            assert!(rendered.contains(r#"base_url = "https://node-b:18443""#));
            assert!(rendered.contains(r#"resolve_to = "127.0.0.1:18443""#));
            assert!(rendered.contains(r#"type = "role_tokens""#));
            assert!(rendered.contains(read_token_path.display().to_string().as_str()));
            assert!(rendered.contains(observer_key_path.display().to_string().as_str()));
            Ok(())
        })
        .map_err(|err| err.to_string())
    }

    #[test]
    fn cluster_state_observation_owns_primary_and_replica_analysis() -> Result<(), String> {
        let observation: ClusterStateObservation = [(
            ClusterMember::NodeA,
            sample_primary_state(
                ClusterMember::NodeA,
                &[
                    (ClusterMember::NodeB, Readiness::Ready),
                    (ClusterMember::NodeC, Readiness::NotReady),
                ],
            )?,
        )]
        .into_iter()
        .map(|(member, state)| (member, Ok(state)))
        .collect();

        let primary = observation
            .compatible_primary(&[ClusterMember::NodeA], 1, None)
            .map_err(|err| err.to_string())?;
        let replicas = observation
            .replica_members(primary)
            .map_err(|err| err.to_string())?;
        assert_eq!(primary, ClusterMember::NodeA);
        assert_eq!(replicas, vec![ClusterMember::NodeB]);
        Ok(())
    }

    fn sample_primary_state(
        primary: ClusterMember,
        replicas: &[(ClusterMember, Readiness)],
    ) -> Result<pgtuskmaster_rust::api::NodeState, String> {
        let primary_member_id = member_id(primary);
        let idle = ProcessState::Idle {
            worker: WorkerStatus::Running,
            last_outcome: None,
        };
        let primary_pg = primary_pg_info();
        let dcs_members = std::iter::once(Ok((
            primary_member_id.clone(),
            DcsMemberState {
                cluster_postgres: PgRoute::tcp(primary.service_name().to_string(), 5432)?,
                operator_postgres: None,
                operator_api: None,
                postgres: primary_pg.clone(),
            },
        )))
        .chain(replicas.iter().map(|(member, readiness)| {
            Ok((
                member_id(*member),
                DcsMemberState {
                    cluster_postgres: PgRoute::tcp(member.service_name().to_string(), 5432)?,
                    operator_postgres: None,
                    operator_api: None,
                    postgres: replica_pg_info(readiness.clone()),
                },
            ))
        }))
        .collect::<Result<BTreeMap<_, _>, String>>()?;
        Ok(pgtuskmaster_rust::api::NodeState {
            identity: NodeIdentity {
                cluster_name: ClusterName("cluster-a".to_string()),
                scope: ScopeName("scope-a".to_string()),
                member_id: primary_member_id.clone(),
            },
            pg: primary_pg.clone(),
            process: idle.clone(),
            dcs: DcsSnapshot::quorum(None, SwitchoverState::None, dcs_members),
            ha: HaState {
                worker: WorkerStatus::Running,
                tick: 0,
                managed_roles_reconciled: true,
                publication: PublicationState::Projected(AuthorityProjection::Primary(
                    LeaseEpoch {
                        holder: primary_member_id.clone(),
                        generation: 1,
                    },
                )),
                decision: HaDecision {
                    mode: HaMode::WaitForLeader,
                    publication: None,
                    clear_switchover: false,
                },
                observation: HaObservation {
                    pg: primary_pg,
                    process: idle,
                    dcs: DcsSnapshot::starting(),
                    publication: PublicationState::Unknown,
                    managed_roles_reconciled: false,
                    local_data: LocalDataState::Missing,
                    resolved_upstream: None,
                    self_candidate: CandidateState::Ineligible,
                    storage_stalled: false,
                    ready_primary: None,
                },
                clear_switchover: false,
                steps: Vec::new(),
            },
        })
    }

    fn member_id(member: ClusterMember) -> MemberId {
        MemberId(member.service_name().to_string())
    }

    fn primary_pg_info() -> PgInfoState {
        PgInfoState::Primary {
            common: common(Readiness::Ready),
            wal_lsn: WalLsn(42),
            slots: Vec::new(),
        }
    }

    fn replica_pg_info(readiness: Readiness) -> PgInfoState {
        PgInfoState::Replica {
            common: common(readiness),
            replay_lsn: WalLsn(41),
            follow_lsn: Some(WalLsn(42)),
            upstream: Some(UpstreamInfo {
                member_id: MemberId("node-a".to_string()),
            }),
        }
    }

    fn common(readiness: Readiness) -> PgInfoCommon {
        PgInfoCommon {
            worker: WorkerStatus::Running,
            sql: SqlStatus::Healthy,
            readiness,
            timeline: Some(TimelineId(1)),
            system_identifier: None,
            pg_config: PgConfig {
                port: Some(5432),
                hot_standby: Some(true),
                primary_conninfo: None,
                primary_slot_name: None,
                extra: BTreeMap::new(),
            },
            last_refresh_at: Some(UnixMillis(1)),
        }
    }
}
