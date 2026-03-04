use std::path::{Path, PathBuf};

use crate::config::RuntimeConfig;

pub(crate) const DEFAULT_PGBACKREST_WAL_WRAPPER_NAME: &str =
    "pgtuskmaster-pgbackrest-wal-wrapper.sh";

pub(crate) fn ensure_pgbackrest_wal_wrapper(cfg: &RuntimeConfig) -> Result<PathBuf, String> {
    let log_file = cfg
        .logging
        .postgres
        .archive_command_log_file
        .as_ref()
        .ok_or_else(|| {
            "logging.postgres.archive_command_log_file must be configured when backup is enabled"
                .to_string()
        })?;

    let parent = log_file.parent().ok_or_else(|| {
        format!(
            "archive_command_log_file has no parent directory: {}",
            log_file.display()
        )
    })?;

    let pgbackrest_bin = cfg
        .process
        .binaries
        .pgbackrest
        .as_ref()
        .ok_or_else(|| "process.binaries.pgbackrest must be configured when backup is enabled".to_string())?;
    if pgbackrest_bin.as_os_str().is_empty() {
        return Err("process.binaries.pgbackrest must not be empty".to_string());
    }

    let pg_cfg = cfg
        .backup
        .pgbackrest
        .as_ref()
        .ok_or_else(|| "backup.pgbackrest config block is required when backup is enabled".to_string())?;
    let stanza = pg_cfg
        .stanza
        .as_deref()
        .ok_or_else(|| "backup.pgbackrest.stanza is required when backup is enabled".to_string())?;
    if stanza.trim().is_empty() {
        return Err("backup.pgbackrest.stanza must not be empty".to_string());
    }
    let repo = pg_cfg
        .repo
        .as_deref()
        .ok_or_else(|| "backup.pgbackrest.repo is required when backup is enabled".to_string())?;
    if repo.trim().is_empty() {
        return Err("backup.pgbackrest.repo must not be empty".to_string());
    }

    let pgbackrest_bin_s = pgbackrest_bin.display().to_string();
    let data_dir_s = cfg.postgres.data_dir.display().to_string();
    let log_file_s = log_file.display().to_string();
    let log_dir_s = parent.display().to_string();

    let pgbackrest_bin_q = shell_single_quote(pgbackrest_bin_s.as_str(), "process.binaries.pgbackrest")?;
    let stanza_q = shell_single_quote(stanza, "backup.pgbackrest.stanza")?;
    let repo_q = shell_single_quote(repo, "backup.pgbackrest.repo")?;
    let pg1_path_q = shell_single_quote(data_dir_s.as_str(), "postgres.data_dir")?;
    let log_file_q = shell_single_quote(
        log_file_s.as_str(),
        "logging.postgres.archive_command_log_file",
    )?;
    let log_dir_q = shell_single_quote(
        log_dir_s.as_str(),
        "logging.postgres.archive_command_log_file(parent)",
    )?;

    let push_args = quote_pgbackrest_option_tokens(
        "backup.pgbackrest.options.archive_push",
        &pg_cfg.options.archive_push,
    )?;
    let get_args = quote_pgbackrest_option_tokens(
        "backup.pgbackrest.options.archive_get",
        &pg_cfg.options.archive_get,
    )?;

    std::fs::create_dir_all(parent).map_err(|err| {
        format!(
            "failed to create archive_command_log_file parent dir {}: {err}",
            parent.display()
        )
    })?;

    let script_path = parent.join(DEFAULT_PGBACKREST_WAL_WRAPPER_NAME);
    write_pgbackrest_wal_wrapper_script(
        script_path.as_path(),
        PgbackrestWalWrapperScriptArgs {
            pgbackrest_bin: pgbackrest_bin_q.as_str(),
            stanza: stanza_q.as_str(),
            repo: repo_q.as_str(),
            pg1_path: pg1_path_q.as_str(),
            log_file: log_file_q.as_str(),
            log_dir: log_dir_q.as_str(),
            archive_push_args: push_args.as_str(),
            archive_get_args: get_args.as_str(),
        },
    )?;

    Ok(script_path)
}

