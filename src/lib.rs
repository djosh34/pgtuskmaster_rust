#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented
)]

pub mod api;
pub mod cli;
pub mod config;
pub(crate) mod backup;
pub mod dcs;
pub(crate) mod debug_api;
pub(crate) mod ha;
pub(crate) mod logging;
pub mod pginfo;
pub(crate) mod postgres_managed;
pub(crate) mod process;
pub(crate) mod self_exe;
pub(crate) mod tls;
pub mod runtime;
pub mod state;
pub mod wal;
#[cfg(test)]
pub(crate) mod test_harness;

#[cfg(test)]
mod worker_contract_tests;
