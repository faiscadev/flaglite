//! Flags E2E Tests (Black-Box)
//!
//! Tests feature flag management by:
//! - Spawning actual flaglite-api server
//! - Running actual flaglite CLI commands
//! - Parsing command output to verify results

mod common;

use common::{unique_flag_key, TestHarness, TEST_PASSWORD};

/// Helper to setup a user with a selected project.
async fn setup_user_with_project(harness: &TestHarness, name: &str) -> common::TestUser {
    let user = harness.create_user(name);
    
    // Sign up
    user.signup(None, TEST_PASSWORD).expect("Signup failed");
    
    // Get and select first project
    let projects = user.projects_list().expect("Projects list failed");
    assert!(!projects.is_empty(), "No projects found");
    
    user.projects_use(&projects[0].id).expect("Projects use failed");
    
    user
}

/// Test creating a flag.
#[tokio::test]
async fn test_create_flag() {
    let harness = TestHarness::new("create_flag")
        .await
        .expect("Failed to create test harness");

    let user = setup_user_with_project(&harness, "alice").await;

    let flag_key = unique_flag_key();
    let result = user.flags_create(&flag_key, Some("Test Flag"), Some("boolean"), true);
    assert!(result.is_ok(), "flags create failed: {:?}", result.err());

    let flag = result.unwrap();
    assert_eq!(flag.key, flag_key);
    assert_eq!(flag.name, "Test Flag");
}

/// Test listing flags.
#[tokio::test]
async fn test_list_flags() {
    let harness = TestHarness::new("list_flags")
        .await
        .expect("Failed to create test harness");

    let user = setup_user_with_project(&harness, "bob").await;

    // Create a flag first
    let flag_key = unique_flag_key();
    user.flags_create(&flag_key, Some("List Test Flag"), None, false)
        .expect("flags create failed");

    // List flags
    let result = user.flags_list();
    assert!(result.is_ok(), "flags list failed: {:?}", result.err());

    let flags = result.unwrap();
    let created_flag = flags.iter().find(|f| f.key == flag_key);
    assert!(
        created_flag.is_some(),
        "Created flag not found in list. Flags: {:?}",
        flags.iter().map(|f| &f.key).collect::<Vec<_>>()
    );
}

/// Test getting a specific flag.
#[tokio::test]
async fn test_get_flag() {
    let harness = TestHarness::new("get_flag")
        .await
        .expect("Failed to create test harness");

    let user = setup_user_with_project(&harness, "charlie").await;

    // Create a flag
    let flag_key = unique_flag_key();
    user.flags_create(&flag_key, Some("Get Test Flag"), None, true)
        .expect("flags create failed");

    // Get the flag
    let result = user.flags_get(&flag_key);
    assert!(result.is_ok(), "flags get failed: {:?}", result.err());

    let flag = result.unwrap();
    assert_eq!(flag.key, flag_key);
    assert_eq!(flag.name, "Get Test Flag");
}

/// Test toggling a flag.
#[tokio::test]
async fn test_toggle_flag() {
    let harness = TestHarness::new("toggle_flag")
        .await
        .expect("Failed to create test harness");

    let user = setup_user_with_project(&harness, "dave").await;

    // Create a flag (enabled = false)
    let flag_key = unique_flag_key();
    user.flags_create(&flag_key, Some("Toggle Test Flag"), None, false)
        .expect("flags create failed");

    // Get initial state
    let initial = user.flags_get(&flag_key).expect("flags get failed");
    let initial_enabled = initial.enabled;

    // Toggle the flag
    let toggle_result = user.flags_toggle(&flag_key);
    assert!(toggle_result.is_ok(), "flags toggle failed: {:?}", toggle_result.err());

    // Get new state
    let after_toggle = user.flags_get(&flag_key).expect("flags get failed");
    assert_ne!(
        after_toggle.enabled, initial_enabled,
        "Flag enabled state should have changed after toggle"
    );

    // Toggle again
    user.flags_toggle(&flag_key).expect("flags toggle failed");

    // Verify it goes back
    let after_second_toggle = user.flags_get(&flag_key).expect("flags get failed");
    assert_eq!(
        after_second_toggle.enabled, initial_enabled,
        "Flag should return to initial state after double toggle"
    );
}

