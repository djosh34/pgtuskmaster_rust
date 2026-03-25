use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::Command,
    sync::mpsc,
    time::{SystemTime, UNIX_EPOCH},
};

use pgtuskmaster_test_support::config_v2::render_runtime_test_config_toml;

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

fn binary_path(env_var: &str, binary_name: &str) -> Result<std::path::PathBuf, String> {
    if let Ok(path) = std::env::var(env_var) {
        return Ok(std::path::PathBuf::from(path));
    }

    let current = std::env::current_exe().map_err(|err| format!("current_exe failed: {err}"))?;
    let debug_dir = current
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| "failed to derive target/debug directory".to_string())?;
    let mut candidate = debug_dir.join(binary_name);
    if cfg!(windows) {
        candidate.set_extension("exe");
    }
    if candidate.exists() {
        Ok(candidate)
    } else {
        Err(format!(
            "{binary_name} binary not found at {}",
            candidate.display()
        ))
    }
}

fn write_temp_toml(label: &str, contents: impl AsRef<str>) -> Result<PathBuf, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| format!("system clock before unix epoch: {err}"))?
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "pgtm-{label}-{}-{timestamp}.toml",
        std::process::id()
    ));
    fs::write(&path, contents.as_ref())
        .map_err(|err| format!("write config {} failed: {err}", path.display()))?;
    Ok(path)
}

#[test]
fn help_exits_success() -> Result<(), String> {
    let bin = binary_path("CARGO_BIN_EXE_pgtm", "pgtm")?;
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
        stdout.contains("primary") && stdout.contains("replicas") && stdout.contains("switchover"),
        "help output should include connection helper and switchover commands"
    );
    assert!(
        stdout.contains("--config") && stdout.contains("-c"),
        "help output should advertise config loading"
    );
    Ok(())
}

