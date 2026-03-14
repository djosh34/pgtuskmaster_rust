use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{BinaryPathOverrides, BinaryResolutionConfig};
use crate::dev_support::provenance;
use crate::dev_support::HarnessError;

pub fn validate_executable_file(path: &Path, label: &str) -> Result<(), HarnessError> {
    let metadata = fs::metadata(path).map_err(|err| {
        HarnessError::InvalidInput(format!(
            "{label} binary missing or inaccessible: {} ({err})",
            path.display()
        ))
    })?;

    if !metadata.is_file() {
        return Err(HarnessError::InvalidInput(format!(
            "{label} binary is not a regular file: {}",
            path.display()
        )));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        if (mode & 0o111) == 0 {
            return Err(HarnessError::InvalidInput(format!(
                "{label} binary is not executable (mode {mode:o}): {}",
                path.display()
            )));
        }
    }

    Ok(())
}

pub fn require_binary(path: &Path) -> Result<PathBuf, HarnessError> {
    if !path.exists() {
        return Err(HarnessError::InvalidInput(format!(
            "couldn't find binary {}, please either change path or install the binary for this to pass",
            path.display()
        )));
    }

    validate_executable_file(path, "binary")?;
    Ok(path.to_path_buf())
}

pub fn require_etcd_bin_for_real_tests() -> Result<PathBuf, HarnessError> {
    provenance::require_verified_real_binary("etcd")
}

pub fn require_pg16_bin_for_real_tests(name: &str) -> Result<PathBuf, HarnessError> {
    provenance::require_verified_real_binary(name)
}

pub fn require_pg16_process_binaries_for_real_tests() -> Result<BinaryResolutionConfig, HarnessError>
{
    Ok(BinaryResolutionConfig {
        overrides: BinaryPathOverrides {
            postgres: Some(require_pg16_bin_for_real_tests("postgres")?),
            pg_ctl: Some(require_pg16_bin_for_real_tests("pg_ctl")?),
            pg_rewind: Some(require_pg16_bin_for_real_tests("pg_rewind")?),
            initdb: Some(require_pg16_bin_for_real_tests("initdb")?),
            pg_basebackup: Some(require_pg16_bin_for_real_tests("pg_basebackup")?),
            psql: Some(require_pg16_bin_for_real_tests("psql")?),
        },
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::require_binary;
    use crate::dev_support::HarnessError;

    fn unique_tmp_path(prefix: &str) -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        PathBuf::from(format!("/tmp/{prefix}_{pid}_{unique}"))
    }

    #[test]
    fn require_binary_missing_path_returns_invalid_input() {
        let missing = unique_tmp_path("pgtuskmaster_missing_bin");

        let result = require_binary(missing.as_path());
        assert!(matches!(result, Err(HarnessError::InvalidInput(_))));
    }
}
