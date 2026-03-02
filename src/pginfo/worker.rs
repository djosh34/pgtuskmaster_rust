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
    use std::time::Duration;

    use tokio_postgres::NoTls;

    use crate::pginfo::state::{PgConfig, PgInfoCommon};
    use crate::state::{new_state_channel, MemberId, UnixMillis, WorkerStatus};
    use crate::test_harness::binaries::require_pg16_bin;
    use crate::test_harness::namespace::{cleanup_namespace, create_namespace, NamespaceGuard};
    use crate::test_harness::pg16::{prepare_pgdata_dir, spawn_pg16, PgInstanceSpec};
    use crate::test_harness::ports::allocate_ports;

    use super::{step_once, PgInfoWorkerCtx, SqlStatus};
    use crate::pginfo::state::{PgInfoState, Readiness};

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_transitions_unreachable_to_primary_and_tracks_wal_and_slots() {
        let postgres_bin = require_pg16_bin("postgres");
        let initdb_bin = require_pg16_bin("initdb");

        let guard = match NamespaceGuard::new("pginfo-primary-flow") {
            Ok(guard) => guard,
            Err(err) => panic!("namespace guard create failed: {err}"),
        };
        let namespace = match guard.namespace() {
            Ok(ns) => ns,
            Err(err) => panic!("namespace lookup failed: {err}"),
        };

        let reservation = match allocate_ports(1) {
            Ok(res) => res,
            Err(err) => panic!("port allocation failed: {err}"),
        };
        let port = reservation.as_slice()[0];

        let data_dir = match prepare_pgdata_dir(namespace, "primary") {
            Ok(path) => path,
            Err(err) => panic!("prepare data dir failed: {err}"),
        };
        let socket_dir = namespace.child_dir("run/primary");
        let log_dir = namespace.child_dir("logs/primary");
        if let Err(err) = fs::create_dir_all(&socket_dir) {
            panic!("create socket dir failed: {err}");
        }
        if let Err(err) = fs::create_dir_all(&log_dir) {
            panic!("create log dir failed: {err}");
        }

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
        let mut handle = match spawn_pg16(spec).await {
            Ok(handle) => handle,
            Err(err) => panic!("spawn pg16 failed: {err}"),
        };

        let dsn = format!("host=127.0.0.1 port={} user=postgres dbname=postgres", port);

        let unknown = PgInfoState::Unknown {
            common: PgInfoCommon {
                worker: WorkerStatus::Starting,
                sql: SqlStatus::Unknown,
                readiness: Readiness::Unknown,
                timeline: None,
                pg_config: PgConfig {
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

        if let Err(err) = step_once(&mut ctx).await {
            let _ = handle.shutdown().await;
            panic!("first step_once failed: {err}");
        }

        let first = subscriber.latest().value;
        let first_wal = match first {
            PgInfoState::Primary { wal_lsn, .. } => wal_lsn,
            other => {
                let _ = handle.shutdown().await;
                panic!("expected primary state after first poll, got: {other:?}");
            }
        };

        let (client, connection) = match tokio_postgres::connect(&dsn, NoTls).await {
            Ok(pair) => pair,
            Err(err) => {
                let _ = handle.shutdown().await;
                panic!("connect for writes failed: {err}");
            }
        };
        let conn_task = tokio::spawn(connection);

        if let Err(err) = client
            .batch_execute(
                "CREATE TABLE IF NOT EXISTS t_pginfo(id integer);
                 INSERT INTO t_pginfo(id) VALUES (1);
                 SELECT pg_create_physical_replication_slot('slot_pginfo_worker_test');",
            )
            .await
        {
            let _ = handle.shutdown().await;
            panic!("write workload failed: {err}");
        }
        drop(client);
        let connection_result = conn_task.await;
        match connection_result {
            Ok(Ok(())) => {}
            Ok(Err(err)) => {
                let _ = handle.shutdown().await;
                panic!("connection completion failed: {err}");
            }
            Err(err) => {
                let _ = handle.shutdown().await;
                panic!("connection join failed: {err}");
            }
        }

        if let Err(err) = step_once(&mut ctx).await {
            let _ = handle.shutdown().await;
            panic!("second step_once failed: {err}");
        }

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
                let _ = handle.shutdown().await;
                panic!("expected primary after writes, got: {other:?}");
            }
        }

        if let Err(err) = handle.shutdown().await {
            panic!("postgres shutdown failed: {err}");
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn step_once_maps_replica_when_polling_standby() {
        let postgres_bin = require_pg16_bin("postgres");
        let initdb_bin = require_pg16_bin("initdb");
        let basebackup_bin = require_pg16_bin("pg_basebackup");

        let ns = match create_namespace("pginfo-replica-flow") {
            Ok(ns) => ns,
            Err(err) => panic!("namespace create failed: {err}"),
        };

        let primary_data = match prepare_pgdata_dir(&ns, "primary") {
            Ok(path) => path,
            Err(err) => {
                let _ = cleanup_namespace(ns);
                panic!("primary data dir failed: {err}");
            }
        };
        let primary_socket = ns.child_dir("run/primary");
        let primary_logs = ns.child_dir("logs/primary");
        if let Err(err) = fs::create_dir_all(&primary_socket) {
            let _ = cleanup_namespace(ns);
            panic!("primary socket dir failed: {err}");
        }
        if let Err(err) = fs::create_dir_all(&primary_logs) {
            let _ = cleanup_namespace(ns);
            panic!("primary log dir failed: {err}");
        }

        let primary_reservation = match allocate_ports(1) {
            Ok(res) => res,
            Err(err) => {
                let _ = cleanup_namespace(ns);
                panic!("port allocation failed: {err}");
            }
        };
        let primary_port = primary_reservation.as_slice()[0];
        drop(primary_reservation);

        let mut primary = match spawn_pg16(PgInstanceSpec {
            postgres_bin: postgres_bin.clone(),
            initdb_bin: initdb_bin.clone(),
            data_dir: primary_data.clone(),
            socket_dir: primary_socket.clone(),
            log_dir: primary_logs.clone(),
            port: primary_port,
            startup_timeout: Duration::from_secs(25),
        })
        .await
        {
            Ok(handle) => handle,
            Err(err) => {
                let _ = cleanup_namespace(ns);
                panic!("spawn primary failed: {err}");
            }
        };

        let primary_dsn = format!(
            "host=127.0.0.1 port={} user=postgres dbname=postgres",
            primary_port
        );
        let (primary_client, primary_conn) =
            match tokio_postgres::connect(&primary_dsn, NoTls).await {
                Ok(pair) => pair,
                Err(err) => {
                    let _ = primary.shutdown().await;
                    let _ = cleanup_namespace(ns);
                    panic!("connect to primary failed: {err}");
                }
            };
        let primary_conn_task = tokio::spawn(primary_conn);

        if let Err(err) = primary_client.simple_query("SELECT 1;").await {
            let _ = primary.shutdown().await;
            let _ = cleanup_namespace(ns);
            panic!("verify primary connectivity failed: {err}");
        }
        drop(primary_client);
        let conn_result = primary_conn_task.await;
        match conn_result {
            Ok(Ok(())) => {}
            Ok(Err(err)) => {
                let _ = primary.shutdown().await;
                let _ = cleanup_namespace(ns);
                panic!("primary connection completion failed: {err}");
            }
            Err(err) => {
                let _ = primary.shutdown().await;
                let _ = cleanup_namespace(ns);
                panic!("primary connection join failed: {err}");
            }
        }

        let replica_data = ns.child_dir("pg16/replica/data");
        let replica_parent = match replica_data.parent() {
            Some(parent) => parent,
            None => &ns.root_dir,
        };
        if let Err(err) = fs::create_dir_all(replica_parent) {
            let _ = primary.shutdown().await;
            let _ = cleanup_namespace(ns);
            panic!("replica parent dir failed: {err}");
        }
        let basebackup_output = tokio::process::Command::new(basebackup_bin)
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
            .await;
        let output = match basebackup_output {
            Ok(output) => output,
            Err(err) => {
                let _ = primary.shutdown().await;
                let _ = cleanup_namespace(ns);
                panic!("pg_basebackup spawn failed: {err}");
            }
        };
        if !output.status.success() {
            let _ = primary.shutdown().await;
            let _ = cleanup_namespace(ns);
            panic!("pg_basebackup failed with status: {}", output.status);
        }

        let replica_socket = ns.child_dir("run/replica");
        let replica_logs = ns.child_dir("logs/replica");
        if let Err(err) = fs::create_dir_all(&replica_socket) {
            let _ = primary.shutdown().await;
            let _ = cleanup_namespace(ns);
            panic!("replica socket dir failed: {err}");
        }
        if let Err(err) = fs::create_dir_all(&replica_logs) {
            let _ = primary.shutdown().await;
            let _ = cleanup_namespace(ns);
            panic!("replica log dir failed: {err}");
        }

        let replica_reservation = match allocate_ports(1) {
            Ok(res) => res,
            Err(err) => {
                let _ = primary.shutdown().await;
                let _ = cleanup_namespace(ns);
                panic!("port allocation failed: {err}");
            }
        };
        let replica_port = replica_reservation.as_slice()[0];
        drop(replica_reservation);

        let mut replica = match spawn_pg16(PgInstanceSpec {
            postgres_bin,
            initdb_bin,
            data_dir: replica_data.clone(),
            socket_dir: replica_socket,
            log_dir: replica_logs,
            port: replica_port,
            startup_timeout: Duration::from_secs(30),
        })
        .await
        {
            Ok(handle) => handle,
            Err(err) => {
                let _ = primary.shutdown().await;
                let _ = cleanup_namespace(ns);
                panic!("spawn replica failed: {err}");
            }
        };

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

        let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
        let snapshot = loop {
            if let Err(err) = step_once(&mut ctx).await {
                let _ = replica.shutdown().await;
                let _ = primary.shutdown().await;
                let _ = cleanup_namespace(ns);
                panic!("replica step_once failed: {err}");
            }

            let polled = subscriber.latest().value;
            if matches!(polled, PgInfoState::Replica { .. }) {
                break polled;
            }

            if tokio::time::Instant::now() >= deadline {
                let _ = replica.shutdown().await;
                let _ = primary.shutdown().await;
                let _ = cleanup_namespace(ns);
                panic!("timed out waiting for replica state, got: {polled:?}");
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        };

        match snapshot {
            PgInfoState::Replica { common, .. } => {
                assert_eq!(common.sql, SqlStatus::Healthy);
                assert_eq!(common.readiness, Readiness::Ready);
            }
            other => {
                let _ = replica.shutdown().await;
                let _ = primary.shutdown().await;
                let _ = cleanup_namespace(ns);
                panic!("expected replica state, got: {other:?}");
            }
        }

        if let Err(err) = replica.shutdown().await {
            let _ = primary.shutdown().await;
            let _ = cleanup_namespace(ns);
            panic!("replica shutdown failed: {err}");
        }
        if let Err(err) = primary.shutdown().await {
            let _ = cleanup_namespace(ns);
            panic!("primary shutdown failed: {err}");
        }
        if let Err(err) = cleanup_namespace(ns) {
            panic!("namespace cleanup failed: {err}");
        }
    }
}
