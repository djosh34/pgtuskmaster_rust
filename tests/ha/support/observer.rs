use pgtuskmaster_rust::{api::HaPhaseResponse, api::HaStateResponse, state::WorkerError};

#[derive(Clone, Default, serde::Serialize)]
pub struct HaObservationStats {
    pub sample_count: u64,
    pub api_error_count: u64,
    pub max_concurrent_primaries: usize,
    pub leader_change_count: u64,
    pub failsafe_sample_count: u64,
    pub recent_samples: Vec<String>,
}

#[derive(Clone, Copy)]
pub struct HaObserverConfig {
    pub min_successful_samples: u64,
    pub ring_capacity: usize,
}

pub struct HaInvariantObserver {
    config: HaObserverConfig,
    stats: HaObservationStats,
    poll_attempts: u64,
    poll_errors: u64,
    last_poll_error: Option<String>,
    last_leader_signature: Option<String>,
}

impl HaInvariantObserver {
    pub fn new(config: HaObserverConfig) -> Self {
        Self {
            config,
            stats: HaObservationStats::default(),
            poll_attempts: 0,
            poll_errors: 0,
            last_poll_error: None,
            last_leader_signature: None,
        }
    }

    pub fn record_poll_attempt(&mut self) {
        self.poll_attempts = self.poll_attempts.saturating_add(1);
    }

    pub fn record_api_states(
        &mut self,
        states: &[HaStateResponse],
        errors: &[String],
    ) -> Result<(), WorkerError> {
        self.stats.api_error_count = self
            .stats
            .api_error_count
            .saturating_add(len_to_u64(errors.len()));

        if states.is_empty() {
            if !errors.is_empty() {
                self.push_recent(format!("api_blind_spot: {}", errors.join(" | ")));
            }
            return Ok(());
        }

        self.stats.sample_count = self.stats.sample_count.saturating_add(1);
        let primary_count = states
            .iter()
            .filter(|state| state.ha_phase == HaPhaseResponse::Primary)
            .count();
        self.stats.max_concurrent_primaries =
            self.stats.max_concurrent_primaries.max(primary_count);

        let mut leaders = states
            .iter()
            .filter_map(|state| state.leader.clone())
            .collect::<Vec<_>>();
        leaders.sort();
        leaders.dedup();
        let leader_signature = leaders.join("|");
        if self
            .last_leader_signature
            .as_deref()
            .map(|prior| prior != leader_signature.as_str())
            .unwrap_or(false)
        {
            self.stats.leader_change_count = self.stats.leader_change_count.saturating_add(1);
        }
        self.last_leader_signature = Some(leader_signature);

        if states
            .iter()
            .all(|state| state.ha_phase == HaPhaseResponse::FailSafe)
        {
            self.stats.failsafe_sample_count = self.stats.failsafe_sample_count.saturating_add(1);
        }

        let mut fragments = states
            .iter()
            .map(|state| {
                let leader = state.leader.as_deref().unwrap_or("none");
                format!(
                    "{}:{}:leader={leader}",
                    state.self_member_id, state.ha_phase
                )
            })
            .collect::<Vec<_>>();
        fragments.extend(errors.iter().map(|error| format!("api_error={error}")));
        self.push_recent(fragments.join(", "));

        if primary_count > 1 {
            return Err(WorkerError::Message(format!(
                "split-brain detected: more than one primary; observations={} errors={}",
                states
                    .iter()
                    .map(|state| format!("{}:{}", state.self_member_id, state.ha_phase))
                    .collect::<Vec<_>>()
                    .join(","),
                summarize_errors(errors)
            )));
        }

        Ok(())
    }

    pub fn record_sql_roles(
        &mut self,
        roles: &[(String, String)],
        errors: &[String],
    ) -> Result<(), WorkerError> {
        if roles.is_empty() {
            if !errors.is_empty() {
                self.push_recent(format!("sql_blind_spot: {}", errors.join(" | ")));
            }
            return Ok(());
        }

        self.stats.sample_count = self.stats.sample_count.saturating_add(1);
        let primary_count = roles.iter().filter(|(_, role)| role == "primary").count();
        self.stats.max_concurrent_primaries =
            self.stats.max_concurrent_primaries.max(primary_count);

        let mut fragments = roles
            .iter()
            .map(|(node_id, role)| format!("{node_id}:{role}"))
            .collect::<Vec<_>>();
        fragments.extend(errors.iter().map(|error| format!("sql_error={error}")));
        self.push_recent(format!("sql_roles=[{}]", fragments.join(", ")));

        if primary_count > 1 {
            return Err(WorkerError::Message(format!(
                "split-brain detected via SQL roles: observations={} errors={}",
                roles
                    .iter()
                    .map(|(node_id, role)| format!("{node_id}:{role}"))
                    .collect::<Vec<_>>()
                    .join(","),
                summarize_errors(errors)
            )));
        }

        Ok(())
    }

