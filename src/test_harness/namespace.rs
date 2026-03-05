use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::HarnessError;

static NAMESPACE_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TestNamespace {
    pub(crate) id: String,
    pub(crate) root_dir: PathBuf,
}

impl TestNamespace {
    pub(crate) fn child_dir(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.root_dir.join(relative)
    }
}

#[derive(Debug)]
pub(crate) struct NamespaceGuard {
    namespace: Option<TestNamespace>,
}

impl NamespaceGuard {
    pub(crate) fn new(test_name: &str) -> Result<Self, HarnessError> {
        let namespace = create_namespace(test_name)?;
        Ok(Self {
            namespace: Some(namespace),
        })
    }

    pub(crate) fn namespace(&self) -> Result<&TestNamespace, HarnessError> {
        self.namespace.as_ref().ok_or_else(|| {
            HarnessError::InvalidInput("namespace guard no longer owns namespace".to_string())
        })
    }

}

impl Drop for NamespaceGuard {
    fn drop(&mut self) {
        if let Some(ns) = self.namespace.take() {
            if should_keep_namespace() {
                eprintln!(
                    "test harness: keeping namespace at {} (set by env PGTM_TEST_KEEP_NAMESPACE=1)",
                    ns.root_dir.display()
                );
                return;
            }
            let _ = cleanup_namespace(ns);
        }
    }
}

fn should_keep_namespace() -> bool {
    match std::env::var("PGTM_TEST_KEEP_NAMESPACE") {
        Ok(value) => parse_env_bool(value.as_str()),
        Err(std::env::VarError::NotPresent) => false,
        Err(std::env::VarError::NotUnicode(_)) => false,
    }
}

fn parse_env_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "y" | "on"
    )
}

pub(crate) fn create_namespace(test_name: &str) -> Result<TestNamespace, HarnessError> {
    let sanitized_name = sanitize_name(test_name);
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| {
            HarnessError::InvalidInput(format!("system clock before unix epoch: {err}"))
        })?
        .as_millis();
    let pid = std::process::id();
    let counter = NAMESPACE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let id = format!("pgtm-{sanitized_name}-{now_ms}-{pid}-{counter}");
    let root_dir = std::env::temp_dir()
        // Keep this directory name short: postgres unix socket paths are capped (107 bytes on linux).
        .join("pgtm")
        .join(id.as_str());

    fs::create_dir_all(root_dir.join("logs"))?;
    fs::create_dir_all(root_dir.join("run"))?;

    Ok(TestNamespace { id, root_dir })
}

pub(crate) fn cleanup_namespace(ns: TestNamespace) -> Result<(), HarnessError> {
    match fs::remove_dir_all(&ns.root_dir) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(HarnessError::Io(err)),
    }
}

fn sanitize_name(name: &str) -> String {
    const MAX_LEN: usize = 24;
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return "test".to_string();
    }

    let mut out = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    if out.len() > MAX_LEN {
        out.truncate(MAX_LEN);
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::test_harness::HarnessError;

    use super::{cleanup_namespace, create_namespace, NamespaceGuard};

    #[test]
    fn create_namespace_is_unique() -> Result<(), HarnessError> {
        let ns_one = create_namespace("alpha")?;
        let ns_two = create_namespace("alpha")?;

        assert_ne!(ns_one.id, ns_two.id);
        assert_ne!(ns_one.root_dir, ns_two.root_dir);

        cleanup_namespace(ns_one)?;
        cleanup_namespace(ns_two)?;
        Ok(())
    }

    #[test]
    fn cleanup_is_idempotent() -> Result<(), HarnessError> {
        let ns = create_namespace("cleanup-idempotent")?;
        let clone = ns.clone();

        cleanup_namespace(ns)?;
        cleanup_namespace(clone)?;
        Ok(())
    }

    #[test]
    fn guard_cleans_up_on_drop() -> Result<(), HarnessError> {
        let path = {
            let guard = NamespaceGuard::new("drop-cleanup")?;
            let ns = guard.namespace()?;
            ns.root_dir.clone()
        };

        assert!(!path.exists());
        Ok(())
    }
}
