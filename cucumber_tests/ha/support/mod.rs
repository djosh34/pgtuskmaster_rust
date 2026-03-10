mod error;
mod process;
pub mod docker;
pub mod givens;
pub mod observer;
pub mod runner;
pub mod steps;
pub mod timeouts;
pub mod world;

use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};

use cucumber::{World as _, WriterExt as _, writer};
use futures::FutureExt as _;

use crate::support::{
    error::{HarnessError, Result},
    world::{HarnessShared, HaWorld},
};

#[derive(Clone, Debug)]
pub struct FeatureMetadata {
    pub feature_name: String,
}

#[derive(Clone, Debug)]
pub struct BinaryPaths {
    pub pgtuskmaster: PathBuf,
    pub pgtm: PathBuf,
}

static FEATURE_METADATA: OnceLock<FeatureMetadata> = OnceLock::new();
static BINARY_PATHS: OnceLock<BinaryPaths> = OnceLock::new();
static ACTIVE_RUNS: OnceLock<Mutex<BTreeMap<String, Arc<Mutex<HarnessShared>>>>> =
    OnceLock::new();

// This runner is intentionally independent from the legacy HA harness so the old
// `tests/ha` and `src/test_harness/ha_e2e` flows can be deleted later.
pub async fn run_feature(feature_name: &str, feature_path: &str) -> std::result::Result<(), String> {
    install_context(feature_name, feature_path).map_err(|err| err.to_string())?;

    let writer = HaWorld::cucumber()
        .before(|_, _, _, world| {
            async move {
                world.reset();
            }
            .boxed_local()
        })
        .with_writer(writer::Basic::stdout().summarized())
        .with_default_cli()
        .run(feature_path)
        .await;

    let stats_error = summarize_result(writer.scenarios_stats(), writer.steps_stats()).err();
    let cleanup_error = cleanup_active_runs().err();

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

pub fn binary_paths() -> Result<&'static BinaryPaths> {
    BINARY_PATHS
        .get()
        .ok_or_else(|| HarnessError::message("binary paths have not been initialized"))
}

pub fn register_run(run_id: String, shared: Arc<Mutex<HarnessShared>>) -> Result<()> {
    let registry = ACTIVE_RUNS.get_or_init(|| Mutex::new(BTreeMap::new()));
    let mut guard = registry
        .lock()
        .map_err(|_| HarnessError::message("run registry mutex was poisoned"))?;
    guard.insert(run_id, shared);
    Ok(())
}

fn install_context(feature_name: &str, _feature_path: &str) -> Result<()> {
    FEATURE_METADATA
        .set(FeatureMetadata {
            feature_name: feature_name.to_string(),
        })
        .map_err(|_| HarnessError::message("feature metadata was already initialized"))?;
    BINARY_PATHS
        .set(BinaryPaths {
            pgtuskmaster: PathBuf::from(env!("CARGO_BIN_EXE_pgtuskmaster")),
            pgtm: PathBuf::from(env!("CARGO_BIN_EXE_pgtm")),
        })
        .map_err(|_| HarnessError::message("binary paths were already initialized"))?;
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

fn cleanup_active_runs() -> Result<()> {
    let registry = ACTIVE_RUNS.get_or_init(|| Mutex::new(BTreeMap::new()));
    let runs = {
        let mut guard = registry
            .lock()
            .map_err(|_| HarnessError::message("run registry mutex was poisoned"))?;
        std::mem::take(&mut *guard)
            .into_values()
            .collect::<Vec<_>>()
    };

    let errors = runs
        .into_iter()
        .filter_map(|shared: Arc<Mutex<HarnessShared>>| {
            let mut guard = match shared.lock() {
                Ok(guard) => guard,
                Err(_) => {
                    return Some("failed to lock harness state for cleanup".to_string());
                }
            };
            guard.cleanup().err().map(|err| err.to_string())
        })
        .collect::<Vec<_>>();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(HarnessError::message(errors.join("\n")))
    }
}
