use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::process::{Child, Command};
use tokio::time::{sleep, timeout, Instant};

use super::binaries::validate_executable_file;
use super::HarnessError;
use crate::test_harness::namespace::TestNamespace;

#[derive(Debug, Clone)]
pub(crate) struct PgInstanceSpec {
    pub(crate) postgres_bin: PathBuf,
    pub(crate) initdb_bin: PathBuf,
    pub(crate) data_dir: PathBuf,
    pub(crate) socket_dir: PathBuf,
    pub(crate) log_dir: PathBuf,
    pub(crate) port: u16,
    pub(crate) startup_timeout: Duration,
}

#[derive(Debug)]
pub(crate) struct PgHandle {
    child: Child,
    pub(crate) port: u16,
    pub(crate) data_dir: PathBuf,
}

impl PgHandle {
    pub(crate) async fn shutdown(&mut self) -> Result<(), HarnessError> {
        if let Some(pid) = self.child.id() {
            let mut term_cmd = Command::new("kill");
            term_cmd.arg("-TERM").arg(pid.to_string());
            let status = timeout(Duration::from_secs(2), term_cmd.status()).await;
            if let Ok(Ok(exit_status)) = status {
                if !exit_status.success() {
                    // Continue with fallback kill path below.
                }
            }
        }

        let wait_result = timeout(Duration::from_secs(5), self.child.wait()).await;
        match wait_result {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(err)) => Err(HarnessError::Io(err)),
            Err(_) => {
                self.child.start_kill().map_err(HarnessError::Io)?;
                self.child.wait().await.map_err(HarnessError::Io)?;
                Ok(())
            }
        }
    }
}

pub(crate) fn prepare_pgdata_dir(
    namespace: &TestNamespace,
    node_id: &str,
) -> Result<PathBuf, HarnessError> {
    if node_id.trim().is_empty() {
        return Err(HarnessError::InvalidInput(
            "node_id must not be empty".to_string(),
        ));
    }

    let safe_node = sanitize_node_name(node_id);
    let data_dir = namespace.child_dir(format!("pg16/{safe_node}/data"));
    if data_dir.exists() {
        return Err(HarnessError::StalePath { path: data_dir });
    }
    fs::create_dir_all(&data_dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&data_dir, fs::Permissions::from_mode(0o700))?;
    }
    Ok(data_dir)
}

pub(crate) async fn spawn_pg16(spec: PgInstanceSpec) -> Result<PgHandle, HarnessError> {
    spawn_pg16_with_conf_lines(spec, &[]).await
}

pub(crate) async fn spawn_pg16_with_conf_lines(
    spec: PgInstanceSpec,
    postgresql_conf_lines: &[String],
) -> Result<PgHandle, HarnessError> {
    validate_executable_file(spec.postgres_bin.as_path(), "postgres")?;
    validate_executable_file(spec.initdb_bin.as_path(), "initdb")?;

    fs::create_dir_all(&spec.socket_dir)?;
    fs::create_dir_all(&spec.log_dir)?;

    initialize_pgdata_if_needed(&spec).await?;
    append_postgresql_conf_lines(&spec.data_dir, postgresql_conf_lines)?;

    let stdout_path = spec.log_dir.join("postgres.stdout.log");
    let stderr_path = spec.log_dir.join("postgres.stderr.log");
    let stdout_file = File::create(stdout_path)?;
    let stderr_file = File::create(stderr_path)?;

    let mut command = Command::new(&spec.postgres_bin);
    command
        .arg("-D")
        .arg(&spec.data_dir)
        .arg("-k")
        .arg(&spec.socket_dir)
        .arg("-h")
        .arg("127.0.0.1")
        .arg("-p")
        .arg(spec.port.to_string())
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file));

    let mut child = command
        .spawn()
        .map_err(|source| HarnessError::SpawnFailure {
            binary: spec.postgres_bin.display().to_string(),
            source,
        })?;

    wait_for_port("postgres", spec.port, spec.startup_timeout, &mut child).await?;

    Ok(PgHandle {
        child,
        port: spec.port,
        data_dir: spec.data_dir,
    })
}

