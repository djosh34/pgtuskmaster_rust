use std::fs::{self, File};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::process::{Child, Command};
use tokio::time::{sleep, timeout, Instant};

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
    Ok(data_dir)
}

pub(crate) async fn spawn_pg16(spec: PgInstanceSpec) -> Result<PgHandle, HarnessError> {
    if !spec.postgres_bin.exists() {
        return Err(HarnessError::InvalidInput(format!(
            "postgres binary does not exist: {}",
            spec.postgres_bin.display()
        )));
    }
    if !spec.initdb_bin.exists() {
        return Err(HarnessError::InvalidInput(format!(
            "initdb binary does not exist: {}",
            spec.initdb_bin.display()
        )));
    }

    fs::create_dir_all(&spec.socket_dir)?;
    fs::create_dir_all(&spec.log_dir)?;

    initialize_pgdata_if_needed(&spec).await?;

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
    use std::path::Path;
    use std::time::Duration;

    use super::{prepare_pgdata_dir, spawn_pg16, PgInstanceSpec};
    use crate::test_harness::namespace::{cleanup_namespace, create_namespace};
    use crate::test_harness::ports::allocate_ports;

    #[test]
    fn prepare_pgdata_dir_rejects_reuse() {
        let ns = match create_namespace("prepare-pgdata") {
            Ok(ns) => ns,
            Err(err) => panic!("namespace create failed: {err}"),
        };

        let first = match prepare_pgdata_dir(&ns, "node-a") {
            Ok(path) => path,
            Err(err) => {
                if let Err(clean_err) = cleanup_namespace(ns) {
                    panic!("prepare failed: {err}; cleanup failed: {clean_err}");
                }
                panic!("prepare pgdata first failed: {err}");
            }
        };
        assert!(first.exists());

        let second = prepare_pgdata_dir(&ns, "node-a");
        assert!(second.is_err());

        if let Err(err) = cleanup_namespace(ns) {
            panic!("cleanup failed: {err}");
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn spawn_pg16_is_skipped_when_binaries_missing() {
        let postgres_bin = Path::new("/usr/lib/postgresql/16/bin/postgres");
        let initdb_bin = Path::new("/usr/lib/postgresql/16/bin/initdb");
        if !postgres_bin.exists() || !initdb_bin.exists() {
            return;
        }

        let ns = match create_namespace("spawn-pg16") {
            Ok(ns) => ns,
            Err(err) => panic!("namespace create failed: {err}"),
        };

        let data_dir = match prepare_pgdata_dir(&ns, "node-a") {
            Ok(path) => path,
            Err(err) => {
                if let Err(clean_err) = cleanup_namespace(ns) {
                    panic!("prepare failed: {err}; cleanup failed: {clean_err}");
                }
                panic!("prepare pgdata failed: {err}");
            }
        };

        let reservation = match allocate_ports(1) {
            Ok(res) => res,
            Err(err) => {
                if let Err(clean_err) = cleanup_namespace(ns) {
                    panic!("port alloc failed: {err}; cleanup failed: {clean_err}");
                }
                panic!("allocate ports failed: {err}");
            }
        };
        let port = reservation.as_slice()[0];

        let socket_dir = ns.child_dir("pg16/node-a/socket");
        let log_dir = ns.child_dir("logs/pg16-node-a");

        let spec = PgInstanceSpec {
            postgres_bin: postgres_bin.to_path_buf(),
            initdb_bin: initdb_bin.to_path_buf(),
            data_dir,
            socket_dir,
            log_dir,
            port,
            startup_timeout: Duration::from_secs(10),
        };

        // Release the reserved port immediately before spawning postgres so the
        // child can bind the requested port.
        drop(reservation);
        let mut handle = match spawn_pg16(spec).await {
            Ok(handle) => handle,
            Err(err) => {
                if let Err(clean_err) = cleanup_namespace(ns) {
                    panic!("spawn failed: {err}; cleanup failed: {clean_err}");
                }
                panic!("spawn pg16 failed: {err}");
            }
        };

        if let Err(err) = handle.shutdown().await {
            if let Err(clean_err) = cleanup_namespace(ns) {
                panic!("shutdown failed: {err}; cleanup failed: {clean_err}");
            }
            panic!("shutdown pg16 failed: {err}");
        }

        if let Err(err) = cleanup_namespace(ns) {
            panic!("cleanup failed: {err}");
        }
    }
}
