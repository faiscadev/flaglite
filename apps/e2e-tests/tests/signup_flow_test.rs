//! Signup Flow E2E Tests (Black-Box)
//!
//! Tests that signup properly saves project_id to config, allowing
//! immediate flag operations without manual project selection.
//!
//! This verifies the critical user flow:
//! 1. User signs up
//! 2. User can immediately create flags (no `projects use` required)

mod common;

use common::{unique_flag_key, TestHarness, TEST_PASSWORD};
use std::fs;

/// Test that signup saves project_id, enabling immediate flag creation.
///
/// This is the key test: after signup, the user should be able to create
/// flags without manually selecting a project via `projects use`.
#[tokio::test]
async fn test_signup_enables_immediate_flag_creation() {
    let harness = TestHarness::new("signup_immediate_flags")
        .await
        .expect("Failed to create test harness");

    let user = harness.create_user("alice");

    // Step 1: Sign up
    let signup_result = user.signup(None, TEST_PASSWORD);
    assert!(
        signup_result.is_ok(),
        "Signup failed: {:?}",
        signup_result.err()
    );

    // Step 2: Immediately create a flag WITHOUT calling `projects use`
    // This should work because signup saved the project_id to config
    let flag_key = unique_flag_key();
    let flag_result = user.flags_create(&flag_key, Some("E2E Test Flag"), None, true);

    assert!(
        flag_result.is_ok(),
        "Flag creation failed after signup (project_id not saved?): {:?}",
        flag_result.err()
    );

    let flag = flag_result.unwrap();
    assert_eq!(flag.key, flag_key, "Flag key should match");
    assert_eq!(flag.name, "E2E Test Flag", "Flag name should match");
}

/// Test that project_id is persisted in credentials file after signup.
#[tokio::test]
async fn test_signup_saves_project_id_to_config() {
    let harness = TestHarness::new("signup_saves_project_id")
        .await
        .expect("Failed to create test harness");

    let user = harness.create_user("bob");

    // Sign up
    let signup_result = user.signup(None, TEST_PASSWORD);
    assert!(
        signup_result.is_ok(),
        "Signup failed: {:?}",
        signup_result.err()
    );

    // Check credentials file contains project_id
    let credentials_path = user.home_dir.join(".flaglite").join("credentials.json");

    assert!(
        credentials_path.exists(),
        "Credentials file should exist at {credentials_path:?}"
    );

    let content = fs::read_to_string(&credentials_path).expect("Failed to read credentials file");

    // Parse as JSON and check for project_id
    let creds: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse credentials JSON");

    assert!(
        creds.get("project_id").is_some(),
        "Credentials should contain project_id. Got: {content}"
    );

    let project_id = creds["project_id"]
        .as_str()
        .expect("project_id should be a string");
    assert!(!project_id.is_empty(), "project_id should not be empty");
}

/// Test the complete signup flow: signup -> create flag -> verify flag exists.
#[tokio::test]
async fn test_signup_flow_complete() {
    let harness = TestHarness::new("signup_flow_complete")
        .await
        .expect("Failed to create test harness");

    let user = harness.create_user("charlie");

    // Step 1: Sign up
    let signup_result = user.signup(None, TEST_PASSWORD);
    assert!(
        signup_result.is_ok(),
        "Signup failed: {:?}",
        signup_result.err()
    );

    let signup_info = signup_result.unwrap();
    assert!(
        !signup_info.username.is_empty(),
        "Username should not be empty"
    );
    assert!(
        signup_info.api_key.starts_with("flg_"),
        "API key should start with 'flg_'"
    );

    // Step 2: Create a flag immediately (no project selection needed)
    let flag_key = unique_flag_key();
    let flag_result = user.flags_create(&flag_key, Some("Complete Flow Flag"), None, false);
    assert!(
        flag_result.is_ok(),
        "Flag creation failed: {:?}",
        flag_result.err()
    );

    // Step 3: Verify the flag exists via list
    let flags = user.flags_list().expect("flags list failed");
    let found = flags.iter().any(|f| f.key == flag_key);
    assert!(
        found,
        "Created flag should appear in flags list. Got: {:?}",
        flags.iter().map(|f| &f.key).collect::<Vec<_>>()
    );

    // Step 4: Verify the flag via get
    let flag = user.flags_get(&flag_key).expect("flags get failed");
    assert_eq!(flag.key, flag_key);
    assert_eq!(flag.name, "Complete Flow Flag");
}

/// Test that multiple flag operations work after signup without project selection.
#[tokio::test]
async fn test_signup_enables_all_flag_operations() {
    let harness = TestHarness::new("signup_all_flag_ops")
        .await
        .expect("Failed to create test harness");

    let user = harness.create_user("dave");

    // Sign up
    user.signup(None, TEST_PASSWORD).expect("Signup failed");

    // Create flag
    let flag_key = unique_flag_key();
    user.flags_create(&flag_key, Some("Ops Test Flag"), None, true)
        .expect("flags create failed");

    // List flags
    let flags = user.flags_list().expect("flags list failed");
    assert!(
        flags.iter().any(|f| f.key == flag_key),
        "Flag should be in list"
    );

    // Get flag
    let flag = user.flags_get(&flag_key).expect("flags get failed");
    assert!(flag.enabled, "Flag should be enabled");

    // Toggle flag
    let toggled = user.flags_toggle(&flag_key).expect("flags toggle failed");
    assert!(!toggled, "Flag should be disabled after toggle");

    // Verify toggle persisted
    let flag_after = user.flags_get(&flag_key).expect("flags get failed");
    assert!(!flag_after.enabled, "Flag should remain disabled");
}
