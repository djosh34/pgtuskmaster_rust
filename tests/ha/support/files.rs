use std::{fs, path::Path};

use crate::support::error::{HarnessError, Result};

pub(crate) fn create_dir_all(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|source| HarnessError::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub(crate) fn write_text_file(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).map_err(|source| HarnessError::Io {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
pub(crate) fn with_temporary_directory<T>(
    prefix: &str,
    name: &str,
    run: impl FnOnce(&Path) -> Result<T>,
) -> Result<T> {
    let root = temporary_directory(prefix, name)?;
    let result = run(root.as_path());
    let cleanup_result = cleanup_directory(root.as_path());
    match (result, cleanup_result) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(err), Ok(())) => Err(err),
        (Ok(_), Err(cleanup)) => Err(cleanup),
        (Err(err), Err(cleanup)) => Err(HarnessError::message(format!(
            "{err}\ncleanup also failed: {cleanup}"
        ))),
    }
}

#[cfg(test)]
pub(crate) fn temporary_directory(prefix: &str, name: &str) -> Result<std::path::PathBuf> {
    let root = std::env::temp_dir().join(format!(
        "{prefix}-{name}-{}-{}",
        std::process::id(),
        timestamp_millis()?
    ));
    match fs::remove_dir_all(root.as_path()) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => {
            return Err(HarnessError::Io { path: root, source });
        }
    }
    fs::create_dir_all(root.as_path()).map_err(|source| HarnessError::Io {
        path: root.clone(),
        source,
    })?;
    Ok(root)
}

#[cfg(test)]
pub(crate) fn cleanup_directory(path: &Path) -> Result<()> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(HarnessError::Io {
            path: path.to_path_buf(),
            source,
        }),
    }
}

#[cfg(test)]
fn timestamp_millis() -> Result<u128> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|source| HarnessError::message(format!("system clock error: {source}")))
}
