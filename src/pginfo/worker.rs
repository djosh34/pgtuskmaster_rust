use crate::state::{UnixMillis, WorkerStatus};
use crate::{
    logging::{InternalEvent, LogEvent, PgInfoEvent, SeverityText},
    state::WorkerError,
};

use super::query::poll_once;
use super::state::{to_member_status, PgInfoState, PgInfoWorkerCtx, SqlStatus};

fn emit_pginfo_event(
    ctx: &PgInfoWorkerCtx,
    origin: &str,
    event: PgInfoEvent,
    severity: SeverityText,
    error_prefix: &str,
) -> Result<(), WorkerError> {
    ctx.runtime
        .log
        .emit(origin, LogEvent::PgInfo(InternalEvent::new(severity, event)))
        .map_err(|err| WorkerError::Message(format!("{error_prefix}: {err}")))
}

pub(crate) async fn run(mut ctx: PgInfoWorkerCtx) -> Result<(), WorkerError> {
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(ctx.cadence.poll_interval).await;
    }
}

pub(crate) async fn step_once(ctx: &mut PgInfoWorkerCtx) -> Result<(), WorkerError> {
    let now = now_unix_millis()?;
    let poll = poll_once(&ctx.probe.to_conninfo()).await;
    let next_state = match poll {
        Ok(polled) => {
            to_member_status(WorkerStatus::Running, SqlStatus::Healthy, now, Some(polled))?
        }
        Err(ref err) => {
            emit_pginfo_event(
                ctx,
                "pginfo_worker::step_once",
                PgInfoEvent::PollFailed {
                    member_id: ctx.identity.self_id.0.clone(),
                    error: err.to_string(),
                },
                SeverityText::Warn,
                "pginfo poll failure log emit failed",
            )?;
            to_member_status(WorkerStatus::Running, SqlStatus::Unreachable, now, None)?
        }
    };

    let next_sql = pginfo_sql_status(&next_state);
    let prev_sql = ctx
        .state_channel
        .last_emitted_sql_status
        .clone()
        .unwrap_or(SqlStatus::Unknown);
    if prev_sql != next_sql {
        let severity = match (prev_sql.clone(), next_sql.clone()) {
            (SqlStatus::Healthy, SqlStatus::Unreachable) => SeverityText::Warn,
            (SqlStatus::Unreachable, SqlStatus::Healthy) => SeverityText::Info,
            _ => SeverityText::Debug,
        };
        emit_pginfo_event(
            ctx,
            "pginfo_worker::step_once",
            PgInfoEvent::SqlTransition {
                member_id: ctx.identity.self_id.0.clone(),
                previous: prev_sql.clone(),
                next: next_sql.clone(),
            },
            severity,
            "pginfo sql transition log emit failed",
        )?;
        ctx.state_channel.last_emitted_sql_status = Some(next_sql.clone());
    }

    ctx.state_channel
        .publisher
        .publish(next_state)
        .map_err(|err| {
            WorkerError::Message(format!(
                "pginfo publish failed for {:?}: {err}",
                ctx.identity.self_id
            ))
        })?;
    Ok(())
}

fn pginfo_sql_status(state: &PgInfoState) -> SqlStatus {
    match state {
        PgInfoState::Unknown { common } => common.sql.clone(),
        PgInfoState::Primary { common, .. } => common.sql.clone(),
        PgInfoState::Replica { common, .. } => common.sql.clone(),
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
