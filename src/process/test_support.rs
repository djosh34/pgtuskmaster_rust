use std::{
    fs, io,
    path::Path,
    process::{Child, Command},
    time::Duration,
};

use crate::process::postmaster::{lookup_managed_postmaster, ManagedPostmasterTarget};

pub(crate) struct FakeManagedPostmaster {
    child: Child,
}

impl FakeManagedPostmaster {
    pub(crate) fn pid(&self) -> u32 {
        self.child.id()
    }

    pub(crate) fn kill_and_wait(&mut self) -> Result<(), String> {
        let pid = self.pid();
        self.child
            .kill()
            .map_err(|err| format!("kill fake postgres pid={pid} failed: {err}"))?;
        self.child
            .wait()
            .map_err(|err| format!("wait fake postgres pid={pid} failed: {err}"))?;
        Ok(())
    }
}

impl Drop for FakeManagedPostmaster {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(unix)]
pub(crate) fn spawn_fake_managed_postmaster(
    root: &Path,
    data_dir: &Path,
    signal_log: Option<&Path>,
) -> Result<FakeManagedPostmaster, String> {
    let script = root.join("fake-postgres.py");
    let ready_file = root.join("fake-postgres.ready");
    let signal_log = signal_log.map_or_else(String::new, |path| path.display().to_string());
    let script_contents = r#"#!/usr/bin/env python3
import signal
import sys
import time
from pathlib import Path

ready_file = Path(sys.argv[1])
data_dir = sys.argv[2]
signal_log = sys.argv[3]

def on_hup(_signum, _frame):
    if not signal_log:
        return
    with open(signal_log, "a", encoding="utf-8") as handle:
        handle.write("hup")
        handle.flush()

signal.signal(signal.SIGHUP, on_hup)
ready_file.write_text(data_dir, encoding="utf-8")

while True:
    time.sleep(1)
"#;
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
    let child = Command::new("/usr/bin/env")
        .arg("bash")
        .arg("-lc")
        .arg(format!(
            "exec -a postgres python3 '{}' '{}' '{}' '{}'",
            script.display(),
            ready_file.display(),
            data_dir.display(),
            signal_log,
        ))
        .spawn()
        .map_err(|err| {
            format!(
                "spawn fake postgres process via {} failed: {err}",
                script.display()
            )
        })?;
    wait_for_ready_file(ready_file.as_path())?;
    Ok(FakeManagedPostmaster { child })
}

#[cfg(not(unix))]
pub(crate) fn spawn_fake_managed_postmaster(
    _root: &Path,
    _data_dir: &Path,
    _signal_log: Option<&Path>,
) -> Result<FakeManagedPostmaster, String> {
    Err("fake postgres helper is only implemented on unix".to_string())
}

pub(crate) fn write_postmaster_pid(
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

pub(crate) fn wait_for_lookup_ready(target: &ManagedPostmasterTarget) -> Result<(), String> {
    let mut attempts = 0_u8;
    while attempts < 150 {
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

pub(crate) fn wait_for_signal_log(signal_log: &Path) -> Result<String, String> {
    wait_for_non_empty_file(signal_log, "signal log")
}

fn wait_for_ready_file(ready_file: &Path) -> Result<(), String> {
    wait_for_non_empty_file(ready_file, "fake postgres ready file").map(|_| ())
}

fn wait_for_non_empty_file(path: &Path, label: &str) -> Result<String, String> {
    let mut attempts = 0_u8;
    while attempts < 150 {
        match fs::read_to_string(path) {
            Ok(contents) if !contents.trim().is_empty() => return Ok(contents),
            Ok(_) => {}
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => {
                return Err(format!("read {label} {} failed: {err}", path.display()));
            }
        }
        std::thread::sleep(Duration::from_millis(10));
        attempts = attempts.saturating_add(1);
    }
    Err(format!(
        "{label} {} was not written in time",
        path.display()
    ))
}
