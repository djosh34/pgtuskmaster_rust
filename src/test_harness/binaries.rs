use std::path::{Path, PathBuf};

use crate::config::BinaryPaths;

const PG16_BIN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/.tools/postgres16/bin");
const ETCD_BIN_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/.tools/etcd/bin/etcd");

pub(crate) fn require_binary(path: &Path) -> PathBuf {
    if path.exists() {
        return path.to_path_buf();
    }

    panic!(
        "couldn't find binary {}, please either change path or install the binary for this to pass",
        path.display()
    );
}

pub(crate) fn require_etcd_bin() -> PathBuf {
    require_binary(Path::new(ETCD_BIN_PATH))
}

pub(crate) fn require_pg16_bin(name: &str) -> PathBuf {
    let path = Path::new(PG16_BIN_DIR).join(name);
    require_binary(path.as_path())
}

pub(crate) fn require_pg16_process_binaries() -> BinaryPaths {
    BinaryPaths {
        postgres: require_pg16_bin("postgres"),
        pg_ctl: require_pg16_bin("pg_ctl"),
        pg_rewind: require_pg16_bin("pg_rewind"),
        initdb: require_pg16_bin("initdb"),
        psql: require_pg16_bin("psql"),
    }
}
