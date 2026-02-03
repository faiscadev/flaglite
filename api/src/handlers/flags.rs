use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::Utc;
use std::collections::HashMap;
use std::io::Cursor;
use uuid::Uuid;

use crate::auth::{AuthProject, FlexAuth};
use crate::error::{AppError, Result};
use crate::models::{
    AppState, CreateFlagRequest, Environment, EvaluateFlagQuery, Flag, FlagEnvironmentValue,
    FlagEvaluationResponse, FlagResponse, FlagToggleResponse, FlagValue, ToggleFlagQuery,
    UpdateFlagValueRequest,
};

/// Deterministic percentage rollout using murmur3 hash
fn is_enabled_for_user(flag_key: &str, user_id: &str, rollout_percentage: i32) -> bool {
    let input = format!("{}:{}", flag_key, user_id);
    let hash = murmur3::murmur3_32(&mut Cursor::new(input.as_bytes()), 0).unwrap_or(0);
    let bucket = (hash % 100) as i32;
    bucket < rollout_percentage
}

/// Evaluate a flag (SDK endpoint - uses environment API key)
pub async fn evaluate_flag(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Query(query): Query<EvaluateFlagQuery>,
    auth: FlexAuth,
) -> Result<Json<FlagEvaluationResponse>> {
    let (project_id, environment_id) = match &auth {
        FlexAuth::Environment(env, project) => (project.id.clone(), Some(env.id.clone())),
        FlexAuth::Project(project) => (project.id.clone(), None),
    };

    // Get the flag
    let flag: Flag = sqlx::query_as(
        "SELECT id, project_id, key, name, description, created_at FROM flags WHERE project_id = ? AND key = ?",
    )
    .bind(&project_id)
    .bind(&key)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Flag '{}' not found", key)))?;

    // If using env API key, get that specific environment's value
    // If using project key, default to production
    let env_id = match environment_id {
        Some(id) => id,
        None => {
            // Default to production environment
            let env: Environment = sqlx::query_as(
                "SELECT id, project_id, name, api_key, created_at FROM environments WHERE project_id = ? AND name = 'production'",
            )
            .bind(&project_id)
            .fetch_optional(&state.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Production environment not found".to_string()))?;
            env.id
        }
    };

    // Get flag value for this environment
    let flag_value: Option<FlagValue> = sqlx::query_as(
        "SELECT id, flag_id, environment_id, enabled, rollout_percentage, updated_at FROM flag_values WHERE flag_id = ? AND environment_id = ?",
    )
    .bind(&flag.id)
    .bind(&env_id)
    .fetch_optional(&state.pool)
    .await?;

    let enabled = match flag_value {
        Some(fv) => {
            if !fv.enabled {
                false
            } else if fv.rollout_percentage >= 100 {
                true
            } else if fv.rollout_percentage <= 0 {
                false
            } else {
                // Percentage rollout
                match &query.user_id {
                    Some(user_id) => is_enabled_for_user(&key, user_id, fv.rollout_percentage),
                    None => {
                        // No user ID = random evaluation
                        let random = rand::random::<u32>() % 100;
                        (random as i32) < fv.rollout_percentage
                    }
                }
            }
        }
        None => false, // No flag value = disabled
    };

    Ok(Json(FlagEvaluationResponse { key, enabled }))
}

/// List all flags for a project
pub async fn list_flags(
    State(state): State<AppState>,
    AuthProject(project): AuthProject,
) -> Result<Json<Vec<FlagResponse>>> {
    // Get all flags for the project
    let flags: Vec<Flag> = sqlx::query_as(
        "SELECT id, project_id, key, name, description, created_at FROM flags WHERE project_id = ? ORDER BY created_at DESC",
    )
    .bind(&project.id)
    .fetch_all(&state.pool)
    .await?;

    // Get all environments
    let environments: Vec<Environment> = sqlx::query_as(
        "SELECT id, project_id, name, api_key, created_at FROM environments WHERE project_id = ?",
    )
    .bind(&project.id)
    .fetch_all(&state.pool)
    .await?;

    let env_map: HashMap<String, String> = environments
        .iter()
        .map(|e| (e.id.clone(), e.name.clone()))
        .collect();

    // Get all flag values
    let flag_ids: Vec<String> = flags.iter().map(|f| f.id.clone()).collect();
    
    let flag_values: Vec<FlagValue> = if !flag_ids.is_empty() {
        let placeholders = flag_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query_str = format!(
            "SELECT id, flag_id, environment_id, enabled, rollout_percentage, updated_at FROM flag_values WHERE flag_id IN ({})",
            placeholders
        );
        
        let mut query = sqlx::query_as(&query_str);
        for id in &flag_ids {
            query = query.bind(id);
        }
        query.fetch_all(&state.pool).await?
    } else {
        vec![]
    };

    // Group flag values by flag_id
    let mut flag_value_map: HashMap<String, Vec<FlagValue>> = HashMap::new();
    for fv in flag_values {
        flag_value_map
            .entry(fv.flag_id.clone())
            .or_default()
            .push(fv);
    }

    // Build response
    let mut responses = Vec::new();
    for flag in flags {
        let mut env_values: HashMap<String, FlagEnvironmentValue> = HashMap::new();

        // Initialize all environments with defaults
        for env in &environments {
            env_values.insert(
                env.name.clone(),
                FlagEnvironmentValue {
                    enabled: false,
                    rollout: 100,
                },
            );
        }

        // Override with actual values
        if let Some(values) = flag_value_map.get(&flag.id) {
            for fv in values {
                if let Some(env_name) = env_map.get(&fv.environment_id) {
                    env_values.insert(
                        env_name.clone(),
                        FlagEnvironmentValue {
                            enabled: fv.enabled,
                            rollout: fv.rollout_percentage,
                        },
                    );
                }
            }
        }

        responses.push(FlagResponse {
            key: flag.key,
            name: flag.name,
            description: flag.description,
            environments: env_values,
        });
    }

    Ok(Json(responses))
}

/// Create a new flag
pub async fn create_flag(
    State(state): State<AppState>,
    AuthProject(project): AuthProject,
    Json(req): Json<CreateFlagRequest>,
) -> Result<Json<FlagResponse>> {
    // Validate key format
    if req.key.is_empty() || req.key.len() > 255 {
        return Err(AppError::BadRequest("Invalid flag key".to_string()));
    }
    if !req.key.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(AppError::BadRequest(
            "Flag key can only contain alphanumeric characters, hyphens, and underscores".to_string(),
        ));
    }

    // Check for duplicate
    let existing: Option<Flag> = sqlx::query_as(
        "SELECT id, project_id, key, name, description, created_at FROM flags WHERE project_id = ? AND key = ?",
    )
    .bind(&project.id)
    .bind(&req.key)
    .fetch_optional(&state.pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::BadRequest(format!(
            "Flag '{}' already exists",
            req.key
        )));
    }

    let now = Utc::now();
    let flag_id = Uuid::new_v4().to_string();

    // Create the flag
    sqlx::query(
        "INSERT INTO flags (id, project_id, key, name, description, created_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&flag_id)
    .bind(&project.id)
    .bind(&req.key)
    .bind(&req.name)
    .bind(&req.description)
    .bind(now)
    .execute(&state.pool)
    .await?;

    // Get all environments and create default flag values
    let environments: Vec<Environment> = sqlx::query_as(
        "SELECT id, project_id, name, api_key, created_at FROM environments WHERE project_id = ?",
    )
    .bind(&project.id)
    .fetch_all(&state.pool)
    .await?;

    let mut env_values: HashMap<String, FlagEnvironmentValue> = HashMap::new();

    for env in &environments {
        let fv_id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO flag_values (id, flag_id, environment_id, enabled, rollout_percentage, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&fv_id)
        .bind(&flag_id)
        .bind(&env.id)
        .bind(false)
        .bind(100)
        .bind(now)
        .execute(&state.pool)
        .await?;

        env_values.insert(
            env.name.clone(),
            FlagEnvironmentValue {
                enabled: false,
                rollout: 100,
            },
        );
    }

    Ok(Json(FlagResponse {
        key: req.key,
        name: req.name,
        description: req.description,
        environments: env_values,
    }))
}

