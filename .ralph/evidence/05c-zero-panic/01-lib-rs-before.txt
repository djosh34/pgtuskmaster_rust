#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::todo,
        clippy::unimplemented
    )
)]

pub mod api;
pub mod config;
pub mod dcs;
pub(crate) mod debug_api;
pub(crate) mod ha;
pub(crate) mod pginfo;
pub(crate) mod process;
pub mod state;
#[cfg(any(test, feature = "test-harness"))]
pub(crate) mod test_harness;

#[cfg(test)]
mod worker_contract_tests;
