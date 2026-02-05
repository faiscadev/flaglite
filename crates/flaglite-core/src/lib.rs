//! FlagLite Core - Shared types and error definitions
//!
//! This crate provides common types used by both the CLI and API server.

pub mod error;
pub mod types;

pub use error::FlagLiteError;
pub use types::*;
