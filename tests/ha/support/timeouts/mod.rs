use std::{path::Path, time::Duration};

use crate::support::error::{HarnessError, Result};

const FAILOVER_SLACK_LOOPS: u64 = 3;
const DCS_DETECTION_SLACK_LOOPS: u64 = 1;
const FAILOVER_EXTRA_BUFFER_MS: u64 = 4_000;
const RECOVERY_SLACK_LOOPS: u64 = 4;
const HARNESS_POLL_INTERVAL_MULTIPLIER: u64 = 1;
const MIN_HARNESS_POLL_INTERVAL_MS: u64 = 1_000;
const MAX_FAILOVER_DEADLINE_MS: u64 = 20_000;
const MAX_STARTUP_DEADLINE_MS: u64 = 20_000;
const MAX_RECOVERY_DEADLINE_MS: u64 = 30_000;
const WRITE_CONVERGENCE_EXTRA_BUFFER_MS: u64 = 20_000;
const MAX_WRITE_CONVERGENCE_DEADLINE_MS: u64 = 90_000;

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
        let (ha_loop_interval, ha_lease_ttl, bootstrap_timeout, pg_rewind_timeout) =
            pgtuskmaster_test_support::config_v2::load_runtime_timing_values(path).map_err(
                |err| {
                    HarnessError::message(format!(
                        "failed to derive timeout inputs from runtime config `{}`: {err}",
                        path.display()
                    ))
                },
            )?;
        Ok(derive_timeout_model(
            ha_loop_interval,
            ha_lease_ttl,
            bootstrap_timeout,
            pg_rewind_timeout,
        ))
    }
}

fn derive_timeout_model(
    ha_loop_interval: Duration,
    lease_ttl: Duration,
    bootstrap_timeout: Duration,
    pg_rewind_timeout: Duration,
) -> TimeoutModel {
    let failover_slack =
        ha_loop_interval.mul_f64((FAILOVER_SLACK_LOOPS + DCS_DETECTION_SLACK_LOOPS) as f64);
    let failover_buffer = Duration::from_millis(FAILOVER_EXTRA_BUFFER_MS);
    let recovery_slack = ha_loop_interval.mul_f64(RECOVERY_SLACK_LOOPS as f64);
    let failover_deadline = (lease_ttl + failover_slack + failover_buffer)
        .min(Duration::from_millis(MAX_FAILOVER_DEADLINE_MS));
    let startup_deadline =
        (bootstrap_timeout + recovery_slack).min(Duration::from_millis(MAX_STARTUP_DEADLINE_MS));
    let recovery_base = bootstrap_timeout.max(pg_rewind_timeout);
    let recovery_deadline =
        (recovery_base + recovery_slack).min(Duration::from_millis(MAX_RECOVERY_DEADLINE_MS));
    let write_convergence_deadline = (failover_deadline
        + recovery_deadline
        + Duration::from_millis(WRITE_CONVERGENCE_EXTRA_BUFFER_MS))
    .min(Duration::from_millis(MAX_WRITE_CONVERGENCE_DEADLINE_MS));
    let poll_interval = Duration::from_millis(
        duration_millis_u64(ha_loop_interval)
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

fn duration_millis_u64(value: Duration) -> u64 {
    u64::try_from(value.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::derive_timeout_model;

    #[test]
    fn doubles_harness_poll_interval_for_fast_ha_loops() {
        let model = derive_timeout_model(
            Duration::from_millis(1_000),
            Duration::from_millis(10_000),
            Duration::from_millis(300_000),
            Duration::from_millis(120_000),
        );
        assert_eq!(model.poll_interval, Duration::from_secs(1));
        assert_eq!(model.failover_deadline, Duration::from_secs(18));
        assert_eq!(model.startup_deadline, Duration::from_secs(20));
        assert_eq!(model.recovery_deadline, Duration::from_secs(30));
        assert_eq!(model.write_convergence_deadline, Duration::from_secs(68));
    }

    #[test]
    fn preserves_longer_harness_poll_intervals_above_the_minimum() {
        let model = derive_timeout_model(
            Duration::from_millis(3_000),
            Duration::from_millis(10_000),
            Duration::from_millis(300_000),
            Duration::from_millis(120_000),
        );
        assert_eq!(model.poll_interval, Duration::from_secs(3));
        assert_eq!(model.failover_deadline, Duration::from_secs(20));
        assert_eq!(model.startup_deadline, Duration::from_secs(20));
        assert_eq!(model.recovery_deadline, Duration::from_secs(30));
        assert_eq!(model.write_convergence_deadline, Duration::from_secs(70));
    }
}
