use std::collections::BTreeSet;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::process::{ExitStatus, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::task::JoinHandle;

use crate::cli::client::{CliApiClient, HaStateResponse};
use crate::cli::error::CliError;
use crate::state::{UnixMillis, WorkerError};
use crate::test_harness::ports::{allocate_ports, PortReservation};

const LOG_TAIL_LINE_LIMIT: usize = 40;
const API_READY_POLL_INTERVAL: Duration = Duration::from_millis(100);
const FORCE_KILL_GRACE_PERIOD: Duration = Duration::from_millis(200);
const NON_OVERLAPPING_PORT_ALLOCATION_ATTEMPTS: usize = 30;

pub async fn run_with_local_set<F, T>(future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    tokio::task::LocalSet::new().run_until(future).await
}

pub fn http_timeout_ms(timeout: Duration) -> Result<u64, WorkerError> {
    u64::try_from(timeout.as_millis())
        .map_err(|_| WorkerError::Message("http timeout does not fit into u64".to_string()))
}

pub async fn wait_for_node_api_ready_or_task_exit(
    node_addr: SocketAddr,
    node_id: &str,
    postgres_log_file: &Path,
    task: JoinHandle<Result<(), WorkerError>>,
    http_step_timeout: Duration,
    timeout: Duration,
) -> Result<JoinHandle<Result<(), WorkerError>>, WorkerError> {
    let deadline = tokio::time::Instant::now() + timeout;
    let timeout_ms = http_timeout_ms(http_step_timeout)?;
    let client = CliApiClient::new(format!("http://{node_addr}"), timeout_ms, None, None).map_err(
        |err| {
            WorkerError::Message(format!(
                "build CliApiClient failed for api readiness probe on {node_id}: {err}"
            ))
        },
    )?;

    loop {
        if task.is_finished() {
            let joined = task.await.map_err(|err| {
                WorkerError::Message(format!("runtime task join failed for {node_id}: {err}"))
            })?;
            return match joined {
                Ok(()) => Err(WorkerError::Message(format!(
                    "runtime task exited unexpectedly for {node_id} before API became ready"
                ))),
                Err(err) => Err(WorkerError::Message(format!(
                    "runtime task failed for {node_id} before API became ready: {err}; postgres_log_tail={}",
                    read_log_tail(postgres_log_file, LOG_TAIL_LINE_LIMIT)
                ))),
            };
        }

        let observation = match client.get_ha_state().await {
            Ok(_) => return Ok(task),
            Err(err) => err.to_string(),
        };

        if tokio::time::Instant::now() >= deadline {
            task.abort();
            let _ = task.await;
            return Err(WorkerError::Message(format!(
                "timed out waiting for api readiness for {node_id} at {node_addr}; last_observation={observation}; postgres_log_tail={}",
                read_log_tail(postgres_log_file, LOG_TAIL_LINE_LIMIT)
            )));
        }
        tokio::time::sleep(API_READY_POLL_INTERVAL).await;
    }
}

pub fn read_log_tail(path: &Path, max_lines: usize) -> String {
    let content = match fs::read_to_string(path) {
        Ok(value) => value,
        Err(err) => return format!("log-read-failed: {err}"),
    };
    let mut lines = content.lines().collect::<Vec<_>>();
    if lines.is_empty() {
        return "empty".to_string();
    }
    if lines.len() > max_lines {
        let start = lines.len().saturating_sub(max_lines);
        lines = lines[start..].to_vec();
    }
    lines.join(" | ")
}

pub async fn fetch_ha_state_via_tcp(
    node_addr: SocketAddr,
    http_step_timeout: Duration,
) -> Result<HaStateResponse, WorkerError> {
    let mut stream =
        match tokio::time::timeout(http_step_timeout, tokio::net::TcpStream::connect(node_addr))
            .await
        {
            Ok(Ok(stream)) => stream,
            Ok(Err(err)) => {
                return Err(WorkerError::Message(format!(
                    "fallback connect to {node_addr} failed: {err}"
                )));
            }
            Err(_) => {
                return Err(WorkerError::Message(format!(
                    "fallback connect to {node_addr} timed out after {}s",
                    http_step_timeout.as_secs()
                )));
            }
        };

    let request =
        format!("GET /ha/state HTTP/1.1\r\nHost: {node_addr}\r\nConnection: close\r\n\r\n");
    match tokio::time::timeout(http_step_timeout, stream.write_all(request.as_bytes())).await {
        Ok(Ok(())) => {}
        Ok(Err(err)) => {
            return Err(WorkerError::Message(format!(
                "fallback write request to {node_addr} failed: {err}"
            )));
        }
        Err(_) => {
            return Err(WorkerError::Message(format!(
                "fallback write request to {node_addr} timed out after {}s",
                http_step_timeout.as_secs()
            )));
        }
    }

    let mut raw = Vec::new();
    match tokio::time::timeout(http_step_timeout, stream.read_to_end(&mut raw)).await {
        Ok(Ok(_)) => {}
        Ok(Err(err)) => {
            return Err(WorkerError::Message(format!(
                "fallback read response from {node_addr} failed: {err}"
            )));
        }
        Err(_) => {
            return Err(WorkerError::Message(format!(
                "fallback read response from {node_addr} timed out after {}s",
                http_step_timeout.as_secs()
            )));
        }
    }

    let (status_code, body) = parse_raw_http_response(raw.as_slice())?;
    if status_code != 200 {
        let body_text = String::from_utf8_lossy(body);
        return Err(WorkerError::Message(format!(
            "fallback GET /ha/state returned status {status_code} body={}",
            body_text.trim()
        )));
    }

    serde_json::from_slice::<HaStateResponse>(body)
        .map_err(|err| WorkerError::Message(format!("fallback decode /ha/state failed: {err}")))
}

fn parse_raw_http_response(raw: &[u8]) -> Result<(u16, &[u8]), WorkerError> {
    let status_line_end = raw
        .windows(2)
        .position(|window| window == b"\r\n")
        .ok_or_else(|| WorkerError::Message("fallback response missing status line".to_string()))?;
    let status_line = String::from_utf8_lossy(&raw[..status_line_end]);
    let mut parts = status_line.split_whitespace();
    let _http_version = parts.next().ok_or_else(|| {
        WorkerError::Message("fallback response missing http version".to_string())
    })?;
    let status_code = parts
        .next()
        .ok_or_else(|| WorkerError::Message("fallback response missing status code".to_string()))?
        .parse::<u16>()
        .map_err(|err| {
            WorkerError::Message(format!("fallback response status parse failed: {err}"))
        })?;

    let header_end = raw
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| {
            WorkerError::Message("fallback response missing header/body boundary".to_string())
        })?;
    let body_start = header_end.checked_add(4).ok_or_else(|| {
        WorkerError::Message("fallback response body offset overflow".to_string())
    })?;
    let body = raw.get(body_start..).ok_or_else(|| {
        WorkerError::Message("fallback response body offset out of bounds".to_string())
    })?;
    Ok((status_code, body))
}

