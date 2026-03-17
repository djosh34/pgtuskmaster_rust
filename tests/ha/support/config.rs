use std::{fs, path::PathBuf, sync::OnceLock};

use serde::Deserialize;

use crate::support::error::{HarnessError, Result};

static HARNESS_SETTINGS: OnceLock<HarnessSettings> = OnceLock::new();

#[derive(Clone, Debug, Deserialize)]
pub struct HarnessSettings {
    pub docker: ExecutableDiscoverySettings,
    pub pgtm: ExecutableDiscoverySettings,
    pub psql: ExecutableDiscoverySettings,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ExecutableDiscoverySettings {
    pub executable_candidates: Vec<PathBuf>,
}

pub fn harness_settings() -> Result<&'static HarnessSettings> {
    if let Some(settings) = HARNESS_SETTINGS.get() {
        return Ok(settings);
    }

    let loaded = load_harness_settings()?;
    HARNESS_SETTINGS
        .set(loaded)
        .map_err(|_| HarnessError::message("harness settings were already initialized"))?;
    HARNESS_SETTINGS
        .get()
        .ok_or_else(|| HarnessError::message("harness settings were not available after load"))
}

fn load_harness_settings() -> Result<HarnessSettings> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/ha/harness.toml");
    let raw = fs::read_to_string(path.as_path()).map_err(|source| HarnessError::Io {
        path: path.clone(),
        source,
    })?;
    toml::from_str(raw.as_str()).map_err(|err| {
        HarnessError::message(format!(
            "failed to parse harness config `{}`: {err}",
            path.display()
        ))
    })
}

pub fn configured_executable(
    candidates: &[PathBuf],
    config_field: &str,
    label: &str,
) -> Result<PathBuf> {
    let candidate = candidates
        .iter()
        .find(|path| path.exists())
        .ok_or_else(|| {
            HarnessError::message(format!(
                "{label} binary was not found in tests/ha/harness.toml {config_field}"
            ))
        })?;
    Ok(candidate.clone())
}
