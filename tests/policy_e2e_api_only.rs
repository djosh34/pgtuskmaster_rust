use std::{fs, path::PathBuf};

const FORBIDDEN_PATTERNS: &[&str] = &[
    ".write_path(",
    ".delete_path(",
    "api::controller::",
    "post_switchover(",
    "\"/ha/leader\"",
    "post_set_leader_via_api",
    "delete_leader_via_api",
    "ProcessWorkerCtx::contract_stub(",
    "HaWorkerCtx::contract_stub(",
    "ApiWorkerCtx::contract_stub(",
    "DebugApiCtx::contract_stub(",
    "crate::pginfo::worker::run(",
    "crate::dcs::worker::run(",
    "ha_worker::run(",
    "crate::api::worker::step_once(",
    "crate::debug_api::worker::step_once(",
    "initialize_pgdata(",
];

#[test]
fn e2e_sources_must_use_api_only_control_paths() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let ha_dir = repo_root.join("src/ha");

    let mut matched_files = 0usize;
    let mut violations: Vec<String> = Vec::new();

    for entry in fs::read_dir(&ha_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !(file_name.starts_with("e2e_") && file_name.ends_with(".rs")) {
            continue;
        }

        matched_files = matched_files.saturating_add(1);
        let source = fs::read_to_string(&path)?;
        for pattern in FORBIDDEN_PATTERNS {
            if source.contains(pattern) {
                violations.push(format!(
                    "{} contains forbidden pattern `{pattern}`",
                    path.display()
                ));
            }
        }
    }

    if matched_files == 0 {
        return Err("no e2e source files matched src/ha/e2e_*.rs policy scope".into());
    }

    if violations.is_empty() {
        return Ok(());
    }

    Err(format!(
        "e2e API-only policy violations detected:\n{}",
        violations.join("\n")
    )
    .into())
}
