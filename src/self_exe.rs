use std::{
    path::PathBuf,
    sync::{OnceLock},
};

static SELF_EXE: OnceLock<PathBuf> = OnceLock::new();

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum SelfExeError {
    #[error("failed to read current_exe: {0}")]
    CurrentExe(String),
    #[error("self exe already initialized to `{existing}`, cannot set to `{attempt}`")]
    AlreadyInitialized { existing: String, attempt: String },
}

pub(crate) fn init_from_current_exe() -> Result<(), SelfExeError> {
    let path = std::env::current_exe().map_err(|err| SelfExeError::CurrentExe(err.to_string()))?;
    set(path)
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
    match SELF_EXE.get() {
        Some(path) => Ok(path.clone()),
        None => Err(SelfExeError::CurrentExe(
            "self exe not initialized".to_string(),
        )),
    }
}
