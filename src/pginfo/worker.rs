use crate::state::{UnixMillis, WorkerError, WorkerStatus};

use super::log_event::PgInfoLogEvent;
use super::query::poll_once;
use super::state::{to_member_status, PgInfoState, PgInfoWorkerCtx, SqlStatus};

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
            to_member_status(WorkerStatus::Running, SqlStatus::Healthy, now, Some(polled))
        }
        Err(ref err) => {
            ctx.runtime
                .log
                .send(PgInfoLogEvent::PollFailed {
                    cause: err.to_string(),
                })
                .map_err(|err| {
                    WorkerError::Message(format!("pginfo poll failure log emit failed: {err}"))
                })?;
            to_member_status(WorkerStatus::Running, SqlStatus::Unreachable, now, None)
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
