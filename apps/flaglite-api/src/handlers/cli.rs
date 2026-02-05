//! CLI-compatible handlers
//! These handlers provide responses in the format expected by the CLI client

use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::{AppError, Result};
use crate::models::{
    generate_env_api_key, generate_project_api_key, AppState, Environment, Flag, FlagValue,
    Project,
};

const DEFAULT_ENVIRONMENTS: [&str; 3] = ["development", "staging", "production"];

// ============ CLI-compatible response types ============

/// Project response matching CLI expectations
#[derive(Debug, Serialize)]
pub struct CliProject {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Project> for CliProject {
    fn from(p: Project) -> Self {
        let slug = p.name.to_lowercase().replace(' ', "-");
        CliProject {
            id: Uuid::parse_str(&p.id).unwrap_or_else(|_| Uuid::nil()),
            name: p.name,
            description: None,
            slug,
            created_at: p.created_at,
            updated_at: p.created_at, // API doesn't track updated_at
        }
    }
}

/// Environment response matching CLI expectations
#[derive(Debug, Serialize)]
pub struct CliEnvironment {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub project_id: Uuid,
    pub is_production: bool,
    pub created_at: DateTime<Utc>,
}

impl CliEnvironment {
    fn from_env(e: Environment) -> Self {
        CliEnvironment {
            id: Uuid::parse_str(&e.id).unwrap_or_else(|_| Uuid::nil()),
            name: e.name.clone(),
            slug: e.name.to_lowercase(),
            project_id: Uuid::parse_str(&e.project_id).unwrap_or_else(|_| Uuid::nil()),
            is_production: e.name == "production",
            created_at: e.created_at,
        }
    }
}

/// Flag type enum matching CLI expectations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CliFlagType {
    #[default]
    Boolean,
    String,
    Number,
    Json,
}

