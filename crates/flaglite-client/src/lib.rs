//! FlagLite HTTP Client
//!
//! This crate provides an HTTP client for interacting with the FlagLite API.

mod client;

pub use client::FlagLiteClient;

// Re-export core types for convenience
pub use flaglite_core::*;
