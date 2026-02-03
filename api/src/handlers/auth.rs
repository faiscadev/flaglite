use axum::{extract::State, Json};
use chrono::Utc;
use uuid::Uuid;

use crate::auth::{create_jwt, hash_password, verify_password, AuthUser};
use crate::error::{AppError, Result};
use crate::models::{
    generate_env_api_key, generate_project_api_key, AppState, AuthResponse, Environment,
    LoginRequest, Project, SignupRequest, User, UserResponse,
};

const DEFAULT_ENVIRONMENTS: [&str; 3] = ["development", "staging", "production"];

pub async fn signup(
    State(state): State<AppState>,
    Json(req): Json<SignupRequest>,
) -> Result<Json<AuthResponse>> {
    // Validate input
    if req.email.is_empty() || !req.email.contains('@') {
        return Err(AppError::BadRequest("Invalid email".to_string()));
    }
    if req.password.len() < 8 {
        return Err(AppError::BadRequest(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    // Check if user exists
    let existing: Option<User> = sqlx::query_as(
        "SELECT id, email, password_hash, created_at FROM users WHERE email = ?",
    )
    .bind(&req.email)
    .fetch_optional(&state.pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::UserAlreadyExists);
    }

    // Create user
    let user_id = Uuid::new_v4().to_string();
    let password_hash = hash_password(&req.password)?;
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(&user_id)
    .bind(&req.email)
    .bind(&password_hash)
    .bind(now)
    .execute(&state.pool)
    .await?;

    let user = User {
        id: user_id.clone(),
        email: req.email.clone(),
        password_hash,
        created_at: now,
    };

    // Create first project
    let project_name = req.project_name.unwrap_or_else(|| "My Project".to_string());
    let project_id = Uuid::new_v4().to_string();
    let project_api_key = generate_project_api_key();

    sqlx::query(
        "INSERT INTO projects (id, user_id, name, api_key, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&project_id)
    .bind(&user_id)
    .bind(&project_name)
    .bind(&project_api_key)
    .bind(now)
    .execute(&state.pool)
    .await?;

    let project = Project {
        id: project_id.clone(),
        user_id: user_id.clone(),
        name: project_name,
        api_key: project_api_key,
        created_at: now,
    };

    // Create 3 default environments
    let mut environments = Vec::new();
    for env_name in DEFAULT_ENVIRONMENTS {
        let env_id = Uuid::new_v4().to_string();
        let env_api_key = generate_env_api_key();

        sqlx::query(
            "INSERT INTO environments (id, project_id, name, api_key, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&env_id)
        .bind(&project_id)
        .bind(env_name)
        .bind(&env_api_key)
        .bind(now)
        .execute(&state.pool)
        .await?;

        environments.push(Environment {
            id: env_id,
            project_id: project_id.clone(),
            name: env_name.to_string(),
            api_key: env_api_key,
            created_at: now,
        });
    }

    // Create JWT
    let token = create_jwt(&user, &state.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
        project: Some(project.into()),
        environments: Some(environments.into_iter().map(|e| e.into()).collect()),
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    // Find user
    let user: User = sqlx::query_as(
        "SELECT id, email, password_hash, created_at FROM users WHERE email = ?",
    )
    .bind(&req.email)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::InvalidCredentials)?;

    // Verify password
    if !verify_password(&req.password, &user.password_hash)? {
        return Err(AppError::InvalidCredentials);
    }

    // Create JWT
    let token = create_jwt(&user, &state.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
        project: None,
        environments: None,
    }))
}

pub async fn me(AuthUser(user): AuthUser) -> Result<Json<UserResponse>> {
    Ok(Json(user.into()))
}