pub async fn wait_for_bootstrap_primary(
    node_addr: SocketAddr,
    expected_member_id: &str,
    http_step_timeout: Duration,
    timeout: Duration,
) -> Result<(), WorkerError> {
    let deadline = tokio::time::Instant::now() + timeout;
    let timeout_ms = http_timeout_ms(http_step_timeout)?;
    let client = CliApiClient::new(format!("http://{node_addr}"), timeout_ms, None, None).map_err(
        |err| {
            WorkerError::Message(format!(
                "build CliApiClient failed for bootstrap probe on {expected_member_id}: {err}"
            ))
        },
    )?;

    loop {
        let observation = match client.get_ha_state().await {
            Ok(state) => {
                let is_expected_primary =
                    state.self_member_id == expected_member_id && state.ha_phase == "Primary";
                if is_expected_primary {
                    return Ok(());
                }
                let leader = state.leader.as_deref().unwrap_or("none");
                format!(
                    "member={} phase={} leader={leader}",
                    state.self_member_id, state.ha_phase
                )
            }
            Err(err) => err.to_string(),
        };

        if tokio::time::Instant::now() >= deadline {
            return Err(WorkerError::Message(format!(
                "timed out waiting for bootstrap primary {expected_member_id} at {node_addr}; last_observation={observation}"
            )));
        }
        tokio::time::sleep(API_READY_POLL_INTERVAL).await;
    }
}

