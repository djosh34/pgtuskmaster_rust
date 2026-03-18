use std::{path::Path, time::Duration};

use pgtuskmaster_rust::config::RuntimeConfig;

use crate::support::error::{HarnessError, Result};

const FAILOVER_SLACK_LOOPS: u64 = 3;
const DCS_DETECTION_SLACK_LOOPS: u64 = 1;
const FAILOVER_EXTRA_BUFFER_MS: u64 = 12_000;
const RECOVERY_SLACK_LOOPS: u64 = 10;
const HARNESS_POLL_INTERVAL_MULTIPLIER: u64 = 2;
const MIN_HARNESS_POLL_INTERVAL_MS: u64 = 2_000;

#[derive(Clone, Debug)]
pub struct TimeoutModel {
    pub startup_deadline: Duration,
    pub failover_deadline: Duration,
    pub recovery_deadline: Duration,
    pub write_convergence_deadline: Duration,
    pub poll_interval: Duration,
}

impl TimeoutModel {
    pub fn from_runtime_config(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path).map_err(|err| {
            HarnessError::message(format!(
                "failed to read runtime config `{}` for timeout derivation: {err}",
                path.display()
            ))
        })?;
        let config = toml::from_str::<RuntimeConfig>(contents.as_str()).map_err(|err| {
            HarnessError::message(format!(
                "failed to parse runtime config `{}` for timeout derivation: {err}",
                path.display()
            ))
        })?;
        Ok(derive_timeout_model(
            config.ha.loop_interval_ms,
            config.ha.lease_ttl_ms,
            config.process.timeouts.bootstrap_ms,
            config.process.timeouts.pg_rewind_ms,
        ))
    }
}

fn derive_timeout_model(
    ha_loop_interval_ms: u64,
    lease_ttl_ms: u64,
    bootstrap_ms: u64,
    pg_rewind_ms: u64,
) -> TimeoutModel {
    let ha_loop_interval = Duration::from_millis(ha_loop_interval_ms);
    let failover_slack =
        ha_loop_interval.mul_f64((FAILOVER_SLACK_LOOPS + DCS_DETECTION_SLACK_LOOPS) as f64);
    let failover_buffer = Duration::from_millis(FAILOVER_EXTRA_BUFFER_MS);
    let recovery_slack = ha_loop_interval.mul_f64(RECOVERY_SLACK_LOOPS as f64);
    let failover_deadline = Duration::from_millis(lease_ttl_ms) + failover_slack + failover_buffer;
    let startup_deadline = Duration::from_millis(bootstrap_ms) + recovery_slack;
    let recovery_base = bootstrap_ms.max(pg_rewind_ms);
    let recovery_deadline = Duration::from_millis(recovery_base) + recovery_slack;
    let write_convergence_deadline = failover_deadline + recovery_deadline;
    let poll_interval = Duration::from_millis(
        ha_loop_interval_ms
            .saturating_mul(HARNESS_POLL_INTERVAL_MULTIPLIER)
            .max(MIN_HARNESS_POLL_INTERVAL_MS),
    );
    TimeoutModel {
        startup_deadline,
        failover_deadline,
        recovery_deadline,
        write_convergence_deadline,
        poll_interval,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::derive_timeout_model;

    #[test]
    fn doubles_harness_poll_interval_for_fast_ha_loops() {
        let model = derive_timeout_model(1_000, 10_000, 300_000, 120_000);
        assert_eq!(model.poll_interval, Duration::from_secs(2));
        assert_eq!(model.failover_deadline, Duration::from_secs(26));
        assert_eq!(model.write_convergence_deadline, Duration::from_secs(336));
    }

    #[test]
    fn preserves_longer_harness_poll_intervals_above_the_minimum() {
        let model = derive_timeout_model(3_000, 10_000, 300_000, 120_000);
        assert_eq!(model.poll_interval, Duration::from_secs(6));
        assert_eq!(model.failover_deadline, Duration::from_secs(34));
        assert_eq!(model.write_convergence_deadline, Duration::from_secs(364));
    }
}
