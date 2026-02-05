//! Utility functions for e2e tests.

use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;

/// Counter for generating unique test identifiers.
static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate a unique test identifier.
///
/// Combines an atomic counter with a UUID fragment for uniqueness.
pub fn unique_id() -> String {
    let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let uuid = Uuid::new_v4();
    format!("{}_{}", counter, &uuid.to_string()[..8])
}

/// Generate a unique username for testing.
pub fn unique_username() -> String {
    format!("user_{}", unique_id())
}

/// Generate a unique flag key for testing.
pub fn unique_flag_key() -> String {
    format!("flag_{}", unique_id())
}

/// Generate a unique project name for testing.
pub fn unique_project_name() -> String {
    format!("Project {}", unique_id())
}

/// Default test password.
pub const TEST_PASSWORD: &str = "testpassword123";
