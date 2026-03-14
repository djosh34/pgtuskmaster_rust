use std::{
    fs, io,
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

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub(crate) enum ManagedPostmasterError {
    #[cfg(not(unix))]
    #[error("managed postmaster lookup is unsupported on this platform")]
    UnsupportedPlatform,
    #[error("read postmaster pid file {pid_file} failed: {message}")]
    ReadPidFile { pid_file: PathBuf, message: String },
    #[error("postmaster pid file {pid_file} is missing")]
    MissingPidFile { pid_file: PathBuf },
    #[error("postmaster pid file {pid_file} is missing pid line")]
    MissingPidLine { pid_file: PathBuf },
    #[error("postmaster pid file {pid_file} has an empty pid line")]
    EmptyPidLine { pid_file: PathBuf },
    #[error("parse postmaster pid '{value}' from {pid_file} failed: {message}")]
    InvalidPid {
        pid_file: PathBuf,
        value: String,
        message: String,
    },
    #[error("postmaster pid {pid} from {pid_file} is not running")]
    PidNotRunning { pid: u32, pid_file: PathBuf },
    #[error("postmaster pid {pid} does not match managed data dir {expected_data_dir}")]
    DataDirMismatch {
        pid: u32,
        expected_data_dir: PathBuf,
        pid_file: PathBuf,
    },
    #[error("read process metadata {path} failed: {message}")]
    ReadProcessMetadata { path: PathBuf, message: String },
    #[error("postmaster pid {pid} cannot be converted to pid_t: {message}")]
    PidOutOfRange { pid: u32, message: String },
    #[error("send {signal} to postmaster pid {pid} failed: {message}")]
    SignalDelivery {
        pid: u32,
        signal: &'static str,
        message: String,
    },
}

pub(crate) fn reload_managed_postmaster(
    target: &ManagedPostmasterTarget,
) -> Result<ManagedPostmasterSignalDelivery, ManagedPostmasterError> {
    let postmaster = lookup_managed_postmaster(target)?;
    signal_managed_postmaster(&postmaster, ManagedPostmasterSignal::Sighup)
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
    let contents = fs::read_to_string(pid_file).map_err(|err| match err.kind() {
        io::ErrorKind::NotFound => ManagedPostmasterError::MissingPidFile {
            pid_file: pid_file.to_path_buf(),
        },
        _ => ManagedPostmasterError::ReadPidFile {
            pid_file: pid_file.to_path_buf(),
            message: err.to_string(),
        },
    })?;
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
        .map_err(|err| ManagedPostmasterError::InvalidPid {
            pid_file: pid_file.to_path_buf(),
            value: trimmed.to_string(),
            message: err.to_string(),
        })
}

fn postmaster_pid_data_dir_matches(
    pid_file: &Path,
    data_dir: &Path,
) -> Result<bool, ManagedPostmasterError> {
    let contents = fs::read_to_string(pid_file).map_err(|err| match err.kind() {
        io::ErrorKind::NotFound => ManagedPostmasterError::MissingPidFile {
            pid_file: pid_file.to_path_buf(),
        },
        _ => ManagedPostmasterError::ReadPidFile {
            pid_file: pid_file.to_path_buf(),
            message: err.to_string(),
        },
    })?;
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
        let pid_value = pid.value();
        let cmdline_path = PathBuf::from(format!("/proc/{pid_value}/cmdline"));
        let cmdline = match fs::read(&cmdline_path) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                return Err(ManagedPostmasterError::PidNotRunning {
                    pid: pid_value,
                    pid_file: pid_file.to_path_buf(),
                });
            }
            Err(err) => {
                return Err(ManagedPostmasterError::ReadProcessMetadata {
                    path: cmdline_path,
                    message: err.to_string(),
                });
            }
        };
        let data_dir_text = data_dir.display().to_string();
        let cmdline_args = cmdline
            .split(|byte| *byte == 0)
            .filter(|arg| !arg.is_empty())
            .map(|arg| String::from_utf8_lossy(arg))
            .collect::<Vec<_>>();
        let has_data_dir = cmdline_args
            .iter()
            .any(|arg| arg.contains(data_dir_text.as_str()));
        let has_postgres_argv = cmdline_args.iter().any(|arg| {
            Path::new(arg.as_ref())
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| matches!(name, "postgres" | "postmaster"))
                .unwrap_or(false)
        });
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