fn append_postgresql_conf_lines(
    data_dir: &Path,
    lines: &[String],
) -> Result<(), HarnessError> {
    if lines.is_empty() {
        return Ok(());
    }

    let conf = data_dir.join("postgresql.conf");
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(conf)
        .map_err(HarnessError::Io)?;

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        std::io::Write::write_all(&mut file, b"\n").map_err(HarnessError::Io)?;
        std::io::Write::write_all(&mut file, line.as_bytes()).map_err(HarnessError::Io)?;
        std::io::Write::write_all(&mut file, b"\n").map_err(HarnessError::Io)?;
    }

    Ok(())
}

async fn initialize_pgdata_if_needed(spec: &PgInstanceSpec) -> Result<(), HarnessError> {
    let marker = spec.data_dir.join("PG_VERSION");
    if marker.exists() {
        return Ok(());
    }

    let stdout_path = spec.log_dir.join("initdb.stdout.log");
    let stderr_path = spec.log_dir.join("initdb.stderr.log");
    let stdout_file = File::create(stdout_path)?;
    let stderr_file = File::create(stderr_path)?;

    let mut command = Command::new(&spec.initdb_bin);
    command
        .arg("-D")
        .arg(&spec.data_dir)
        .arg("-A")
        .arg("trust")
        .arg("-U")
        .arg("postgres")
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file));

    let status = command
        .status()
        .await
        .map_err(|source| HarnessError::SpawnFailure {
            binary: spec.initdb_bin.display().to_string(),
            source,
        })?;

    if !status.success() {
        return Err(HarnessError::EarlyExit {
            component: "initdb",
            status,
        });
    }

    Ok(())
}

async fn wait_for_port(
    component: &'static str,
    port: u16,
    startup_timeout: Duration,
    child: &mut Child,
) -> Result<(), HarnessError> {
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(HarnessError::Io)? {
            return Err(HarnessError::EarlyExit { component, status });
        }

        if TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            return Ok(());
        }

        if started.elapsed() >= startup_timeout {
            let _ = child.start_kill();
            let _ = child.wait().await;
            return Err(HarnessError::StartupTimeout {
                component,
                timeout: startup_timeout,
            });
        }

        sleep(Duration::from_millis(50)).await;
    }
}

fn sanitize_node_name(node_id: &str) -> String {
    let mut out = String::with_capacity(node_id.len());
    for ch in node_id.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    if out.is_empty() {
        "node".to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{prepare_pgdata_dir, spawn_pg16, PgInstanceSpec};
    use crate::test_harness::binaries::require_pg16_bin_for_real_tests;
    use crate::test_harness::namespace::NamespaceGuard;
    use crate::test_harness::ports::allocate_ports;
    use crate::test_harness::HarnessError;

    #[test]
    fn prepare_pgdata_dir_rejects_reuse() -> Result<(), HarnessError> {
        let guard = NamespaceGuard::new("prepare-pgdata")?;
        let ns = guard.namespace()?;

        let first = prepare_pgdata_dir(ns, "node-a")?;
        assert!(first.exists());

        let second = prepare_pgdata_dir(ns, "node-a");
        assert!(second.is_err());
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn spawn_pg16_requires_binaries_and_spawns() -> Result<(), HarnessError> {
        let postgres_bin = require_pg16_bin_for_real_tests("postgres")?;
        let initdb_bin = require_pg16_bin_for_real_tests("initdb")?;

        let guard = NamespaceGuard::new("spawn-pg16")?;
        let ns = guard.namespace()?;

        let data_dir = prepare_pgdata_dir(ns, "node-a")?;
        let reservation = allocate_ports(1)?;
        let port = reservation.as_slice()[0];

        let socket_dir = ns.child_dir("pg16/node-a/socket");
        let log_dir = ns.child_dir("logs/pg16-node-a");

        let spec = PgInstanceSpec {
            postgres_bin,
            initdb_bin,
            data_dir,
            socket_dir,
            log_dir,
            port,
            startup_timeout: Duration::from_secs(10),
        };

        // Release the reserved port immediately before spawning postgres so the
        // child can bind the requested port.
        drop(reservation);
        let mut handle = spawn_pg16(spec).await?;
        handle.shutdown().await?;
        Ok(())
    }
}
