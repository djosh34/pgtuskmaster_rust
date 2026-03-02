use std::path::{Path, PathBuf};

use crate::config::BinaryPaths;
use crate::test_harness::HarnessError;

const PG16_BIN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/.tools/postgres16/bin");
const ETCD_BIN_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/.tools/etcd/bin/etcd");

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
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::require_binary;
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
}