/// Flag response matching CLI expectations
#[derive(Debug, Serialize)]
pub struct CliFlag {
    pub id: Uuid,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub flag_type: CliFlagType,
    pub project_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CliFlag {
    fn from_flag(f: Flag) -> Self {
        CliFlag {
            id: Uuid::parse_str(&f.id).unwrap_or_else(|_| Uuid::nil()),
            key: f.key,
            name: f.name,
            description: f.description,
            flag_type: CliFlagType::Boolean,
            project_id: Uuid::parse_str(&f.project_id).unwrap_or_else(|_| Uuid::nil()),
            created_at: f.created_at,
            updated_at: f.created_at,
        }
    }
}

/// Flag with state matching CLI expectations
#[derive(Debug, Serialize)]
pub struct CliFlagWithState {
    #[serde(flatten)]
    pub flag: CliFlag,
    pub enabled: bool,
    pub value: Option<serde_json::Value>,
}

/// Request to create a project
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Request to create a flag
#[derive(Debug, Deserialize)]
pub struct CreateFlagRequest {
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub flag_type: CliFlagType,
    #[serde(default)]
    pub enabled: bool,
}

/// Query params for flag operations
#[derive(Debug, Deserialize)]
pub struct FlagQuery {
    pub environment: Option<String>,
}

// ============ Handlers ============

/// GET /projects - List all projects for authenticated user
pub async fn list_projects(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<CliProject>>> {
    let projects = state.storage.list_projects_by_user(&user.id).await?;
    let responses: Vec<CliProject> = projects.into_iter().map(|p| p.into()).collect();
    Ok(Json(responses))
}

/// POST /projects - Create a new project
pub async fn create_project(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<CliProject>> {
    let name = req.name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest(
            "Project name cannot be empty".to_string(),
        ));
    }
    if name.len() > 255 {
        return Err(AppError::BadRequest(
            "Project name must be at most 255 characters".to_string(),
        ));
    }

    let now = Utc::now();
    let project_id = Uuid::new_v4().to_string();
    let project_api_key = generate_project_api_key();

    let project = Project {
        id: project_id.clone(),
        user_id: user.id.clone(),
        name: name.to_string(),
        api_key: project_api_key,
        created_at: now,
    };

    state.storage.create_project(&project).await?;

    // Create 3 default environments
    for env_name in DEFAULT_ENVIRONMENTS {
        let env_id = Uuid::new_v4().to_string();
        let env_api_key = generate_env_api_key();

        let env = Environment {
            id: env_id,
            project_id: project_id.clone(),
            name: env_name.to_string(),
            api_key: env_api_key,
            created_at: now,
        };

        state.storage.create_environment(&env).await?;
    }

    Ok(Json(project.into()))
}

/// GET /projects/:project_id/environments - List environments for a project
pub async fn list_environments(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<CliEnvironment>>> {
    // Verify project belongs to user
    let project = state
        .storage
        .get_project_by_id(&project_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    if project.user_id != user.id {
        return Err(AppError::NotFound("Project not found".to_string()));
    }

    let environments = state
        .storage
        .list_environments_by_project(&project_id)
        .await?;
    let responses: Vec<CliEnvironment> = environments
        .into_iter()
        .map(CliEnvironment::from_env)
        .collect();
    Ok(Json(responses))
}

/// GET /projects/:project_id/flags - List flags for a project
pub async fn list_flags(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(project_id): Path<String>,
    Query(query): Query<FlagQuery>,
) -> Result<Json<Vec<CliFlagWithState>>> {
    // Verify project belongs to user
    let project = state
        .storage
        .get_project_by_id(&project_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    if project.user_id != user.id {
        return Err(AppError::NotFound("Project not found".to_string()));
    }

    let flags = state.storage.list_flags_by_project(&project_id).await?;

    // Get environment for state lookup (default to production)
    let env_name = query.environment.as_deref().unwrap_or("production");
    let environment = state
        .storage
        .get_environment_by_name(&project_id, env_name)
        .await?;

    let mut responses = Vec::new();
    for flag in flags {
        let enabled = if let Some(ref env) = environment {
            state
                .storage
                .get_flag_value(&flag.id, &env.id)
                .await?
                .map(|fv| fv.enabled)
                .unwrap_or(false)
        } else {
            false
        };

        responses.push(CliFlagWithState {
            flag: CliFlag::from_flag(flag),
            enabled,
            value: None,
        });
    }

    Ok(Json(responses))
}

/// POST /projects/:project_id/flags - Create a new flag
pub async fn create_flag(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(project_id): Path<String>,
    Json(req): Json<CreateFlagRequest>,
) -> Result<Json<CliFlag>> {
    // Verify project belongs to user
    let project = state
        .storage
        .get_project_by_id(&project_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    if project.user_id != user.id {
        return Err(AppError::NotFound("Project not found".to_string()));
    }

    // Validate key
    if req.key.is_empty() || req.key.len() > 255 {
        return Err(AppError::BadRequest("Invalid flag key".to_string()));
    }
    if !req
        .key
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::BadRequest(
            "Flag key can only contain alphanumeric characters, hyphens, and underscores"
                .to_string(),
        ));
    }

    // Check for duplicate
    if state
        .storage
        .get_flag_by_key(&project_id, &req.key)
        .await?
        .is_some()
    {
        return Err(AppError::BadRequest(format!(
            "Flag '{}' already exists",
            req.key
        )));
    }

    let now = Utc::now();
    let flag_id = Uuid::new_v4().to_string();

    let flag = Flag {
        id: flag_id.clone(),
        project_id: project_id.clone(),
        key: req.key.clone(),
        name: req.name.clone(),
        description: req.description.clone(),
        created_at: now,
    };

    state.storage.create_flag(&flag).await?;

    // Create flag values for all environments
    let environments = state
        .storage
        .list_environments_by_project(&project_id)
        .await?;

    for env in &environments {
        let fv_id = Uuid::new_v4().to_string();
        let flag_value = FlagValue {
            id: fv_id,
            flag_id: flag_id.clone(),
            environment_id: env.id.clone(),
            enabled: req.enabled,
            rollout_percentage: 100,
            updated_at: now,
        };

        state.storage.create_flag_value(&flag_value).await?;
    }

    Ok(Json(CliFlag::from_flag(flag)))
}

/// GET /projects/:project_id/flags/:key - Get a specific flag
pub async fn get_flag(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path((project_id, key)): Path<(String, String)>,
    Query(query): Query<FlagQuery>,
) -> Result<Json<CliFlagWithState>> {
    // Verify project belongs to user
    let project = state
        .storage
        .get_project_by_id(&project_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    if project.user_id != user.id {
        return Err(AppError::NotFound("Project not found".to_string()));
    }

    let flag = state
        .storage
        .get_flag_by_key(&project_id, &key)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Flag '{}' not found", key)))?;

    // Get environment for state lookup
    let env_name = query.environment.as_deref().unwrap_or("production");
    let environment = state
        .storage
        .get_environment_by_name(&project_id, env_name)
        .await?;

    let enabled = if let Some(ref env) = environment {
        state
            .storage
            .get_flag_value(&flag.id, &env.id)
            .await?
            .map(|fv| fv.enabled)
            .unwrap_or(false)
    } else {
        false
    };

    Ok(Json(CliFlagWithState {
        flag: CliFlag::from_flag(flag),
        enabled,
        value: None,
    }))
}

/// POST /projects/:project_id/flags/:key/toggle - Toggle a flag
pub async fn toggle_flag(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path((project_id, key)): Path<(String, String)>,
    Query(query): Query<FlagQuery>,
) -> Result<Json<CliFlagWithState>> {
    // Verify project belongs to user
    let project = state
        .storage
        .get_project_by_id(&project_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    if project.user_id != user.id {
        return Err(AppError::NotFound("Project not found".to_string()));
    }

    let flag = state
        .storage
        .get_flag_by_key(&project_id, &key)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Flag '{}' not found", key)))?;

    let env_name = query
        .environment
        .ok_or_else(|| AppError::BadRequest("environment query param is required".to_string()))?;

    let environment = state
        .storage
        .get_environment_by_name(&project_id, &env_name)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Environment '{}' not found", env_name)))?;

    let now = Utc::now();

    // Toggle the flag
    let existing = state
        .storage
        .get_flag_value(&flag.id, &environment.id)
        .await?;

    let new_enabled = match existing {
        Some(fv) => {
            let toggled = !fv.enabled;
            let updated_fv = FlagValue {
                id: fv.id,
                flag_id: flag.id.clone(),
                environment_id: environment.id,
                enabled: toggled,
                rollout_percentage: fv.rollout_percentage,
                updated_at: now,
            };
            state.storage.update_flag_value(&updated_fv).await?;
            toggled
        }
        None => {
            let fv_id = Uuid::new_v4().to_string();
            let flag_value = FlagValue {
                id: fv_id,
                flag_id: flag.id.clone(),
                environment_id: environment.id,
                enabled: true,
                rollout_percentage: 100,
                updated_at: now,
            };
            state.storage.create_flag_value(&flag_value).await?;
            true
        }
    };

    Ok(Json(CliFlagWithState {
        flag: CliFlag::from_flag(flag),
        enabled: new_enabled,
        value: None,
    }))
}

/// DELETE /projects/:project_id/flags/:key - Delete a flag
pub async fn delete_flag(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path((project_id, key)): Path<(String, String)>,
) -> Result<()> {
    // Verify project belongs to user
    let project = state
        .storage
        .get_project_by_id(&project_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    if project.user_id != user.id {
        return Err(AppError::NotFound("Project not found".to_string()));
    }

    let flag = state
        .storage
        .get_flag_by_key(&project_id, &key)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Flag '{}' not found", key)))?;

    // Delete flag (cascade should handle flag_values)
    state.storage.delete_flag(&flag.id).await?;

    Ok(())
}
