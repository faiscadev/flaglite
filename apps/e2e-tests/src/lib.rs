//! E2E Test Utilities for FlagLite
//!
//! Provides helpers for creating test clients with unique users.

use flaglite_client::{FlagLiteClient, SignupResponse};
use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;

/// Counter for generating unique test identifiers
static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Get the API URL from environment or default to localhost:3456
pub fn api_url() -> String {
    std::env::var("FLAGLITE_API_URL").unwrap_or_else(|_| "http://localhost:3456".to_string())
}

/// Generate a unique test identifier
pub fn unique_id() -> String {
    let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let uuid = Uuid::new_v4();
    format!("test_{counter}_{}", &uuid.to_string()[..8])
}

/// Test client wrapper with authentication state
pub struct TestClient {
    pub client: FlagLiteClient,
    pub api_key: Option<String>,
    pub token: Option<String>,
    pub user_id: Option<String>,
    pub username: Option<String>,
    pub project_id: Option<Uuid>,
}

impl TestClient {
    /// Create a new unauthenticated client
    pub fn new() -> Self {
        Self {
            client: FlagLiteClient::new(api_url()),
            api_key: None,
            token: None,
            user_id: None,
            username: None,
            project_id: None,
        }
    }

    /// Create a new client and sign up with a unique user
    pub async fn signup() -> anyhow::Result<Self> {
        let username = format!("user_{}", unique_id());
        Self::signup_with_username(&username).await
    }

    /// Create a new client and sign up with a specific username
    pub async fn signup_with_username(username: &str) -> anyhow::Result<Self> {
        let client = FlagLiteClient::new(api_url());
        let password = "testpassword123";

        let response = client.signup(Some(username), password).await?;

        Ok(Self::from_signup_response(response))
    }

    /// Create a client from a signup response
    fn from_signup_response(response: SignupResponse) -> Self {
        let api_key = response.api_key.key.clone();
        let token = response.token.clone();
        let user_id = response.user.id.clone();
        let username = response.user.username.clone();

        // Note: SignupResponse may include a default project
        // For now we don't extract it since the type doesn't include it
        let project_id = None;

        Self {
            client: FlagLiteClient::new(api_url()).with_api_key(&api_key),
            api_key: Some(api_key),
            token: Some(token),
            user_id: Some(user_id),
            username: Some(username),
            project_id,
        }
    }

    /// Get an authenticated client using the stored API key
    pub fn authenticated(&self) -> FlagLiteClient {
        let mut client = FlagLiteClient::new(api_url());
        if let Some(ref key) = self.api_key {
            client = client.with_api_key(key);
        } else if let Some(ref token) = self.token {
            client = client.with_token(token);
        }
        client
    }

    /// Get an authenticated client using the JWT token
    pub fn with_token(&self) -> FlagLiteClient {
        let mut client = FlagLiteClient::new(api_url());
        if let Some(ref token) = self.token {
            client = client.with_token(token);
        }
        client
    }
}

impl Default for TestClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_id_is_unique() {
        let id1 = unique_id();
        let id2 = unique_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_api_url_default() {
        // Only test default if env var is not set
        if std::env::var("FLAGLITE_API_URL").is_err() {
            assert_eq!(api_url(), "http://localhost:3456");
        }
    }
}
