use std::{fs, path::PathBuf};

const FORBIDDEN_PATTERNS: &[&str] = &[
    ".write_path(",
    ".delete_path(",
    "api::controller::",
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
    "crate::ha::worker::step_once(",
    "crate::dcs::worker::step_once(",
    "crate::api::worker::step_once(",
    "crate::debug_api::worker::step_once(",
    "EtcdDcsStore::connect(",
    "refresh_from_etcd_watch(",
    "initialize_pgdata(",
    "send_http_request(",
    "parse_http_response(",
    "post_switchover_via_api(",
];

const ALLOWED_POST_START_PATTERNS: &[&str] = &[
    "\"/switchover\"",
    "CliApiClient::get_ha_state(",
    "crate::cli::run(",
    ".run_sql_on_node(",
    ".run_sql_on_node_with_retry(",
];

#[test]
fn e2e_sources_must_use_post_start_hands_off_control_paths(
) -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let ha_dirs = [repo_root.join("tests/ha/support"), repo_root.join("tests")];

    let mut matched_files = 0usize;
    let mut scanned_files: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    for dir in ha_dirs {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let matches_scope = if path.starts_with(repo_root.join("tests/ha/support")) {
                matches!(file_name, "multi_node.rs" | "partition.rs")
            } else {
                file_name.starts_with("ha_") && file_name.ends_with(".rs")
            };
            if !matches_scope {
                continue;
            }

            matched_files = matched_files.saturating_add(1);
            scanned_files.push(path.display().to_string());
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
    }

    if matched_files == 0 {
        return Err("no e2e source files matched tests/ha/support/{multi_node,partition}.rs or tests/ha_*.rs policy scope".into());
    }

    if violations.is_empty() {
        return Ok(());
    }

    Err(format!(
        "post-start hands-off policy violations detected in tests/ha/support/{{multi_node,partition}}.rs or tests/ha_*.rs.\nallowed post-start controls: GET /ha/state observation, admin switchover API requests, SQL reads/writes for scenario intent, and external process/network fault injection.\nforbidden: direct DCS writes/deletes and internal worker/controller steering after startup.\nscanned_files={}:\n{}\nviolations:\n{}",
        matched_files,
        scanned_files.join("\n"),
        violations.join("\n")
    )
    .into())
}

#[test]
fn e2e_policy_documents_allowed_post_start_actions() {
    let policy = FORBIDDEN_PATTERNS.join("\n");
    for allowed in ALLOWED_POST_START_PATTERNS {
        assert!(
            !policy.contains(allowed),
            "allowed post-start action token `{allowed}` must not be listed as forbidden"
        );
    }
}
