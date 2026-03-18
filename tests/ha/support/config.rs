use std::{fs, path::PathBuf, sync::OnceLock};

use serde::{
    de::{Deserializer, Error as _},
    Deserialize,
};

use crate::support::error::{HarnessError, Result};

static HARNESS_SETTINGS: OnceLock<HarnessSettings> = OnceLock::new();

#[derive(Clone, Debug, Deserialize)]
pub struct HarnessSettings {
    #[serde(deserialize_with = "deserialize_docker_executable")]
    docker: PathBuf,
    #[serde(deserialize_with = "deserialize_pgtm_executable")]
    pgtm: PathBuf,
    #[serde(deserialize_with = "deserialize_psql_executable")]
    psql: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
struct ExecutableDiscoverySettings {
    executable_candidates: Vec<PathBuf>,
}

impl HarnessSettings {
    pub fn docker_executable(&self) -> &std::path::Path {
        self.docker.as_path()
    }

    pub fn pgtm_executable(&self) -> &std::path::Path {
        self.pgtm.as_path()
    }

    pub fn psql_executable(&self) -> &std::path::Path {
        self.psql.as_path()
    }
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
    toml::from_str::<HarnessSettings>(raw.as_str()).map_err(|err| {
        HarnessError::message(format!(
            "failed to parse harness config `{}`: {err}",
            path.display()
        ))
    })
}

fn deserialize_docker_executable<'de, D>(deserializer: D) -> std::result::Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_configured_executable(deserializer, "docker.executable_candidates", "docker", None)
}

fn deserialize_pgtm_executable<'de, D>(deserializer: D) -> std::result::Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_configured_executable(
        deserializer,
        "pgtm.executable_candidates",
        "pgtm",
        Some("pgtm"),
    )
}

fn deserialize_psql_executable<'de, D>(deserializer: D) -> std::result::Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_configured_executable(deserializer, "psql.executable_candidates", "psql", None)
}

fn deserialize_configured_executable<'de, D>(
    deserializer: D,
    config_field: &'static str,
    label: &'static str,
    workspace_binary: Option<&str>,
) -> std::result::Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let settings = ExecutableDiscoverySettings::deserialize(deserializer)?;
    let candidates = match workspace_binary {
        Some(name) => workspace_debug_binary_candidates(name)
            .into_iter()
            .chain(settings.executable_candidates)
            .collect::<Vec<_>>(),
        None => settings.executable_candidates,
    };
    resolve_configured_executable(candidates.as_slice(), config_field, label)
        .map_err(D::Error::custom)
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

fn resolve_configured_executable(
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
    crate::support::process::ensure_absolute_executable(candidate.as_path())?;
    Ok(candidate.clone())
}

#[cfg(test)]
mod tests {
    use super::{
        resolve_configured_executable, workspace_debug_binary_candidates, HarnessError,
        HarnessSettings, Result,
    };
    use std::path::PathBuf;

    fn current_executable() -> Result<PathBuf> {
        std::env::current_exe().map_err(|source| {
            HarnessError::message(format!("current executable path was unavailable: {source}"))
        })
    }

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

    #[test]
    fn resolve_configured_executable_picks_first_existing_candidate() -> Result<()> {
        let current = current_executable()?;
        let selected = resolve_configured_executable(
            &[PathBuf::from("/definitely/missing"), current.clone()],
            "test.executable_candidates",
            "test",
        )?;
        assert_eq!(selected, current);
        Ok(())
    }

    #[test]
    fn resolve_configured_executable_rejects_relative_path() -> Result<()> {
        let error = match resolve_configured_executable(
            &[PathBuf::from("Cargo.toml")],
            "test.executable_candidates",
            "test",
        ) {
            Ok(path) => {
                return Err(HarnessError::message(format!(
                    "expected relative path rejection, got `{}`",
                    path.display()
                )));
            }
            Err(error) => error,
        };
        assert!(
            error
                .to_string()
                .contains("expected an absolute executable path"),
            "unexpected error: {error}"
        );
        Ok(())
    }

    #[test]
    fn harness_settings_deserializes_directly_into_resolved_paths() -> Result<()> {
        let current = current_executable()?;
        let raw = format!(
            r#"[docker]
executable_candidates = ["{path}"]

[pgtm]
executable_candidates = ["{path}"]

[psql]
executable_candidates = ["{path}"]
"#,
            path = current.display(),
        );
        let settings = toml::from_str::<HarnessSettings>(raw.as_str()).map_err(|err| {
            HarnessError::message(format!(
                "expected harness settings to parse directly: {err}"
            ))
        })?;

        assert_eq!(settings.docker_executable(), current.as_path());
        assert!(settings.pgtm_executable().is_absolute());
        assert!(settings.pgtm_executable().exists());
        assert_eq!(settings.psql_executable(), current.as_path());
        Ok(())
    }
}
