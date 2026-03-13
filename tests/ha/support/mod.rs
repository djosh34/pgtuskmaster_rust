mod config;
pub mod docker;
mod error;
pub mod faults;
pub mod givens;
pub mod observer;
mod process;
pub mod runner;
pub mod steps;
pub mod timeouts;
pub mod topology;
pub mod workload;
pub mod world;

use std::sync::{Mutex, OnceLock};

use cucumber::{writer, World as _, WriterExt as _};
use futures::FutureExt as _;

use crate::support::{
    error::{HarnessError, Result},
    world::HaWorld,
};

#[derive(Clone, Debug)]
pub struct FeatureMetadata {
    pub feature_name: String,
}

static FEATURE_METADATA: OnceLock<FeatureMetadata> = OnceLock::new();
static CLEANUP_ERRORS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

// This runner is intentionally independent from the legacy HA harness so the old
// `tests/ha` and `src/test_harness/ha_e2e` flows can be deleted later.
pub async fn run_feature(
    feature_name: &str,
    feature_path: &str,
) -> std::result::Result<(), String> {
    install_context(feature_name, feature_path).map_err(|err| err.to_string())?;

    let writer = HaWorld::cucumber()
        .before(|_, _, _, world| {
            async move {
                world.reset();
            }
            .boxed_local()
        })
        .after(|_, _, _, _, world| {
            async move {
                if let Some(world) = world {
                    if let Err(err) = world.cleanup() {
                        record_cleanup_error(err.to_string());
                    }
                }
            }
            .boxed_local()
        })
        .with_writer(writer::Basic::stdout().summarized())
        .with_default_cli()
        .run(feature_path)
        .await;

    let stats_error = summarize_result(writer.scenarios_stats(), writer.steps_stats()).err();
    let cleanup_error = cleanup_recorded_errors().err();

    match (stats_error, cleanup_error) {
        (None, None) => Ok(()),
        (Some(stats), None) => Err(stats.to_string()),
        (None, Some(cleanup)) => Err(cleanup.to_string()),
        (Some(stats), Some(cleanup)) => Err(format!("{stats}\ncleanup also failed: {cleanup}")),
    }
}

pub fn feature_metadata() -> Result<&'static FeatureMetadata> {
    FEATURE_METADATA
        .get()
        .ok_or_else(|| HarnessError::message("feature metadata has not been initialized"))
}

fn install_context(feature_name: &str, _feature_path: &str) -> Result<()> {
    FEATURE_METADATA
        .set(FeatureMetadata {
            feature_name: feature_name.to_string(),
        })
        .map_err(|_| HarnessError::message("feature metadata was already initialized"))?;
    Ok(())
}

fn summarize_result(
    scenario_stats: &cucumber::writer::summarize::Stats,
    step_stats: &cucumber::writer::summarize::Stats,
) -> Result<()> {
    if scenario_stats.total() == 0 {
        return Err(HarnessError::message("cucumber executed zero scenarios"));
    }
    if scenario_stats.failed > 0 || step_stats.failed > 0 {
        return Err(HarnessError::message(format!(
            "cucumber feature failed: scenarios_failed={} steps_failed={}",
            scenario_stats.failed, step_stats.failed
        )));
    }
    if scenario_stats.skipped > 0 || step_stats.skipped > 0 {
        return Err(HarnessError::message(format!(
            "cucumber feature skipped steps unexpectedly: scenarios_skipped={} steps_skipped={}",
            scenario_stats.skipped, step_stats.skipped
        )));
    }
    Ok(())
}

fn cleanup_recorded_errors() -> Result<()> {
    let recorded = CLEANUP_ERRORS.get_or_init(|| Mutex::new(Vec::new()));
    let errors = {
        let mut guard = recorded
            .lock()
            .map_err(|_| HarnessError::message("cleanup error registry mutex was poisoned"))?;
        std::mem::take(&mut *guard)
    };

    if errors.is_empty() {
        Ok(())
    } else {
        Err(HarnessError::message(errors.join("\n")))
    }
}

fn record_cleanup_error(error: String) {
    let recorded = CLEANUP_ERRORS.get_or_init(|| Mutex::new(Vec::new()));
    match recorded.lock() {
        Ok(mut guard) => guard.push(error),
        Err(poisoned) => poisoned.into_inner().push(error),
    }
}
