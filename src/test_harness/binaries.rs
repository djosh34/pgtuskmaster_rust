use std::env::VarError;
use std::path::{Path, PathBuf};

use crate::config::BinaryPaths;
use crate::test_harness::HarnessError;

const PG16_BIN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/.tools/postgres16/bin");
const ETCD_BIN_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/.tools/etcd/bin/etcd");
pub(crate) const REQUIRE_REAL_BINARIES_ENV: &str = "PGTUSKMASTER_REQUIRE_REAL_BINARIES";

pub(crate) fn require_binary(path: &Path) -> Result<PathBuf, HarnessError> {
    if path.exists() {
        return Ok(path.to_path_buf());
    }

    Err(HarnessError::InvalidInput(format!(
        "couldn't find binary {}, please either change path or install the binary for this to pass",
        path.display()
    )))
}

pub(crate) fn require_etcd_bin() -> Result<PathBuf, HarnessError> {
    require_binary(Path::new(ETCD_BIN_PATH))
}

pub(crate) fn require_pg16_bin(name: &str) -> Result<PathBuf, HarnessError> {
    let path = Path::new(PG16_BIN_DIR).join(name);
    require_binary(path.as_path())
}

fn real_binaries_enforced() -> Result<bool, HarnessError> {
    match std::env::var(REQUIRE_REAL_BINARIES_ENV) {
        Ok(raw) => parse_bool_env(raw.as_str()),
        Err(VarError::NotPresent) => Ok(false),
        Err(VarError::NotUnicode(_)) => Err(HarnessError::InvalidInput(format!(
            "{REQUIRE_REAL_BINARIES_ENV} contains non-utf8 data"
        ))),
    }
}

fn parse_bool_env(raw: &str) -> Result<bool, HarnessError> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "" | "0" | "false" | "no" | "off" => Ok(false),
        other => Err(HarnessError::InvalidInput(format!(
            "invalid {REQUIRE_REAL_BINARIES_ENV} value '{other}'; use one of 1,true,yes,on,0,false,no,off"
        ))),
    }
}

fn real_binary_missing_message(path: &Path) -> String {
    format!(
        "real-binary prerequisite missing: {} (set {REQUIRE_REAL_BINARIES_ENV}=1 to enforce fail-fast)",
        path.display()
    )
}

fn require_or_skip_binary(
    path: &Path,
    enforce_real_binaries: bool,
) -> Result<Option<PathBuf>, HarnessError> {
    if path.exists() {
        return Ok(Some(path.to_path_buf()));
    }

    let message = real_binary_missing_message(path);
    if enforce_real_binaries {
        Err(HarnessError::InvalidInput(message))
    } else {
        eprintln!("{message}; skipping real-binary test");
        Ok(None)
    }
}

pub(crate) fn require_etcd_bin_for_real_tests() -> Result<Option<PathBuf>, HarnessError> {
    require_or_skip_binary(Path::new(ETCD_BIN_PATH), real_binaries_enforced()?)
}

pub(crate) fn require_pg16_bin_for_real_tests(name: &str) -> Result<Option<PathBuf>, HarnessError> {
    let path = Path::new(PG16_BIN_DIR).join(name);
    require_or_skip_binary(path.as_path(), real_binaries_enforced()?)
}

pub(crate) fn require_pg16_process_binaries_for_real_tests(
) -> Result<Option<BinaryPaths>, HarnessError> {
    let postgres = match require_pg16_bin_for_real_tests("postgres")? {
        Some(path) => path,
        None => return Ok(None),
    };
    let pg_ctl = match require_pg16_bin_for_real_tests("pg_ctl")? {
        Some(path) => path,
        None => return Ok(None),
    };
    let pg_rewind = match require_pg16_bin_for_real_tests("pg_rewind")? {
        Some(path) => path,
        None => return Ok(None),
    };
    let initdb = match require_pg16_bin_for_real_tests("initdb")? {
        Some(path) => path,
        None => return Ok(None),
    };
    let psql = match require_pg16_bin_for_real_tests("psql")? {
        Some(path) => path,
        None => return Ok(None),
    };

    Ok(Some(BinaryPaths {
        postgres,
        pg_ctl,
        pg_rewind,
        initdb,
        psql,
    }))
}

pub(crate) fn require_pg16_process_binaries() -> Result<BinaryPaths, HarnessError> {
    Ok(BinaryPaths {
        postgres: require_pg16_bin("postgres")?,
        pg_ctl: require_pg16_bin("pg_ctl")?,
        pg_rewind: require_pg16_bin("pg_rewind")?,
        initdb: require_pg16_bin("initdb")?,
        psql: require_pg16_bin("psql")?,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{parse_bool_env, require_binary, require_or_skip_binary};
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
    fn parse_bool_env_accepts_expected_values() -> Result<(), HarnessError> {
        assert!(parse_bool_env("1")?);
        assert!(parse_bool_env("TRUE")?);
        assert!(!parse_bool_env("0")?);
        assert!(!parse_bool_env("off")?);
        Ok(())
    }

    #[test]
    fn parse_bool_env_rejects_invalid_values() {
        let result = parse_bool_env("sometimes");
        assert!(matches!(result, Err(HarnessError::InvalidInput(_))));
    }

    #[test]
    fn require_or_skip_binary_returns_none_when_missing_and_not_enforced(
    ) -> Result<(), HarnessError> {
        let millis = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        let missing = PathBuf::from(format!(
            "/tmp/pgtuskmaster_missing_optional_bin_{millis}_{}",
            std::process::id()
        ));
        let result = require_or_skip_binary(missing.as_path(), false)?;
        assert!(result.is_none());
        Ok(())
    }

    #[test]
    fn require_or_skip_binary_returns_error_when_missing_and_enforced() {
        let millis = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        let missing = PathBuf::from(format!(
            "/tmp/pgtuskmaster_missing_required_bin_{millis}_{}",
            std::process::id()
        ));
        let result = require_or_skip_binary(missing.as_path(), true);
        assert!(matches!(result, Err(HarnessError::InvalidInput(_))));
    }

    #[test]
    fn require_or_skip_binary_returns_path_when_present() -> Result<(), HarnessError> {
        let millis = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => 0,
        };
        let present = PathBuf::from(format!(
            "/tmp/pgtuskmaster_present_bin_{millis}_{}",
            std::process::id()
        ));
        fs::write(&present, b"bin")?;
        let result = require_or_skip_binary(present.as_path(), true)?;
        assert_eq!(result, Some(present.clone()));
        fs::remove_file(present)?;
        Ok(())
    }
}
