use std::{fs, path::PathBuf, sync::OnceLock};

use serde::Deserialize;

use crate::support::error::{HarnessError, Result};

static HARNESS_SETTINGS: OnceLock<HarnessSettings> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct HarnessSettings {
    docker: PathBuf,
    pgtm: PathBuf,
    psql: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
struct RawHarnessSettings {
    docker: ExecutableDiscoverySettings,
    pgtm: ExecutableDiscoverySettings,
    psql: ExecutableDiscoverySettings,
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
    let raw_settings = toml::from_str::<RawHarnessSettings>(raw.as_str()).map_err(|err| {
        HarnessError::message(format!(
            "failed to parse harness config `{}`: {err}",
            path.display()
        ))
    })?;
    raw_settings.into_harness_settings()
}

impl RawHarnessSettings {
    fn into_harness_settings(self) -> Result<HarnessSettings> {
        Ok(HarnessSettings {
            docker: self
                .docker
                .resolve("docker.executable_candidates", "docker", None)?,
            pgtm: self
                .pgtm
                .resolve("pgtm.executable_candidates", "pgtm", Some("pgtm"))?,
            psql: self
                .psql
                .resolve("psql.executable_candidates", "psql", None)?,
        })
    }
}

impl ExecutableDiscoverySettings {
    fn resolve(
        self,
        config_field: &'static str,
        label: &'static str,
        workspace_binary: Option<&str>,
    ) -> Result<PathBuf> {
        let candidates = workspace_binary
            .into_iter()
            .flat_map(workspace_debug_binary_candidates)
            .chain(self.executable_candidates)
            .collect::<Vec<_>>();
        resolve_configured_executable(candidates.as_slice(), config_field, label)
    }
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
        resolve_configured_executable, workspace_debug_binary_candidates,
        ExecutableDiscoverySettings, HarnessError, RawHarnessSettings, Result,
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

    fn parse_harness_settings(raw: &str) -> Result<super::HarnessSettings> {
        let settings = toml::from_str::<RawHarnessSettings>(raw).map_err(|err| {
            HarnessError::message(format!(
                "expected raw harness settings to parse directly: {err}"
            ))
        })?;
        settings.into_harness_settings()
    }

    #[test]
    fn executable_discovery_settings_resolve_uses_workspace_binary_fallback() -> Result<()> {
        let current = current_executable()?;
        let resolved = ExecutableDiscoverySettings {
            executable_candidates: vec![current.clone()],
        }
        .resolve("pgtm.executable_candidates", "pgtm", Some("pgtm"))?;
        let target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target");

        assert!(resolved.starts_with(target_dir) || resolved == current);
        assert!(resolved.is_absolute());
        assert!(resolved.exists());
        Ok(())
    }

    #[test]
    fn harness_settings_parse_into_resolved_paths() -> Result<()> {
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
        let settings = parse_harness_settings(raw.as_str())?;

        assert_eq!(settings.docker_executable(), current.as_path());
        assert!(settings.pgtm_executable().is_absolute());
        assert!(settings.pgtm_executable().exists());
        assert_eq!(settings.psql_executable(), current.as_path());
        Ok(())
    }
}
