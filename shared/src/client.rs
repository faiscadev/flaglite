//! FlagLite API client

use crate::error::FlagLiteError;
use crate::types::*;
use reqwest::{Client, StatusCode};

/// FlagLite API client
pub struct FlagLiteClient {
    client: Client,
    base_url: String,
    token: Option<String>,
    api_key: Option<String>,
}

impl FlagLiteClient {
    /// Create a new client with the given base URL
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            token: None,
            api_key: None,
        }
    }

    /// Set the authentication token (JWT)
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Set the API key for authentication
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    fn auth_header(&self) -> Result<String, FlagLiteError> {
        // Prefer API key over token
        if let Some(key) = &self.api_key {
            return Ok(format!("Bearer {}", key));
        }
        self.token
            .as_ref()
            .map(|t| format!("Bearer {}", t))
            .ok_or(FlagLiteError::NotAuthenticated)
    }

    async fn handle_error(&self, status: StatusCode, body: &str) -> FlagLiteError {
        if status == StatusCode::UNAUTHORIZED {
            return FlagLiteError::InvalidCredentials;
        }

        if status == StatusCode::TOO_MANY_REQUESTS {
            return FlagLiteError::RateLimited { retry_after: 60 };
        }

        if let Ok(err) = serde_json::from_str::<ApiErrorResponse>(body) {
            return FlagLiteError::ApiError {
                status: status.as_u16(),
                message: err.error,
            };
        }

        FlagLiteError::ApiError {
            status: status.as_u16(),
            message: body.to_string(),
        }
    }

    // === Auth ===

    /// Signup with optional username and password
    pub async fn signup(
        &self,
        username: Option<&str>,
        password: &str,
    ) -> Result<SignupResponse, FlagLiteError> {
        let url = format!("{}/v1/auth/signup", self.base_url);
        let req = SignupRequest {
            username: username.map(|s| s.to_string()),
            password: password.to_string(),
        };

        let resp = self.client.post(&url).json(&req).send().await?;

        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            return Err(self.handle_error(status, &body).await);
        }

        serde_json::from_str(&body).map_err(|e| FlagLiteError::InvalidResponse(e.to_string()))
    }

    /// Login with username and password
    pub async fn login(&self, username: &str, password: &str) -> Result<AuthResponse, FlagLiteError> {
        let url = format!("{}/v1/auth/login", self.base_url);
        let req = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let resp = self.client.post(&url).json(&req).send().await?;

        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            return Err(self.handle_error(status, &body).await);
        }

        serde_json::from_str(&body).map_err(|e| FlagLiteError::InvalidResponse(e.to_string()))
    }

    /// Get current user info
    pub async fn whoami(&self) -> Result<User, FlagLiteError> {
        let url = format!("{}/v1/auth/me", self.base_url);
        let auth = self.auth_header()?;

        let resp = self
            .client
            .get(&url)
            .header("Authorization", auth)
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            return Err(self.handle_error(status, &body).await);
        }

        serde_json::from_str(&body).map_err(|e| FlagLiteError::InvalidResponse(e.to_string()))
    }

    // === Projects ===

    /// List all projects
    pub async fn list_projects(&self) -> Result<Vec<Project>, FlagLiteError> {
        let url = format!("{}/v1/projects", self.base_url);
        let auth = self.auth_header()?;

        let resp = self
            .client
            .get(&url)
            .header("Authorization", auth)
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            return Err(self.handle_error(status, &body).await);
        }

        // Try paginated first, then plain array
        if let Ok(paginated) = serde_json::from_str::<PaginatedResponse<Project>>(&body) {
            return Ok(paginated.data);
        }

        serde_json::from_str(&body).map_err(|e| FlagLiteError::InvalidResponse(e.to_string()))
    }

    /// Create a new project
    pub async fn create_project(&self, req: CreateProjectRequest) -> Result<Project, FlagLiteError> {
        let url = format!("{}/v1/projects", self.base_url);
        let auth = self.auth_header()?;

        let resp = self
            .client
            .post(&url)
            .header("Authorization", auth)
            .json(&req)
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            return Err(self.handle_error(status, &body).await);
        }

        serde_json::from_str(&body).map_err(|e| FlagLiteError::InvalidResponse(e.to_string()))
    }

    // === Environments ===

    /// List environments for a project
    pub async fn list_environments(
        &self,
        project_id: &str,
    ) -> Result<Vec<Environment>, FlagLiteError> {
        let url = format!("{}/v1/projects/{}/environments", self.base_url, project_id);
        let auth = self.auth_header()?;

        let resp = self
            .client
            .get(&url)
            .header("Authorization", auth)
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            return Err(self.handle_error(status, &body).await);
        }

        if let Ok(paginated) = serde_json::from_str::<PaginatedResponse<Environment>>(&body) {
            return Ok(paginated.data);
        }

        serde_json::from_str(&body).map_err(|e| FlagLiteError::InvalidResponse(e.to_string()))
    }

    // === Flags ===

    /// List flags for a project (optionally in a specific environment)
    pub async fn list_flags(
        &self,
        project_id: &str,
        environment: Option<&str>,
    ) -> Result<Vec<FlagWithState>, FlagLiteError> {
        let mut url = format!("{}/v1/projects/{}/flags", self.base_url, project_id);
        if let Some(env) = environment {
            url = format!("{}?environment={}", url, env);
        }
        let auth = self.auth_header()?;

        let resp = self
            .client
            .get(&url)
            .header("Authorization", auth)
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            return Err(self.handle_error(status, &body).await);
        }

        if let Ok(paginated) = serde_json::from_str::<PaginatedResponse<FlagWithState>>(&body) {
            return Ok(paginated.data);
        }

        serde_json::from_str(&body).map_err(|e| FlagLiteError::InvalidResponse(e.to_string()))
    }

    /// Get a specific flag
    pub async fn get_flag(
        &self,
        project_id: &str,
        key: &str,
        environment: Option<&str>,
    ) -> Result<FlagWithState, FlagLiteError> {
        let mut url = format!("{}/v1/projects/{}/flags/{}", self.base_url, project_id, key);
        if let Some(env) = environment {
            url = format!("{}?environment={}", url, env);
        }
        let auth = self.auth_header()?;

        let resp = self
            .client
            .get(&url)
            .header("Authorization", auth)
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;

        if status == StatusCode::NOT_FOUND {
            return Err(FlagLiteError::FlagNotFound(key.to_string()));
        }

        if !status.is_success() {
            return Err(self.handle_error(status, &body).await);
        }

        serde_json::from_str(&body).map_err(|e| FlagLiteError::InvalidResponse(e.to_string()))
    }

    /// Create a new flag
    pub async fn create_flag(
        &self,
        project_id: &str,
        req: CreateFlagRequest,
    ) -> Result<Flag, FlagLiteError> {
        let url = format!("{}/v1/projects/{}/flags", self.base_url, project_id);
        let auth = self.auth_header()?;

        let resp = self
            .client
            .post(&url)
            .header("Authorization", auth)
            .json(&req)
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            return Err(self.handle_error(status, &body).await);
        }

        serde_json::from_str(&body).map_err(|e| FlagLiteError::InvalidResponse(e.to_string()))
    }

    /// Toggle a flag's enabled state
    pub async fn toggle_flag(
        &self,
        project_id: &str,
        key: &str,
        environment: &str,
    ) -> Result<FlagWithState, FlagLiteError> {
        let url = format!(
            "{}/v1/projects/{}/flags/{}/toggle?environment={}",
            self.base_url, project_id, key, environment
        );
        let auth = self.auth_header()?;

        let resp = self
            .client
            .post(&url)
            .header("Authorization", auth)
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;

        if status == StatusCode::NOT_FOUND {
            return Err(FlagLiteError::FlagNotFound(key.to_string()));
        }

        if !status.is_success() {
            return Err(self.handle_error(status, &body).await);
        }

        serde_json::from_str(&body).map_err(|e| FlagLiteError::InvalidResponse(e.to_string()))
    }

    /// Delete a flag
    pub async fn delete_flag(&self, project_id: &str, key: &str) -> Result<(), FlagLiteError> {
        let url = format!("{}/v1/projects/{}/flags/{}", self.base_url, project_id, key);
        let auth = self.auth_header()?;

        let resp = self
            .client
            .delete(&url)
            .header("Authorization", auth)
            .send()
            .await?;

        let status = resp.status();

        if status == StatusCode::NOT_FOUND {
            return Err(FlagLiteError::FlagNotFound(key.to_string()));
        }

        if !status.is_success() {
            let body = resp.text().await?;
            return Err(self.handle_error(status, &body).await);
        }

        Ok(())
    }
}
