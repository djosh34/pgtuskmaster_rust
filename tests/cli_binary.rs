use std::{
    io::{Read, Write},
    net::TcpListener,
    process::Command,
    sync::mpsc,
};

fn write_temp_config(label: &str, toml: &str) -> Result<std::path::PathBuf, String> {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| format!("system time error: {err}"))?
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "pgtuskmaster-cli-config-{label}-{unique}-{}",
        std::process::id()
    ));

    std::fs::write(&path, toml).map_err(|err| format!("write config failed: {err}"))?;
    Ok(path)
}

fn pgtm_config_toml(api_listen_addr: &str, api_security: &str, pgtm_section: &str) -> String {
    format!(
        r##"
[cluster]
name = "cluster-a"
member_id = "node-a"

[postgres]
data_dir = "/tmp/pgdata"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtm/socket"
log_file = "/tmp/pgtm/postgres.log"
local_conn_identity = {{ user = "postgres", dbname = "postgres", ssl_mode = "prefer" }}
rewind_conn_identity = {{ user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }}
tls = {{ mode = "disabled" }}
roles = {{ superuser = {{ username = "postgres", auth = {{ type = "password", password = {{ content = "secret-password" }} }} }}, replicator = {{ username = "replicator", auth = {{ type = "password", password = {{ content = "secret-password" }} }} }}, rewinder = {{ username = "rewinder", auth = {{ type = "password", password = {{ content = "secret-password" }} }} }} }}
pg_hba = {{ source = {{ content = "local all all trust" }} }}
pg_ident = {{ source = {{ content = "empty" }} }}

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
pg_rewind_timeout_ms = 1000
bootstrap_timeout_ms = 1000
fencing_timeout_ms = 1000
binaries = {{ postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }}

[api]
listen_addr = "{api_listen_addr}"
security = {api_security}

{pgtm_section}
"##
    )
}

fn sample_status_json(api_url: &str) -> String {
    format!(
        r#"{{"cluster_name":"cluster-a","scope":"scope-a","self_member_id":"node-a","leader":"node-a","switchover_pending":false,"switchover_to":null,"member_count":1,"members":[{{"member_id":"node-a","postgres_host":"127.0.0.1","postgres_port":5432,"api_url":"{api_url}","role":"primary","sql":"healthy","readiness":"ready","timeline":7,"write_lsn":10,"replay_lsn":null,"updated_at_ms":1,"pg_version":1}}],"dcs_trust":"full_quorum","ha_phase":"primary","ha_tick":1,"ha_decision":{{"kind":"become_primary","promote":true}},"snapshot_sequence":10}}"#
    )
}

fn http_json_ok(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
        body.len(),
        body
    )
}

fn spawn_single_request_server(
    response: &str,
) -> Result<(std::net::SocketAddr, mpsc::Receiver<String>), String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|err| format!("bind failed: {err}"))?;
    let addr = listener
        .local_addr()
        .map_err(|err| format!("local_addr failed: {err}"))?;
    let (tx, rx) = mpsc::channel();
    let response = response.to_string();
    std::thread::spawn(move || {
        let result = (|| -> Result<(), String> {
            let (mut stream, _) = listener
                .accept()
                .map_err(|err| format!("accept failed: {err}"))?;
            let mut buf = [0_u8; 4096];
            let bytes = stream
                .read(&mut buf)
                .map_err(|err| format!("read failed: {err}"))?;
            let request = String::from_utf8(buf[..bytes].to_vec())
                .map_err(|err| format!("request utf8 decode failed: {err}"))?;
            stream
                .write_all(response.as_bytes())
                .map_err(|err| format!("write failed: {err}"))?;
            tx.send(request)
                .map_err(|err| format!("send request failed: {err}"))?;
            Ok(())
        })();
        if let Err(err) = result {
            let _ = tx.send(format!("server-error: {err}"));
        }
    });
    Ok((addr, rx))
}

fn cli_bin_path() -> Result<std::path::PathBuf, String> {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_pgtm") {
        return Ok(std::path::PathBuf::from(path));
    }

    let current = std::env::current_exe().map_err(|err| format!("current_exe failed: {err}"))?;
    let debug_dir = current
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| "failed to derive target/debug directory".to_string())?;
    let candidate = if cfg!(windows) {
        debug_dir.join("pgtm.exe")
    } else {
        debug_dir.join("pgtm")
    };
    if candidate.exists() {
        Ok(candidate)
    } else {
        Err(format!("cli binary not found at {}", candidate.display()))
    }
}