pub async fn wait_for_node_api_unavailable(
    node_addr: SocketAddr,
    node_id: &str,
    http_step_timeout: Duration,
    timeout: Duration,
) -> Result<(), WorkerError> {
    let deadline = tokio::time::Instant::now() + timeout;
    let timeout_ms = http_timeout_ms(http_step_timeout)?;
    let client = CliApiClient::new(format!("http://{node_addr}"), timeout_ms, None, None).map_err(
        |err| {
            WorkerError::Message(format!(
                "build CliApiClient failed for api unavailability probe on {node_id}: {err}"
            ))
        },
    )?;

    loop {
        match client.get_ha_state().await {
            Ok(state) => {
                if tokio::time::Instant::now() >= deadline {
                    return Err(WorkerError::Message(format!(
                        "timed out waiting for api unavailability for {node_id} at {node_addr}; last_observation=member={} phase={} leader={:?}",
                        state.self_member_id,
                        state.ha_phase,
                        state.leader
                    )));
                }
            }
            Err(_) => return Ok(()),
        }

        tokio::time::sleep(API_READY_POLL_INTERVAL).await;
    }
}

pub fn unix_now() -> Result<UnixMillis, WorkerError> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| WorkerError::Message(format!("system time before epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| WorkerError::Message(format!("millis conversion failed: {err}")))?;
    Ok(UnixMillis(millis))
}

pub async fn pg_ctl_stop_immediate(
    pg_ctl: &Path,
    data_dir: &Path,
    command_timeout: Duration,
    command_kill_wait_timeout: Duration,
) -> Result<(), WorkerError> {
    let pid_file = data_dir.join("postmaster.pid");
    if !pid_file.exists() {
        return Ok(());
    }

    let mut child = Command::new(pg_ctl)
        .arg("-D")
        .arg(data_dir)
        .arg("stop")
        .arg("-m")
        .arg("immediate")
        .arg("-w")
        .spawn()
        .map_err(|err| WorkerError::Message(format!("pg_ctl stop spawn failed: {err}")))?;
    let label = format!("pg_ctl stop for {}", data_dir.display());
    let wait_result = wait_for_child_exit_with_timeout(
        &label,
        &mut child,
        command_timeout,
        command_kill_wait_timeout,
    )
    .await;

    match wait_result {
        Ok(status) => {
            if status.success() || !pid_file.exists() {
                return Ok(());
            }
        }
        Err(_) => {
            // Continue to fallback kill path below.
        }
    }

    let pid = read_postmaster_pid(&pid_file)?;
    force_kill_postmaster_pid(pid, &label).await?;
    if pid_is_alive(pid, &label).await? {
        Err(WorkerError::Message(format!(
            "{label} postgres pid {pid} still alive after fallback kill"
        )))
    } else {
        Ok(())
    }
}

fn read_postmaster_pid(pid_file: &Path) -> Result<u32, WorkerError> {
    let contents = fs::read_to_string(pid_file).map_err(|err| {
        WorkerError::Message(format!(
            "read postmaster.pid failed for {}: {err}",
            pid_file.display()
        ))
    })?;
    let first_line = contents
        .lines()
        .next()
        .ok_or_else(|| WorkerError::Message("postmaster.pid missing pid line".to_string()))?;
    let trimmed = first_line.trim();
    if trimmed.is_empty() {
        return Err(WorkerError::Message(
            "postmaster.pid pid line is empty".to_string(),
        ));
    }
    trimmed.parse::<u32>().map_err(|err| {
        WorkerError::Message(format!("parse postmaster pid '{trimmed}' failed: {err}"))
    })
}

async fn kill_best_effort(pid: u32, signal: &str, label: &str) -> Result<(), WorkerError> {
    #[cfg(unix)]
    const SIGTERM: i32 = libc::SIGTERM;
    #[cfg(not(unix))]
    const SIGTERM: i32 = 15;

    #[cfg(unix)]
    const SIGKILL: i32 = libc::SIGKILL;
    #[cfg(not(unix))]
    const SIGKILL: i32 = 9;

    let signal_num = match signal {
        "TERM" => SIGTERM,
        "KILL" => SIGKILL,
        other => {
            return Err(WorkerError::Message(format!(
                "{label} unsupported signal '{other}' for pid={pid}"
            )));
        }
    };

    crate::test_harness::signals::send_signal(pid, signal_num).map_err(|err| {
        WorkerError::Message(format!(
            "{label} kill -{signal} failed for pid={pid}: {err}"
        ))
    })
}

async fn pid_is_alive(pid: u32, label: &str) -> Result<bool, WorkerError> {
    crate::test_harness::signals::pid_exists(pid)
        .map_err(|err| WorkerError::Message(format!("{label} kill -0 failed for pid={pid}: {err}")))
}

async fn force_kill_postmaster_pid(pid: u32, label: &str) -> Result<(), WorkerError> {
    let _ = kill_best_effort(pid, "TERM", label).await;
    tokio::time::sleep(FORCE_KILL_GRACE_PERIOD).await;

    if !pid_is_alive(pid, label).await? {
        return Ok(());
    }

    let _ = kill_best_effort(pid, "KILL", label).await;
    tokio::time::sleep(FORCE_KILL_GRACE_PERIOD).await;
    if pid_is_alive(pid, label).await? {
        Err(WorkerError::Message(format!(
            "{label} postgres pid {pid} still alive after kill"
        )))
    } else {
        Ok(())
    }
}

pub async fn wait_for_child_exit_with_timeout(
    label: &str,
    child: &mut Child,
    timeout: Duration,
    kill_wait_timeout: Duration,
) -> Result<ExitStatus, WorkerError> {
    match tokio::time::timeout(timeout, child.wait()).await {
        Ok(wait_result) => {
            wait_result.map_err(|err| WorkerError::Message(format!("{label} wait failed: {err}")))
        }
        Err(_) => {
            child.start_kill().map_err(|err| {
                WorkerError::Message(format!(
                    "{label} timed out after {}s and kill failed: {err}",
                    timeout.as_secs()
                ))
            })?;
            match tokio::time::timeout(kill_wait_timeout, child.wait()).await {
                Ok(Ok(_)) | Ok(Err(_)) | Err(_) => {}
            }
            Err(WorkerError::Message(format!(
                "{label} timed out after {}s and was killed",
                timeout.as_secs()
            )))
        }
    }
}

pub async fn run_psql_statement(
    psql: &Path,
    port: u16,
    user: &str,
    dbname: &str,
    sql: &str,
    command_timeout: Duration,
    command_kill_wait_timeout: Duration,
) -> Result<String, WorkerError> {
    let mut command = Command::new(psql);
    command
        .arg("-h")
        .arg("127.0.0.1")
        .arg("-p")
        .arg(port.to_string())
        .arg("-U")
        .arg(user)
        .arg("-d")
        .arg(dbname)
        .arg("-v")
        .arg("ON_ERROR_STOP=1")
        .arg("-AXqt")
        .arg("-c")
        .arg(sql)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .map_err(|err| WorkerError::Message(format!("psql spawn failed: {err}")))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| WorkerError::Message("psql stdout pipe unavailable".to_string()))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| WorkerError::Message("psql stderr pipe unavailable".to_string()))?;

    let stdout_task = tokio::spawn(async move {
        let mut buffer = Vec::new();
        stdout
            .read_to_end(&mut buffer)
            .await
            .map(|_| buffer)
            .map_err(|err| WorkerError::Message(format!("psql stdout read failed: {err}")))
    });
    let stderr_task = tokio::spawn(async move {
        let mut buffer = Vec::new();
        stderr
            .read_to_end(&mut buffer)
            .await
            .map(|_| buffer)
            .map_err(|err| WorkerError::Message(format!("psql stderr read failed: {err}")))
    });

    let label = format!("psql port={port}");
    let status = wait_for_child_exit_with_timeout(
        &label,
        &mut child,
        command_timeout,
        command_kill_wait_timeout,
    )
    .await?;
    let stdout_bytes = stdout_task
        .await
        .map_err(|err| WorkerError::Message(format!("psql stdout join failed: {err}")))??;
    let stderr_bytes = stderr_task
        .await
        .map_err(|err| WorkerError::Message(format!("psql stderr join failed: {err}")))??;

    let stdout_text = String::from_utf8(stdout_bytes)
        .map_err(|err| WorkerError::Message(format!("psql stdout utf8 decode failed: {err}")))?;
    if status.success() {
        return Ok(stdout_text);
    }

    let stderr_text = String::from_utf8(stderr_bytes)
        .map_err(|err| WorkerError::Message(format!("psql stderr utf8 decode failed: {err}")))?;
    Err(WorkerError::Message(format!(
        "psql exited unsuccessfully with status {status}; stderr={}",
        stderr_text.trim()
    )))
}

