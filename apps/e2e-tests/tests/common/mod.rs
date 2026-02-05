//! Common test utilities for e2e tests
//!
//! This module provides a black-box test harness that:
//! - Spawns actual server processes (flaglite-api)
//! - Runs actual CLI commands (flaglite) as subprocesses
//! - Parses command output to verify results

#![allow(dead_code)]

pub mod harness;
pub mod utils;

pub use harness::{CommandResult, TestHarness, TestUser};
pub use utils::*;
