use thiserror::Error;

use super::state::{DecideInput, DecideOutput};

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub(crate) enum DecideError {
    #[error("decision failed")]
    Failed,
}

pub(crate) fn decide(input: DecideInput) -> Result<DecideOutput, DecideError> {
    Ok(DecideOutput {
        next: input.current,
        actions: Vec::new(),
    })
}