pub fn parse_psql_rows(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect()
}

pub fn parse_single_u64(output: &str) -> Result<u64, WorkerError> {
    let rows = parse_psql_rows(output);
    if rows.len() != 1 {
        return Err(WorkerError::Message(format!(
            "expected one scalar row, got {} rows: {rows:?}",
            rows.len()
        )));
    }
    rows[0].parse::<u64>().map_err(|err| {
        WorkerError::Message(format!("parse scalar u64 from '{}' failed: {err}", rows[0]))
    })
}

pub fn parse_loopback_socket(port: u16) -> Result<SocketAddr, WorkerError> {
    format!("127.0.0.1:{port}")
        .parse::<SocketAddr>()
        .map_err(|err| WorkerError::Message(format!("parse socket failed for port={port}: {err}")))
}

pub fn reserve_non_overlapping_ports(
    count: usize,
    forbidden: &BTreeSet<u16>,
) -> Result<PortReservation, WorkerError> {
    if count == 0 {
        return Ok(PortReservation::empty());
    }

    for _attempt in 0..NON_OVERLAPPING_PORT_ALLOCATION_ATTEMPTS {
        let candidate = allocate_ports(count)?;
        let overlaps = candidate
            .as_slice()
            .iter()
            .any(|port| forbidden.contains(port));
        if !overlaps {
            return Ok(candidate);
        }
    }

    Err(WorkerError::Message(format!(
        "failed to allocate {count} non-overlapping ports after retries"
    )))
}

