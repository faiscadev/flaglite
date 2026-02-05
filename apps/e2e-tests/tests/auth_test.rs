//! Authentication E2E Tests

use e2e_tests::{api_url, unique_id, TestClient};
use flaglite_client::{FlagLiteClient, FlagLiteError};
use serial_test::serial;

/// Test signup creates a user with auto-generated username
#[tokio::test]
#[serial]
async fn test_signup_creates_user() {
    let client = FlagLiteClient::new(api_url());
    let password = "testpassword123";

    let result = client.signup(None, password).await;
    assert!(result.is_ok(), "Signup failed: {:?}", result.err());

    let response = result.unwrap();
    assert!(!response.user.id.is_empty());
    assert!(!response.user.username.is_empty());
    assert!(!response.api_key.key.is_empty());
    assert!(response.api_key.key.starts_with("flg_"));
    assert!(!response.token.is_empty());
}

/// Test signup with custom username
#[tokio::test]
#[serial]
async fn test_signup_with_custom_username() {
    let username = format!("custom_{}", unique_id());
    let password = "testpassword123";

    let client = FlagLiteClient::new(api_url());
    let result = client.signup(Some(&username), password).await;

    assert!(result.is_ok(), "Signup failed: {:?}", result.err());

    let response = result.unwrap();
    assert_eq!(response.user.username, username);
}

/// Test login with correct password returns token
#[tokio::test]
#[serial]
async fn test_login_with_correct_password() {
    // First signup
    let test_client = TestClient::signup().await.expect("Signup failed");
    let username = test_client.username.as_ref().unwrap();
    let password = "testpassword123";

    // Now login
    let client = FlagLiteClient::new(api_url());
    let result = client.login(username, password).await;

    assert!(result.is_ok(), "Login failed: {:?}", result.err());

    let response = result.unwrap();
    assert!(!response.token.is_empty());
    assert_eq!(response.user.username, *username);
}

/// Test login with wrong password is rejected
#[tokio::test]
#[serial]
async fn test_login_with_wrong_password() {
    // First signup
    let test_client = TestClient::signup().await.expect("Signup failed");
    let username = test_client.username.as_ref().unwrap();

    // Try to login with wrong password
    let client = FlagLiteClient::new(api_url());
    let result = client.login(username, "wrongpassword").await;

    assert!(
        result.is_err(),
        "Login should have failed with wrong password"
    );

    match result.unwrap_err() {
        FlagLiteError::InvalidCredentials => {}
        FlagLiteError::ApiError { status, .. } => {
            // Accept 400 or 401 as both indicate auth failure
            assert!(
                status == 400 || status == 401,
                "Expected 400 or 401, got {status}"
            );
        }
        other => panic!("Unexpected error type: {other:?}"),
    }
}

/// Test /auth/me returns user info when authenticated
#[tokio::test]
#[serial]
async fn test_me_returns_user_info() {
    let test_client = TestClient::signup().await.expect("Signup failed");
    let username = test_client.username.as_ref().unwrap().clone();

    // Use API key authentication
    let client = test_client.authenticated();
    let result = client.whoami().await;

    assert!(result.is_ok(), "whoami failed: {:?}", result.err());

    let user = result.unwrap();
    assert_eq!(user.username, username);
}

/// Test /auth/me fails without authentication
#[tokio::test]
#[serial]
async fn test_me_requires_auth() {
    let client = FlagLiteClient::new(api_url());
    let result = client.whoami().await;

    assert!(result.is_err(), "whoami should fail without auth");

    match result.unwrap_err() {
        FlagLiteError::NotAuthenticated => {}
        other => panic!("Expected NotAuthenticated error, got: {other:?}"),
    }
}

/// Test duplicate signup is rejected
#[tokio::test]
#[serial]
async fn test_signup_duplicate_username_rejected() {
    let username = format!("dupe_{}", unique_id());
    let password = "testpassword123";

    let client = FlagLiteClient::new(api_url());

    // First signup should succeed
    let result1 = client.signup(Some(&username), password).await;
    assert!(result1.is_ok(), "First signup failed: {:?}", result1.err());

    // Second signup with same username should fail
    let result2 = client.signup(Some(&username), password).await;
    assert!(
        result2.is_err(),
        "Second signup should have failed for duplicate username"
    );

    match result2.unwrap_err() {
        FlagLiteError::ApiError { status, .. } => {
            // Accept 400 or 409 as both indicate conflict
            assert!(
                status == 400 || status == 409,
                "Expected 400 or 409, got {status}"
            );
        }
        other => panic!("Unexpected error type: {other:?}"),
    }
}
