//! FlagLite shared types and API client
//!
//! This crate provides common types used by both the CLI and API server.

pub mod types;
pub mod client;
pub mod error;

pub use types::*;
pub use client::FlagLiteClient;
pub use error::FlagLiteError;