fn shell_single_quote(value: &str, field: &str) -> Result<String, String> {
    if value.is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    if value.as_bytes().iter().any(|b| matches!(b, 0 | b'\n' | b'\r')) {
        return Err(format!("{field} must not contain NUL/newline characters"));
    }
    let mut out = String::with_capacity(value.len().saturating_add(2));
    out.push('\'');
    for ch in value.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    Ok(out)
}

fn quote_pgbackrest_option_tokens(field: &str, tokens: &[String]) -> Result<String, String> {
    let mut out = String::new();
    for token in tokens {
        if token.trim().is_empty() {
            return Err(format!("{field} must not contain empty tokens"));
        }
        if token
            .as_bytes()
            .iter()
            .any(|b| matches!(b, 0 | b'\n' | b'\r'))
        {
            return Err(format!("{field} must not contain NUL/newline characters"));
        }
        let quoted = shell_single_quote(token.as_str(), field)?;
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(quoted.as_str());
    }
    Ok(out)
}

struct PgbackrestWalWrapperScriptArgs<'a> {
    pgbackrest_bin: &'a str,
    stanza: &'a str,
    repo: &'a str,
    pg1_path: &'a str,
    log_file: &'a str,
    log_dir: &'a str,
    archive_push_args: &'a str,
    archive_get_args: &'a str,
}

fn write_pgbackrest_wal_wrapper_script(
    script_path: &Path,
    args: PgbackrestWalWrapperScriptArgs<'_>,
) -> Result<(), String> {
    let script = format!(
        r#"#!/bin/sh
set -eu

PGBACKREST_BIN={pgbackrest_bin}
STANZA={stanza}
REPO={repo}
PG1_PATH={pg1_path}
LOG_FILE={log_file}
LOG_DIR={log_dir}
SCHEMA_VERSION='1'
PROVIDER='pgbackrest'

json_escape() {{
  # Escape backslashes and double quotes for JSON string contexts.
  # Note: input is expected to be a single line (newlines should be normalized first).
  printf '%s' "$1" | sed -e 's/\\/\\\\/g' -e 's/\"/\\\"/g'
}}

now_ts_ms() {{
  TS_S="$(date +%s 2>/dev/null || echo 0)"
  echo $((TS_S * 1000))
}}

OP="${{1:-}}"
case "$OP" in
  archive-push)
    WAL_PATH="${{2:-}}"
    if [ -z "$WAL_PATH" ]; then
      echo "usage: $0 archive-push <wal_path>" >&2
      exit 2
    fi
    ;;
  archive-get)
    WAL_SEGMENT="${{2:-}}"
    DEST_PATH="${{3:-}}"
    if [ -z "$WAL_SEGMENT" ] || [ -z "$DEST_PATH" ]; then
      echo "usage: $0 archive-get <wal_segment> <destination_path>" >&2
      exit 2
    fi
    ;;
  *)
    echo "usage: $0 archive-push|archive-get ..." >&2
    exit 2
    ;;
esac

mkdir -p "$LOG_DIR"

TS_MS="$(now_ts_ms)"
INVOCATION_ID="${{TS_MS}}-${{PPID}}-$$"

STATUS=0
OUTPUT=""
set +e
if [ "$OP" = "archive-push" ]; then
  OUTPUT="$("$PGBACKREST_BIN" --stanza="$STANZA" --repo="$REPO" --pg1-path="$PG1_PATH" {archive_push_args} archive-push "$WAL_PATH" 2>&1)"
  STATUS="$?"
else
  OUTPUT="$("$PGBACKREST_BIN" --stanza="$STANZA" --repo="$REPO" --pg1-path="$PG1_PATH" {archive_get_args} archive-get "$WAL_SEGMENT" "$DEST_PATH" 2>&1)"
  STATUS="$?"
fi
set -e

SUCCESS=false
SEVERITY=ERROR
if [ "$STATUS" -eq 0 ]; then
  SUCCESS=true
  SEVERITY=LOG
fi

OUTPUT_ONE_LINE="$(printf '%s' "$OUTPUT" | tr '\n' ' ' | tr '\r' ' ')"
OUTPUT_TRUNCATED=false
OUTPUT_LIMIT=4096
if [ "${{#OUTPUT_ONE_LINE}}" -gt "$OUTPUT_LIMIT" ]; then
  OUTPUT_TRUNCATED=true