/// Update flag value for a specific environment
pub async fn update_flag_value(
    State(state): State<AppState>,
    AuthProject(project): AuthProject,
    Path((key, env_name)): Path<(String, String)>,
    Json(req): Json<UpdateFlagValueRequest>,
) -> Result<Json<FlagEnvironmentValue>> {
    // Get the flag
    let flag: Flag = sqlx::query_as(
        "SELECT id, project_id, key, name, description, created_at FROM flags WHERE project_id = ? AND key = ?",
    )
    .bind(&project.id)
    .bind(&key)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Flag '{}' not found", key)))?;

    // Get the environment
    let environment: Environment = sqlx::query_as(
        "SELECT id, project_id, name, api_key, created_at FROM environments WHERE project_id = ? AND name = ?",
    )
    .bind(&project.id)
    .bind(&env_name)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Environment '{}' not found", env_name)))?;

    // Get or create flag value
    let existing: Option<FlagValue> = sqlx::query_as(
        "SELECT id, flag_id, environment_id, enabled, rollout_percentage, updated_at FROM flag_values WHERE flag_id = ? AND environment_id = ?",
    )
    .bind(&flag.id)
    .bind(&environment.id)
    .fetch_optional(&state.pool)
    .await?;

    let now = Utc::now();

    let (enabled, rollout) = match existing {
        Some(fv) => {
            let new_enabled = req.enabled.unwrap_or(fv.enabled);
            let new_rollout = req.rollout_percentage.unwrap_or(fv.rollout_percentage);

            // Validate rollout percentage
            if new_rollout < 0 || new_rollout > 100 {
                return Err(AppError::BadRequest(
                    "Rollout percentage must be between 0 and 100".to_string(),
                ));
            }

            sqlx::query(
                "UPDATE flag_values SET enabled = ?, rollout_percentage = ?, updated_at = ? WHERE id = ?",
            )
            .bind(new_enabled)
            .bind(new_rollout)
            .bind(now)
            .bind(&fv.id)
            .execute(&state.pool)
            .await?;

            (new_enabled, new_rollout)
        }
        None => {
            let enabled = req.enabled.unwrap_or(false);
            let rollout = req.rollout_percentage.unwrap_or(100);

            if rollout < 0 || rollout > 100 {
                return Err(AppError::BadRequest(
                    "Rollout percentage must be between 0 and 100".to_string(),
                ));
            }

            let fv_id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO flag_values (id, flag_id, environment_id, enabled, rollout_percentage, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&fv_id)
            .bind(&flag.id)
            .bind(&environment.id)
            .bind(enabled)
            .bind(rollout)
            .bind(now)
            .execute(&state.pool)
            .await?;

            (enabled, rollout)
        }
    };

    Ok(Json(FlagEnvironmentValue {
        enabled,
        rollout,
    }))
}

