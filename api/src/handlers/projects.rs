use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::{AppError, Result};
use crate::models::{
    generate_env_api_key, generate_project_api_key, AppState, Environment, EnvironmentResponse,
    Project, ProjectResponse,
};

const DEFAULT_ENVIRONMENTS: [&str; 3] = ["development", "staging", "production"];

/// Request to create a new project
#[derive(Debug, serde::Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
}

/// Response when creating a project (includes environments)
#[derive(Debug, serde::Serialize)]
pub struct CreateProjectResponse {
    pub project: ProjectResponse,
    pub environments: Vec<EnvironmentResponse>,
}

/// GET /v1/projects
/// List all projects for the authenticated user
pub async fn list_projects(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<ProjectResponse>>> {
    let projects = state.storage.list_projects_by_user(&user.id).await?;
    let responses: Vec<ProjectResponse> = projects.into_iter().map(|p| p.into()).collect();
    Ok(Json(responses))
}

/// POST /v1/projects
/// Create a new project with default environments
pub async fn create_project(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<CreateProjectResponse>> {
    // Validate project name
    let name = req.name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest("Project name cannot be empty".to_string()));
    }
    if name.len() > 255 {
        return Err(AppError::BadRequest("Project name must be at most 255 characters".to_string()));
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
    let mut environments = Vec::new();
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
        environments.push(env);
    }

    Ok(Json(CreateProjectResponse {
        project: project.into(),
        environments: environments.into_iter().map(|e| e.into()).collect(),
    }))
}

/// GET /v1/projects/:project_id/environments
/// List all environments for a project
pub async fn list_environments(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<EnvironmentResponse>>> {
    // First verify the project belongs to the user
    let project = state
        .storage
        .get_project_by_id(&project_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    if project.user_id != user.id {
        return Err(AppError::NotFound("Project not found".to_string()));
    }

    let environments = state.storage.list_environments_by_project(&project_id).await?;
    let responses: Vec<EnvironmentResponse> = environments.into_iter().map(|e| e.into()).collect();
    Ok(Json(responses))
}