#[test]
fn missing_required_subcommand_arg_exits_usage_code() -> Result<(), String> {
    let bin = binary_path("CARGO_BIN_EXE_pgtm", "pgtm")?;
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
fn status_command_uses_state_endpoint() -> Result<(), String> {
    let bin = binary_path("CARGO_BIN_EXE_pgtm", "pgtm")?;
    let (addr, rx) = spawn_single_request_server(
        "HTTP/1.1 503 Service Unavailable\r\ncontent-type: text/plain\r\ncontent-length: 7\r\n\r\nunready",
    )?;

    let output = Command::new(&bin)
        .args(["--base-url", &format!("http://{addr}"), "status"])
        .output()
        .map_err(|err| format!("failed to run status command: {err}"))?;

    assert_eq!(output.status.code(), Some(4));
    let request = rx
        .recv_timeout(std::time::Duration::from_secs(2))
        .map_err(|err| format!("failed to receive request: {err}"))?;
    assert!(request.starts_with("GET /state HTTP/1.1"));
    Ok(())
}

#[test]
fn switchover_clear_uses_delete_switchover_endpoint() -> Result<(), String> {
    let bin = binary_path("CARGO_BIN_EXE_pgtm", "pgtm")?;
    let (addr, rx) = spawn_single_request_server(
        "HTTP/1.1 202 Accepted\r\ncontent-type: application/json\r\ncontent-length: 17\r\n\r\n{\"accepted\":true}",
    )?;

    let output = Command::new(&bin)
        .args([
            "--base-url",
            &format!("http://{addr}"),
            "switchover",
            "clear",
        ])
        .output()
        .map_err(|err| format!("failed to run switchover clear: {err}"))?;

    assert!(
        output.status.success(),
        "switchover clear should succeed, got {:?}",
        output.status.code()
    );
    let request = rx
        .recv_timeout(std::time::Duration::from_secs(2))
        .map_err(|err| format!("failed to receive request: {err}"))?;
    assert!(request.starts_with("DELETE /switchover HTTP/1.1"));
    Ok(())
}

#[test]
fn status_auth_failure_maps_to_exit_4() -> Result<(), String> {
    let bin = binary_path("CARGO_BIN_EXE_pgtm", "pgtm")?;
    let (addr, _rx) = spawn_single_request_server(
        "HTTP/1.1 401 Unauthorized\r\ncontent-type: text/plain\r\ncontent-length: 13\r\n\r\nmissing token",
    )?;

    let output = Command::new(&bin)
        .args(["--base-url", &format!("http://{addr}"), "status"])
        .output()
        .map_err(|err| format!("failed to run status auth failure: {err}"))?;

    assert_eq!(output.status.code(), Some(4));
    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(stderr.contains("status 401"));
    Ok(())
}

#[test]
fn node_help_exits_success() -> Result<(), String> {
    let bin = binary_path("CARGO_BIN_EXE_pgtuskmaster", "pgtuskmaster")?;
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
    let bin = binary_path("CARGO_BIN_EXE_pgtuskmaster", "pgtuskmaster")?;
    let path = write_temp_toml(
        "missing-config-version",
        r#"
[cluster]
name = "cluster-a"
member_id = "member-a"
"#,
    )
    .map_err(|err| format!("write config failed: {err}"))?;

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
    let bin = binary_path("CARGO_BIN_EXE_pgtuskmaster", "pgtuskmaster")?;
    let path = write_temp_toml(
        "missing-process-binaries",
        render_runtime_test_config_toml(
            "cluster-a",
            "scope-a",
            "member-a",
            (
                Path::new("/var/lib/postgresql/data"),
                Path::new("/tmp/pgtm-socket"),
                Path::new("/tmp/pgtm.log"),
            ),
            ["http://127.0.0.1:2379"],
            [
                r#"[process.timeouts]
pg_rewind_ms = 120000
bootstrap_ms = 300000
fencing_ms = 30000"#,
                r#"[process.binaries.overrides]
initdb = "/definitely/missing/initdb"
pg_basebackup = "/definitely/missing/pg_basebackup"
pg_rewind = "/definitely/missing/pg_rewind"
pg_ctl = "/definitely/missing/pg_ctl""#,
                r#"[api]
transport = { transport = "http" }
auth = { type = "disabled" }"#,
            ],
        ),
    )
    .map_err(|err| format!("write config failed: {err}"))?;

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
fn node_rejects_empty_dcs_basic_auth_username_with_stable_field_path() -> Result<(), String> {
    let bin = binary_path("CARGO_BIN_EXE_pgtuskmaster", "pgtuskmaster")?;
    let path = write_temp_toml(
        "dcs-basic-auth-empty-username",
        render_runtime_test_config_toml(
            "cluster-a",
            "scope-a",
            "member-a",
            (
                Path::new("/var/lib/postgresql/data"),
                Path::new("/tmp/pgtm-socket"),
                Path::new("/tmp/pgtm.log"),
            ),
            ["http://127.0.0.1:2379"],
            [
                r#"[dcs.client.auth]
type = "basic"
username = ""
password = { type = "string", value = "secret-password" }"#,
                r#"[process.binaries.overrides]
initdb = "/usr/bin/initdb"
pg_basebackup = "/usr/bin/pg_basebackup"
pg_rewind = "/usr/bin/pg_rewind"
pg_ctl = "/usr/bin/pg_ctl""#,
                r#"[api]
transport = { transport = "http" }
auth = { type = "disabled" }"#,
            ],
        ),
    )
    .map_err(|err| format!("write config failed: {err}"))?;

    let output = Command::new(&bin)
        .args(["--config", path.to_string_lossy().as_ref()])
        .output()
        .map_err(|err| format!("failed to run node with invalid config: {err}"))?;

    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("`dcs.client.auth.username`"),
        "stderr should mention stable field path, got: {stderr}"
    );

    let _ = std::fs::remove_file(path);
    Ok(())
}

#[test]
fn node_rejects_https_dcs_without_tls_config() -> Result<(), String> {
    let bin = binary_path("CARGO_BIN_EXE_pgtuskmaster", "pgtuskmaster")?;
    let path = write_temp_toml(
        "https-dcs-without-client-tls",
        render_runtime_test_config_toml(
            "cluster-a",
            "scope-a",
            "member-a",
            (
                Path::new("/var/lib/postgresql/data"),
                Path::new("/tmp/pgtm-socket"),
                Path::new("/tmp/pgtm.log"),
            ),
            ["https://127.0.0.1:2379"],
            [
                r#"[process.binaries.overrides]
initdb = "/usr/bin/initdb"
pg_basebackup = "/usr/bin/pg_basebackup"
pg_rewind = "/usr/bin/pg_rewind"
pg_ctl = "/usr/bin/pg_ctl""#,
                r#"[api]
transport = { transport = "http" }
auth = { type = "disabled" }"#,
            ],
        ),
    )
    .map_err(|err| format!("write config failed: {err}"))?;

    let output = Command::new(&bin)
        .args(["--config", path.to_string_lossy().as_ref()])
        .output()
        .map_err(|err| format!("failed to run node with invalid config: {err}"))?;

    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8(output.stderr)
        .map_err(|err| format!("stderr utf8 decode failed: {err}"))?;
    assert!(
        stderr.contains("`dcs.client.tls`"),
        "stderr should mention stable field path, got: {stderr}"
    );

    let _ = std::fs::remove_file(path);
    Ok(())
}