/// Toggle a flag in a specific environment
pub async fn toggle_flag(
    State(state): State<AppState>,
    AuthProject(project): AuthProject,
    Path(key): Path<String>,
    Query(query): Query<ToggleFlagQuery>,
) -> Result<Json<FlagToggleResponse>> {
    // Get the flag
    let flag: Flag = sqlx::query_as(
        "SELECT id, project_id, key, name, description, created_at FROM flags WHERE project_id = ? AND key = ?",
    )
    .bind(&project.id)
    .bind(&key)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Flag '{}' not found", key)))?;

    // Get the environment
    let environment: Environment = sqlx::query_as(
        "SELECT id, project_id, name, api_key, created_at FROM environments WHERE project_id = ? AND name = ?",
    )
    .bind(&project.id)
    .bind(&query.environment)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Environment '{}' not found", query.environment)))?;

    let now = Utc::now();

    // Get current value and toggle
    let existing: Option<FlagValue> = sqlx::query_as(
        "SELECT id, flag_id, environment_id, enabled, rollout_percentage, updated_at FROM flag_values WHERE flag_id = ? AND environment_id = ?",
    )
    .bind(&flag.id)
    .bind(&environment.id)
    .fetch_optional(&state.pool)
    .await?;

    let new_enabled = match existing {
        Some(fv) => {
            let toggled = !fv.enabled;
            sqlx::query("UPDATE flag_values SET enabled = ?, updated_at = ? WHERE id = ?")
                .bind(toggled)
                .bind(now)
                .bind(&fv.id)
                .execute(&state.pool)
                .await?;
            toggled
        }
        None => {
            // No value exists, create with enabled = true (toggle from default false)
            let fv_id = Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO flag_values (id, flag_id, environment_id, enabled, rollout_percentage, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(&fv_id)
            .bind(&flag.id)
            .bind(&environment.id)
            .bind(true)
            .bind(100)
            .bind(now)
            .execute(&state.pool)
            .await?;
            true
        }
    };

    Ok(Json(FlagToggleResponse {
        key,
        environment: query.environment,
        enabled: new_enabled,
    }))
}
