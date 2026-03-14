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
pub mod dcs;
#[cfg(test)]
#[allow(dead_code)]
pub(crate) mod dev_support;
#[cfg(all(not(test), feature = "internal-test-support"))]
pub mod dev_support;
pub mod ha;
pub(crate) mod logging;
pub mod pginfo;
pub(crate) mod postgres;
pub mod process;
pub mod runtime;
pub mod state;
pub(crate) mod tls;
