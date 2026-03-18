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
    let mut settings: HarnessSettings = toml::from_str(raw.as_str()).map_err(|err| {
        HarnessError::message(format!(
            "failed to parse harness config `{}`: {err}",
            path.display()
        ))
    })?;
    let workspace_candidates = workspace_debug_binary_candidates("pgtm");
    settings
        .pgtm
        .executable_candidates
        .splice(0..0, workspace_candidates);
    Ok(settings)
}

fn workspace_debug_binary_candidates(name: &str) -> Vec<PathBuf> {
    let target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target");
    let executable_name = format!("{name}{}", std::env::consts::EXE_SUFFIX);
    let mut candidates = vec![target_dir.join("debug").join(executable_name.as_str())];

    if let Ok(current_executable) = std::env::current_exe() {
        if let Some(debug_dir) = current_executable
            .parent()
            .and_then(|deps_dir| deps_dir.parent())
        {
            let candidate = debug_dir.join(executable_name);
            if candidate.starts_with(&target_dir) && !candidates.contains(&candidate) {
                candidates.push(candidate);
            }
        }
    }

    candidates
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

#[cfg(test)]
mod tests {
    use super::workspace_debug_binary_candidates;
    use std::path::PathBuf;

    #[test]
    fn workspace_debug_binary_candidates_stay_under_target_dir() {
        let target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target");
        let candidates = workspace_debug_binary_candidates("pgtm");
        assert!(candidates.iter().all(|path| path.starts_with(&target_dir)));
        assert_eq!(
            candidates.first(),
            Some(
                &target_dir
                    .join("debug")
                    .join(format!("pgtm{}", std::env::consts::EXE_SUFFIX))
            )
        );
    }
}