fi
OUTPUT_TRUNC="$(printf '%.4096s' "$OUTPUT_ONE_LINE")"

ESC_OUTPUT="$(json_escape "$OUTPUT_TRUNC")"
ESC_STANZA="$(json_escape "$STANZA")"
ESC_REPO="$(json_escape "$REPO")"
ESC_PG1_PATH="$(json_escape "$PG1_PATH")"

MESSAGE="pgbackrest op=$OP status=$STATUS invocation_id=$INVOCATION_ID"
ESC_MESSAGE="$(json_escape "$MESSAGE")"

EVENT_KIND="$(printf '%s' "$OP" | tr '-' '_')"

DETAILS=""
if [ "$OP" = "archive-push" ]; then
  ESC_WAL_PATH="$(json_escape "$WAL_PATH")"
  DETAILS="\"wal_path\":\"$ESC_WAL_PATH\""
else
  ESC_WAL_SEGMENT="$(json_escape "$WAL_SEGMENT")"
  ESC_DEST_PATH="$(json_escape "$DEST_PATH")"
  DETAILS="\"wal_segment\":\"$ESC_WAL_SEGMENT\",\"destination_path\":\"$ESC_DEST_PATH\""
fi

RECORD="{{\"severity\":\"$SEVERITY\",\"message\":\"$ESC_MESSAGE\",\"pgtuskmaster\":{{\"backup\":{{\"schema_version\":$SCHEMA_VERSION,\"provider\":\"$PROVIDER\",\"event_kind\":\"$EVENT_KIND\",\"invocation_id\":\"$INVOCATION_ID\",\"ts_ms\":$TS_MS,\"stanza\":\"$ESC_STANZA\",\"repo\":\"$ESC_REPO\",\"pg1_path\":\"$ESC_PG1_PATH\",$DETAILS,\"status_code\":$STATUS,\"success\":$SUCCESS,\"output\":\"$ESC_OUTPUT\",\"output_truncated\":$OUTPUT_TRUNCATED}}}}}}"

printf '%s\n' "$RECORD" >> "$LOG_FILE"

