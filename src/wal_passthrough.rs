use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::api::events::WalEventIngestInput;

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum WalPassthroughError {
    #[error("{0}")]
    Message(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WalPassthroughKind {
    ArchivePush { wal_path: String },
    ArchiveGet {
        wal_segment: String,
        destination_path: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WalPassthroughExit {
    pub code: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CaptureResult {
    bytes: Vec<u8>,
    truncated: bool,
}

struct CaptureConfig {
    max_bytes: usize,
}

fn now_millis() -> Result<u64, WalPassthroughError> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| WalPassthroughError::Message(format!("system clock before unix epoch: {err}")))?;
    u64::try_from(elapsed.as_millis())
        .map_err(|err| WalPassthroughError::Message(format!("millis conversion failed: {err}")))
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len().saturating_mul(2));
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn invocation_id(kind: &str, wal_a: &str, wal_b: Option<&str>) -> Result<String, WalPassthroughError> {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    let millis = now_millis()?;
    hasher.update(millis.to_le_bytes());
    hasher.update(std::process::id().to_le_bytes());
    hasher.update(kind.as_bytes());
    hasher.update(wal_a.as_bytes());
    if let Some(extra) = wal_b {
        hasher.update(extra.as_bytes());
    }
    Ok(hex_encode(&hasher.finalize()[..16]))
}

fn capture_stream(
    mut reader: impl Read + Send + 'static,
    mut writer: impl Write + Send + 'static,
    cfg: CaptureConfig,
) -> std::thread::JoinHandle<Result<CaptureResult, WalPassthroughError>> {
    std::thread::spawn(move || {
        let mut captured: Vec<u8> = Vec::new();
        let mut truncated = false;
        let mut buf = [0_u8; 8192];
        loop {
            let n = match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(err) => {
                    return Err(WalPassthroughError::Message(format!(
                        "stream read failed: {err}"
                    )));
                }
            };

            if let Err(err) = writer.write_all(&buf[..n]) {
                // Best-effort forwarding: do not abort passthrough due to local stdout/stderr issues.
                let _ = err;
            }

            if captured.len() < cfg.max_bytes {
                let remaining = cfg.max_bytes.saturating_sub(captured.len());
                let take = remaining.min(n);
                captured.extend_from_slice(&buf[..take]);
                if take < n {
                    truncated = true;
                }
            } else {
                truncated = true;
            }
        }
        Ok(CaptureResult {
            bytes: captured,
            truncated,
        })
    })
}

fn map_exit_status(status: ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        if (0..=255).contains(&code) {
            return code;
        }
        return 1;
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            let mapped = 128_i32.saturating_add(signal);
            if (0..=255).contains(&mapped) {
                return mapped;
            }
        }
    }

    1
}

fn best_effort_post_json<T: Serialize>(
    url: &str,
    token: Option<&str>,
    body: &T,
) -> Result<(), WalPassthroughError> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(250))
        .build()
        .map_err(|err| WalPassthroughError::Message(format!("http client build failed: {err}")))?;

    let mut request = client.post(url).json(body);
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    let response = request
        .send()
        .map_err(|err| WalPassthroughError::Message(format!("http request failed: {err}")))?;
    if !response.status().is_success() {
        return Err(WalPassthroughError::Message(format!(
            "http response status was {}",
            response.status()
        )));
    }
    Ok(())
}

pub fn run(pgdata: &Path, kind: WalPassthroughKind) -> Result<WalPassthroughExit, WalPassthroughError> {
    if pgdata.as_os_str().is_empty() {
        return Err(WalPassthroughError::Message("pgdata must not be empty".to_string()));
    }

    let (event_kind, wal_path, wal_segment, destination_path, rendered) = match kind {
        WalPassthroughKind::ArchivePush { wal_path } => {
            let rendered = crate::wal::render_archive_push(pgdata, wal_path.as_str())
                .map_err(|err| WalPassthroughError::Message(err.to_string()))?;
            (
                "archive-push",
                Some(wal_path),
                None,
                None,
                rendered,
            )
        }
        WalPassthroughKind::ArchiveGet {
            wal_segment,
            destination_path,
        } => {
            let rendered = crate::wal::render_archive_get(
                pgdata,
                wal_segment.as_str(),
                destination_path.as_str(),
            )
                .map_err(|err| WalPassthroughError::Message(err.to_string()))?;
            (
                "archive-get",
                None,
                Some(wal_segment),
                Some(destination_path),
                rendered,
            )
        }
    };

    if !rendered.program.is_absolute() {
        return Err(WalPassthroughError::Message(format!(
            "rendered program must be an absolute path, got `{}`",
            rendered.program.display()
        )));
    }

    let helper_cfg = crate::backup::archive_command::load_archive_command_config(pgdata)
        .map_err(|err| WalPassthroughError::Message(err.to_string()))?;

    let invocation_id = invocation_id(
        event_kind,
        wal_path.as_deref().unwrap_or(""),
        wal_segment.as_deref(),
    )?;
    let started_at_ms = now_millis()?;

    let mut command = Command::new(&rendered.program);
    command
        .args(&rendered.args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|err| {
        WalPassthroughError::Message(format!(
            "spawn failed for {}: {err}",
            rendered.program.display()
        ))
    })?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| WalPassthroughError::Message("child stdout pipe missing".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| WalPassthroughError::Message("child stderr pipe missing".to_string()))?;

    let stdout_handle = capture_stream(stdout, std::io::stdout(), CaptureConfig { max_bytes: 64 * 1024 });
    let stderr_handle = capture_stream(stderr, std::io::stderr(), CaptureConfig { max_bytes: 64 * 1024 });

    let status = child
        .wait()
        .map_err(|err| WalPassthroughError::Message(format!("wait failed: {err}")))?;

    let stdout_capture = match stdout_handle.join() {
        Ok(value) => value?,
        Err(_) => {
            return Err(WalPassthroughError::Message(
                "stdout capture thread panicked".to_string(),
            ));
        }
    };
    let stderr_capture = match stderr_handle.join() {
        Ok(value) => value?,
        Err(_) => {
            return Err(WalPassthroughError::Message(
                "stderr capture thread panicked".to_string(),
            ));
        }
    };

    let ended_at_ms = now_millis()?;
    let duration_ms = ended_at_ms.saturating_sub(started_at_ms).max(1);
    let status_code = map_exit_status(status);
    let success = status_code == 0;

    let stdout = String::from_utf8_lossy(stdout_capture.bytes.as_slice()).to_string();
    let stderr = String::from_utf8_lossy(stderr_capture.bytes.as_slice()).to_string();

    let payload = WalEventIngestInput {
        provider: "pgbackrest".to_string(),
        event_kind: event_kind.to_string(),
        invocation_id,
        status_code,
        success,
        started_at_ms,
        duration_ms,
        stdout,
        stderr,
        stdout_truncated: stdout_capture.truncated,
        stderr_truncated: stderr_capture.truncated,
        wal_path,
        wal_segment,
        destination_path,
        command_program: rendered.program.display().to_string(),
        command_args: rendered.args.clone(),
    };

    let url = format!("http://{}/events/wal", helper_cfg.api_local_addr);
    let token = helper_cfg.api_token.as_deref();
    if let Err(err) = best_effort_post_json(url.as_str(), token, &payload) {
        eprintln!("wal event emit failed: {err}");
    }

    let code_u8 = if (0..=255).contains(&status_code) {
        status_code as u8
    } else {
        1
    };
    Ok(WalPassthroughExit { code: code_u8 })
}
