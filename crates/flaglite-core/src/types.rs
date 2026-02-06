//! Shared types for FlagLite

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(default)]
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Project containing feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Environment within a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub project_id: Uuid,
    #[serde(default)]
    pub is_production: bool,
    pub created_at: DateTime<Utc>,
}

/// Feature flag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flag {
    pub id: Uuid,
    pub key: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub flag_type: FlagType,
    pub project_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Flag state in an environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagState {
    pub flag_id: Uuid,
    pub environment_id: Uuid,
    pub enabled: bool,
    #[serde(default)]
    pub value: Option<serde_json::Value>,
    pub updated_at: DateTime<Utc>,
}

/// Flag with its state in current environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagWithState {
    #[serde(flatten)]
    pub flag: Flag,
    pub enabled: bool,
    #[serde(default)]
    pub value: Option<serde_json::Value>,
}

/// Type of feature flag
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FlagType {
    Boolean,
    String,
    Number,
    Json,
}

impl std::fmt::Display for FlagType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlagType::Boolean => write!(f, "boolean"),
            FlagType::String => write!(f, "string"),
            FlagType::Number => write!(f, "number"),
            FlagType::Json => write!(f, "json"),
        }
    }
}

/// Request to create a flag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFlagRequest {
    pub key: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default = "default_flag_type")]
    pub flag_type: FlagType,
    #[serde(default)]
    pub enabled: bool,
}

fn default_flag_type() -> FlagType {
    FlagType::Boolean
}

/// Signup request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignupRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    pub password: String,
}

/// API key info (only shown on creation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyCreated {
    pub id: String,
    pub key: String, // Full key - only shown once!
    pub key_prefix: String,
    #[serde(default)]
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Signup response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignupResponse {
    pub user: User,
    pub api_key: ApiKeyCreated,
    pub token: String,
    #[serde(default)]
    pub project: Option<Project>,
    #[serde(default)]
    pub environments: Option<Vec<Environment>>,
}

/// Login request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Authentication response (login)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub details: Option<serde_json::Value>,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}
