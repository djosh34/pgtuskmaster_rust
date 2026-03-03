use std::path::PathBuf;
use std::{fs, path::Path};

use super::{namespace::TestNamespace, HarnessError};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TlsMaterial {
    pub(crate) ca_cert: Option<PathBuf>,
    pub(crate) cert: Option<PathBuf>,
    pub(crate) key: Option<PathBuf>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum TlsMode {
    #[default]
    Disabled,
    Optional,
    Required,
}

pub(crate) fn write_tls_material(
    namespace: &TestNamespace,
    profile: &str,
    ca_pem: Option<&[u8]>,
    cert_pem: Option<&[u8]>,
    key_pem: Option<&[u8]>,
) -> Result<TlsMaterial, HarnessError> {
    if profile.trim().is_empty() {
        return Err(HarnessError::InvalidInput(
            "tls profile must not be empty".to_string(),
        ));
    }

    let base = namespace.child_dir(format!("security/tls/{}", sanitize_profile(profile)));
    fs::create_dir_all(&base)?;

    let ca_cert = write_optional_file(&base, "ca.crt", ca_pem)?;
    let cert = write_optional_file(&base, "server.crt", cert_pem)?;
    let key = write_optional_file(&base, "server.key", key_pem)?;

    Ok(TlsMaterial { ca_cert, cert, key })
}

fn write_optional_file(
    base: &Path,
    file_name: &str,
    contents: Option<&[u8]>,
) -> Result<Option<PathBuf>, HarnessError> {
    match contents {
        Some(bytes) => {
            if bytes.is_empty() {
                return Err(HarnessError::InvalidInput(format!(
                    "tls file {file_name} contents must not be empty"
                )));
            }

            let path = base.join(file_name);
            fs::write(&path, bytes)?;
            Ok(Some(path))
        }
        None => Ok(None),
    }
}

fn sanitize_profile(profile: &str) -> String {
    let mut out = String::with_capacity(profile.len());
    for ch in profile.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('-');
        }
    }

    if out.is_empty() {
        "tls-profile".to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use crate::test_harness::{namespace::NamespaceGuard, HarnessError};

    use super::write_tls_material;

    #[test]
    fn write_tls_material_writes_files_under_namespace() -> Result<(), HarnessError> {
        let guard = NamespaceGuard::new("tls-material")?;
        let namespace = guard.namespace()?;
        let material = write_tls_material(
            namespace,
            "node-a",
            Some(b"ca-bytes"),
            Some(b"cert-bytes"),
            Some(b"key-bytes"),
        )?;

        let ca = material
            .ca_cert
            .ok_or_else(|| HarnessError::InvalidInput("expected ca cert path".to_string()))?;
        let cert = material
            .cert
            .ok_or_else(|| HarnessError::InvalidInput("expected cert path".to_string()))?;
        let key = material
            .key
            .ok_or_else(|| HarnessError::InvalidInput("expected key path".to_string()))?;

        assert!(ca.exists());
        assert!(cert.exists());
        assert!(key.exists());
        Ok(())
    }
}
