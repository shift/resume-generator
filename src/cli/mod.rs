//! CLI subcommand handlers.
//!
//! Each subcommand has its own module containing a `run` function
//! with the full handler logic, keeping `main.rs` as a thin dispatcher.

pub mod build;
pub mod init;
pub mod keywords;
pub mod validate;
