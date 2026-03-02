use std::time::Duration;

use crate::state::WorkerError;

use super::state::ProcessWorkerCtx;

pub(crate) async fn run(mut ctx: ProcessWorkerCtx) -> Result<(), WorkerError> {
    loop {
        step_once(&mut ctx).await?;
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}

pub(crate) async fn step_once(_ctx: &mut ProcessWorkerCtx) -> Result<(), WorkerError> {
    Ok(())
}