    pub fn record_observation_gap(&mut self, api_errors: &[String], sql_errors: &[String]) {
        self.poll_errors = self.poll_errors.saturating_add(1);
        let message = format!(
            "api_errors={}; sql_errors={}",
            summarize_errors(api_errors),
            summarize_errors(sql_errors)
        );
        self.last_poll_error = Some(message.clone());
        self.push_recent(format!("observation_gap:{message}"));
    }

    pub fn record_transport_error(&mut self, error: impl Into<String>) {
        self.poll_errors = self.poll_errors.saturating_add(1);
        let message = error.into();
        self.last_poll_error = Some(message.clone());
        self.push_recent(format!("transport_error:{message}"));
    }

    pub fn stats(&self) -> &HaObservationStats {
        &self.stats
    }

    pub fn into_stats(self) -> HaObservationStats {
        self.stats
    }

    pub fn finalize_no_dual_primary_window(&self) -> Result<(), WorkerError> {
        if self.stats.sample_count < self.config.min_successful_samples {
            let detail = self.last_poll_error.as_deref().unwrap_or("none");
            return Err(WorkerError::Message(format!(
                "insufficient evidence for split-brain window assertion: successful_samples={} min_successful_samples={} poll_attempts={} poll_errors={} last_poll_error={} recent_samples={}",
                self.stats.sample_count,
                self.config.min_successful_samples,
                self.poll_attempts,
                self.poll_errors,
                detail,
                summarize_recent_samples(&self.stats.recent_samples),
            )));
        }

        assert_no_dual_primary_in_samples(self.stats(), self.config.min_successful_samples)
    }

    fn push_recent(&mut self, sample: String) {
        if self.config.ring_capacity == 0 {
            return;
        }
        if self.stats.recent_samples.len() >= self.config.ring_capacity {
            let _ = self.stats.recent_samples.remove(0);
        }
        self.stats.recent_samples.push(sample);
    }
}

pub fn assert_no_dual_primary_in_samples(
    stats: &HaObservationStats,
    min_successful_samples: u64,
) -> Result<(), WorkerError> {
    if stats.sample_count < min_successful_samples {
        return Err(WorkerError::Message(format!(
            "insufficient HA sample evidence: sample_count={} min_successful_samples={} api_error_count={} recent_samples={}",
            stats.sample_count,
            min_successful_samples,
            stats.api_error_count,
            summarize_recent_samples(&stats.recent_samples),
        )));
    }
    if stats.max_concurrent_primaries > 1 {
        return Err(WorkerError::Message(format!(
            "dual primary observed during sampled window; max_concurrent_primaries={} recent_samples={}",
            stats.max_concurrent_primaries,
            summarize_recent_samples(&stats.recent_samples),
        )));
    }
    Ok(())
}

fn len_to_u64(value: usize) -> u64 {
    u64::try_from(value)
        .ok()
        .map_or(u64::MAX, core::convert::identity)
}

fn summarize_errors(errors: &[String]) -> String {
    if errors.is_empty() {
        "none".to_string()
    } else {
        errors.join(" | ")
    }
}

fn summarize_recent_samples(samples: &[String]) -> String {
    if samples.is_empty() {
        "none".to_string()
    } else {
        samples.join(" || ")
    }
}

#[cfg(test)]
mod unit_tests {
    use super::{
        assert_no_dual_primary_in_samples, HaInvariantObserver, HaObservationStats,
        HaObserverConfig,
    };
    use pgtuskmaster_rust::api::{
        DcsTrustResponse, HaDecisionResponse, HaPhaseResponse, HaStateResponse, MemberRoleResponse,
        ReadinessResponse, SqlStatusResponse,
    };
    use std::collections::BTreeMap;