fn node_bin_path() -> Result<std::path::PathBuf, String> {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_pgtuskmaster") {
        return Ok(std::path::PathBuf::from(path));
    }

    let current = std::env::current_exe().map_err(|err| format!("current_exe failed: {err}"))?;
    let debug_dir = current
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| "failed to derive target/debug directory".to_string())?;
    let mut candidate = debug_dir.join("pgtuskmaster");
    if cfg!(windows) {
        candidate.set_extension("exe");
    }
    if candidate.exists() {
        Ok(candidate)
    } else {
        Err(format!("node binary not found at {}", candidate.display()))
    }
}

#[test]
fn help_exits_success() -> Result<(), String> {
    let bin = cli_bin_path()?;
    let output = Command::new(&bin)
        .arg("--help")
        .output()
        .map_err(|err| format!("failed to run --help: {err}"))?;

    assert!(
        output.status.success(),
        "--help should exit successfully, got {:?}",
        output.status.code()
    );

    let stdout = String::from_utf8(output.stdout)
        .map_err(|err| format!("stdout utf8 decode failed: {err}"))?;
    assert!(
        stdout.contains("status"),
        "help output should include status command"
    );
    assert!(
        stdout.contains("--config") && stdout.contains("-c"),
        "help output should advertise config loading"
    );
    Ok(())
}

#[test]
fn missing_required_subcommand_arg_exits_usage_code() -> Result<(), String> {
    let bin = cli_bin_path()?;
    let output = Command::new(&bin)
        .args(["switchover", "leader", "set"])
        .output()
        .map_err(|err| format!("failed to run command: {err}"))?;

    assert_eq!(
        output.status.code(),
        Some(2),
        "clap usage failures should exit with code 2"
    );
    Ok(())
}

#[test]
fn state_command_maps_connection_refused_to_exit_3() -> Result<(), String> {
    let bin = cli_bin_path()?;
    let listener =
        std::net::TcpListener::bind("127.0.0.1:0").map_err(|err| format!("bind failed: {err}"))?;
    let addr = listener
        .local_addr()
        .map_err(|err| format!("local_addr failed: {err}"))?;
    drop(listener);

    let output = Command::new(&bin)
        .args([
            "--base-url",
            &format!("http://{addr}"),
            "--timeout-ms",
            "50",
            "status",
        ])
        .output()
        .map_err(|err| format!("failed to run state command: {err}"))?;

    assert_eq!(
        output.status.code(),
        Some(3),
        "transport errors should map to exit code 3"
    );

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("transport error"),
        "stderr should mention transport error"
    );
    Ok(())
}

#[test]
fn bare_pgtm_defaults_to_status_and_renders_human_table() -> Result<(), String> {
    let bin = cli_bin_path()?;
    let (addr, rx) = spawn_single_request_server(
        http_json_ok(sample_status_json("http://127.0.0.1:8080").as_str()).as_str(),
    )?;

    let output = Command::new(&bin)
        .args(["--base-url", &format!("http://{addr}")])
        .output()
        .map_err(|err| format!("failed to run bare pgtm: {err}"))?;

    assert!(
        output.status.success(),
        "bare pgtm should succeed, got {:?}",
        output.status.code()
    );
    let stdout = String::from_utf8(output.stdout)
        .map_err(|err| format!("stdout utf8 decode failed: {err}"))?;
    assert!(stdout.contains("cluster: cluster-a  health: healthy"));
    assert!(stdout.contains("NODE"));
    assert!(stdout.contains("node-a"));
    assert!(stdout.contains("primary"));
    let request = rx
        .recv_timeout(std::time::Duration::from_secs(2))
        .map_err(|err| format!("failed to receive status request: {err}"))?;
    assert!(
        request.starts_with("GET /ha/state HTTP/1.1"),
        "expected /ha/state request, got: {request}"
    );
    Ok(())
}

#[test]
fn status_json_output_contains_queried_via_identity() -> Result<(), String> {
    let bin = cli_bin_path()?;
    let api_url = "http://127.0.0.1:8080";
    let (addr, _rx) =
        spawn_single_request_server(http_json_ok(sample_status_json(api_url).as_str()).as_str())?;

    let output = Command::new(&bin)
        .args(["--base-url", &format!("http://{addr}"), "--json", "status"])
        .output()
        .map_err(|err| format!("failed to run status --json: {err}"))?;

    assert!(
        output.status.success(),
        "status --json should succeed, got {:?}",
        output.status.code()
    );
    let stdout = String::from_utf8(output.stdout)
        .map_err(|err| format!("stdout utf8 decode failed: {err}"))?;
    assert!(stdout.contains("\"queried_via\""));
    assert!(stdout.contains("\"member_id\": \"node-a\""));
    assert!(stdout.contains("\"health\": \"healthy\""));
    Ok(())
}

