//! High-performance, cross-platform ping utility written in Rust.
//!
//! Provides unprivileged ICMP probing on Windows using the IP Helper API,
//! colored ANSI output, asynchronous scheduling via Tokio, and complete statistics.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

pub mod app;
pub mod cli;
pub mod display;
pub mod engine;
pub mod error;
pub mod stats;

pub use app::run;
pub use cli::Cli;
pub use error::{PingError, Result};
pub use stats::PingStatistics;

/// Returns the current library version.
#[must_use]
pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
