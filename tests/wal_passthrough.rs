use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_root(label: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_nanos(),
        Err(_) => 0,
    };
    std::env::temp_dir().join(format!("pgtuskmaster-test-{label}-{pid}-{nanos}"))
}

fn write_executable_script(path: &Path, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|err| format!("write {} failed: {err}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)
            .map_err(|err| format!("stat {} failed: {err}", path.display()))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms)
            .map_err(|err| format!("chmod {} failed: {err}", path.display()))?;
    }

    Ok(())
}

fn write_archive_helper_config(
    pgdata: &Path,
    pgbackrest_bin: &Path,
    archive_push_options: Vec<String>,
    archive_get_options: Vec<String>,
    api_local_addr: &str,
) -> Result<(), String> {
    let cfg_path = pgdata.join("pgtm.pgbackrest.archive.json");
    let value = serde_json::json!({
        "pgbackrest_bin": pgbackrest_bin,
        "stanza": "stanza-a",
        "repo": "1",
        "pg1_path": pgdata,
        "archive_push_options": archive_push_options,
        "archive_get_options": archive_get_options,
        "api_local_addr": api_local_addr,
        "api_token": null,
    });
    let raw =
        serde_json::to_vec(&value).map_err(|err| format!("config json encode failed: {err}"))?;
    fs::create_dir_all(pgdata)
        .map_err(|err| format!("create {} failed: {err}", pgdata.display()))?;
    fs::write(&cfg_path, raw)
        .map_err(|err| format!("write {} failed: {err}", cfg_path.display()))?;
    Ok(())
}

fn read_lines(path: &Path) -> Result<Vec<String>, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("read {} failed: {err}", path.display()))?;
    Ok(raw
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>())
}

#[test]
fn wal_passthrough_archive_push_preserves_exit_code_and_argv() -> Result<(), String> {
    let exe = env!("CARGO_BIN_EXE_pgtuskmaster");
    let root = unique_temp_root("wal-push");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).map_err(|err| format!("mkdir {} failed: {err}", root.display()))?;

    let pgdata = root.join("pgdata");
    let pgdata_arg = pgdata.to_string_lossy().to_string();
    let argv_path = root.join("argv.txt");
    let argv_arg = argv_path.to_string_lossy().to_string();
    let stub = root.join("pgbackrest-stub.sh");
    write_executable_script(
        &stub,
        r#"#!/bin/sh
set -eu
: "${PGTUSKMASTER_STUB_ARGV_PATH:?missing}"
: "${PGTUSKMASTER_STUB_EXIT_CODE:?missing}"
printf '%s\n' "$@" > "$PGTUSKMASTER_STUB_ARGV_PATH"
stdout_n=${PGTUSKMASTER_STUB_STDOUT_N:-70000}
stderr_n=${PGTUSKMASTER_STUB_STDERR_N:-70000}
i=0
while [ "$i" -lt "$stdout_n" ]; do
  printf a
  i=$((i+1))
done
printf '\n'
i=0
while [ "$i" -lt "$stderr_n" ]; do
  printf b >&2
  i=$((i+1))
done
printf '\n' >&2
exit "$PGTUSKMASTER_STUB_EXIT_CODE"
"#,
    )?;

    write_archive_helper_config(
        pgdata.as_path(),
        stub.as_path(),
        vec!["--opt=hello world".to_string(), "--quote=it's fine".to_string()],
        Vec::new(),
        "127.0.0.1:1",
    )?;

    let wal_path = "/tmp/wal path/000000010000000000000001";
    let status = Command::new(exe)
        .args(["wal", "--pgdata", pgdata_arg.as_str(), "archive-push", wal_path])
        .env("PGTUSKMASTER_STUB_ARGV_PATH", argv_arg.as_str())
        .env("PGTUSKMASTER_STUB_EXIT_CODE", "0")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|err| format!("spawn helper failed: {err}"))?;

    let code = status
        .code()
        .ok_or_else(|| "expected helper to exit with a status code".to_string())?;
    if code != 0 {
        return Err(format!("expected exit code 0, got {code}"));
    }

    let argv = read_lines(argv_path.as_path())?;
    let expected = vec![
        "--stanza".to_string(),
        "stanza-a".to_string(),
        "--repo".to_string(),
        "1".to_string(),
        "--pg1-path".to_string(),
        pgdata_arg.clone(),
        "--opt=hello world".to_string(),
        "--quote=it's fine".to_string(),
        "archive-push".to_string(),
        wal_path.to_string(),
    ];
    if argv != expected {
        return Err(format!("argv mismatch:\nexpected: {expected:?}\nactual:   {argv:?}"));
    }

    let _ = fs::remove_dir_all(&root);
    Ok(())
}

#[test]
fn wal_passthrough_archive_get_preserves_failure_exit_code() -> Result<(), String> {
    let exe = env!("CARGO_BIN_EXE_pgtuskmaster");
    let root = unique_temp_root("wal-get");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).map_err(|err| format!("mkdir {} failed: {err}", root.display()))?;

    let pgdata = root.join("pgdata");
    let pgdata_arg = pgdata.to_string_lossy().to_string();
    let argv_path = root.join("argv.txt");
    let argv_arg = argv_path.to_string_lossy().to_string();
    let stub = root.join("pgbackrest-stub.sh");
    write_executable_script(
        &stub,
        r#"#!/bin/sh
set -eu
: "${PGTUSKMASTER_STUB_ARGV_PATH:?missing}"
: "${PGTUSKMASTER_STUB_EXIT_CODE:?missing}"
printf '%s\n' "$@" > "$PGTUSKMASTER_STUB_ARGV_PATH"
exit "$PGTUSKMASTER_STUB_EXIT_CODE"
"#,
    )?;

    write_archive_helper_config(
        pgdata.as_path(),
        stub.as_path(),
        Vec::new(),
        vec!["--spaced=hello world".to_string()],
        "127.0.0.1:1",
    )?;

    let wal_segment = "000000010000000000000001";
    let destination = "/tmp/dest path";
    let status = Command::new(exe)
        .args([
            "wal",
            "--pgdata",
            pgdata_arg.as_str(),
            "archive-get",
            wal_segment,
            destination,
        ])
        .env("PGTUSKMASTER_STUB_ARGV_PATH", argv_arg.as_str())
        .env("PGTUSKMASTER_STUB_EXIT_CODE", "37")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|err| format!("spawn helper failed: {err}"))?;

    let code = status
        .code()
        .ok_or_else(|| "expected helper to exit with a status code".to_string())?;
    if code != 37 {
        return Err(format!("expected exit code 37, got {code}"));
    }

    let argv = read_lines(argv_path.as_path())?;
    let expected = vec![
        "--stanza".to_string(),
        "stanza-a".to_string(),
        "--repo".to_string(),
        "1".to_string(),
        "--pg1-path".to_string(),
        pgdata_arg.clone(),
        "--spaced=hello world".to_string(),
        "archive-get".to_string(),
        wal_segment.to_string(),
        destination.to_string(),
    ];
    if argv != expected {
        return Err(format!("argv mismatch:\nexpected: {expected:?}\nactual:   {argv:?}"));
    }

    let _ = fs::remove_dir_all(&root);
    Ok(())
}
