#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented
)]

pub mod api;
pub(crate) mod backup;
pub mod cli;
pub mod config;
pub mod dcs;
pub(crate) mod debug_api;
pub(crate) mod ha;
pub(crate) mod logging;
pub mod pginfo;
pub(crate) mod postgres_managed;
pub(crate) mod process;
pub mod runtime;
pub(crate) mod self_exe;
pub mod state;
#[cfg(test)]
pub(crate) mod test_harness;
pub(crate) mod tls;
pub mod wal;
pub mod wal_passthrough;

#[cfg(test)]
mod worker_contract_tests;