#[test]
fn state_command_with_config_only_maps_connection_refused_to_exit_3() -> Result<(), String> {
    let bin = cli_bin_path()?;
    let listener =
        std::net::TcpListener::bind("127.0.0.1:0").map_err(|err| format!("bind failed: {err}"))?;
    let addr = listener
        .local_addr()
        .map_err(|err| format!("local_addr failed: {err}"))?;
    drop(listener);

    let path = write_temp_config(
        "pgtm-config-only-status",
        pgtm_config_toml(
            &addr.to_string(),
            r#"{ tls = { mode = "disabled" }, auth = { type = "disabled" } }"#,
            &format!("[pgtm]\napi_url = \"http://{addr}\"\n"),
        )
        .as_str(),
    )?;

    let output = Command::new(&bin)
        .args([
            "-c",
            path.to_string_lossy().as_ref(),
            "--timeout-ms",
            "50",
            "status",
        ])
        .output()
        .map_err(|err| format!("failed to run state command: {err}"))?;
    let _ = std::fs::remove_file(path);

    assert_eq!(output.status.code(), Some(3));
    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(stderr.contains("transport error"));
    Ok(())
}

#[test]
fn status_command_with_unusable_derived_api_target_exits_6() -> Result<(), String> {
    let bin = cli_bin_path()?;
    let path = write_temp_config(
        "pgtm-config-unusable-derive",
        pgtm_config_toml(
            "0.0.0.0:8080",
            r#"{ tls = { mode = "disabled" }, auth = { type = "disabled" } }"#,
            "",
        )
        .as_str(),
    )?;

    let output = Command::new(&bin)
        .args(["-c", path.to_string_lossy().as_ref(), "status"])
        .output()
        .map_err(|err| format!("failed to run state command: {err}"))?;
    let _ = std::fs::remove_file(path);

    assert_eq!(output.status.code(), Some(6));
    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(stderr.contains("pgtm.api_url"));
    Ok(())
}

#[test]
fn switchover_request_uses_admin_token_from_config() -> Result<(), String> {
    let bin = cli_bin_path()?;
    let (addr, rx) = spawn_single_request_server(
        "HTTP/1.1 202 Accepted\r\ncontent-type: application/json\r\ncontent-length: 17\r\n\r\n{\"accepted\":true}",
    )?;
    let path = write_temp_config(
        "pgtm-config-admin-auth",
        pgtm_config_toml(
            &addr.to_string(),
            r#"{ tls = { mode = "disabled" }, auth = { type = "role_tokens", admin_token = { content = "admin-token" } } }"#,
            &format!(
                "[pgtm]\napi_url = \"http://{addr}\"\n"
            ),
        )
        .as_str(),
    )?;

    let output = Command::new(&bin)
        .args([
            "-c",
            path.to_string_lossy().as_ref(),
            "switchover",
            "request",
        ])
        .output()
        .map_err(|err| format!("failed to run switchover request: {err}"))?;
    let _ = std::fs::remove_file(path);

    assert!(
        output.status.success(),
        "switchover request should succeed, got {:?}",
        output.status.code()
    );
    let request = rx
        .recv_timeout(std::time::Duration::from_secs(2))
        .map_err(|err| format!("failed to receive request: {err}"))?;
    assert!(
        request.contains("Authorization: Bearer admin-token")
            || request.contains("authorization: Bearer admin-token"),
        "request should carry admin token, got: {request}"
    );
    Ok(())
}

#[test]
fn status_command_reports_missing_env_backed_token() -> Result<(), String> {
    let bin = cli_bin_path()?;
    let path = write_temp_config(
        "pgtm-config-missing-env-token",
        pgtm_config_toml(
            "127.0.0.1:8080",
            r#"{ tls = { mode = "disabled" }, auth = { type = "role_tokens", read_token = { env = "PGTM_TEST_MISSING_READ_TOKEN" } } }"#,
            "[pgtm]\napi_url = \"http://127.0.0.1:8080\"\n",
        )
        .as_str(),
    )?;

    let output = Command::new(&bin)
        .env_remove("PGTM_TEST_MISSING_READ_TOKEN")
        .args(["-c", path.to_string_lossy().as_ref(), "status"])
        .output()
        .map_err(|err| format!("failed to run status command: {err}"))?;
    let _ = std::fs::remove_file(path);

    assert_eq!(output.status.code(), Some(6));
    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(stderr.contains("PGTM_TEST_MISSING_READ_TOKEN"));
    Ok(())
}

