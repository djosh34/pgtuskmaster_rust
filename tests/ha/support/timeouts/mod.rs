use std::{path::Path, time::Duration};

use pgtuskmaster_rust::config::RuntimeConfig;

use crate::support::error::{HarnessError, Result};

const FAILOVER_SLACK_LOOPS: u64 = 3;
const DCS_DETECTION_SLACK_LOOPS: u64 = 1;
const RECOVERY_SLACK_LOOPS: u64 = 10;

#[derive(Clone, Debug)]
pub struct TimeoutModel {
    pub startup_deadline: Duration,
    pub failover_deadline: Duration,
    pub recovery_deadline: Duration,
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
        let poll_interval = Duration::from_millis(config.ha.loop_interval_ms);
        let failover_slack =
            poll_interval.mul_f64((FAILOVER_SLACK_LOOPS + DCS_DETECTION_SLACK_LOOPS) as f64);
        let recovery_slack = poll_interval.mul_f64(RECOVERY_SLACK_LOOPS as f64);
        let failover_deadline = Duration::from_millis(config.ha.lease_ttl_ms) + failover_slack;
        let startup_deadline =
            Duration::from_millis(config.process.timeouts.bootstrap_ms) + recovery_slack;
        let recovery_base = config
            .process
            .timeouts
            .bootstrap_ms
            .max(config.process.timeouts.pg_rewind_ms);
        let recovery_deadline = Duration::from_millis(recovery_base) + recovery_slack;
        Ok(Self {
            startup_deadline,
            failover_deadline,
            recovery_deadline,
            poll_interval,
        })
    }
}
