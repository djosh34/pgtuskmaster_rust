use std::{
    fs, io,
    num::{ParseIntError, TryFromIntError},
    path::{Path, PathBuf},
};

use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManagedPostmasterTarget {
    pub(crate) data_dir: PathBuf,
    pub(crate) pid_file: PathBuf,
}

impl ManagedPostmasterTarget {
    pub(crate) fn from_data_dir(data_dir: PathBuf) -> Self {
        let pid_file = data_dir.join("postmaster.pid");
        Self { data_dir, pid_file }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ManagedPostmasterPid(u32);

impl ManagedPostmasterPid {
    pub(crate) fn new(pid: u32) -> Self {
        Self(pid)
    }

    pub(crate) fn value(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VerifiedManagedPostmaster {
    pub(crate) target: ManagedPostmasterTarget,
    pub(crate) pid: ManagedPostmasterPid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StartPostgresPreflight {
    AlreadyRunning,
    SafeToStart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ManagedPostmasterSignal {
    Sighup,
}

impl ManagedPostmasterSignal {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Sighup => "SIGHUP",
        }
    }

    #[cfg(unix)]
    fn raw(self) -> i32 {
        match self {
            Self::Sighup => libc::SIGHUP,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManagedPostmasterSignalDelivery {
    pub(crate) signal: ManagedPostmasterSignal,
    pub(crate) postmaster: VerifiedManagedPostmaster,
}

#[derive(Debug, Error)]
pub(crate) enum ManagedPostmasterError {
    #[cfg(not(unix))]
    #[error("managed postmaster lookup is unsupported on this platform")]
    UnsupportedPlatform,
    #[error("read postmaster pid file {pid_file} failed: {source}")]
    ReadPidFile {
        pid_file: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("postmaster pid file {pid_file} is missing")]
    MissingPidFile { pid_file: PathBuf },
    #[error("postmaster pid file {pid_file} is missing pid line")]
    MissingPidLine { pid_file: PathBuf },
    #[error("postmaster pid file {pid_file} has an empty pid line")]
    EmptyPidLine { pid_file: PathBuf },
    #[error("parse postmaster pid '{value}' from {pid_file} failed: {source}")]
    InvalidPid {
        pid_file: PathBuf,
        value: String,
        #[source]
        source: ParseIntError,
    },
    #[error("postmaster pid {pid} from {pid_file} is not running")]
    PidNotRunning { pid: u32, pid_file: PathBuf },
    #[error("postmaster pid {pid} does not match managed data dir {expected_data_dir}")]
    DataDirMismatch {
        pid: u32,
        expected_data_dir: PathBuf,
        pid_file: PathBuf,
    },
    #[error("read postgres socket lock {lock_file} failed: {source}")]
    ReadSocketLock {
        lock_file: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("parse postgres socket lock pid '{value}' in {lock_file} failed: {source}")]
    InvalidSocketLockPid {
        lock_file: PathBuf,
        value: String,
        #[source]
        source: ParseIntError,
    },
    #[error("read process metadata {path} failed: {source}")]
    ReadProcessMetadata {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("remove file {path} failed: {source}")]
    RemoveFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("postmaster pid {pid} cannot be converted to pid_t: {source}")]
    PidOutOfRange {
        pid: u32,
        #[source]
        source: TryFromIntError,
    },
    #[error("send {signal} to postmaster pid {pid} failed: {source}")]
    SignalDelivery {
        pid: u32,
        signal: &'static str,
        #[source]
        source: io::Error,
    },
}

pub(crate) fn reload_managed_postmaster(
    target: &ManagedPostmasterTarget,
) -> Result<ManagedPostmasterSignalDelivery, ManagedPostmasterError> {
    let postmaster = lookup_managed_postmaster(target)?;
    signal_managed_postmaster(&postmaster, ManagedPostmasterSignal::Sighup)
}

pub(crate) fn start_postgres_preflight(
    data_dir: &Path,
    socket_dir: &Path,
    port: u16,
) -> Result<StartPostgresPreflight, ManagedPostmasterError> {
    let target = ManagedPostmasterTarget::from_data_dir(data_dir.to_path_buf());
    if target.pid_file.exists() {
        match lookup_managed_postmaster(&target) {
            Ok(_) => return Ok(StartPostgresPreflight::AlreadyRunning),
            Err(
                ManagedPostmasterError::MissingPidFile { .. }
                | ManagedPostmasterError::PidNotRunning { .. }
                | ManagedPostmasterError::DataDirMismatch { .. },
            ) => {
                remove_file_if_exists(target.pid_file.as_path())?;
                remove_file_if_exists(data_dir.join("postmaster.opts").as_path())?;
            }
            Err(err) => return Err(err),
        }
    }

    let (_, lock_file) = postgres_socket_paths(socket_dir, port);
    if let Some(pid) = parse_postgres_socket_lock_pid(lock_file.as_path())? {
        if pid_is_running_postgres(ManagedPostmasterPid::new(pid))? {
            return Ok(StartPostgresPreflight::AlreadyRunning);
        }
    }

    let (socket_file, lock_file) = postgres_socket_paths(socket_dir, port);
    remove_file_if_exists(socket_file.as_path())?;
    remove_file_if_exists(lock_file.as_path())?;
    Ok(StartPostgresPreflight::SafeToStart)
}

pub(crate) fn lookup_managed_postmaster(
    target: &ManagedPostmasterTarget,
) -> Result<VerifiedManagedPostmaster, ManagedPostmasterError> {
    let pid = parse_postmaster_pid(target.pid_file.as_path())?;
    if !pid_matches_data_dir(pid, target.data_dir.as_path(), target.pid_file.as_path())? {
        return Err(ManagedPostmasterError::DataDirMismatch {
            pid: pid.value(),
            expected_data_dir: target.data_dir.clone(),
            pid_file: target.pid_file.clone(),
        });
    }

    Ok(VerifiedManagedPostmaster {
        target: target.clone(),
        pid,
    })
}

pub(crate) fn signal_managed_postmaster(
    postmaster: &VerifiedManagedPostmaster,
    signal: ManagedPostmasterSignal,
) -> Result<ManagedPostmasterSignalDelivery, ManagedPostmasterError> {
    send_signal(postmaster.pid, signal)?;
    Ok(ManagedPostmasterSignalDelivery {
        signal,
        postmaster: postmaster.clone(),
    })
}

fn parse_postmaster_pid(pid_file: &Path) -> Result<ManagedPostmasterPid, ManagedPostmasterError> {
    let contents = read_postmaster_pid_file(pid_file)?;
    let first_line =
        contents
            .lines()
            .next()
            .ok_or_else(|| ManagedPostmasterError::MissingPidLine {
                pid_file: pid_file.to_path_buf(),
            })?;
    let trimmed = first_line.trim();
    if trimmed.is_empty() {
        return Err(ManagedPostmasterError::EmptyPidLine {
            pid_file: pid_file.to_path_buf(),
        });
    }

    trimmed
        .parse::<u32>()
        .map(ManagedPostmasterPid::new)
        .map_err(|source| ManagedPostmasterError::InvalidPid {
            pid_file: pid_file.to_path_buf(),
            value: trimmed.to_string(),
            source,
        })
}

fn postmaster_pid_data_dir_matches(
    pid_file: &Path,
    data_dir: &Path,
) -> Result<bool, ManagedPostmasterError> {
    let contents = read_postmaster_pid_file(pid_file)?;
    let Some(raw_data_dir) = contents.lines().nth(1) else {
        return Ok(false);
    };
    let trimmed = raw_data_dir.trim();
    if trimmed.is_empty() {
        return Ok(false);
    }
    Ok(Path::new(trimmed) == data_dir)
}

fn pid_matches_data_dir(
    pid: ManagedPostmasterPid,
    data_dir: &Path,
    pid_file: &Path,
) -> Result<bool, ManagedPostmasterError> {
    if !pid_exists(pid)? {
        return Err(ManagedPostmasterError::PidNotRunning {
            pid: pid.value(),
            pid_file: pid_file.to_path_buf(),
        });
    }

    #[cfg(unix)]
    {
        let Some(cmdline_args) = read_process_cmdline(pid)? else {
            return Err(ManagedPostmasterError::PidNotRunning {
                pid: pid.value(),
                pid_file: pid_file.to_path_buf(),
            });
        };
        let has_data_dir = process_cmdline_has_data_dir(cmdline_args.as_slice(), data_dir);
        let has_postgres_argv = process_cmdline_has_postgres_binary(cmdline_args.as_slice());
        if !has_postgres_argv {
            return Ok(false);
        }
        if has_data_dir {
            return Ok(true);
        }
        postmaster_pid_data_dir_matches(pid_file, data_dir)
    }
    #[cfg(not(unix))]
    {
        let _data_dir = data_dir;
        let _pid_file = pid_file;
        Err(ManagedPostmasterError::UnsupportedPlatform)
    }
}

fn postgres_socket_paths(socket_dir: &Path, port: u16) -> (PathBuf, PathBuf) {
    let socket_file = socket_dir.join(format!(".s.PGSQL.{port}"));
    let lock_file = socket_dir.join(format!(".s.PGSQL.{port}.lock"));
    (socket_file, lock_file)
}

fn parse_postgres_socket_lock_pid(lock_file: &Path) -> Result<Option<u32>, ManagedPostmasterError> {
    let contents = match fs::read_to_string(lock_file) {
        Ok(contents) => contents,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(ManagedPostmasterError::ReadSocketLock {
                lock_file: lock_file.to_path_buf(),
                source,
            });
        }
    };
    let Some(first_line) = contents.lines().next() else {
        return Ok(None);
    };
    let trimmed = first_line.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    trimmed.parse::<u32>().map(Some).map_err(|source| {
        ManagedPostmasterError::InvalidSocketLockPid {
            lock_file: lock_file.to_path_buf(),
            value: trimmed.to_string(),
            source,
        }
    })
}

fn remove_file_if_exists(path: &Path) -> Result<(), ManagedPostmasterError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(ManagedPostmasterError::RemoveFile {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn pid_is_running_postgres(pid: ManagedPostmasterPid) -> Result<bool, ManagedPostmasterError> {
    if !pid_exists(pid)? {
        return Ok(false);
    }

    Ok(read_process_cmdline(pid)?
        .map(|cmdline_args| process_cmdline_has_postgres_binary(cmdline_args.as_slice()))
        .unwrap_or(false))
}

fn process_cmdline_has_data_dir(cmdline_args: &[String], data_dir: &Path) -> bool {
    let data_dir_text = data_dir.display().to_string();
    cmdline_args
        .iter()
        .any(|arg| arg.contains(data_dir_text.as_str()))
}

fn process_cmdline_has_postgres_binary(cmdline_args: &[String]) -> bool {
    cmdline_args.iter().any(|arg| {
        Path::new(arg)
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| matches!(name, "postgres" | "postmaster"))
            .unwrap_or(false)
    })
}

fn read_process_cmdline(
    pid: ManagedPostmasterPid,
) -> Result<Option<Vec<String>>, ManagedPostmasterError> {
    #[cfg(unix)]
    {
        let path = PathBuf::from(format!("/proc/{}/cmdline", pid.value()));
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(source) => {
                return Err(ManagedPostmasterError::ReadProcessMetadata { path, source });
            }
        };
        Ok(Some(
            bytes
                .split(|byte| *byte == 0)
                .filter(|arg| !arg.is_empty())
                .map(|arg| String::from_utf8_lossy(arg).into_owned())
                .collect(),
        ))
    }
    #[cfg(not(unix))]
    {
        let _pid = pid;
        Err(ManagedPostmasterError::UnsupportedPlatform)
    }
}

fn pid_exists(pid: ManagedPostmasterPid) -> Result<bool, ManagedPostmasterError> {
    #[cfg(unix)]
    {
        let pid_value = pid.value();
        let pid_i32 =
            i32::try_from(pid_value).map_err(|source| ManagedPostmasterError::PidOutOfRange {
                pid: pid_value,
                source,
            })?;
        let rc = unsafe { libc::kill(pid_i32, 0) };
        if rc == 0 {
            return Ok(true);
        }
        let err = io::Error::last_os_error();
        let raw = err.raw_os_error();
        if raw == Some(libc::ESRCH) {
            return Ok(false);
        }
        if raw == Some(libc::EPERM) {
            return Ok(true);
        }
        Err(ManagedPostmasterError::ReadProcessMetadata {
            path: PathBuf::from(format!("/proc/{pid_value}")),
            source: err,
        })
    }
    #[cfg(not(unix))]
    {
        let _pid = pid;
        Err(ManagedPostmasterError::UnsupportedPlatform)
    }
}

fn send_signal(
    pid: ManagedPostmasterPid,
    signal: ManagedPostmasterSignal,
) -> Result<(), ManagedPostmasterError> {
    #[cfg(unix)]
    {
        let pid_value = pid.value();
        let pid_i32 =
            i32::try_from(pid_value).map_err(|source| ManagedPostmasterError::PidOutOfRange {
                pid: pid_value,
                source,
            })?;
        let rc = unsafe { libc::kill(pid_i32, signal.raw()) };
        if rc == 0 {
            return Ok(());
        }

        let err = io::Error::last_os_error();
        Err(ManagedPostmasterError::SignalDelivery {
            pid: pid_value,
            signal: signal.label(),
            source: err,
        })
    }
    #[cfg(not(unix))]
    {
        let _pid = pid;
        let _signal = signal;
        Err(ManagedPostmasterError::UnsupportedPlatform)
    }
}

fn read_postmaster_pid_file(pid_file: &Path) -> Result<String, ManagedPostmasterError> {
    fs::read_to_string(pid_file).map_err(|source| match source.kind() {
        io::ErrorKind::NotFound => ManagedPostmasterError::MissingPidFile {
            pid_file: pid_file.to_path_buf(),
        },
        _ => ManagedPostmasterError::ReadPidFile {
            pid_file: pid_file.to_path_buf(),
            source,
        },
    })
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use crate::{
        dev_support::test_fs::unique_test_dir,
        process::test_support::{
            spawn_fake_managed_postmaster, wait_for_lookup_ready, wait_for_signal_log,
            write_postmaster_pid,
        },
    };

    use super::{
        lookup_managed_postmaster, reload_managed_postmaster, signal_managed_postmaster,
        start_postgres_preflight, ManagedPostmasterError, ManagedPostmasterPid,
        ManagedPostmasterSignal, ManagedPostmasterTarget, StartPostgresPreflight,
        VerifiedManagedPostmaster,
    };

    fn write_socket_lock(lock_file: &Path, pid: u32) -> Result<(), String> {
        fs::write(lock_file, format!("{pid}\n"))
            .map_err(|err| format!("write socket lock {} failed: {err}", lock_file.display()))
    }

    #[test]
    fn lookup_managed_postmaster_reports_missing_pid_file() -> Result<(), String> {
        let root = unique_test_dir("postmaster", "missing-pid")?;
        let data_dir = root.join("data");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        let target = ManagedPostmasterTarget::from_data_dir(data_dir.clone());

        let lookup = lookup_managed_postmaster(&target);

        match lookup {
            Err(ManagedPostmasterError::MissingPidFile { pid_file }) => {
                if pid_file != data_dir.join("postmaster.pid") {
                    return Err(format!(
                        "unexpected pid file path: expected={} actual={}",
                        data_dir.join("postmaster.pid").display(),
                        pid_file.display()
                    ));
                }
                Ok(())
            }
            other => Err(format!(
                "expected missing pid file error, observed {other:?}"
            )),
        }
    }

    #[cfg(unix)]
    #[test]
    fn lookup_managed_postmaster_reports_stale_pid_file() -> Result<(), String> {
        let root = unique_test_dir("postmaster", "stale-pid")?;
        let data_dir = root.join("data");
        let signal_log = root.join("signal.log");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        let mut child = spawn_fake_managed_postmaster(&root, &data_dir, Some(&signal_log))?;
        let pid = child.pid();
        child.kill_and_wait()?;
        write_postmaster_pid(&data_dir, pid, &data_dir)?;
        let target = ManagedPostmasterTarget::from_data_dir(data_dir.clone());

        let lookup = lookup_managed_postmaster(&target);

        match lookup {
            Err(ManagedPostmasterError::PidNotRunning {
                pid: actual_pid, ..
            }) => {
                if actual_pid != pid {
                    return Err(format!(
                        "unexpected stale pid: expected={pid} actual={actual_pid}"
                    ));
                }
                Ok(())
            }
            other => Err(format!("expected stale pid error, observed {other:?}")),
        }
    }

    #[cfg(unix)]
    #[test]
    fn lookup_managed_postmaster_reports_data_dir_mismatch() -> Result<(), String> {
        let root = unique_test_dir("postmaster", "mismatch")?;
        let target_data_dir = root.join("target-data");
        let real_data_dir = root.join("real-data");
        let signal_log = root.join("signal.log");
        fs::create_dir_all(&target_data_dir).map_err(|err| {
            format!(
                "create target data dir {} failed: {err}",
                target_data_dir.display()
            )
        })?;
        fs::create_dir_all(&real_data_dir).map_err(|err| {
            format!(
                "create real data dir {} failed: {err}",
                real_data_dir.display()
            )
        })?;
        let child = spawn_fake_managed_postmaster(&root, &real_data_dir, Some(&signal_log))?;
        let pid = child.pid();
        write_postmaster_pid(&target_data_dir, pid, &real_data_dir)?;
        let _child = child;
        let target = ManagedPostmasterTarget::from_data_dir(target_data_dir.clone());

        let lookup = lookup_managed_postmaster(&target);

        match lookup {
            Err(ManagedPostmasterError::DataDirMismatch {
                pid: actual_pid,
                expected_data_dir,
                ..
            }) => {
                if actual_pid != pid {
                    return Err(format!(
                        "unexpected mismatch pid: expected={pid} actual={actual_pid}"
                    ));
                }
                if expected_data_dir != target_data_dir {
                    return Err(format!(
                        "unexpected mismatch target data dir: expected={} actual={}",
                        target_data_dir.display(),
                        expected_data_dir.display()
                    ));
                }
                Ok(())
            }
            other => Err(format!(
                "expected data dir mismatch error, observed {other:?}"
            )),
        }
    }

    #[cfg(unix)]
    #[test]
    fn reload_managed_postmaster_sends_sighup() -> Result<(), String> {
        let root = unique_test_dir("postmaster", "reload-success")?;
        let data_dir = root.join("data");
        let signal_log = root.join("signal.log");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        let child = spawn_fake_managed_postmaster(&root, &data_dir, Some(&signal_log))?;
        let pid = child.pid();
        write_postmaster_pid(&data_dir, pid, &data_dir)?;
        let _child = child;
        let target = ManagedPostmasterTarget::from_data_dir(data_dir.clone());
        wait_for_lookup_ready(&target)?;

        let delivery = reload_managed_postmaster(&target).map_err(|err| err.to_string())?;

        if delivery.signal != ManagedPostmasterSignal::Sighup {
            return Err(format!(
                "unexpected signal delivery: expected={:?} actual={:?}",
                ManagedPostmasterSignal::Sighup,
                delivery.signal
            ));
        }
        if delivery.postmaster.pid.value() != pid {
            return Err(format!(
                "unexpected delivered pid: expected={pid} actual={}",
                delivery.postmaster.pid.value()
            ));
        }
        let contents = wait_for_signal_log(&signal_log)?;
        if !contents.contains("hup") {
            return Err(format!(
                "signal log {} did not record hup: {contents:?}",
                signal_log.display()
            ));
        }
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn start_postgres_preflight_returns_already_running_for_managed_postmaster(
    ) -> Result<(), String> {
        let root = unique_test_dir("postmaster", "preflight-managed-running")?;
        let data_dir = root.join("data");
        let socket_dir = root.join("socket");
        let signal_log = root.join("signal.log");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        fs::create_dir_all(&socket_dir)
            .map_err(|err| format!("create socket dir {} failed: {err}", socket_dir.display()))?;
        let child = spawn_fake_managed_postmaster(&root, &data_dir, Some(&signal_log))?;
        let pid = child.pid();
        write_postmaster_pid(&data_dir, pid, &data_dir)?;
        let _child = child;
        let target = ManagedPostmasterTarget::from_data_dir(data_dir.clone());
        wait_for_lookup_ready(&target)?;

        let result = start_postgres_preflight(&data_dir, &socket_dir, 5432)
            .map_err(|err| err.to_string())?;

        if result != StartPostgresPreflight::AlreadyRunning {
            return Err(format!(
                "unexpected preflight result: expected={:?} actual={result:?}",
                StartPostgresPreflight::AlreadyRunning,
            ));
        }
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn start_postgres_preflight_removes_stale_pid_and_opts_files() -> Result<(), String> {
        let root = unique_test_dir("postmaster", "preflight-stale-pid")?;
        let data_dir = root.join("data");
        let socket_dir = root.join("socket");
        let signal_log = root.join("signal.log");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        fs::create_dir_all(&socket_dir)
            .map_err(|err| format!("create socket dir {} failed: {err}", socket_dir.display()))?;
        let mut child = spawn_fake_managed_postmaster(&root, &data_dir, Some(&signal_log))?;
        let pid = child.pid();
        child.kill_and_wait()?;
        write_postmaster_pid(&data_dir, pid, &data_dir)?;
        let opts_file = data_dir.join("postmaster.opts");
        fs::write(&opts_file, "-D stale\n")
            .map_err(|err| format!("write opts file {} failed: {err}", opts_file.display()))?;

        let result = start_postgres_preflight(&data_dir, &socket_dir, 5432)
            .map_err(|err| err.to_string())?;

        if result != StartPostgresPreflight::SafeToStart {
            return Err(format!(
                "unexpected preflight result: expected={:?} actual={result:?}",
                StartPostgresPreflight::SafeToStart,
            ));
        }
        if data_dir.join("postmaster.pid").exists() {
            return Err("stale postmaster.pid was not removed".to_string());
        }
        if opts_file.exists() {
            return Err("stale postmaster.opts was not removed".to_string());
        }
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn start_postgres_preflight_removes_stale_socket_files() -> Result<(), String> {
        let root = unique_test_dir("postmaster", "preflight-stale-socket")?;
        let data_dir = root.join("data");
        let socket_dir = root.join("socket");
        let signal_log = root.join("signal.log");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        fs::create_dir_all(&socket_dir)
            .map_err(|err| format!("create socket dir {} failed: {err}", socket_dir.display()))?;
        let mut child = spawn_fake_managed_postmaster(&root, &data_dir, Some(&signal_log))?;
        let pid = child.pid();
        child.kill_and_wait()?;
        let socket_file = socket_dir.join(".s.PGSQL.5432");
        let lock_file = socket_dir.join(".s.PGSQL.5432.lock");
        fs::write(&socket_file, [])
            .map_err(|err| format!("write socket file {} failed: {err}", socket_file.display()))?;
        write_socket_lock(&lock_file, pid)?;

        let result = start_postgres_preflight(&data_dir, &socket_dir, 5432)
            .map_err(|err| err.to_string())?;

        if result != StartPostgresPreflight::SafeToStart {
            return Err(format!(
                "unexpected preflight result: expected={:?} actual={result:?}",
                StartPostgresPreflight::SafeToStart,
            ));
        }
        if socket_file.exists() {
            return Err("stale postgres socket file was not removed".to_string());
        }
        if lock_file.exists() {
            return Err("stale postgres socket lock file was not removed".to_string());
        }
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn start_postgres_preflight_keeps_live_socket_lock_files() -> Result<(), String> {
        let root = unique_test_dir("postmaster", "preflight-live-socket")?;
        let data_dir = root.join("data");
        let socket_dir = root.join("socket");
        let signal_log = root.join("signal.log");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        fs::create_dir_all(&socket_dir)
            .map_err(|err| format!("create socket dir {} failed: {err}", socket_dir.display()))?;
        let child = spawn_fake_managed_postmaster(&root, &data_dir, Some(&signal_log))?;
        let pid = child.pid();
        let _child = child;
        let socket_file = socket_dir.join(".s.PGSQL.5432");
        let lock_file = socket_dir.join(".s.PGSQL.5432.lock");
        fs::write(&socket_file, [])
            .map_err(|err| format!("write socket file {} failed: {err}", socket_file.display()))?;
        write_socket_lock(&lock_file, pid)?;

        let result = start_postgres_preflight(&data_dir, &socket_dir, 5432)
            .map_err(|err| err.to_string())?;

        if result != StartPostgresPreflight::AlreadyRunning {
            return Err(format!(
                "unexpected preflight result: expected={:?} actual={result:?}",
                StartPostgresPreflight::AlreadyRunning,
            ));
        }
        if !socket_file.exists() {
            return Err("live postgres socket file was unexpectedly removed".to_string());
        }
        if !lock_file.exists() {
            return Err("live postgres socket lock file was unexpectedly removed".to_string());
        }
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn signal_managed_postmaster_reports_signal_delivery_failure() -> Result<(), String> {
        let root = unique_test_dir("postmaster", "signal-failure")?;
        let data_dir = root.join("data");
        let signal_log = root.join("signal.log");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        let mut child = spawn_fake_managed_postmaster(&root, &data_dir, Some(&signal_log))?;
        let pid = child.pid();
        write_postmaster_pid(&data_dir, pid, &data_dir)?;
        child.kill_and_wait()?;
        let verified = VerifiedManagedPostmaster {
            target: ManagedPostmasterTarget::from_data_dir(data_dir),
            pid: ManagedPostmasterPid::new(pid),
        };

        let delivery = signal_managed_postmaster(&verified, ManagedPostmasterSignal::Sighup);

        match delivery {
            Err(ManagedPostmasterError::SignalDelivery {
                pid: actual_pid,
                signal,
                ..
            }) => {
                if actual_pid != pid {
                    return Err(format!(
                        "unexpected signal failure pid: expected={pid} actual={actual_pid}"
                    ));
                }
                if signal != "SIGHUP" {
                    return Err(format!(
                        "unexpected signal failure label: expected=SIGHUP actual={signal}"
                    ));
                }
                Ok(())
            }
            other => Err(format!(
                "expected signal delivery failure error, observed {other:?}"
            )),
        }
    }
}