#[test]
fn node_help_exits_success() -> Result<(), String> {
    let bin = node_bin_path()?;
    let output = Command::new(&bin)
        .arg("--help")
        .output()
        .map_err(|err| format!("failed to run node --help: {err}"))?;

    assert!(
        output.status.success(),
        "--help should exit successfully, got {:?}",
        output.status.code()
    );

    let stdout = String::from_utf8(output.stdout)
        .map_err(|err| format!("stdout utf8 decode failed: {err}"))?;
    assert!(
        stdout.contains("--config"),
        "help output should include --config option"
    );
    Ok(())
}

#[test]
fn node_missing_incomplete_config_reports_parse_error() -> Result<(), String> {
    let bin = node_bin_path()?;
    let path = write_temp_config(
        "missing-config-version",
        r#"
[cluster]
name = "cluster-a"
member_id = "member-a"
"#,
    )?;

    let output = Command::new(&bin)
        .args(["--config", path.to_string_lossy().as_ref()])
        .output()
        .map_err(|err| format!("failed to run node with incomplete config: {err}"))?;

    assert_eq!(
        output.status.code(),
        Some(1),
        "invalid configs should exit with code 1"
    );

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("failed to parse config file"),
        "stderr should include parse failure details, got: {stderr}"
    );

    let _ = std::fs::remove_file(path);
    Ok(())
}

#[test]
fn node_missing_secure_field_prints_stable_field_path() -> Result<(), String> {
    let bin = node_bin_path()?;
    let path = write_temp_config(
        "missing-process-binaries",
        r#"
[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
pg_rewind_timeout_ms = 120000
bootstrap_timeout_ms = 300000
fencing_timeout_ms = 30000

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#,
    )?;

    let output = Command::new(&bin)
        .args(["--config", path.to_string_lossy().as_ref()])
        .output()
        .map_err(|err| format!("failed to run node with invalid config: {err}"))?;

    assert_eq!(
        output.status.code(),
        Some(1),
        "invalid configs should exit with code 1"
    );

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("`process.binaries`"),
        "stderr should mention stable field path, got: {stderr}"
    );

    let _ = std::fs::remove_file(path);
    Ok(())
}

#[test]
fn node_rejects_postgres_role_tls_auth_with_stable_field_path() -> Result<(), String> {
    let bin = node_bin_path()?;
    let path = write_temp_config(
        "postgres-role-tls-auth",
        r#"
[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "prefer" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "tls" } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#,
    )?;

    let output = Command::new(&bin)
        .args(["--config", path.to_string_lossy().as_ref()])
        .output()
        .map_err(|err| format!("failed to run node with invalid config: {err}"))?;

    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("`postgres.roles.superuser.auth`"),
        "stderr should mention stable field path, got: {stderr}"
    );

    let _ = std::fs::remove_file(path);
    Ok(())
}

#[test]
fn node_rejects_ssl_mode_requiring_tls_when_postgres_tls_disabled() -> Result<(), String> {
    let bin = node_bin_path()?;
    let path = write_temp_config(
        "postgres-ssl-mode-requires-tls",
        r#"
[cluster]
name = "cluster-a"
member_id = "member-a"

[postgres]
data_dir = "/var/lib/postgresql/data"
listen_host = "127.0.0.1"
listen_port = 5432
socket_dir = "/tmp/pgtuskmaster/socket"
log_file = "/tmp/pgtuskmaster/postgres.log"
local_conn_identity = { user = "postgres", dbname = "postgres", ssl_mode = "require" }
rewind_conn_identity = { user = "rewinder", dbname = "postgres", ssl_mode = "prefer" }
tls = { mode = "disabled" }
roles = { superuser = { username = "postgres", auth = { type = "password", password = { content = "secret-password" } } }, replicator = { username = "replicator", auth = { type = "password", password = { content = "secret-password" } } }, rewinder = { username = "rewinder", auth = { type = "password", password = { content = "secret-password" } } } }
pg_hba = { source = { content = "local all all trust" } }
pg_ident = { source = { content = "empty" } }

[dcs]
endpoints = ["http://127.0.0.1:2379"]
scope = "scope-a"

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process]
binaries = { postgres = "/usr/bin/postgres", pg_ctl = "/usr/bin/pg_ctl", pg_rewind = "/usr/bin/pg_rewind", initdb = "/usr/bin/initdb", pg_basebackup = "/usr/bin/pg_basebackup", psql = "/usr/bin/psql" }

[api]
security = { tls = { mode = "disabled" }, auth = { type = "disabled" } }
"#,
    )?;

    let output = Command::new(&bin)
        .args(["--config", path.to_string_lossy().as_ref()])
        .output()
        .map_err(|err| format!("failed to run node with invalid config: {err}"))?;

    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("`postgres.local_conn_identity.ssl_mode`"),
        "stderr should mention stable field path, got: {stderr}"
    );

    let _ = std::fs::remove_file(path);
    Ok(())
}
