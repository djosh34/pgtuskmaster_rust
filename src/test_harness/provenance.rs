use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::test_harness::HarnessError;

const POLICY_REL_PATH: &str = "tools/real-binaries-policy.json";
const ATTESTATION_REL_PATH: &str = ".tools/real-binaries-attestation.json";

#[derive(Debug, Deserialize)]
struct RealBinariesPolicy {
    schema_version: u32,
    required_binaries: Vec<PolicyBinary>,
}

#[derive(Debug, Deserialize)]
struct PolicyBinary {
    label: String,
    path: String,
    #[serde(default)]
    kind: ArtifactKind,
    #[serde(default)]
    expected_version: Option<ExpectedVersion>,
    #[serde(default)]
    allowed_target_prefixes: Option<Vec<String>>,
    #[allow(dead_code)]
    pinned_archive: Option<PinnedArchive>,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
enum ArtifactKind {
    #[default]
    Executable,
    DataFile,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ExpectedVersion {
    Etcd { exact: String },
    Postgres { major: u32 },
}

#[derive(Debug, Deserialize)]
struct PinnedArchive {
    #[allow(dead_code)]
    kind: String,
    #[allow(dead_code)]
    repo: String,
    #[allow(dead_code)]
    tag: String,
    #[allow(dead_code)]
    os: String,
    #[allow(dead_code)]
    sha256_by_arch: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct RealBinariesAttestation {
    schema_version: u32,
    #[allow(dead_code)]
    generated_by: Option<String>,
    entries: Vec<AttestedBinary>,
}

#[derive(Debug, Deserialize)]
struct AttestedBinary {
    label: String,
    path: String,
    sha256: String,
    size_bytes: u64,
    #[allow(dead_code)]
    installed_at_utc: Option<String>,
    #[allow(dead_code)]
    resolved_path_abs: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct VerifiedRealBinaries {
    by_label: BTreeMap<String, PathBuf>,
}

static VERIFIED: OnceLock<VerifiedRealBinaries> = OnceLock::new();

pub(crate) fn require_verified_real_binary(label: &str) -> Result<PathBuf, HarnessError> {
    let verified = verified_real_binaries()?;
    verified
        .by_label
        .get(label)
        .cloned()
        .ok_or_else(|| {
            HarnessError::InvalidInput(format!(
                "real-binary policy does not define required label {label:?} (check {POLICY_REL_PATH})"
            ))
        })
}

fn verified_real_binaries() -> Result<&'static VerifiedRealBinaries, HarnessError> {
    if let Some(existing) = VERIFIED.get() {
        return Ok(existing);
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let verified = verify_real_binaries_from_repo_root(repo_root)?;
    if VERIFIED.set(verified).is_err() {
        // Another thread won the race; use the stored value.
    }
    VERIFIED
        .get()
        .ok_or_else(|| HarnessError::InvalidInput("failed to cache verified real binaries".to_string()))
}

pub(crate) fn verify_real_binaries_from_repo_root(
    repo_root: &Path,
) -> Result<VerifiedRealBinaries, HarnessError> {
    let policy_path = repo_root.join(POLICY_REL_PATH);
    let attestation_path = repo_root.join(ATTESTATION_REL_PATH);

    let policy = load_policy(policy_path.as_path())?;
    let attestation = load_attestation(attestation_path.as_path())?;

    if policy.schema_version != 1 {
        return Err(HarnessError::InvalidInput(format!(
            "unsupported real-binaries policy schema_version={} (expected 1)",
            policy.schema_version
        )));
    }
    if attestation.schema_version != 1 {
        return Err(HarnessError::InvalidInput(format!(
            "unsupported real-binaries attestation schema_version={} (expected 1); regenerate by running ./tools/install-etcd.sh and ./tools/install-postgres16.sh",
            attestation.schema_version
        )));
    }

    verify_manifest_permissions(attestation_path.as_path())?;

    let tools_root = repo_root.join(".tools");
    let tools_root_canon = tools_root
        .canonicalize()
        .map_err(|err| HarnessError::InvalidInput(format!("failed to canonicalize .tools directory: {err}")))?;

    let mut attest_by_path: BTreeMap<String, AttestedBinary> = BTreeMap::new();
    for entry in attestation.entries {
        attest_by_path.insert(entry.path.clone(), entry);
    }

    let mut by_label = BTreeMap::new();
    for required in policy.required_binaries {
        enforce_relative_tools_path(required.path.as_str())?;

        let attested = attest_by_path.get(required.path.as_str()).ok_or_else(|| {
            HarnessError::InvalidInput(format!(
                "real-binary attestation missing required entry for {} at {} (regenerate by running ./tools/install-etcd.sh and ./tools/install-postgres16.sh)",
                required.label, required.path
            ))
        })?;

        if attested.label != required.label {
            return Err(HarnessError::InvalidInput(format!(
                "real-binary attestation label mismatch for {}: policy label={} attestation label={}",
                required.path, required.label, attested.label
            )));
        }

        let abs_path = repo_root.join(required.path.as_str());
        let allow_leaf_symlink = required.allowed_target_prefixes.is_some();
        verify_no_symlinks_in_ancestors(repo_root, abs_path.as_path(), allow_leaf_symlink)?;
        match required.kind {
            ArtifactKind::Executable => {
                verify_regular_executable_file(abs_path.as_path(), required.label.as_str(), allow_leaf_symlink)?;
            }
            ArtifactKind::DataFile => {
                verify_regular_file(abs_path.as_path(), required.label.as_str())?;
            }
        }

        let abs_canon = abs_path.canonicalize().map_err(|err| {
            HarnessError::InvalidInput(format!(
                "failed to canonicalize real-binary path for {}: {} ({err})",
                required.label,
                abs_path.display()
            ))
        })?;
        if let Some(prefixes) = required.allowed_target_prefixes.as_ref() {
            let mut allowed = false;
            for prefix in prefixes {
                let prefix_path = Path::new(prefix.as_str());
                if abs_canon.starts_with(prefix_path) {
                    allowed = true;
                    break;
                }
            }
            if !allowed {
                return Err(HarnessError::InvalidInput(format!(
                    "real-binary canonical target is outside allowlist for {}: {} -> {} (allowed={prefixes:?})",
                    required.label,
                    abs_path.display(),
                    abs_canon.display()
                )));
            }
        } else if !abs_canon.starts_with(tools_root_canon.as_path()) {
            return Err(HarnessError::InvalidInput(format!(
                "real-binary path escapes .tools after canonicalize for {}: {} -> {}",
                required.label,
                abs_path.display(),
                abs_canon.display()
            )));
        }

        if let Some(attested_resolved) = attested.resolved_path_abs.as_ref() {
            let canon_str = abs_canon.to_string_lossy();
            if canon_str.as_ref() != attested_resolved.as_str() {
                return Err(HarnessError::InvalidInput(format!(
                    "real-binary resolved path mismatch for {} at {}: attested={} actual={}",
                    required.label,
                    required.path,
                    attested_resolved,
                    canon_str
                )));
            }
        }

        verify_sha256_matches(attested, abs_path.as_path())?;
        verify_size_matches(attested, abs_path.as_path())?;
        if required.kind == ArtifactKind::Executable {
            let expected = required.expected_version.ok_or_else(|| {
                HarnessError::InvalidInput(format!(
                    "real-binary policy missing expected_version for executable {} at {}",
                    required.label, required.path
                ))
            })?;
            verify_version(required.label.as_str(), abs_path.as_path(), &expected)?;
        }

        by_label.insert(required.label, abs_path);
    }

    Ok(VerifiedRealBinaries { by_label })
}

fn load_policy(path: &Path) -> Result<RealBinariesPolicy, HarnessError> {
    let bytes = fs::read(path).map_err(|err| {
        HarnessError::InvalidInput(format!(
            "real-binary policy missing or unreadable: {} ({err})",
            path.display()
        ))
    })?;
    serde_json::from_slice::<RealBinariesPolicy>(bytes.as_slice()).map_err(|err| {
        HarnessError::InvalidInput(format!(
            "failed to parse real-binary policy JSON {}: {err}",
            path.display()
        ))
    })
}

fn load_attestation(path: &Path) -> Result<RealBinariesAttestation, HarnessError> {
    let bytes = fs::read(path).map_err(|err| {
        HarnessError::InvalidInput(format!(
            "real-binary attestation missing or unreadable: {} ({err}); run ./tools/install-etcd.sh and ./tools/install-postgres16.sh",
            path.display()
        ))
    })?;
    serde_json::from_slice::<RealBinariesAttestation>(bytes.as_slice()).map_err(|err| {
        HarnessError::InvalidInput(format!(
            "failed to parse real-binary attestation JSON {}: {err}; regenerate by running ./tools/install-etcd.sh and ./tools/install-postgres16.sh",
            path.display()
        ))
    })
}

fn enforce_relative_tools_path(rel: &str) -> Result<(), HarnessError> {
    if !rel.starts_with(".tools/") {
        return Err(HarnessError::InvalidInput(format!(
            "policy path must be under .tools/: {rel}"
        )));
    }
    let path = Path::new(rel);
    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(HarnessError::InvalidInput(format!(
                    "policy path must be repo-relative without '..' or absolute prefixes: {rel}"
                )));
            }
        }
    }
    Ok(())
}

fn verify_no_symlinks_in_ancestors(
    repo_root: &Path,
    abs_path: &Path,
    allow_leaf_symlink: bool,
) -> Result<(), HarnessError> {
    let rel = abs_path.strip_prefix(repo_root).map_err(|_| {
        HarnessError::InvalidInput(format!(
            "real-binary path is not under repo root: {}",
            abs_path.display()
        ))
    })?;

    let mut current = repo_root.to_path_buf();
    let mut components: Vec<_> = rel.components().collect();
    if allow_leaf_symlink && components.pop().is_none() {
        return Err(HarnessError::InvalidInput(format!(
            "real-binary path is unexpectedly empty: {}",
            abs_path.display()
        )));
    }

    for component in components {
        let comp_str = match component {
            Component::Normal(s) => s,
            Component::CurDir => continue,
            _ => {
                return Err(HarnessError::InvalidInput(format!(
                    "unexpected path component in real-binary path: {}",
                    abs_path.display()
                )))
            }
        };
        current.push(comp_str);
        let meta = fs::symlink_metadata(current.as_path()).map_err(|err| {
            HarnessError::InvalidInput(format!(
                "real-binary path component missing or inaccessible: {} ({err})",
                current.display()
            ))
        })?;
        if meta.file_type().is_symlink() {
            return Err(HarnessError::InvalidInput(format!(
                "real-binary path must not contain symlinks: {}",
                current.display()
            )));
        }
    }

    Ok(())
}

fn verify_manifest_permissions(attestation_path: &Path) -> Result<(), HarnessError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta = fs::metadata(attestation_path).map_err(|err| {
            HarnessError::InvalidInput(format!(
                "attestation manifest missing or inaccessible: {} ({err})",
                attestation_path.display()
            ))
        })?;
        let mode = meta.permissions().mode();
        if (mode & 0o022) != 0 {
            return Err(HarnessError::InvalidInput(format!(
                "attestation manifest must not be group/other writable (mode {mode:o}): {}",
                attestation_path.display()
            )));
        }
    }
    Ok(())
}

