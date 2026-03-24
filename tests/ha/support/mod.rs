mod config;
mod docker;
mod error;
mod faults;
mod files;
mod givens;
mod invariants;
mod observer;
mod process;
mod steps;
mod timeouts;
mod topology;
mod world;

use std::{
    future::Future,
    sync::{Mutex, OnceLock},
    thread,
};

use cucumber::{writer, World as _, WriterExt as _};
use futures::FutureExt as _;
use tokio::runtime::{Builder, Handle, RuntimeFlavor};

use crate::support::{
    error::{HarnessError, Result},
    world::HaWorld,
};

static FEATURE_NAME: OnceLock<String> = OnceLock::new();
static CLEANUP_ERRORS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

// This runner is intentionally independent from the legacy HA harness so the old
// `tests/ha` and `src/test_harness/ha_e2e` flows can be deleted later.
pub fn run_feature(
    feature_name: &str,
    feature_path: &str,
    scenario_name: Option<&str>,
) -> std::result::Result<(), String> {
    block_on_current_thread(
        run_feature_async(feature_name, feature_path, scenario_name),
        "failed to build tokio runtime",
    )
}

async fn run_feature_async(
    feature_name: &str,
    feature_path: &str,
    scenario_name: Option<&str>,
) -> std::result::Result<(), String> {
    install_context(feature_name).map_err(|err| err.to_string())?;

    let cucumber = HaWorld::cucumber()
        .before(|_, _, scenario, world| {
            async move {
                world.reset();
                world.set_scenario_name(scenario.name.clone());
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
        .max_concurrent_scenarios(1)
        .with_writer(writer::Basic::stdout().summarized())
        .with_default_cli();

    let writer = match scenario_name.map(str::to_owned) {
        Some(target_scenario_name) => {
            cucumber
                .filter_run(feature_path, move |_, _, scenario| {
                    scenario.name == target_scenario_name
                })
                .await
        }
        None => cucumber.run(feature_path).await,
    };

    let stats_error = summarize_result(
        writer.scenarios_stats(),
        writer.steps_stats(),
        scenario_name,
    )
    .err();
    let cleanup_error = cleanup_recorded_errors().err();

    match (stats_error, cleanup_error) {
        (None, None) => Ok(()),
        (Some(stats), None) => Err(stats.to_string()),
        (None, Some(cleanup)) => Err(cleanup.to_string()),
        (Some(stats), Some(cleanup)) => Err(format!("{stats}\ncleanup also failed: {cleanup}")),
    }
}

pub fn feature_name() -> Result<&'static str> {
    FEATURE_NAME
        .get()
        .map(String::as_str)
        .ok_or_else(|| HarnessError::message("feature name has not been initialized"))
}

fn install_context(feature_name: &str) -> Result<()> {
    FEATURE_NAME
        .set(feature_name.to_string())
        .map_err(|_| HarnessError::message("feature name was already initialized"))?;
    Ok(())
}

fn summarize_result(
    scenario_stats: &cucumber::writer::summarize::Stats,
    step_stats: &cucumber::writer::summarize::Stats,
    scenario_name: Option<&str>,
) -> Result<()> {
    if scenario_name.is_some() && scenario_stats.total() != 1 {
        return Err(HarnessError::message(format!(
            "cucumber executed {} scenarios for a single-scenario test",
            scenario_stats.total()
        )));
    }
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

pub(crate) fn block_on_current_thread<T>(
    future: impl Future<Output = std::result::Result<T, String>>,
    runtime_build_error: &'static str,
) -> std::result::Result<T, String> {
    Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| format!("{runtime_build_error}: {err}"))?
        .block_on(future)
}

pub(crate) fn block_on_support_future<T, E>(
    future: impl Future<Output = std::result::Result<T, E>> + Send + 'static,
    runtime_build_error: &'static str,
    thread_panic_error: &'static str,
) -> std::result::Result<T, String>
where
    T: Send + 'static,
    E: ToString + Send + 'static,
{
    let future = async move { future.await.map_err(|err| err.to_string()) };
    match Handle::try_current() {
        Ok(handle) if handle.runtime_flavor() == RuntimeFlavor::MultiThread => {
            tokio::task::block_in_place(|| handle.block_on(future))
        }
        Ok(_) | Err(_) => {
            thread::spawn(move || block_on_current_thread(future, runtime_build_error))
                .join()
                .map_err(|_| thread_panic_error.to_string())?
        }
    }
}