    type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn ha_state(member_id: &str, phase: HaPhaseResponse, leader: Option<&str>) -> HaStateResponse {
        HaStateResponse {
            cluster_name: "cluster-e2e".to_string(),
            scope: "scope-ha-e2e".to_string(),
            self_member_id: member_id.to_string(),
            leader: leader.map(ToString::to_string),
            switchover_pending: false,
            switchover_to: None,
            member_count: 3,
            members: [
                (member_id.to_string(), phase.clone()),
                (
                    "node-b".to_string(),
                    if member_id == "node-b" {
                        phase.clone()
                    } else {
                        HaPhaseResponse::Replica
                    },
                ),
                (
                    "node-c".to_string(),
                    if member_id == "node-c" {
                        phase.clone()
                    } else {
                        HaPhaseResponse::Replica
                    },
                ),
            ]
            .into_iter()
            .collect::<BTreeMap<_, _>>()
            .into_iter()
            .map(|(sample_member_id, sample_phase)| {
                pgtuskmaster_rust::api::HaClusterMemberResponse {
                    member_id: sample_member_id,
                    postgres_host: "127.0.0.1".to_string(),
                    postgres_port: 5432,
                    api_url: Some("http://127.0.0.1:8080".to_string()),
                    role: match sample_phase {
                        HaPhaseResponse::Primary => MemberRoleResponse::Primary,
                        HaPhaseResponse::Replica => MemberRoleResponse::Replica,
                        _ => MemberRoleResponse::Unknown,
                    },
                    sql: SqlStatusResponse::Healthy,
                    readiness: ReadinessResponse::Ready,
                    timeline: Some(7),
                    write_lsn: None,
                    replay_lsn: Some(5),
                    updated_at_ms: 1,
                    pg_version: 1,
                }
            })
            .collect(),
            dcs_trust: DcsTrustResponse::FreshQuorum,
            ha_phase: phase,
            ha_tick: 1,
            ha_decision: HaDecisionResponse::NoChange,
            snapshot_sequence: 1,
        }
    }

    #[test]
    fn zero_sample_finalization_fails_closed() -> TestResult {
        let observer = HaInvariantObserver::new(HaObserverConfig {
            min_successful_samples: 1,
            ring_capacity: 4,
        });
        let result = observer.finalize_no_dual_primary_window();
        if result.is_ok() {
            return Err(Box::new(std::io::Error::other(
                "expected finalization to fail with zero samples",
            )));
        }
        Ok(())
    }

    #[test]
    fn insufficient_sample_threshold_fails() -> TestResult {
        let mut observer = HaInvariantObserver::new(HaObserverConfig {
            min_successful_samples: 2,
            ring_capacity: 4,
        });
        observer.record_poll_attempt();
        observer.record_api_states(
            &[ha_state("node-1", HaPhaseResponse::Primary, Some("node-1"))],
            &[],
        )?;
        let result = observer.finalize_no_dual_primary_window();
        if result.is_ok() {
            return Err(Box::new(std::io::Error::other(
                "expected finalization to fail when the minimum sample threshold is not met",
            )));
        }
        Ok(())
    }

    #[test]
    fn successful_finalization_with_enough_samples() -> TestResult {
        let mut observer = HaInvariantObserver::new(HaObserverConfig {
            min_successful_samples: 2,
            ring_capacity: 4,
        });
        for _ in 0..2 {
            observer.record_poll_attempt();
            observer.record_api_states(
                &[ha_state("node-1", HaPhaseResponse::Primary, Some("node-1"))],
                &[],
            )?;
        }
        observer.finalize_no_dual_primary_window()?;
        Ok(())
    }

    #[test]
    fn dual_primary_sample_is_rejected() -> TestResult {
        let mut observer = HaInvariantObserver::new(HaObserverConfig {
            min_successful_samples: 1,
            ring_capacity: 4,
        });
        observer.record_poll_attempt();
        let result = observer.record_api_states(
            &[
                ha_state("node-1", HaPhaseResponse::Primary, Some("node-1")),
                ha_state("node-2", HaPhaseResponse::Primary, Some("node-2")),
            ],
            &[],
        );
        if result.is_ok() {
            return Err(Box::new(std::io::Error::other(
                "expected dual-primary sample to fail",
            )));
        }
        Ok(())
    }

    #[test]
    fn standalone_sample_assertion_rejects_dual_primary_stats() -> TestResult {
        let stats = HaObservationStats {
            sample_count: 3,
            api_error_count: 1,
            max_concurrent_primaries: 2,
            leader_change_count: 0,
            failsafe_sample_count: 0,
            recent_samples: vec!["node-1:Primary,node-2:Primary".to_string()],
        };
        let result = assert_no_dual_primary_in_samples(&stats, 1);
        if result.is_ok() {
            return Err(Box::new(std::io::Error::other(
                "expected dual-primary stats assertion to fail",
            )));
        }
        Ok(())
    }

    #[test]
    fn observer_transport_and_stats_helpers_are_reachable() {
        let mut observer = HaInvariantObserver::new(HaObserverConfig {
            min_successful_samples: 1,
            ring_capacity: 4,
        });
        observer.record_transport_error("synthetic");
        let stats = observer.into_stats();
        assert_eq!(stats.recent_samples.len(), 1);
    }
}
