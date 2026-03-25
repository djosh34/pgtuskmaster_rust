use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_TEST_PATH_ID: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn unique_test_dir(prefix: &str, label: &str) -> Result<PathBuf, String> {
    let unique_id = NEXT_TEST_PATH_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "pgtm-{prefix}-{label}-{}-{unique_id}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir)
        .map_err(|err| format!("create test dir {} failed: {err}", dir.display()))?;
    Ok(dir)
}

pub(crate) fn remove_dir_if_exists(path: &Path) -> Result<(), String> {
    match std::fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(format!("remove {} failed: {err}", path.display())),
    }
}

pub(crate) fn write_text_file(dir: &Path, name: &str, contents: &str) -> Result<PathBuf, String> {
    let path = dir.join(name);
    std::fs::write(&path, contents)
        .map_err(|err| format!("write {} failed: {err}", path.display()))?;
    Ok(path)
}
