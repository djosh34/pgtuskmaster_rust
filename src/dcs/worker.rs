use std::time::Duration;

use crate::state::WorkerError;

use super::state::DcsWorkerCtx;

pub(crate) async fn run(mut ctx: DcsWorkerCtx) -> Result<(), WorkerError> {
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}

pub(crate) async fn step_once(_ctx: &mut DcsWorkerCtx) -> Result<(), WorkerError> {
    Ok(())
}