fn verify_regular_executable_file(
    path: &Path,
    label: &str,
    allow_leaf_symlink: bool,
) -> Result<(), HarnessError> {
    let leaf_meta = fs::symlink_metadata(path).map_err(|err| {
        HarnessError::InvalidInput(format!(
            "real-binary missing or inaccessible for {label}: {} ({err})",
            path.display()
        ))
    })?;

    if leaf_meta.file_type().is_symlink() && !allow_leaf_symlink {
        return Err(HarnessError::InvalidInput(format!(
            "real-binary must not be a symlink for {label}: {}",
            path.display()
        )));
    }

    // Use dereferenced metadata so we validate the actual executable, not just the link.
    let meta = fs::metadata(path).map_err(|err| {
        HarnessError::InvalidInput(format!(
            "real-binary missing or inaccessible for {label}: {} ({err})",
            path.display()
        ))
    })?;

    if !meta.is_file() {
        return Err(HarnessError::InvalidInput(format!(
            "real-binary is not a regular file for {label}: {}",
            path.display()
        )));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = meta.permissions().mode();
        if (mode & 0o111) == 0 {
            return Err(HarnessError::InvalidInput(format!(
                "real-binary is not executable for {label} (mode {mode:o}): {}",
                path.display()
            )));
        }
        if (mode & 0o022) != 0 {
            return Err(HarnessError::InvalidInput(format!(
                "real-binary must not be group/other writable for {label} (mode {mode:o}): {}",
                path.display()
            )));
        }
    }

    Ok(())
}

