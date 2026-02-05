use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use std::sync::Arc;

use crate::storage::Storage;

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<dyn Storage>,
    pub jwt_secret: String,
}

// ============ User ============

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String, // UUID stored as string for SQLite compat
    pub username: String,
    pub password_hash: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
        }
    }
}

// ============ API Key ============

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: String,
    pub user_id: String,
    pub key_hash: String,
    pub key_prefix: String, // First 8 chars for display (e.g., "flg_a1b2")
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub id: String,
    pub key_prefix: String,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<ApiKey> for ApiKeyResponse {
    fn from(key: ApiKey) -> Self {
        ApiKeyResponse {
            id: key.id,
            key_prefix: key.key_prefix,
            name: key.name,
            created_at: key.created_at,
        }
    }
}

/// Response returned only on API key creation (includes full key)
#[derive(Debug, Serialize)]
pub struct ApiKeyCreatedResponse {
    pub id: String,
    pub key: String, // Full key - only shown once
    pub key_prefix: String,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============ Project ============

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub api_key: String, // ffl_proj_*
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub id: String,
    pub name: String,
    pub api_key: String,
    pub created_at: DateTime<Utc>,
}

impl From<Project> for ProjectResponse {
    fn from(p: Project) -> Self {
        ProjectResponse {
            id: p.id,
            name: p.name,
            api_key: p.api_key,
            created_at: p.created_at,
        }
    }
}

// ============ Environment ============

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Environment {
    pub id: String,
    pub project_id: String,
    pub name: String,    // development, staging, production
    pub api_key: String, // ffl_env_*
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct EnvironmentResponse {
    pub id: String,
    pub name: String,
    pub api_key: String,
    pub created_at: DateTime<Utc>,
}

impl From<Environment> for EnvironmentResponse {
    fn from(e: Environment) -> Self {
        EnvironmentResponse {
            id: e.id,
            name: e.name,
            api_key: e.api_key,
            created_at: e.created_at,
        }
    }
}

// ============ Flag ============

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Flag {
    pub id: String,
    pub project_id: String,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FlagValue {
    pub id: String,
    pub flag_id: String,
    pub environment_id: String,
    pub enabled: bool,
    pub rollout_percentage: i32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct FlagEnvironmentValue {
    pub enabled: bool,
    pub rollout: i32,
}

#[derive(Debug, Serialize)]
pub struct FlagResponse {
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub environments: HashMap<String, FlagEnvironmentValue>,
}

#[derive(Debug, Serialize)]
pub struct FlagEvaluationResponse {
    pub key: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct FlagToggleResponse {
    pub key: String,
    pub environment: String,
    pub enabled: bool,
}

// ============ API Requests ============

#[derive(Debug, Deserialize)]
pub struct SignupRequest {
    pub username: Option<String>, // Optional - auto-generated if not provided
    pub password: String,
    pub project_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SignupResponse {
    pub user: UserResponse,
    pub api_key: ApiKeyCreatedResponse,
    pub token: String,
    pub project: Option<ProjectResponse>,
    pub environments: Option<Vec<EnvironmentResponse>>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
    pub project: Option<ProjectResponse>,
    pub environments: Option<Vec<EnvironmentResponse>>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFlagRequest {
    pub key: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFlagValueRequest {
    pub enabled: Option<bool>,
    pub rollout_percentage: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ToggleFlagQuery {
    pub environment: String,
}

#[derive(Debug, Deserialize)]
pub struct EvaluateFlagQuery {
    pub user_id: Option<String>,
}

// ============ JWT Claims ============

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub username: String,
    pub exp: i64,
    pub iat: i64,
}

// ============ API Key Types ============

fn generate_random_alphanumeric(len: usize) -> String {
    (0..len)
        .map(|_| {
            let idx = rand::random::<usize>() % 36;
            if idx < 10 {
                (b'0' + idx as u8) as char
            } else {
                (b'a' + (idx - 10) as u8) as char
            }
        })
        .collect()
}

/// Generate user API key with flg_ prefix (32 random alphanumeric chars)
/// Example: flg_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6
pub fn generate_user_api_key() -> String {
    format!("flg_{}", generate_random_alphanumeric(32))
}

pub fn generate_project_api_key() -> String {
    format!("ffl_proj_{}", generate_random_alphanumeric(32))
}

pub fn generate_env_api_key() -> String {
    format!("ffl_env_{}", generate_random_alphanumeric(32))
}

/// Check if key is a user API key (flg_ prefix)
pub fn is_user_api_key(key: &str) -> bool {
    key.starts_with("flg_")
}

#[allow(dead_code)]
pub fn is_project_api_key(key: &str) -> bool {
    key.starts_with("ffl_proj_")
}

#[allow(dead_code)]
pub fn is_env_api_key(key: &str) -> bool {
    key.starts_with("ffl_env_")
}
