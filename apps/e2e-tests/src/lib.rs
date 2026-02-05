//! E2E Tests Library for FlagLite
//!
//! This crate provides black-box end-to-end tests that:
//! - Spawn actual flaglite-api server processes
//! - Run actual flaglite CLI commands as subprocesses
//! - Parse command output to verify results
//!
//! The test harness and utilities are defined in `tests/common/` module.
//!
//! ## Running Tests
//!
//! First, build the binaries:
//! ```sh
//! cargo build --bins
//! ```
//!
//! Then run the tests:
//! ```sh
//! cargo test --package e2e-tests
//! ```
//!
//! ## Environment Variables
//!
//! - `FLAGLITE_API_BIN` - Path to flaglite-api binary (optional)
//! - `FLAGLITE_CLI_BIN` - Path to flaglite CLI binary (optional)
//! - `FLAGLITE_E2E_SERVER_TIMEOUT_SECS` - Server startup timeout (default: 30)

// This lib.rs is intentionally minimal.
// The actual test utilities are in tests/common/ module.
