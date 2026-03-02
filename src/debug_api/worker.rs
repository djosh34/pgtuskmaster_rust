use std::time::Duration;

use crate::state::WorkerError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DebugApiCtx;

pub(crate) async fn run(mut ctx: DebugApiCtx) -> Result<(), WorkerError> {
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}

pub(crate) async fn step_once(_ctx: &mut DebugApiCtx) -> Result<(), WorkerError> {
    Ok(())
}
