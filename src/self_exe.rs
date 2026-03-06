use std::{fs, path::Path, path::PathBuf, sync::OnceLock};

static SELF_EXE: OnceLock<PathBuf> = OnceLock::new();

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum SelfExeError {
    #[error("failed to read current_exe: {0}")]
    CurrentExe(String),
    #[error("self exe already initialized to `{existing}`, cannot set to `{attempt}`")]
    AlreadyInitialized { existing: String, attempt: String },
}

pub(crate) fn init_from_current_exe() -> Result<(), SelfExeError> {
    let current =
        std::env::current_exe().map_err(|err| SelfExeError::CurrentExe(err.to_string()))?;
    let resolved = resolve_self_exe_override(current.as_path()).unwrap_or(current);
    set(resolved)
}

fn resolve_self_exe_override(current: &Path) -> Option<PathBuf> {
    if let Ok(value) = std::env::var("PGTM_SELF_EXE_OVERRIDE") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }

    // When running as a library test (`cargo test --lib`), `current_exe()` points at the libtest
    // harness under `target/*/deps/`. That binary does not implement the `wal` subcommand used by
    // our managed `archive_command`/`restore_command`, so prefer the real `pgtuskmaster` binary
    // when it exists next to the build artifacts.
    let deps_dir = current.parent()?;
    if deps_dir.file_name()? != "deps" {
        return None;
    }
    let target_dir = deps_dir.parent()?;

    let mut candidate = target_dir.join("pgtuskmaster");
    if cfg!(windows) {
        candidate.set_extension("exe");
    }
    if fs::metadata(&candidate).is_ok() {
        return Some(candidate);
    }

    None
}

pub(crate) fn set(path: PathBuf) -> Result<(), SelfExeError> {
    if path.as_os_str().is_empty() {
        return Err(SelfExeError::CurrentExe(
            "current_exe returned an empty path".to_string(),
        ));
    }

    match SELF_EXE.get() {
        Some(existing) => {
            if existing == &path {
                Ok(())
            } else {
                Err(SelfExeError::AlreadyInitialized {
                    existing: existing.display().to_string(),
                    attempt: path.display().to_string(),
                })
            }
        }
        None => {
            if SELF_EXE.set(path).is_err() {
                // Another thread won the race. Re-check for consistency.
                match SELF_EXE.get() {
                    Some(_existing) => Ok(()),
                    None => Err(SelfExeError::CurrentExe(
                        "failed to set self exe path".to_string(),
                    )),
                }
            } else {
                Ok(())
            }
        }
    }
}

pub(crate) fn get() -> Result<PathBuf, SelfExeError> {
    if let Some(path) = SELF_EXE.get() {
        return Ok(path.clone());
    }

    init_from_current_exe()?;
    match SELF_EXE.get() {
        Some(path) => Ok(path.clone()),
        None => Err(SelfExeError::CurrentExe(
            "self exe not initialized".to_string(),
        )),
    }
}
