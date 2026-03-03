use crate::state::WorkerError;
use crate::state::{UnixMillis, WorkerStatus};

use super::query::poll_once;
use super::state::{to_member_status, PgInfoWorkerCtx, SqlStatus};

pub(crate) async fn run(mut ctx: PgInfoWorkerCtx) -> Result<(), WorkerError> {
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.poll_interval).await;
    }
}

pub(crate) async fn step_once(ctx: &mut PgInfoWorkerCtx) -> Result<(), WorkerError> {
    let now = now_unix_millis()?;
    let next_state = match poll_once(&ctx.postgres_dsn).await {
        Ok(polled) => {
            to_member_status(WorkerStatus::Running, SqlStatus::Healthy, now, Some(polled))
        }
        Err(_) => to_member_status(WorkerStatus::Running, SqlStatus::Unreachable, now, None),
    };

    ctx.publisher
        .publish(next_state, now)
        .map_err(|err| WorkerError::Message(format!("pginfo publish failed: {err}")))?;
    Ok(())
}

fn now_unix_millis() -> Result<UnixMillis, WorkerError> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| WorkerError::Message(format!("system clock before unix epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| WorkerError::Message(format!("unix millis conversion failed: {err}")))?;
    Ok(UnixMillis(millis))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io;
    use std::time::Duration;

    use tokio::time::Instant;
    use tokio_postgres::NoTls;

    use crate::pginfo::state::{PgConfig, PgInfoCommon};
    use crate::state::{new_state_channel, MemberId, UnixMillis, WorkerStatus};
    use crate::test_harness::binaries::require_pg16_bin_for_real_tests;
    use crate::test_harness::namespace::NamespaceGuard;
    use crate::test_harness::pg16::{prepare_pgdata_dir, spawn_pg16, PgHandle, PgInstanceSpec};
    use crate::test_harness::ports::allocate_ports;

    use super::{step_once, PgInfoWorkerCtx, SqlStatus};
    use crate::pginfo::state::{PgInfoState, Readiness};

    type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

    fn test_error(message: impl Into<String>) -> Box<dyn std::error::Error + Send + Sync> {
        Box::new(io::Error::other(message.into()))
    }

    async fn wait_for_postgres_ready(dsn: &str, timeout: Duration) -> TestResult {
        let deadline = Instant::now() + timeout;
        loop {
            match tokio_postgres::connect(dsn, NoTls).await {
                Ok((client, connection)) => {
                    let conn_task = tokio::spawn(connection);
                    client.simple_query("SELECT 1;").await?;
                    drop(client);
                    conn_task.await??;
                    return Ok(());
                }
                Err(err) => {
                    if Instant::now() >= deadline {
                        return Err(Box::new(err));
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    async fn shutdown_with_context(label: &str, handle: &mut PgHandle) -> TestResult {
        handle
            .shutdown()
            .await
            .map_err(|err| test_error(format!("{label} shutdown failed: {err}")))
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_transitions_unreachable_to_primary_and_tracks_wal_and_slots() -> TestResult {
        let postgres_bin = match require_pg16_bin_for_real_tests("postgres")? {
            Some(path) => path,
            None => return Ok(()),
        };
        let initdb_bin = match require_pg16_bin_for_real_tests("initdb")? {
            Some(path) => path,
            None => return Ok(()),
        };

        let guard = NamespaceGuard::new("pginfo-primary-flow")?;
        let namespace = guard.namespace()?;

        let reservation = allocate_ports(1)?;
        let port = reservation.as_slice()[0];

        let data_dir = prepare_pgdata_dir(namespace, "primary")?;
        let socket_dir = namespace.child_dir("run/primary");
        let log_dir = namespace.child_dir("logs/primary");
        fs::create_dir_all(&socket_dir)?;
        fs::create_dir_all(&log_dir)?;

        let spec = PgInstanceSpec {
            postgres_bin,
            initdb_bin,
            data_dir: data_dir.clone(),
            socket_dir: socket_dir.clone(),
            log_dir,
            port,
            startup_timeout: Duration::from_secs(20),
        };

        // Release the reserved port immediately before spawning postgres so the
        // child can bind the same port.
        drop(reservation);
        let mut handle = spawn_pg16(spec).await?;

        let dsn = format!("host=127.0.0.1 port={} user=postgres dbname=postgres", port);

        let unknown = PgInfoState::Unknown {
            common: PgInfoCommon {
                worker: WorkerStatus::Starting,
                sql: SqlStatus::Unknown,
                readiness: Readiness::Unknown,
                timeline: None,
                pg_config: PgConfig {
                    port: None,
                    hot_standby: None,
                    primary_conninfo: None,
                    primary_slot_name: None,
                    extra: std::collections::BTreeMap::new(),
                },
                last_refresh_at: Some(UnixMillis(1)),
            },
        };
        let (publisher, subscriber) = new_state_channel(unknown, UnixMillis(1));
        let mut ctx = PgInfoWorkerCtx {
            self_id: MemberId("node-a".to_string()),
            postgres_dsn: dsn.clone(),
            poll_interval: Duration::from_millis(25),
            publisher,
        };

        let run_result: TestResult = async {
            wait_for_postgres_ready(&dsn, Duration::from_secs(10)).await?;
            step_once(&mut ctx).await?;

            let first = subscriber.latest().value;
            let first_wal = match first {
                PgInfoState::Primary { wal_lsn, .. } => wal_lsn,
                other => {
                    return Err(test_error(format!(
                        "expected primary state after first poll, got: {other:?}"
                    )));
                }
            };

            let (client, connection) = tokio_postgres::connect(&dsn, NoTls).await?;
            let conn_task = tokio::spawn(connection);

            client
                .batch_execute(
                    "CREATE TABLE IF NOT EXISTS t_pginfo(id integer);
                     INSERT INTO t_pginfo(id) VALUES (1);
                     SELECT pg_create_physical_replication_slot('slot_pginfo_worker_test');",
                )
                .await?;
            drop(client);
            conn_task.await??;

            step_once(&mut ctx).await?;

            let second = subscriber.latest().value;
            match second {
                PgInfoState::Primary {
                    wal_lsn,
                    slots,
                    common,
                } => {
                    assert!(wal_lsn >= first_wal);
                    assert!(slots
                        .iter()
                        .any(|slot| slot.name == "slot_pginfo_worker_test"));
                    assert_eq!(common.sql, SqlStatus::Healthy);
                    assert_eq!(common.readiness, Readiness::Ready);
                }
                other => {
                    return Err(test_error(format!(
                        "expected primary after writes, got: {other:?}"
                    )));
                }
            }
            Ok(())
        }
        .await;

        let shutdown_result = shutdown_with_context("postgres", &mut handle).await;
        match (run_result, shutdown_result) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(err), Ok(())) => Err(err),
            (Ok(()), Err(err)) => Err(err),
            (Err(err), Err(clean_err)) => Err(test_error(format!("{err}; {clean_err}"))),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_maps_replica_when_polling_standby() -> TestResult {
        let postgres_bin = match require_pg16_bin_for_real_tests("postgres")? {
            Some(path) => path,
            None => return Ok(()),
        };
        let initdb_bin = match require_pg16_bin_for_real_tests("initdb")? {
            Some(path) => path,
            None => return Ok(()),
        };
        let basebackup_bin = match require_pg16_bin_for_real_tests("pg_basebackup")? {
            Some(path) => path,
            None => return Ok(()),
        };

        let guard = NamespaceGuard::new("pginfo-replica-flow")?;
        let ns = guard.namespace()?;

        let primary_data = prepare_pgdata_dir(ns, "primary")?;
        let primary_socket = ns.child_dir("run/primary");
        let primary_logs = ns.child_dir("logs/primary");
        fs::create_dir_all(&primary_socket)?;
        fs::create_dir_all(&primary_logs)?;

        let primary_reservation = allocate_ports(1)?;
        let primary_port = primary_reservation.as_slice()[0];
        drop(primary_reservation);

        let mut primary = spawn_pg16(PgInstanceSpec {
            postgres_bin: postgres_bin.clone(),
            initdb_bin: initdb_bin.clone(),
            data_dir: primary_data.clone(),
            socket_dir: primary_socket.clone(),
            log_dir: primary_logs.clone(),
            port: primary_port,
            startup_timeout: Duration::from_secs(25),
        })
        .await?;

        let primary_dsn = format!(
            "host=127.0.0.1 port={} user=postgres dbname=postgres",
            primary_port
        );
        let mut replica: Option<PgHandle> = None;
        let run_result: TestResult = async {
            wait_for_postgres_ready(&primary_dsn, Duration::from_secs(20)).await?;

            let replica_data = ns.child_dir("pg16/replica/data");
            let replica_parent = replica_data
                .parent()
                .ok_or_else(|| test_error("replica data dir has no parent"))?;
            fs::create_dir_all(replica_parent)?;

            let output = tokio::process::Command::new(&basebackup_bin)
                .arg("-h")
                .arg("127.0.0.1")
                .arg("-p")
                .arg(primary_port.to_string())
                .arg("-D")
                .arg(&replica_data)
                .arg("-U")
                .arg("postgres")
                .arg("-Fp")
                .arg("-Xs")
                .arg("-R")
                .output()
                .await?;
            if !output.status.success() {
                return Err(test_error(format!(
                    "pg_basebackup failed with status {}",
                    output.status
                )));
            }

            let replica_socket = ns.child_dir("run/replica");
            let replica_logs = ns.child_dir("logs/replica");
            fs::create_dir_all(&replica_socket)?;
            fs::create_dir_all(&replica_logs)?;

            let replica_reservation = allocate_ports(1)?;
            let replica_port = replica_reservation.as_slice()[0];
            drop(replica_reservation);

            let replica_handle = spawn_pg16(PgInstanceSpec {
                postgres_bin: postgres_bin.clone(),
                initdb_bin: initdb_bin.clone(),
                data_dir: replica_data.clone(),
                socket_dir: replica_socket,
                log_dir: replica_logs,
                port: replica_port,
                startup_timeout: Duration::from_secs(30),
            })
            .await?;
            replica = Some(replica_handle);

            let replica_dsn = format!(
                "host=127.0.0.1 port={} user=postgres dbname=postgres",
                replica_port
            );
            let initial = PgInfoState::Unknown {
                common: PgInfoCommon {
                    worker: WorkerStatus::Starting,
                    sql: SqlStatus::Unknown,
                    readiness: Readiness::Unknown,
                    timeline: None,
                    pg_config: PgConfig {
                        port: None,
                        hot_standby: None,
                        primary_conninfo: None,
                        primary_slot_name: None,
                        extra: std::collections::BTreeMap::new(),
                    },
                    last_refresh_at: Some(UnixMillis(1)),
                },
            };
            let (publisher, subscriber) = new_state_channel(initial, UnixMillis(1));
            let mut ctx = PgInfoWorkerCtx {
                self_id: MemberId("node-b".to_string()),
                postgres_dsn: replica_dsn,
                poll_interval: Duration::from_millis(50),
                publisher,
            };

            let deadline = Instant::now() + Duration::from_secs(20);
            let snapshot = loop {
                step_once(&mut ctx).await?;

                let polled = subscriber.latest().value;
                if matches!(polled, PgInfoState::Replica { .. }) {
                    break polled;
                }

                if Instant::now() >= deadline {
                    return Err(test_error(format!(
                        "timed out waiting for replica state, got: {polled:?}"
                    )));
                }

                tokio::time::sleep(Duration::from_millis(100)).await;
            };

            match snapshot {
                PgInfoState::Replica { common, .. } => {
                    assert_eq!(common.sql, SqlStatus::Healthy);
                    assert_eq!(common.readiness, Readiness::Ready);
                }
                other => {
                    return Err(test_error(format!(
                        "expected replica state, got: {other:?}"
                    )));
                }
            }
            Ok(())
        }
        .await;

        let mut cleanup_errors = Vec::new();
        if let Some(handle) = replica.as_mut() {
            if let Err(err) = shutdown_with_context("replica postgres", handle).await {
                cleanup_errors.push(err.to_string());
            }
        }
        if let Err(err) = shutdown_with_context("primary postgres", &mut primary).await {
            cleanup_errors.push(err.to_string());
        }

        if let Err(err) = run_result {
            if cleanup_errors.is_empty() {
                return Err(err);
            }
            return Err(test_error(format!(
                "{err}; cleanup errors: {}",
                cleanup_errors.join("; ")
            )));
        }

        if cleanup_errors.is_empty() {
            Ok(())
        } else {
            Err(test_error(format!(
                "cleanup errors: {}",
                cleanup_errors.join("; ")
            )))
        }
    }
}
