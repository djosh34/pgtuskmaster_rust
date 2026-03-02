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

    pub(crate) fn into_inner(mut self) -> Result<TestNamespace, HarnessError> {
        self.namespace.take().ok_or_else(|| {
            HarnessError::InvalidInput("namespace guard no longer owns namespace".to_string())
        })
    }
}

impl Drop for NamespaceGuard {
    fn drop(&mut self) {
        if let Some(ns) = self.namespace.take() {
            let _ = cleanup_namespace(ns);
        }
    }
}

pub(crate) fn create_namespace(test_name: &str) -> Result<TestNamespace, HarnessError> {
    let sanitized_name = sanitize_name(test_name);
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| HarnessError::InvalidInput(format!("system clock before unix epoch: {err}")))?
        .as_millis();
    let pid = std::process::id();
    let counter = NAMESPACE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let id = format!("pgtm-{sanitized_name}-{now_ms}-{pid}-{counter}");
    let root_dir = std::env::temp_dir()
        .join("pgtuskmaster-rust")
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
    out
}

#[cfg(test)]
mod tests {
    use super::{cleanup_namespace, create_namespace, NamespaceGuard};

    #[test]
    fn create_namespace_is_unique() {
        let ns_one = match create_namespace("alpha") {
            Ok(ns) => ns,
            Err(err) => panic!("create namespace one failed: {err}"),
        };
        let ns_two = match create_namespace("alpha") {
            Ok(ns) => ns,
            Err(err) => panic!("create namespace two failed: {err}"),
        };

        assert_ne!(ns_one.id, ns_two.id);
        assert_ne!(ns_one.root_dir, ns_two.root_dir);

        if let Err(err) = cleanup_namespace(ns_one) {
            panic!("cleanup namespace one failed: {err}");
        }
        if let Err(err) = cleanup_namespace(ns_two) {
            panic!("cleanup namespace two failed: {err}");
        }
    }

    #[test]
    fn cleanup_is_idempotent() {
        let ns = match create_namespace("cleanup-idempotent") {
            Ok(ns) => ns,
            Err(err) => panic!("create namespace failed: {err}"),
        };
        let clone = ns.clone();

        if let Err(err) = cleanup_namespace(ns) {
            panic!("first cleanup failed: {err}");
        }
        if let Err(err) = cleanup_namespace(clone) {
            panic!("second cleanup failed: {err}");
        }
    }

    #[test]
    fn guard_cleans_up_on_drop() {
        let path = {
            let guard = match NamespaceGuard::new("drop-cleanup") {
                Ok(guard) => guard,
                Err(err) => panic!("guard create failed: {err}"),
            };
            let ns = match guard.namespace() {
                Ok(ns) => ns,
                Err(err) => panic!("namespace lookup failed: {err}"),
            };
            ns.root_dir.clone()
        };

        assert!(!path.exists());
    }
}