fn verify_regular_file(path: &Path, label: &str) -> Result<(), HarnessError> {
    let meta = fs::symlink_metadata(path).map_err(|err| {
        HarnessError::InvalidInput(format!(
            "real-binary missing or inaccessible for {label}: {} ({err})",
            path.display()
        ))
    })?;

    if meta.file_type().is_symlink() {
        return Err(HarnessError::InvalidInput(format!(
            "real-binary file must not be a symlink for {label}: {}",
            path.display()
        )));
    }
    if !meta.is_file() {
        return Err(HarnessError::InvalidInput(format!(
            "real-binary file is not a regular file for {label}: {}",
            path.display()
        )));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = meta.permissions().mode();
        if (mode & 0o022) != 0 {
            return Err(HarnessError::InvalidInput(format!(
                "real-binary file must not be group/other writable for {label} (mode {mode:o}): {}",
                path.display()
            )));
        }
    }

    Ok(())
}

fn verify_sha256_matches(attested: &AttestedBinary, path: &Path) -> Result<(), HarnessError> {
    let actual = sha256_hex(path)?;
    if actual != attested.sha256 {
        return Err(HarnessError::InvalidInput(format!(
            "real-binary sha256 mismatch for {} at {}: expected={} actual={}",
            attested.label,
            attested.path,
            attested.sha256,
            actual
        )));
    }
    Ok(())
}

