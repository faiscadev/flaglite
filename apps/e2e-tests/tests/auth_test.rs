//! Authentication E2E Tests (Black-Box)
//!
//! Tests authentication flows by:
//! - Spawning actual flaglite-api server
//! - Running actual flaglite CLI commands
//! - Parsing command output to verify results

mod common;

use common::{unique_username, TestHarness, TEST_PASSWORD};

/// Test signup creates a user with auto-generated username.
#[tokio::test]
async fn test_signup_creates_user() {
    let harness = TestHarness::new("signup_creates_user")
        .await
        .expect("Failed to create test harness");

    let user = harness.create_user("alice");
    let result = user.signup(None, TEST_PASSWORD);

    assert!(result.is_ok(), "Signup failed: {:?}", result.err());

    let info = result.unwrap();
    assert!(!info.username.is_empty(), "Username should not be empty");
    assert!(
        info.api_key.starts_with("flg_"),
        "API key should start with 'flg_', got: {}",
        info.api_key
    );
}

/// Test signup with custom username.
#[tokio::test]
async fn test_signup_with_custom_username() {
    let harness = TestHarness::new("signup_custom_username")
        .await
        .expect("Failed to create test harness");

    let username = unique_username();
    let user = harness.create_user("bob");
    let result = user.signup(Some(&username), TEST_PASSWORD);

    assert!(result.is_ok(), "Signup failed: {:?}", result.err());

    let info = result.unwrap();
    assert_eq!(info.username, username);
}

/// Test whoami returns user info after signup.
#[tokio::test]
async fn test_whoami_after_signup() {
    let harness = TestHarness::new("whoami_after_signup")
        .await
        .expect("Failed to create test harness");

    let username = unique_username();
    let user = harness.create_user("charlie");

    // Sign up first
    let signup_result = user.signup(Some(&username), TEST_PASSWORD);
    assert!(
        signup_result.is_ok(),
        "Signup failed: {:?}",
        signup_result.err()
    );

    // Now check whoami
    let whoami_result = user.whoami();
    assert!(
        whoami_result.is_ok(),
        "Whoami failed: {:?}",
        whoami_result.err()
    );

    let info = whoami_result.unwrap();
    assert_eq!(info.username, username);
}

/// Test login with correct password works.
#[tokio::test]
async fn test_login_with_correct_password() {
    let harness = TestHarness::new("login_correct_password")
        .await
        .expect("Failed to create test harness");

    let username = unique_username();
    let user = harness.create_user("dave");

    // Sign up first
    let signup_result = user.signup(Some(&username), TEST_PASSWORD);
    assert!(
        signup_result.is_ok(),
        "Signup failed: {:?}",
        signup_result.err()
    );

    // Logout
    let logout_result = user.logout();
    assert!(
        logout_result.is_ok(),
        "Logout failed: {:?}",
        logout_result.err()
    );

    // Now login
    let login_result = user.login(&username, TEST_PASSWORD);
    assert!(
        login_result.is_ok(),
        "Login failed: {:?}",
        login_result.err()
    );

    // Verify we're logged in with whoami
    let whoami_result = user.whoami();
    assert!(
        whoami_result.is_ok(),
        "Whoami failed after login: {:?}",
        whoami_result.err()
    );
}

/// Test login with wrong password is rejected.
#[tokio::test]
async fn test_login_with_wrong_password() {
    let harness = TestHarness::new("login_wrong_password")
        .await
        .expect("Failed to create test harness");

    let username = unique_username();
    let user = harness.create_user("eve");

    // Sign up first
    let signup_result = user.signup(Some(&username), TEST_PASSWORD);
    assert!(
        signup_result.is_ok(),
        "Signup failed: {:?}",
        signup_result.err()
    );

    // Logout
    let _ = user.logout();

    // Try to login with wrong password
    let login_result = user.login(&username, "wrongpassword");
    assert!(
        login_result.is_err(),
        "Login should have failed with wrong password"
    );
}

/// Test whoami fails without authentication.
#[tokio::test]
async fn test_whoami_requires_auth() {
    let harness = TestHarness::new("whoami_requires_auth")
        .await
        .expect("Failed to create test harness");

    // Create user without signup
    let user = harness.create_user("frank");

    // Try whoami without signing up - should fail
    let result = user.whoami();
    assert!(result.is_err(), "Whoami should fail without auth");
}

/// Test duplicate signup is rejected.
#[tokio::test]
async fn test_signup_duplicate_username_rejected() {
    let harness = TestHarness::new("signup_duplicate")
        .await
        .expect("Failed to create test harness");

    let username = unique_username();

    // First user signs up
    let user1 = harness.create_user("grace");
    let result1 = user1.signup(Some(&username), TEST_PASSWORD);
    assert!(result1.is_ok(), "First signup failed: {:?}", result1.err());

    // Second user tries same username - should fail
    let user2 = harness.create_user("henry");
    let result2 = user2.signup(Some(&username), TEST_PASSWORD);
    assert!(
        result2.is_err(),
        "Second signup should have failed for duplicate username"
    );
}

/// Test logout clears credentials.
#[tokio::test]
async fn test_logout_clears_auth() {
    let harness = TestHarness::new("logout_clears_auth")
        .await
        .expect("Failed to create test harness");

    let user = harness.create_user("ivy");

    // Sign up
    let signup_result = user.signup(None, TEST_PASSWORD);
    assert!(
        signup_result.is_ok(),
        "Signup failed: {:?}",
        signup_result.err()
    );

    // Verify logged in
    let whoami_before = user.whoami();
    assert!(whoami_before.is_ok(), "Should be logged in after signup");

    // Logout
    let logout_result = user.logout();
    assert!(
        logout_result.is_ok(),
        "Logout failed: {:?}",
        logout_result.err()
    );

    // Verify no longer authenticated
    let whoami_after = user.whoami();
    assert!(
        whoami_after.is_err(),
        "Should not be authenticated after logout"
    );
}
