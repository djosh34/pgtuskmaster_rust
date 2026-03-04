use std::fs;
use std::path::{Path, PathBuf};

use crate::config::BinaryPaths;
use crate::test_harness::HarnessError;

const PG16_BIN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/.tools/postgres16/bin");
const ETCD_BIN_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/.tools/etcd/bin/etcd");

pub(crate) fn validate_executable_file(path: &Path, label: &str) -> Result<(), HarnessError> {
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

pub(crate) fn require_binary(path: &Path) -> Result<PathBuf, HarnessError> {
    if !path.exists() {
        return Err(HarnessError::InvalidInput(format!(
            "couldn't find binary {}, please either change path or install the binary for this to pass",
            path.display()
        )));
    }

    validate_executable_file(path, "binary")?;
    Ok(path.to_path_buf())
}

pub(crate) fn require_etcd_bin() -> Result<PathBuf, HarnessError> {
    require_binary(Path::new(ETCD_BIN_PATH))
}

pub(crate) fn require_pg16_bin(name: &str) -> Result<PathBuf, HarnessError> {
    let path = Path::new(PG16_BIN_DIR).join(name);
    require_binary(path.as_path())
}

fn require_real_binary(path: &Path) -> Result<PathBuf, HarnessError> {
    if !path.exists() {
        return Err(HarnessError::InvalidInput(format!(
            "real-binary prerequisite missing: {} (install required test binaries under .tools/postgres16/bin and .tools/etcd/bin)",
            path.display()
        )));
    }

    validate_executable_file(path, "real-binary prerequisite")?;
    Ok(path.to_path_buf())
}

pub(crate) fn require_etcd_bin_for_real_tests() -> Result<PathBuf, HarnessError> {
    require_real_binary(Path::new(ETCD_BIN_PATH))
}

pub(crate) fn require_pg16_bin_for_real_tests(name: &str) -> Result<PathBuf, HarnessError> {
    let path = Path::new(PG16_BIN_DIR).join(name);
    require_real_binary(path.as_path())
}

pub(crate) fn require_pg16_process_binaries_for_real_tests() -> Result<BinaryPaths, HarnessError> {
    Ok(BinaryPaths {
        postgres: require_pg16_bin_for_real_tests("postgres")?,
        pg_ctl: require_pg16_bin_for_real_tests("pg_ctl")?,
        pg_rewind: require_pg16_bin_for_real_tests("pg_rewind")?,
        initdb: require_pg16_bin_for_real_tests("initdb")?,
        pg_basebackup: require_pg16_bin_for_real_tests("pg_basebackup")?,
        psql: require_pg16_bin_for_real_tests("psql")?,
    })
}

pub(crate) fn require_pg16_process_binaries() -> Result<BinaryPaths, HarnessError> {
    Ok(BinaryPaths {
        postgres: require_pg16_bin("postgres")?,
        pg_ctl: require_pg16_bin("pg_ctl")?,
        pg_rewind: require_pg16_bin("pg_rewind")?,
        initdb: require_pg16_bin("initdb")?,
        pg_basebackup: require_pg16_bin("pg_basebackup")?,
        psql: require_pg16_bin("psql")?,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{require_binary, require_real_binary};
    use crate::test_harness::HarnessError;

    #[test]
    fn require_binary_missing_path_returns_invalid_input() {
        let millis = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        let missing = PathBuf::from(format!(
            "/tmp/pgtuskmaster_missing_bin_{millis}_{}",
            std::process::id()
        ));

        let result = require_binary(missing.as_path());
        assert!(matches!(result, Err(HarnessError::InvalidInput(_))));
    }

    #[test]
    fn require_real_binary_returns_error_when_missing() {
        let millis = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        let missing = PathBuf::from(format!(
            "/tmp/pgtuskmaster_missing_required_bin_{millis}_{}",
            std::process::id()
        ));
        let result = require_real_binary(missing.as_path());
        assert!(matches!(result, Err(HarnessError::InvalidInput(_))));
    }

    #[test]
    fn require_real_binary_returns_path_when_present() -> Result<(), HarnessError> {
        let millis = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        let present = PathBuf::from(format!(
            "/tmp/pgtuskmaster_present_bin_{millis}_{}",
            std::process::id()
        ));
        fs::write(&present, b"bin")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&present, fs::Permissions::from_mode(0o755))?;
        }
        let result = require_real_binary(present.as_path())?;
        assert_eq!(result, present.clone());
        fs::remove_file(present)?;
        Ok(())
    }

    #[test]
    fn require_real_binary_rejects_directories() -> Result<(), HarnessError> {
        let millis = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        let dir_path = PathBuf::from(format!(
            "/tmp/pgtuskmaster_present_bin_dir_{millis}_{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir_path)?;
        let result = require_real_binary(dir_path.as_path());
        assert!(matches!(result, Err(HarnessError::InvalidInput(_))));
        fs::remove_dir_all(dir_path)?;
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn require_real_binary_rejects_non_executable_files_on_unix() -> Result<(), HarnessError> {
        use std::os::unix::fs::PermissionsExt;

        let millis = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        let present = PathBuf::from(format!(
            "/tmp/pgtuskmaster_present_nonexec_bin_{millis}_{}",
            std::process::id()
        ));
        fs::write(&present, b"bin")?;
        fs::set_permissions(&present, fs::Permissions::from_mode(0o644))?;

        let result = require_real_binary(present.as_path());
        assert!(matches!(result, Err(HarnessError::InvalidInput(_))));

        fs::remove_file(present)?;
        Ok(())
    }
}
