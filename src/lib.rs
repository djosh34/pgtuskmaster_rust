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

pub mod config;
pub mod state;
pub(crate) mod api;
pub(crate) mod dcs;
pub(crate) mod debug_api;
pub(crate) mod ha;
pub(crate) mod pginfo;
pub(crate) mod process;
#[cfg(any(test, feature = "test-harness"))]
pub(crate) mod test_harness;

#[cfg(test)]
mod worker_contract_tests;