fn pid_exists(pid: ManagedPostmasterPid) -> Result<bool, ManagedPostmasterError> {
    #[cfg(unix)]
    {
        let pid_value = pid.value();
        let pid_i32 =
            i32::try_from(pid_value).map_err(|err| ManagedPostmasterError::PidOutOfRange {
                pid: pid_value,
                message: err.to_string(),
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
            message: err.to_string(),
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
            i32::try_from(pid_value).map_err(|err| ManagedPostmasterError::PidOutOfRange {
                pid: pid_value,
                message: err.to_string(),
            })?;
        let rc = unsafe { libc::kill(pid_i32, signal.raw()) };
        if rc == 0 {
            return Ok(());
        }

        let err = io::Error::last_os_error();
        Err(ManagedPostmasterError::SignalDelivery {
            pid: pid_value,
            signal: signal.label(),
            message: err.to_string(),
        })
    }
    #[cfg(not(unix))]
    {
        let _pid = pid;
        let _signal = signal;
        Err(ManagedPostmasterError::UnsupportedPlatform)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs, io,
        path::{Path, PathBuf},
        process::{Child, Command},
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use super::{
        lookup_managed_postmaster, reload_managed_postmaster, signal_managed_postmaster,
        ManagedPostmasterError, ManagedPostmasterPid, ManagedPostmasterSignal,
        ManagedPostmasterTarget, VerifiedManagedPostmaster,
    };

    struct ChildGuard(Option<Child>);

    impl ChildGuard {
        fn child(&self) -> Result<&Child, String> {
            self.0
                .as_ref()
                .ok_or_else(|| "fake postgres child handle missing".to_string())
        }

        fn child_mut(&mut self) -> Result<&mut Child, String> {
            self.0
                .as_mut()
                .ok_or_else(|| "fake postgres child handle missing".to_string())
        }
    }

    impl Drop for ChildGuard {
        fn drop(&mut self) {
            if let Some(child) = self.0.as_mut() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }

    fn unique_test_dir(label: &str) -> Result<PathBuf, String> {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("clock error for test dir: {err}"))?
            .as_millis();
        let dir = std::env::temp_dir().join(format!(
            "pgtm-postmaster-{label}-{}-{millis}",
            std::process::id()
        ));
        fs::create_dir_all(&dir)
            .map_err(|err| format!("create test dir {} failed: {err}", dir.display()))?;
        Ok(dir)
    }

    #[cfg(unix)]
    fn spawn_fake_postgres_process(
        root: &Path,
        data_dir: &Path,
        signal_log: &Path,
    ) -> Result<ChildGuard, String> {
        let script = root.join("fake-postgres.sh");
        let script_contents = format!(
            "#!/bin/bash\ntrap 'printf hup >> \"{}\"' HUP\nwhile true; do read -r -t 1 _ || true; done\n",
            signal_log.display()
        );
        fs::write(&script, script_contents).map_err(|err| {
            format!(
                "write fake postgres script {} failed: {err}",
                script.display()
            )
        })?;
        let mut permissions = fs::metadata(&script)
            .map_err(|err| {
                format!(
                    "read fake postgres script metadata {} failed: {err}",
                    script.display()
                )
            })?
            .permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
        fs::set_permissions(&script, permissions).map_err(|err| {
            format!(
                "set fake postgres script permissions {} failed: {err}",
                script.display()
            )
        })?;
        let child = Command::new("/bin/bash")
            .arg("-lc")
            .arg(format!(
                "exec -a postgres /bin/bash '{}' '{}'",
                script.display(),
                data_dir.display()
            ))
            .spawn()
            .map_err(|err| {
                format!(
                    "spawn fake postgres process via {} failed: {err}",
                    script.display()
                )
            })?;
        Ok(ChildGuard(Some(child)))
    }

    #[cfg(not(unix))]
    fn spawn_fake_postgres_process(
        _root: &Path,
        _data_dir: &Path,
        _signal_log: &Path,
    ) -> Result<ChildGuard, String> {
        Err("fake postgres helper is only implemented on unix".to_string())
    }

    fn write_postmaster_pid(
        data_dir: &Path,
        pid: u32,
        recorded_data_dir: &Path,
    ) -> Result<(), String> {
        let pid_file = data_dir.join("postmaster.pid");
        let contents = format!("{pid}\n{}\n", recorded_data_dir.display());
        fs::write(&pid_file, contents).map_err(|err| {
            format!(
                "write postmaster pid file {} failed: {err}",
                pid_file.display()
            )
        })
    }

    fn wait_for_lookup_ready(target: &ManagedPostmasterTarget) -> Result<(), String> {
        let mut attempts = 0_u8;
        while attempts < 50 {
            if lookup_managed_postmaster(target).is_ok() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(10));
            attempts = attempts.saturating_add(1);
        }
        Err(format!(
            "managed postmaster never became ready for {}",
            target.data_dir.display()
        ))
    }

    fn wait_for_signal_log(signal_log: &Path) -> Result<String, String> {
        let mut attempts = 0_u8;
        while attempts < 150 {
            match fs::read_to_string(signal_log) {
                Ok(contents) if !contents.is_empty() => return Ok(contents),
                Ok(_) => {}
                Err(err) if err.kind() == io::ErrorKind::NotFound => {}
                Err(err) => {
                    return Err(format!(
                        "read signal log {} failed: {err}",
                        signal_log.display()
                    ));
                }
            }
            std::thread::sleep(Duration::from_millis(10));
            attempts = attempts.saturating_add(1);
        }
        Err(format!(
            "signal log {} was not written in time",
            signal_log.display()
        ))
    }

    #[test]
    fn lookup_managed_postmaster_reports_missing_pid_file() -> Result<(), String> {
        let root = unique_test_dir("missing-pid")?;
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
        let root = unique_test_dir("stale-pid")?;
        let data_dir = root.join("data");
        let signal_log = root.join("signal.log");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        let mut child = spawn_fake_postgres_process(&root, &data_dir, &signal_log)?;
        let pid = child.child()?.id();
        child
            .child_mut()?
            .kill()
            .map_err(|err| format!("kill fake postgres pid={pid} failed: {err}"))?;
        child
            .child_mut()?
            .wait()
            .map_err(|err| format!("wait fake postgres pid={pid} failed: {err}"))?;
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
        let root = unique_test_dir("mismatch")?;
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
        let child = spawn_fake_postgres_process(&root, &real_data_dir, &signal_log)?;
        let pid = child.child()?.id();
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
        let root = unique_test_dir("reload-success")?;
        let data_dir = root.join("data");
        let signal_log = root.join("signal.log");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        let child = spawn_fake_postgres_process(&root, &data_dir, &signal_log)?;
        let pid = child.child()?.id();
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
    fn signal_managed_postmaster_reports_signal_delivery_failure() -> Result<(), String> {
        let root = unique_test_dir("signal-failure")?;
        let data_dir = root.join("data");
        let signal_log = root.join("signal.log");
        fs::create_dir_all(&data_dir)
            .map_err(|err| format!("create data dir {} failed: {err}", data_dir.display()))?;
        let mut child = spawn_fake_postgres_process(&root, &data_dir, &signal_log)?;
        let pid = child.child()?.id();
        write_postmaster_pid(&data_dir, pid, &data_dir)?;
        child
            .child_mut()?
            .kill()
            .map_err(|err| format!("kill fake postgres pid={pid} failed: {err}"))?;
        child
            .child_mut()?
            .wait()
            .map_err(|err| format!("wait fake postgres pid={pid} failed: {err}"))?;
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
