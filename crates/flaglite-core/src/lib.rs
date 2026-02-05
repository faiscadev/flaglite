//! FlagLite Core - Shared types and error definitions
//!
//! This crate provides common types used by both the CLI and API server.

pub mod types;
pub mod error;

pub use types::*;
pub use error::FlagLiteError;