exit "$STATUS"
"#,
        pgbackrest_bin = args.pgbackrest_bin,
        stanza = args.stanza,
        repo = args.repo,
        pg1_path = args.pg1_path,
        log_file = args.log_file,
        log_dir = args.log_dir,
        archive_push_args = args.archive_push_args,
        archive_get_args = args.archive_get_args,
    );

    std::fs::write(script_path, script)
        .map_err(|err| format!("write pgbackrest wal wrapper failed: {err}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        std::fs::set_permissions(script_path, perms).map_err(|err| {
            format!("chmod pgbackrest wal wrapper failed: {err}")
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use tokio::process::Command;

    use crate::config::{
        ApiAuthConfig, ApiConfig, ApiSecurityConfig, ApiTlsMode, BackupConfig, BinaryPaths,
        ClusterConfig, DcsConfig, DebugConfig, FileSinkConfig, FileSinkMode, HaConfig, InlineOrPath,
        LogCleanupConfig, LogLevel, LoggingConfig, LoggingSinksConfig, PgHbaConfig, PgIdentConfig,
        PostgresConnIdentityConfig, PostgresConfig, PostgresLoggingConfig, PostgresRoleConfig,
        PostgresRolesConfig, ProcessConfig, RoleAuthConfig, RuntimeConfig, StderrSinkConfig,
        TlsServerConfig,
    };
    use crate::pginfo::conninfo::PgSslMode;

    fn temp_dir(label: &str) -> PathBuf {
        let unique = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(value) => value.as_nanos(),
            Err(_) => 0,
        };
        std::env::temp_dir().join(format!(
            "pgtuskmaster-wrapper-{label}-{unique}-{}",
            std::process::id()
        ))
    }

    fn write_fake_pgbackrest(dir: &PathBuf) -> Result<PathBuf, String> {
        fs::create_dir_all(dir).map_err(|err| format!("create fake bin dir failed: {err}"))?;
        let bin = dir.join("fake_pgbackrest.sh");
        let script = r#"#!/bin/sh
set -eu
echo "fake pgbackrest stdout: ok" >&1
echo "fake pgbackrest stderr: quote=\" backslash=\\" >&2
exit 0
"#;
        fs::write(&bin, script).map_err(|err| format!("write fake pgbackrest failed: {err}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&bin, fs::Permissions::from_mode(0o755))
                .map_err(|err| format!("chmod fake pgbackrest failed: {err}"))?;
        }
        Ok(bin)
    }

    fn sample_cfg(pgbackrest_bin: PathBuf, archive_log: PathBuf) -> RuntimeConfig {
        let mut backup = BackupConfig {
            enabled: true,
            ..BackupConfig::default()
        };
        if let Some(pg_cfg) = backup.pgbackrest.as_mut() {
            pg_cfg.stanza = Some("stanza-a".to_string());
            pg_cfg.repo = Some("1".to_string());
            pg_cfg.options.archive_get = vec!["--repo1-path=/tmp/repo".to_string()];
            pg_cfg.options.archive_push = vec!["--repo1-path=/tmp/repo".to_string()];
        }

        RuntimeConfig {
            cluster: ClusterConfig {
                name: "cluster-a".to_string(),
                member_id: "node-a".to_string(),
            },
            postgres: PostgresConfig {
                data_dir: "/tmp/pgdata".into(),
                connect_timeout_s: 5,
                listen_host: "127.0.0.1".to_string(),
                listen_port: 5432,
                socket_dir: "/tmp/pgtuskmaster/socket".into(),
                log_file: "/tmp/pgtuskmaster/postgres.log".into(),
                rewind_source_host: "127.0.0.1".to_string(),
                rewind_source_port: 5432,
                local_conn_identity: PostgresConnIdentityConfig {
                    user: "postgres".to_string(),
                    dbname: "postgres".to_string(),
                    ssl_mode: PgSslMode::Prefer,
                },
                rewind_conn_identity: PostgresConnIdentityConfig {
                    user: "rewinder".to_string(),
                    dbname: "postgres".to_string(),
                    ssl_mode: PgSslMode::Prefer,
                },
                tls: TlsServerConfig {
                    mode: ApiTlsMode::Disabled,
                    identity: None,
                    client_auth: None,
                },
                roles: PostgresRolesConfig {
                    superuser: PostgresRoleConfig {
                        username: "postgres".to_string(),
                        auth: RoleAuthConfig::Tls,
                    },
                    replicator: PostgresRoleConfig {
                        username: "replicator".to_string(),
                        auth: RoleAuthConfig::Tls,
                    },
                    rewinder: PostgresRoleConfig {
                        username: "rewinder".to_string(),
                        auth: RoleAuthConfig::Tls,
                    },
                },
                pg_hba: PgHbaConfig {
                    source: InlineOrPath::Inline {
                        content: "local all all trust\n".to_string(),
                    },
                },
                pg_ident: PgIdentConfig {
                    source: InlineOrPath::Inline {
                        content: "# empty\n".to_string(),
                    },
                },
            },
            dcs: DcsConfig {
                endpoints: vec!["http://127.0.0.1:2379".to_string()],
                scope: "scope-a".to_string(),
                init: None,
            },
            ha: HaConfig {
                loop_interval_ms: 1000,
                lease_ttl_ms: 10_000,
            },
            process: ProcessConfig {
                pg_rewind_timeout_ms: 1000,
                bootstrap_timeout_ms: 1000,
                fencing_timeout_ms: 1000,
                backup_timeout_ms: 1000,
                binaries: BinaryPaths {
                    postgres: "/usr/bin/postgres".into(),
                    pg_ctl: "/usr/bin/pg_ctl".into(),
                    pg_rewind: "/usr/bin/pg_rewind".into(),
                    initdb: "/usr/bin/initdb".into(),
                    pg_basebackup: "/usr/bin/pg_basebackup".into(),
                    psql: "/usr/bin/psql".into(),
                    pgbackrest: Some(pgbackrest_bin),
                },
            },
            backup,
            logging: LoggingConfig {
                level: LogLevel::Info,
                capture_subprocess_output: true,
                postgres: PostgresLoggingConfig {
                    enabled: true,
                    pg_ctl_log_file: None,
                    log_dir: None,
                    archive_command_log_file: Some(archive_log),
                    poll_interval_ms: 200,
                    cleanup: LogCleanupConfig {
                        enabled: true,
                        max_files: 10,
                        max_age_seconds: 60,
                        protect_recent_seconds: 300,
                    },
                },
                sinks: LoggingSinksConfig {
                    stderr: StderrSinkConfig { enabled: true },
                    file: FileSinkConfig {
                        enabled: false,
                        path: None,
                        mode: FileSinkMode::Append,
                    },
                },
            },
            api: ApiConfig {
                listen_addr: "127.0.0.1:8080".to_string(),
                security: ApiSecurityConfig {
                    tls: TlsServerConfig {
                        mode: ApiTlsMode::Disabled,
                        identity: None,
                        client_auth: None,
                    },
                    auth: ApiAuthConfig::Disabled,
                },
            },
            debug: DebugConfig { enabled: false },
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn wrapper_writes_valid_json_lines_concurrently() -> Result<(), String> {
        let dir = temp_dir("concurrency");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).map_err(|err| format!("create temp dir failed: {err}"))?;

        let fake_bin_dir = dir.join("bin");
        let fake_pgbackrest = write_fake_pgbackrest(&fake_bin_dir)?;

        let archive_log = dir.join("archive logs").join("archive_command.jsonl");
        if let Some(parent) = archive_log.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("create archive log parent failed: {err}"))?;
        }

        let cfg = sample_cfg(fake_pgbackrest, archive_log.clone());
        let wrapper = super::ensure_pgbackrest_wal_wrapper(&cfg)?;

        let mut tasks = Vec::new();
        for idx in 0..25_u32 {
            let wrapper = wrapper.clone();
            tasks.push(tokio::spawn(async move {
                let wal_segment = format!("0000000100000000000000{:02}", idx);
                let dest = format!("/tmp/dest_{idx}");
                let status = Command::new(&wrapper)
                    .args(["archive-get", wal_segment.as_str(), dest.as_str()])
                    .status()
                    .await
                    .map_err(|err| format!("spawn wrapper failed: {err}"))?;
                if !status.success() {
                    return Err(format!("wrapper exited unsuccessfully: {status:?}"));
                }
                Ok::<(), String>(())
            }));
        }
        for task in tasks {
            task.await.map_err(|err| format!("task join failed: {err}"))??;
        }

        let contents =
            fs::read_to_string(&archive_log).map_err(|err| format!("read archive log failed: {err}"))?;
        let lines: Vec<&str> = contents.lines().filter(|l| !l.trim().is_empty()).collect();
        if lines.len() != 25 {
            return Err(format!("expected 25 jsonl lines, got {}", lines.len()));
        }
        for line in lines {
            let value: serde_json::Value =
                serde_json::from_str(line).map_err(|err| format!("invalid json line: {err}"))?;
            if value.get("pgtuskmaster").is_none() {
                return Err("json record missing pgtuskmaster field".to_string());
            }
        }

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn wrapper_supports_paths_with_spaces_and_single_quotes() -> Result<(), String> {
        let dir = temp_dir("paths");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).map_err(|err| format!("create temp dir failed: {err}"))?;

        let fake_bin_dir = dir.join("bin");
        let fake_pgbackrest = write_fake_pgbackrest(&fake_bin_dir)?;

        let archive_log = dir
            .join("arch ive")
            .join("arch'ive_command.jsonl");
        if let Some(parent) = archive_log.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("create archive log parent failed: {err}"))?;
        }

        let cfg = sample_cfg(fake_pgbackrest, archive_log.clone());
        let wrapper = super::ensure_pgbackrest_wal_wrapper(&cfg)?;

        let status = Command::new(&wrapper)
            .args(["archive-push", "/tmp/wal file"])
            .status()
            .await
            .map_err(|err| format!("spawn wrapper failed: {err}"))?;
        if !status.success() {
            return Err(format!("wrapper exited unsuccessfully: {status:?}"));
        }

        let contents =
            fs::read_to_string(&archive_log).map_err(|err| format!("read archive log failed: {err}"))?;
        let first = contents
            .lines()
            .find(|l| !l.trim().is_empty())
            .ok_or_else(|| "expected at least one log line".to_string())?;
        let _: serde_json::Value =
            serde_json::from_str(first).map_err(|err| format!("invalid json line: {err}"))?;

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }
}
