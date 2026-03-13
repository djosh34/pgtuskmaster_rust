use std::path::{Path, PathBuf};

use crate::support::error::{HarnessError, Result};

pub fn given_root(repo_root: &Path, given_name: &str) -> Result<PathBuf> {
    let path = repo_root.join("tests/ha/givens").join(given_name);
    if path.is_dir() {
        Ok(path)
    } else {
        Err(HarnessError::message(format!(
            "unsupported given `{given_name}`; expected directory `{}`",
            path.display()
        )))
    }
}
