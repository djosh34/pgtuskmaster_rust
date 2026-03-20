use crate::config_v2::RuntimeConfigV2;
use crate::state::PgEndpoint;
use crate::state::{UnixMillis, WorkerError, WorkerStatus};

use super::log_event::PgInfoLogEvent;
use super::query::poll_state_once;
use super::state::{PgConnInfo, PgInfoState, PgInfoWorkerCtx, PgSslMode, SqlStatus};

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
                ctx.identity.member_id
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
        endpoint: PgEndpoint::UnixSocket {
            socket_dir: cfg.postgres.socket_dir.clone(),
            port: cfg.postgres.listen_port,
        },
        hostaddr: None,
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
        config_v2::load_runtime_config,
        pginfo::state::PgSslMode,
        state::PgEndpoint,
    };
    use std::{
        path::{Path, PathBuf},
        time::SystemTime,
    };

    #[test]
    fn probe_conninfo_uses_local_socket_without_tls() -> Result<(), String> {
        let path = write_temp_runtime_config()?;
        let cfg = load_runtime_config(path.as_path()).map_err(|err| err.to_string())?;
        let conninfo = probe_conninfo(&cfg);
        let _ = std::fs::remove_file(path);

        match conninfo.endpoint {
            PgEndpoint::UnixSocket { socket_dir, port } => {
                if socket_dir != Path::new("/tmp/pgtm-socket") {
                    return Err(format!("unexpected socket dir {}", socket_dir.display()));
                }
                if port != 5432 {
                    return Err(format!("unexpected port {port}"));
                }
            }
            other => return Err(format!("expected unix socket endpoint, got {other:?}")),
        }

        if conninfo.dbname != "postgres" {
            return Err(format!("unexpected local database {}", conninfo.dbname));
        }
        if conninfo.tls.mode != PgSslMode::Disable {
            return Err(format!("expected sslmode disable, got {:?}", conninfo.tls.mode));
        }
        if conninfo.tls.root_cert.is_some()
            || conninfo.tls.client_cert.is_some()
            || conninfo.tls.client_key.is_some()
        {
            return Err("expected local probe conninfo to omit TLS files".to_string());
        }

        Ok(())
    }

    fn write_temp_runtime_config() -> Result<PathBuf, String> {
        let path = std::env::temp_dir().join(format!(
            "pginfo-probe-runtime-{}-{}.toml",
            std::process::id(),
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|err| err.to_string())?
                .as_nanos()
        ));
        std::fs::write(
            &path,
            r#"
[cluster]
name = "cluster"
scope = "cluster"
member_id = "node-a"

[postgres]
local_database = "postgres"

[postgres.paths]
data_dir = "/tmp/pgtm-data"
socket_dir = "/tmp/pgtm-socket"
log_file = "/tmp/pgtm.log"

[postgres.network]
listen_host = "127.0.0.1"
listen_port = 5432

[postgres.roles.mandatory.superuser]
username = "postgres"
auth = { type = "password", password = { type = "string", value = "secret" } }

[postgres.roles.mandatory.replicator]
username = "replicator"
auth = { type = "password", password = { type = "string", value = "secret" } }

[postgres.roles.mandatory.rewinder]
username = "rewinder"
auth = { type = "password", password = { type = "string", value = "secret" } }

[postgres.access]
hba = { content = "local all all trust" }
ident = { content = "" }

[dcs]
endpoints = ["http://127.0.0.1:2379"]

[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000

[process.timeouts]
pg_rewind_ms = 120000
bootstrap_ms = 300000
fencing_ms = 30000

[logging]
level = "info"
capture_subprocess_output = true

[logging.postgres]
enabled = true
poll_interval_ms = 200
cleanup = { enabled = true, max_files = 20, max_age_seconds = 86400, protect_recent_seconds = 300 }

[logging.sinks.stderr]
enabled = true

[logging.sinks.file]
enabled = false

[api]
listen_addr = "127.0.0.1:8443"
transport = { transport = "http" }
auth = { type = "disabled" }

[debug]
enabled = false
"#,
        )
        .map_err(|err| err.to_string())?;
        Ok(path)
    }
}