fn verify_size_matches(attested: &AttestedBinary, path: &Path) -> Result<(), HarnessError> {
    let meta = fs::metadata(path).map_err(|err| {
        HarnessError::InvalidInput(format!(
            "real-binary missing or inaccessible for {}: {} ({err})",
            attested.label,
            path.display()
        ))
    })?;
    let actual = meta.len();
    if actual != attested.size_bytes {
        return Err(HarnessError::InvalidInput(format!(
            "real-binary size mismatch for {} at {}: expected={} actual={}",
            attested.label, attested.path, attested.size_bytes, actual
        )));
    }
    Ok(())
}

fn sha256_hex(path: &Path) -> Result<String, HarnessError> {
    let bytes = fs::read(path).map_err(|err| {
        HarnessError::InvalidInput(format!(
            "failed to read real-binary for hashing: {} ({err})",
            path.display()
        ))
    })?;
    let mut hasher = Sha256::new();
    hasher.update(bytes.as_slice());
    let digest = hasher.finalize();
    Ok(hex_encode_lower(digest.as_slice()))
}

fn hex_encode_lower(bytes: &[u8]) -> String {
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(LUT[(b >> 4) as usize] as char);
        out.push(LUT[(b & 0x0f) as usize] as char);
    }
    out
}

fn verify_version(label: &str, binary_path: &Path, expected: &ExpectedVersion) -> Result<(), HarnessError> {
    let output = Command::new(binary_path)
        .arg("--version")
        .output()
        .map_err(|err| {
            HarnessError::InvalidInput(format!(
                "failed to execute {label} --version at {}: {err}",
                binary_path.display()
            ))
        })?;

    let stdout = String::from_utf8_lossy(output.stdout.as_slice());
    let stderr = String::from_utf8_lossy(output.stderr.as_slice());
    let combined = format!("{stdout}{stderr}");

    if !output.status.success() {
        return Err(HarnessError::InvalidInput(format!(
            "{label} --version exited non-zero at {}: status={} output={combined:?}",
            binary_path.display(),
            output.status
        )));
    }

    match expected {
        ExpectedVersion::Etcd { exact } => {
            let first = combined.lines().next().unwrap_or("");
            let needle = format!("etcd Version: {exact}");
            if !first.contains(needle.as_str()) {
                return Err(HarnessError::InvalidInput(format!(
                    "unexpected etcd version line for {label} at {}: got={first:?} expected_substring={needle:?}",
                    binary_path.display()
                )));
            }
        }
        ExpectedVersion::Postgres { major } => {
            let needle = format!("PostgreSQL) {major}.");
            if !combined.contains(needle.as_str()) {
                return Err(HarnessError::InvalidInput(format!(
                    "unexpected postgres major version for {label} at {}: expected_substring={needle:?} output={combined:?}",
                    binary_path.display()
                )));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::{verify_real_binaries_from_repo_root, ATTESTATION_REL_PATH, POLICY_REL_PATH};
    use crate::test_harness::HarnessError;

    fn unique_tmp_dir(prefix: &str) -> Result<PathBuf, HarnessError> {
        let mut base = std::env::temp_dir();
        base.push(format!("{prefix}-{}", std::process::id()));
        base.push(format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()));
        fs::create_dir_all(base.as_path())?;
        Ok(base)
    }

    #[cfg(unix)]
    fn write_executable(path: &PathBuf, content: &str) -> Result<(), HarnessError> {
        use std::os::unix::fs::PermissionsExt;
        fs::write(path, content.as_bytes())?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
        Ok(())
    }

    fn write_policy(repo_root: &std::path::Path, policy_json: &str) -> Result<(), HarnessError> {
        let policy_path = repo_root.join(POLICY_REL_PATH);
        if let Some(parent) = policy_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(policy_path, policy_json.as_bytes())?;
        Ok(())
    }

    fn write_attestation(
        repo_root: &std::path::Path,
        attestation_json: &str,
    ) -> Result<(), HarnessError> {
        let att_path = repo_root.join(ATTESTATION_REL_PATH);
        if let Some(parent) = att_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&att_path, attestation_json.as_bytes())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(att_path.as_path(), fs::Permissions::from_mode(0o644))?;
        }
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn provenance_accepts_valid_attested_binary() -> Result<(), HarnessError> {
        use sha2::{Digest, Sha256};

        let repo_root = unique_tmp_dir("pgtuskmaster-provenance-ok")?;
        let tools_dir = repo_root.join(".tools/postgres16/bin");
        fs::create_dir_all(tools_dir.as_path())?;
        let bin = tools_dir.join("postgres");
        write_executable(
            &bin,
            "#!/usr/bin/env bash\nif [[ \"${1:-}\" == \"--version\" ]]; then echo \"postgres (PostgreSQL) 16.1\"; exit 0; fi\nexit 1\n",
        )?;

        let policy = r#"
{
  "schema_version": 1,
  "required_binaries": [
    {
      "label": "postgres",
      "path": ".tools/postgres16/bin/postgres",
      "expected_version": { "kind": "postgres", "major": 16 }
    }
  ]
}
"#;
        write_policy(&repo_root, policy)?;

        let bytes = fs::read(bin.as_path())?;
        let mut hasher = Sha256::new();
        hasher.update(bytes.as_slice());
        let digest = hasher.finalize();
        let sha = digest
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<String>>()
            .join("");
        let size = bytes.len();

        let attestation = format!(
            r#"{{
  "schema_version": 1,
  "generated_by": "unit-test",
  "entries": [
    {{
      "label": "postgres",
      "path": ".tools/postgres16/bin/postgres",
      "sha256": "{sha}",
      "size_bytes": {size},
      "installed_at_utc": "2026-01-01T00:00:00Z"
    }}
  ]
}}
"#
        );
        write_attestation(&repo_root, attestation.as_str())?;

        let verified = verify_real_binaries_from_repo_root(repo_root.as_path())?;
        let path = verified
            .by_label
            .get("postgres")
            .ok_or_else(|| HarnessError::InvalidInput("missing expected verified label".to_string()))?;
        assert!(path.ends_with(".tools/postgres16/bin/postgres"));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn provenance_rejects_checksum_mismatch() -> Result<(), HarnessError> {
        let repo_root = unique_tmp_dir("pgtuskmaster-provenance-mismatch")?;
        let tools_dir = repo_root.join(".tools/postgres16/bin");
        fs::create_dir_all(tools_dir.as_path())?;
        let bin = tools_dir.join("postgres");
        write_executable(
            &bin,
            "#!/usr/bin/env bash\nif [[ \"${1:-}\" == \"--version\" ]]; then echo \"postgres (PostgreSQL) 16.1\"; exit 0; fi\nexit 1\n",
        )?;

        let policy = r#"
{
  "schema_version": 1,
  "required_binaries": [
    {
      "label": "postgres",
      "path": ".tools/postgres16/bin/postgres",
      "expected_version": { "kind": "postgres", "major": 16 }
    }
  ]
}
"#;
        write_policy(&repo_root, policy)?;

        // Wrong sha256 on purpose.
        let attestation = r#"
{
  "schema_version": 1,
  "generated_by": "unit-test",
  "entries": [
    {
      "label": "postgres",
      "path": ".tools/postgres16/bin/postgres",
      "sha256": "deadbeef",
      "size_bytes": 1,
      "installed_at_utc": "2026-01-01T00:00:00Z"
    }
  ]
}
"#;
        write_attestation(&repo_root, attestation)?;

        let result = verify_real_binaries_from_repo_root(repo_root.as_path());
        assert!(matches!(result, Err(HarnessError::InvalidInput(_))));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn provenance_rejects_symlink_leaf() -> Result<(), HarnessError> {
        use std::os::unix::fs::symlink;

        let repo_root = unique_tmp_dir("pgtuskmaster-provenance-symlink-leaf")?;
        let tools_dir = repo_root.join(".tools/postgres16/bin");
        fs::create_dir_all(tools_dir.as_path())?;
        let real = tools_dir.join("postgres.real");
        write_executable(
            &real,
            "#!/usr/bin/env bash\nif [[ \"${1:-}\" == \"--version\" ]]; then echo \"postgres (PostgreSQL) 16.1\"; exit 0; fi\nexit 1\n",
        )?;
        let leaf = tools_dir.join("postgres");
        symlink(real.as_path(), leaf.as_path())?;

        let policy = r#"
{
  "schema_version": 1,
  "required_binaries": [
    {
      "label": "postgres",
      "path": ".tools/postgres16/bin/postgres",
      "expected_version": { "kind": "postgres", "major": 16 }
    }
  ]
}
"#;
        write_policy(&repo_root, policy)?;

        let attestation = r#"
{
  "schema_version": 1,
  "generated_by": "unit-test",
  "entries": [
    {
      "label": "postgres",
      "path": ".tools/postgres16/bin/postgres",
      "sha256": "00",
      "size_bytes": 0,
      "installed_at_utc": "2026-01-01T00:00:00Z"
    }
  ]
}
"#;
        write_attestation(&repo_root, attestation)?;

        let result = verify_real_binaries_from_repo_root(repo_root.as_path());
        assert!(matches!(result, Err(HarnessError::InvalidInput(_))));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn provenance_rejects_symlinked_tools_dir() -> Result<(), HarnessError> {
        use std::os::unix::fs::symlink;

        let repo_root = unique_tmp_dir("pgtuskmaster-provenance-symlink-tools")?;

        let outside_root = unique_tmp_dir("pgtuskmaster-provenance-outside")?;
        let outside_bin_dir = outside_root.join("postgres16/bin");
        fs::create_dir_all(outside_bin_dir.as_path())?;
        let outside_postgres = outside_bin_dir.join("postgres");
        write_executable(
            &outside_postgres,
            "#!/usr/bin/env bash\nif [[ \"${1:-}\" == \"--version\" ]]; then echo \"postgres (PostgreSQL) 16.1\"; exit 0; fi\nexit 1\n",
        )?;

        let tools_root = repo_root.join(".tools");
        fs::create_dir_all(tools_root.as_path())?;
        let postgres_dir = tools_root.join("postgres16");
        symlink(outside_root.join("postgres16").as_path(), postgres_dir.as_path())?;

        let policy = r#"
{
  "schema_version": 1,
  "required_binaries": [
    {
      "label": "postgres",
      "path": ".tools/postgres16/bin/postgres",
      "expected_version": { "kind": "postgres", "major": 16 }
    }
  ]
}
"#;
        write_policy(&repo_root, policy)?;

        let attestation = r#"
{
  "schema_version": 1,
  "generated_by": "unit-test",
  "entries": [
    {
      "label": "postgres",
      "path": ".tools/postgres16/bin/postgres",
      "sha256": "00",
      "size_bytes": 0,
      "installed_at_utc": "2026-01-01T00:00:00Z"
    }
  ]
}
"#;
        write_attestation(&repo_root, attestation)?;

        let result = verify_real_binaries_from_repo_root(repo_root.as_path());
        assert!(matches!(result, Err(HarnessError::InvalidInput(_))));
        Ok(())
    }
}