pub async fn get_ha_state_with_fallback(
    client: &CliApiClient,
    node_id: &str,
    fallback_tcp_addr: SocketAddr,
    http_step_timeout: Duration,
) -> Result<HaStateResponse, WorkerError> {
    let primary_result = tokio::time::timeout(http_step_timeout, client.get_ha_state()).await;

    match primary_result {
        Ok(Ok(state)) => Ok(state),
        Ok(Err(CliError::Transport(primary_err))) => {
            fetch_ha_state_via_tcp(fallback_tcp_addr, http_step_timeout)
                .await
                .map_err(|fallback_err| {
                    WorkerError::Message(format!(
                        "GET /ha/state failed for node {node_id}: primary_transport={primary_err}; fallback={fallback_err}"
                    ))
                })
        }
        Ok(Err(err)) => Err(WorkerError::Message(format!(
            "GET /ha/state failed for node {node_id}: {err}"
        ))),
        Err(_) => fetch_ha_state_via_tcp(fallback_tcp_addr, http_step_timeout)
            .await
            .map_err(|fallback_err| {
                WorkerError::Message(format!(
                    "GET /ha/state failed for node {node_id}: primary_timeout={}s; fallback={fallback_err}",
                    http_step_timeout.as_secs()
                ))
            }),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Stdio;
    use std::time::Duration;

    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command as TokioCommand;

    use crate::test_harness::etcd3::EtcdHandle;
    use crate::test_harness::namespace::NamespaceGuard;
    use crate::test_harness::pg16::PgHandle;

    const INNER_TEST_NAME: &str = "test_harness::ha_e2e::util::tests::kill_path_inner";

    fn find_absolute_sleep() -> Result<PathBuf, String> {
        for candidate in ["/bin/sleep", "/usr/bin/sleep"] {
            let path = Path::new(candidate);
            if path.exists() {
                return Ok(path.to_path_buf());
            }
        }
        Err("no absolute sleep binary found at /bin/sleep or /usr/bin/sleep".to_string())
    }

    #[cfg(unix)]
    fn make_executable(path: &Path) -> Result<(), String> {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(path, perms)
            .map_err(|err| format!("set_permissions failed for {}: {err}", path.display()))
    }

    #[cfg(unix)]
    #[test]
    fn kill_is_not_resolved_via_path() -> Result<(), String> {
        let namespace = NamespaceGuard::new("kill-not-path")
            .map_err(|err| format!("namespace init failed: {err}"))?;
        let ns = namespace
            .namespace()
            .map_err(|err| format!("namespace access failed: {err}"))?;

        let fake_bin_dir = ns.child_dir("fake-bin");
        fs::create_dir_all(&fake_bin_dir)
            .map_err(|err| format!("create fake bin dir failed: {err}"))?;

        let marker = ns.child_dir("kill-marker");
        if marker.exists() {
            fs::remove_file(&marker).map_err(|err| format!("remove stale marker failed: {err}"))?;
        }

        let kill_script = fake_bin_dir.join("kill");
        let script_body = format!("#!/bin/sh\nset -eu\ntouch '{}'\nexit 0\n", marker.display());
        fs::write(&kill_script, script_body)
            .map_err(|err| format!("write fake kill failed: {err}"))?;
        make_executable(&kill_script)?;

        let original_path = std::env::var("PATH").unwrap_or_default();
        let new_path = if original_path.is_empty() {
            fake_bin_dir.display().to_string()
        } else {
            format!("{}:{}", fake_bin_dir.display(), original_path)
        };

        let test_bin =
            std::env::current_exe().map_err(|err| format!("current_exe failed: {err}"))?;

        let status = std::process::Command::new(test_bin)
            .env("PATH", new_path)
            .arg("--exact")
            .arg(INNER_TEST_NAME)
            .arg("--test-threads")
            .arg("1")
            .status()
            .map_err(|err| format!("spawn inner test failed: {err}"))?;

        if !status.success() {
            return Err(format!("inner test failed with status {status}"));
        }

        if marker.exists() {
            Err("fake kill on PATH was executed".to_string())
        } else {
            Ok(())
        }
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn kill_path_inner() -> Result<(), String> {
        let sleep_path = find_absolute_sleep()?;

        // Exercise PgHandle shutdown (previously used `Command::new(\"kill\")`).
        let pg_child = TokioCommand::new(&sleep_path)
            .arg("300")
            .spawn()
            .map_err(|err| format!("spawn sleep for pg shutdown failed: {err}"))?;
        let mut pg_handle = PgHandle::new_for_test(pg_child);
        pg_handle
            .shutdown()
            .await
            .map_err(|err| format!("pg shutdown failed: {err}"))?;

        // Exercise EtcdHandle shutdown (previously used `Command::new(\"kill\")`).
        let etcd_child = TokioCommand::new(&sleep_path)
            .arg("300")
            .spawn()
            .map_err(|err| format!("spawn sleep for etcd shutdown failed: {err}"))?;
        let mut etcd_handle = EtcdHandle::new_for_test(etcd_child);
        etcd_handle
            .shutdown()
            .await
            .map_err(|err| format!("etcd shutdown failed: {err}"))?;

        // Exercise ha_e2e util fallback kill + liveness probe against a PID that we do NOT parent,
        // so we don't accidentally keep it around as a zombie until `wait()`.
        let sh_path = Path::new("/bin/sh");
        if !sh_path.exists() {
            return Err("expected /bin/sh for test helper".to_string());
        }

        let shell_script = format!("{} 300 & echo $!; wait", sleep_path.display());
        let mut wrapper = TokioCommand::new(sh_path);
        wrapper
            .arg("-c")
            .arg(shell_script)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut wrapper_child = wrapper
            .spawn()
            .map_err(|err| format!("spawn sleep wrapper failed: {err}"))?;

        let stdout = wrapper_child
            .stdout
            .take()
            .ok_or_else(|| "sleep wrapper stdout missing".to_string())?;
        let mut reader = BufReader::new(stdout);
        let mut pid_line = String::new();
        reader
            .read_line(&mut pid_line)
            .await
            .map_err(|err| format!("read pid line failed: {err}"))?;

        let pid: u32 = pid_line
            .trim()
            .parse()
            .map_err(|err| format!("parse wrapper pid '{}' failed: {err}", pid_line.trim()))?;

        let label = "kill-path-inner";
        let alive_before = super::pid_is_alive(pid, label)
            .await
            .map_err(|err| format!("pid probe before kill failed: {err}"))?;
        if !alive_before {
            return Err("expected wrapper sleep pid to be alive before kill".to_string());
        }

        super::force_kill_postmaster_pid(pid, label)
            .await
            .map_err(|err| format!("force_kill_postmaster_pid failed: {err}"))?;

        let alive_after = super::pid_is_alive(pid, label)
            .await
            .map_err(|err| format!("pid probe after kill failed: {err}"))?;
        if alive_after {
            return Err("expected pid to be gone after kill".to_string());
        }

        let wait_status = tokio::time::timeout(Duration::from_secs(5), wrapper_child.wait())
            .await
            .map_err(|_| "sleep wrapper did not exit after kill".to_string())?
            .map_err(|err| format!("sleep wrapper wait failed: {err}"))?;

        if !wait_status.success() {
            return Err(format!("sleep wrapper exited non-zero: {wait_status}"));
        }

        Ok(())
    }
}
