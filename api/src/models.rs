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
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            email: user.email,
            created_at: user.created_at,
        }
    }
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
    pub name: String, // development, staging, production
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
    pub email: String,
    pub password: String,
    pub project_name: Option<String>,
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
    pub email: String,
    pub password: String,
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
    pub email: String,
    pub exp: i64,
    pub iat: i64,
}

// ============ API Key Types ============

pub fn generate_project_api_key() -> String {
    let random: String = (0..32)
        .map(|_| {
            let idx = rand::random::<usize>() % 36;
            if idx < 10 {
                (b'0' + idx as u8) as char
            } else {
                (b'a' + (idx - 10) as u8) as char
            }
        })
        .collect();
    format!("ffl_proj_{}", random)
}

pub fn generate_env_api_key() -> String {
    let random: String = (0..32)
        .map(|_| {
            let idx = rand::random::<usize>() % 36;
            if idx < 10 {
                (b'0' + idx as u8) as char
            } else {
                (b'a' + (idx - 10) as u8) as char
            }
        })
        .collect();
    format!("ffl_env_{}", random)
}

#[allow(dead_code)]
pub fn is_project_api_key(key: &str) -> bool {
    key.starts_with("ffl_proj_")
}

#[allow(dead_code)]
pub fn is_env_api_key(key: &str) -> bool {
    key.starts_with("ffl_env_")
}
