use crate::config_v2::RuntimeConfigV2;
use crate::state::PgRoute;
use crate::state::{new_state_channel, StateSubscriber, UnixMillis, WorkerError, WorkerStatus};

use super::log_event::PgInfoLogEvent;
use super::query::poll_state_once;
use super::state::{
    PgConnInfo, PgInfoRuntime, PgInfoState, PgInfoStateChannel, PgInfoWorkerCtx, PgSslMode,
    SqlStatus,
};

pub(crate) fn bootstrap<'a>(
    cfg: &'a RuntimeConfigV2,
    log: crate::logging::LogSender,
) -> (PgInfoWorkerCtx<'a>, StateSubscriber<PgInfoState>) {
    let (publisher, state) = new_state_channel(PgInfoState::starting());

    (
        PgInfoWorkerCtx {
            cfg,
            state_channel: PgInfoStateChannel {
                publisher,
                last_emitted_sql_status: None,
            },
            runtime: PgInfoRuntime { log },
        },
        state,
    )
}

pub(crate) async fn run(mut ctx: PgInfoWorkerCtx<'_>) -> Result<(), WorkerError> {
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.cfg.timing.ha_loop_interval).await;
    }
}

pub(crate) async fn step_once(ctx: &mut PgInfoWorkerCtx<'_>) -> Result<(), WorkerError> {
    let now = now_unix_millis()?;
    let poll = poll_state_once(
        &probe_conninfo(ctx.cfg),
        WorkerStatus::Running,
        SqlStatus::Healthy,
        now,
    )
    .await;
    let next_state = match poll {
        Ok(polled) => polled,
        Err(ref err) => {
            ctx.runtime
                .log
                .send(PgInfoLogEvent::PollFailed {
                    cause: err.to_string(),
                })
                .map_err(|err| {
                    WorkerError::Message(format!("pginfo poll failure log emit failed: {err}"))
                })?;
            PgInfoState::unknown(WorkerStatus::Running, SqlStatus::Unreachable, Some(now))
        }
    };

    let next_sql = pginfo_sql_status(&next_state);
    let prev_sql = ctx
        .state_channel
        .last_emitted_sql_status
        .unwrap_or(SqlStatus::Unknown);
    if prev_sql != next_sql {
        ctx.runtime
            .log
            .send(PgInfoLogEvent::SqlTransition {
                previous: Some(prev_sql),
                next: next_sql,
            })
            .map_err(|err| {
                WorkerError::Message(format!("pginfo sql transition log emit failed: {err}"))
            })?;
        ctx.state_channel.last_emitted_sql_status = Some(next_sql);
    }

    ctx.state_channel
        .publisher
        .publish(next_state)
        .map_err(|err| {
            WorkerError::Message(format!(
                "pginfo publish failed for {:?}: {err}",
                ctx.cfg.member_id
            ))
        })?;
    Ok(())
}

fn pginfo_sql_status(state: &PgInfoState) -> SqlStatus {
    match state {
        PgInfoState::Unknown { common } => common.sql,
        PgInfoState::Primary { common, .. } => common.sql,
        PgInfoState::Replica { common, .. } => common.sql,
    }
}

fn now_unix_millis() -> Result<UnixMillis, WorkerError> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|err| WorkerError::Message(format!("system clock before unix epoch: {err}")))?;
    let millis = u64::try_from(elapsed.as_millis())
        .map_err(|err| WorkerError::Message(format!("unix millis conversion failed: {err}")))?;
    Ok(UnixMillis(millis))
}

fn probe_conninfo(cfg: &RuntimeConfigV2) -> PgConnInfo {
    PgConnInfo {
        route: PgRoute::new(
            crate::state::PgEndpoint::UnixSocket {
                socket_dir: cfg.postgres.socket_dir.clone(),
                port: cfg.postgres.listen_port,
            },
            None,
        ),
        user: cfg.postgres.superuser.username.clone(),
        dbname: cfg.postgres.local_database.clone(),
        application_name: None,
        connect_timeout_s: None,
        options: None,
        tls: super::conninfo::PgClientTls {
            mode: PgSslMode::Disable,
            root_cert: None,
            client_cert: None,
            client_key: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::probe_conninfo;
    use crate::{
        config_v2::{load_runtime_config_contents, render_runtime_test_config_toml},
        pginfo::state::PgSslMode,
        state::PgEndpoint,
    };
    use std::path::Path;

    #[test]
    fn probe_conninfo_uses_local_socket_without_tls() -> Result<(), String> {
        let cfg = load_runtime_config_contents(
            render_runtime_test_config_toml(
                "cluster",
                "cluster",
                "node-a",
                (
                    Path::new("/tmp/pgtm-data"),
                    Path::new("/tmp/pgtm-socket"),
                    Path::new("/tmp/pgtm.log"),
                ),
                ["http://127.0.0.1:2379"],
                [
                    r#"[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000"#,
                    r#"[process.timeouts]
pg_rewind_ms = 120000
bootstrap_ms = 300000
fencing_ms = 30000"#,
                    r#"[logging]
level = "info"
capture_subprocess_output = true"#,
                    r#"[logging.postgres]
enabled = true
poll_interval_ms = 200
cleanup = { enabled = true, max_files = 20, max_age_seconds = 86400, protect_recent_seconds = 300 }"#,
                    r#"[logging.sinks.stderr]
enabled = true"#,
                    r#"[logging.sinks.file]
enabled = false"#,
                    r#"[api]
listen_addr = "127.0.0.1:8443"
transport = { transport = "http" }
auth = { type = "disabled" }"#,
                    r#"[debug]
enabled = false"#,
                ],
            )
            .map_err(|err| err.to_string())?
            .as_str(),
        )
        .map_err(|err| err.to_string())?;
        let conninfo = probe_conninfo(&cfg);

        match conninfo.route.endpoint() {
            PgEndpoint::UnixSocket { socket_dir, port } => {
                if socket_dir != Path::new("/tmp/pgtm-socket") {
                    return Err(format!("unexpected socket dir {}", socket_dir.display()));
                }
                if *port != 5432 {
                    return Err(format!("unexpected port {port}"));
                }
            }
            other => return Err(format!("expected unix socket endpoint, got {other:?}")),
        }

        if conninfo.dbname != "postgres" {
            return Err(format!("unexpected local database {}", conninfo.dbname));
        }
        if conninfo.tls.mode != PgSslMode::Disable {
            return Err(format!(
                "expected sslmode disable, got {:?}",
                conninfo.tls.mode
            ));
        }
        if conninfo.tls.root_cert.is_some()
            || conninfo.tls.client_cert.is_some()
            || conninfo.tls.client_key.is_some()
        {
            return Err("expected local probe conninfo to omit TLS files".to_string());
        }

        Ok(())
    }
}