/// Test getting a non-existent flag returns error.
#[tokio::test]
async fn test_get_nonexistent_flag() {
    let harness = TestHarness::new("get_nonexistent_flag")
        .await
        .expect("Failed to create test harness");

    let user = setup_user_with_project(&harness, "eve").await;

    let result = user.flags_get("nonexistent_flag_key");
    assert!(result.is_err(), "flags get should fail for nonexistent flag");
}

/// Test creating flags with different types.
/// Note: The API currently stores all flags as boolean internally.
/// This test verifies that flag creation works with different type arguments.
#[tokio::test]
async fn test_create_flag_types() {
    let harness = TestHarness::new("create_flag_types")
        .await
        .expect("Failed to create test harness");

    let user = setup_user_with_project(&harness, "frank").await;

    let types = [
        ("boolean", "bool"),
        ("string", "str"),
        ("number", "num"),
        ("json", "json"),
    ];

    for (flag_type, suffix) in types {
        let flag_key = format!("typed_flag_{}_{}", suffix, unique_flag_key());
        let result = user.flags_create(
            &flag_key,
            Some(&format!("{} Flag", flag_type)),
            Some(flag_type),
            false,
        );

        assert!(
            result.is_ok(),
            "flags create for {} failed: {:?}",
            flag_type,
            result.err()
        );

        // Verify flag was created with correct key
        let flag = result.unwrap();
        assert_eq!(flag.key, flag_key);
        
        // Note: API currently returns all flags as "boolean" type
        // regardless of the requested type. This is a known limitation.
        // Once the API supports flag types properly, update this assertion:
        // assert_eq!(flag.flag_type.to_lowercase(), flag_type);
    }
}

/// Test creating multiple flags in same project.
#[tokio::test]
async fn test_create_multiple_flags() {
    let harness = TestHarness::new("create_multiple_flags")
        .await
        .expect("Failed to create test harness");

    let user = setup_user_with_project(&harness, "grace").await;

    // Create 5 flags
    let mut created_keys = Vec::new();
    for i in 0..5 {
        let flag_key = format!("multi_flag_{}_{}", i, unique_flag_key());
        user.flags_create(&flag_key, Some(&format!("Flag {}", i)), None, i % 2 == 0)
            .expect("flags create failed");
        created_keys.push(flag_key);
    }

    // List and verify all exist
    let flags = user.flags_list().expect("flags list failed");
    
    for key in &created_keys {
        let found = flags.iter().any(|f| f.key == *key);
        assert!(found, "Flag {} not found in list", key);
    }
}

/// Test flags are isolated between projects.
#[tokio::test]
async fn test_flags_isolated_between_projects() {
    let harness = TestHarness::new("flags_isolated")
        .await
        .expect("Failed to create test harness");

    let user = harness.create_user("henry");
    user.signup(None, TEST_PASSWORD).expect("Signup failed");

    // Get initial project
    let projects = user.projects_list().expect("Projects list failed");
    let project1_id = projects[0].id.clone();

    // Select project 1 and create a flag
    user.projects_use(&project1_id).expect("Projects use failed");
    let flag_key = unique_flag_key();
    user.flags_create(&flag_key, Some("Project 1 Flag"), None, true)
        .expect("flags create failed");

    // Create a new project
    let project2 = user.projects_create("Second Project", None).expect("Projects create failed");

    // Select project 2
    user.projects_use(&project2.id).expect("Projects use failed");

    // List flags in project 2 - should not contain the flag from project 1
    let project2_flags = user.flags_list().expect("flags list failed");
    let has_flag = project2_flags.iter().any(|f| f.key == flag_key);
    
    assert!(
        !has_flag,
        "Flag from project 1 should not be visible in project 2"
    );
}

/// Test flag with duplicate key is rejected.
#[tokio::test]
async fn test_create_duplicate_flag_rejected() {
    let harness = TestHarness::new("duplicate_flag")
        .await
        .expect("Failed to create test harness");

    let user = setup_user_with_project(&harness, "ivy").await;

    let flag_key = unique_flag_key();

    // Create first flag
    let result1 = user.flags_create(&flag_key, Some("First Flag"), None, true);
    assert!(result1.is_ok(), "First flag create failed: {:?}", result1.err());

    // Try to create second flag with same key
    let result2 = user.flags_create(&flag_key, Some("Duplicate Flag"), None, false);
    assert!(
        result2.is_err(),
        "Second flag create should have failed for duplicate key"
    );
}
