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
    AppState, CreateFlagRequest, EvaluateFlagQuery, Flag, FlagEnvironmentValue,
    FlagEvaluationResponse, FlagResponse, FlagToggleResponse, FlagValue, ToggleFlagQuery,
    UpdateFlagValueRequest,
};

/// Deterministic percentage rollout using murmur3 hash
fn is_enabled_for_user(flag_key: &str, user_id: &str, rollout_percentage: i32) -> bool {
    let input = format!("{flag_key}:{user_id}");
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
    let flag = state
        .storage
        .get_flag_by_key(&project_id, &key)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Flag '{key}' not found")))?;

    // If using env API key, get that specific environment's value
    // If using project key, default to production
    let env_id = match environment_id {
        Some(id) => id,
        None => {
            // Default to production environment
            let env = state
                .storage
                .get_environment_by_name(&project_id, "production")
                .await?
                .ok_or_else(|| {
                    AppError::NotFound("Production environment not found".to_string())
                })?;
            env.id
        }
    };

    // Get flag value for this environment
    let flag_value = state.storage.get_flag_value(&flag.id, &env_id).await?;

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
// Kept for future use
#[allow(dead_code)]
pub async fn list_flags(
    State(state): State<AppState>,
    AuthProject(project): AuthProject,
) -> Result<Json<Vec<FlagResponse>>> {
    // Get all flags for the project
    let flags = state.storage.list_flags_by_project(&project.id).await?;

    // Get all environments
    let environments = state
        .storage
        .list_environments_by_project(&project.id)
        .await?;

    let env_map: HashMap<String, String> = environments
        .iter()
        .map(|e| (e.id.clone(), e.name.clone()))
        .collect();

    // Get all flag values
    let flag_ids: Vec<String> = flags.iter().map(|f| f.id.clone()).collect();
    let flag_values = state
        .storage
        .list_flag_values_by_flag_ids(&flag_ids)
        .await?;

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
// Kept for future use
#[allow(dead_code)]
pub async fn create_flag(
    State(state): State<AppState>,
    AuthProject(project): AuthProject,
    Json(req): Json<CreateFlagRequest>,
) -> Result<Json<FlagResponse>> {
    // Validate key format
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
    let existing = state.storage.get_flag_by_key(&project.id, &req.key).await?;

    if existing.is_some() {
        return Err(AppError::BadRequest(format!(
            "Flag '{}' already exists",
            req.key
        )));
    }

    let now = Utc::now();
    let flag_id = Uuid::new_v4().to_string();

    // Create the flag
    let flag = Flag {
        id: flag_id.clone(),
        project_id: project.id.clone(),
        key: req.key.clone(),
        name: req.name.clone(),
        description: req.description.clone(),
        created_at: now,
    };

    state.storage.create_flag(&flag).await?;

    // Get all environments and create default flag values
    let environments = state
        .storage
        .list_environments_by_project(&project.id)
        .await?;

    let mut env_values: HashMap<String, FlagEnvironmentValue> = HashMap::new();

    for env in &environments {
        let fv_id = Uuid::new_v4().to_string();
        let flag_value = FlagValue {
            id: fv_id,
            flag_id: flag_id.clone(),
            environment_id: env.id.clone(),
            enabled: false,
            rollout_percentage: 100,
            updated_at: now,
        };

        state.storage.create_flag_value(&flag_value).await?;

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
// Kept for future use
#[allow(dead_code)]
pub async fn update_flag_value(
    State(state): State<AppState>,
    AuthProject(project): AuthProject,
    Path((key, env_name)): Path<(String, String)>,
    Json(req): Json<UpdateFlagValueRequest>,
) -> Result<Json<FlagEnvironmentValue>> {
    // Get the flag
    let flag = state
        .storage
        .get_flag_by_key(&project.id, &key)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Flag '{key}' not found")))?;

    // Get the environment
    let environment = state
        .storage
        .get_environment_by_name(&project.id, &env_name)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Environment '{env_name}' not found")))?;

    // Get or create flag value
    let existing = state
        .storage
        .get_flag_value(&flag.id, &environment.id)
        .await?;

    let now = Utc::now();

    let (enabled, rollout) = match existing {
        Some(fv) => {
            let new_enabled = req.enabled.unwrap_or(fv.enabled);
            let new_rollout = req.rollout_percentage.unwrap_or(fv.rollout_percentage);

            // Validate rollout percentage
            if !(0..=100).contains(&new_rollout) {
                return Err(AppError::BadRequest(
                    "Rollout percentage must be between 0 and 100".to_string(),
                ));
            }

            let updated_fv = FlagValue {
                id: fv.id,
                flag_id: flag.id,
                environment_id: environment.id,
                enabled: new_enabled,
                rollout_percentage: new_rollout,
                updated_at: now,
            };

            state.storage.update_flag_value(&updated_fv).await?;

            (new_enabled, new_rollout)
        }
        None => {
            let enabled = req.enabled.unwrap_or(false);
            let rollout = req.rollout_percentage.unwrap_or(100);

            if !(0..=100).contains(&rollout) {
                return Err(AppError::BadRequest(
                    "Rollout percentage must be between 0 and 100".to_string(),
                ));
            }

            let fv_id = Uuid::new_v4().to_string();
            let flag_value = FlagValue {
                id: fv_id,
                flag_id: flag.id,
                environment_id: environment.id,
                enabled,
                rollout_percentage: rollout,
                updated_at: now,
            };

            state.storage.create_flag_value(&flag_value).await?;

            (enabled, rollout)
        }
    };

    Ok(Json(FlagEnvironmentValue { enabled, rollout }))
}

/// Toggle a flag in a specific environment
// Kept for future use
#[allow(dead_code)]
pub async fn toggle_flag(
    State(state): State<AppState>,
    AuthProject(project): AuthProject,
    Path(key): Path<String>,
    Query(query): Query<ToggleFlagQuery>,
) -> Result<Json<FlagToggleResponse>> {
    // Get the flag
    let flag = state
        .storage
        .get_flag_by_key(&project.id, &key)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Flag '{key}' not found")))?;

    // Get the environment
    let env_name = &query.environment;
    let environment = state
        .storage
        .get_environment_by_name(&project.id, env_name)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Environment '{env_name}' not found")))?;

    let now = Utc::now();

    // Get current value and toggle
    let existing = state
        .storage
        .get_flag_value(&flag.id, &environment.id)
        .await?;

    let new_enabled = match existing {
        Some(fv) => {
            let toggled = !fv.enabled;
            let updated_fv = FlagValue {
                id: fv.id,
                flag_id: flag.id,
                environment_id: environment.id,
                enabled: toggled,
                rollout_percentage: fv.rollout_percentage,
                updated_at: now,
            };
            state.storage.update_flag_value(&updated_fv).await?;
            toggled
        }
        None => {
            // No value exists, create with enabled = true (toggle from default false)
            let fv_id = Uuid::new_v4().to_string();
            let flag_value = FlagValue {
                id: fv_id,
                flag_id: flag.id,
                environment_id: environment.id,
                enabled: true,
                rollout_percentage: 100,
                updated_at: now,
            };
            state.storage.create_flag_value(&flag_value).await?;
            true
        }
    };

    Ok(Json(FlagToggleResponse {
        key,
        environment: query.environment,
        enabled: new_enabled,
    }))
}
