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
use crate::test_harness::ports::allocate_ports;

pub(crate) async fn run_with_local_set<F, T>(future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    tokio::task::LocalSet::new().run_until(future).await
}

pub(crate) fn http_timeout_ms(timeout: Duration) -> Result<u64, WorkerError> {
    u64::try_from(timeout.as_millis())
        .map_err(|_| WorkerError::Message("http timeout does not fit into u64".to_string()))
}

pub(crate) async fn wait_for_node_api_ready_or_task_exit(
    node_addr: SocketAddr,
    node_id: &str,
    postgres_log_file: &Path,
    task: &mut JoinHandle<Result<(), WorkerError>>,
    http_step_timeout: Duration,
    timeout: Duration,
) -> Result<(), WorkerError> {
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
                    read_log_tail(postgres_log_file, 40)
                ))),
            };
        }

        let observation = match client.get_ha_state().await {
            Ok(_) => return Ok(()),
            Err(err) => err.to_string(),
        };

        if tokio::time::Instant::now() >= deadline {
            return Err(WorkerError::Message(format!(
                "timed out waiting for api readiness for {node_id} at {node_addr}; last_observation={observation}; postgres_log_tail={}",
                read_log_tail(postgres_log_file, 40)
            )));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

pub(crate) fn read_log_tail(path: &Path, max_lines: usize) -> String {
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

pub(crate) async fn fetch_ha_state_via_tcp(
    node_addr: SocketAddr,
    http_step_timeout: Duration,
) -> Result<HaStateResponse, WorkerError> {
    let mut stream = match tokio::time::timeout(
        http_step_timeout,
        tokio::net::TcpStream::connect(node_addr),
    )
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

pub(crate) async fn wait_for_bootstrap_primary(
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
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

pub(crate) fn unix_now() -> Result<UnixMillis, WorkerError> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| WorkerError::Message(format!("system time before epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| WorkerError::Message(format!("millis conversion failed: {err}")))?;
    Ok(UnixMillis(millis))
}

pub(crate) async fn pg_ctl_stop_immediate(
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
    let status = wait_for_child_exit_with_timeout(
        &label,
        &mut child,
        command_timeout,
        command_kill_wait_timeout,
    )
    .await?;

    if status.success() || !pid_file.exists() {
        Ok(())
    } else {
        Err(WorkerError::Message(format!(
            "pg_ctl stop exited unsuccessfully with status {status} for {}",
            data_dir.display()
        )))
    }
}

pub(crate) async fn wait_for_child_exit_with_timeout(
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

pub(crate) async fn run_psql_statement(
    psql: &Path,
    port: u16,
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
        .arg("postgres")
        .arg("-d")
        .arg("postgres")
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

pub(crate) fn parse_psql_rows(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect()
}

pub(crate) fn parse_single_u64(output: &str) -> Result<u64, WorkerError> {
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

pub(crate) fn parse_loopback_socket(port: u16) -> Result<SocketAddr, WorkerError> {
    format!("127.0.0.1:{port}")
        .parse::<SocketAddr>()
        .map_err(|err| WorkerError::Message(format!("parse socket failed for port={port}: {err}")))
}

pub(crate) fn parse_http_endpoint(endpoint: &str) -> Result<SocketAddr, WorkerError> {
    let host_port = endpoint.strip_prefix("http://").ok_or_else(|| {
        WorkerError::Message(format!(
            "unsupported endpoint format for proxy target: {endpoint}"
        ))
    })?;
    host_port.parse::<SocketAddr>().map_err(|err| {
        WorkerError::Message(format!("parse endpoint socket failed: {endpoint} ({err})"))
    })
}

pub(crate) fn allocate_non_overlapping_ports(
    count: usize,
    forbidden: &BTreeSet<u16>,
) -> Result<Vec<u16>, WorkerError> {
    if count == 0 {
        return Ok(Vec::new());
    }

    for _attempt in 0..30 {
        let candidate = allocate_ports(count)?.into_vec();
        let overlaps = candidate.iter().any(|port| forbidden.contains(port));
        if !overlaps {
            return Ok(candidate);
        }
    }

    Err(WorkerError::Message(format!(
        "failed to allocate {count} non-overlapping ports after retries"
    )))
}

pub(crate) async fn get_ha_state_with_fallback(
    client: &CliApiClient,
    node_id: &str,
    fallback_tcp_addr: SocketAddr,
    http_step_timeout: Duration,
) -> Result<HaStateResponse, WorkerError> {
    let primary_result =
        tokio::time::timeout(http_step_timeout, client.get_ha_state()).await;

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
