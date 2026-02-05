use axum::{extract::State, Json};
use chrono::Utc;
use uuid::Uuid;

use crate::auth::{create_jwt, hash_api_key, hash_password, verify_password, AuthUser};
use crate::error::{AppError, Result};
use crate::models::{
    generate_env_api_key, generate_project_api_key, generate_user_api_key, ApiKey,
    ApiKeyCreatedResponse, AppState, AuthResponse, Environment, LoginRequest, Project,
    SignupRequest, SignupResponse, UpdateUserRequest, User, UserResponse,
};
use crate::username::{generate_username, generate_username_with_suffix};

const DEFAULT_ENVIRONMENTS: [&str; 3] = ["development", "staging", "production"];
const MAX_USERNAME_RETRIES: u32 = 10;

/// POST /v1/auth/signup
/// Creates a new user account with optional username (auto-generated if not provided)
/// Returns user info, API key (shown once), and JWT token
pub async fn signup(
    State(state): State<AppState>,
    Json(req): Json<SignupRequest>,
) -> Result<Json<SignupResponse>> {
    // Validate password
    if req.password.len() < 8 {
        return Err(AppError::BadRequest(
            "Password must be at least 8 characters".to_string(),
        ));
    }

    // Generate or validate username
    let username = if let Some(provided_username) = req.username {
        // Validate provided username
        let username = provided_username.trim().to_lowercase();
        if username.len() < 3 {
            return Err(AppError::BadRequest(
                "Username must be at least 3 characters".to_string(),
            ));
        }
        if username.len() > 32 {
            return Err(AppError::BadRequest(
                "Username must be at most 32 characters".to_string(),
            ));
        }
        if !username.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(AppError::BadRequest(
                "Username can only contain letters, numbers, hyphens, and underscores".to_string(),
            ));
        }
        
        // Check if username exists
        if state.storage.username_exists(&username).await? {
            return Err(AppError::UserAlreadyExists);
        }
        
        username
    } else {
        // Auto-generate username with collision handling
        let mut username = generate_username();
        let mut retries = 0;
        
        while state.storage.username_exists(&username).await? {
            if retries >= MAX_USERNAME_RETRIES {
                // Fall back to username with suffix
                username = generate_username_with_suffix();
            } else {
                username = generate_username();
            }
            retries += 1;
            
            if retries > MAX_USERNAME_RETRIES * 2 {
                return Err(AppError::Internal("Failed to generate unique username".to_string()));
            }
        }
        
        username
    };

    // Create user
    let user_id = Uuid::new_v4().to_string();
    let password_hash = hash_password(&req.password)?;
    let now = Utc::now();

    let user = User {
        id: user_id.clone(),
        username: username.clone(),
        password_hash,
        email: None,
        created_at: now,
        updated_at: now,
    };

    state.storage.create_user(&user).await?;

    // Generate API key for the user
    let api_key_raw = generate_user_api_key();
    let api_key_hash = hash_api_key(&api_key_raw);
    let api_key_prefix = api_key_raw.chars().take(12).collect::<String>(); // flg_a1b2c3d4 (12 chars)
    let api_key_id = Uuid::new_v4().to_string();

    let api_key = ApiKey {
        id: api_key_id.clone(),
        user_id: user_id.clone(),
        key_hash: api_key_hash,
        key_prefix: api_key_prefix.clone(),
        name: Some("Default API Key".to_string()),
        created_at: now,
        revoked_at: None,
    };

    state.storage.create_api_key(&api_key).await?;

    // Create first project
    let project_name = req.project_name.unwrap_or_else(|| "My Project".to_string());
    let project_id = Uuid::new_v4().to_string();
    let project_api_key = generate_project_api_key();

    let project = Project {
        id: project_id.clone(),
        user_id: user_id.clone(),
        name: project_name,
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

    // Create JWT
    let token = create_jwt(&user, &state.jwt_secret)?;

    Ok(Json(SignupResponse {
        user: user.into(),
        api_key: ApiKeyCreatedResponse {
            id: api_key_id,
            key: api_key_raw, // Full key - only shown once!
            key_prefix: api_key_prefix,
            name: Some("Default API Key".to_string()),
            created_at: now,
        },
        token,
        project: Some(project.into()),
        environments: Some(environments.into_iter().map(|e| e.into()).collect()),
    }))
}

/// POST /v1/auth/login
/// Authenticates a user with username and password
/// Returns user info and JWT token
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>> {
    // Find user by username
    let user = state
        .storage
        .get_user_by_username(&req.username.to_lowercase())
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

/// GET /v1/auth/me
/// Returns the authenticated user's info
/// Requires JWT or API key
pub async fn me(AuthUser(user): AuthUser) -> Result<Json<UserResponse>> {
    Ok(Json(user.into()))
}

/// PATCH /v1/auth/me
/// Updates the authenticated user's info (email)
/// Requires JWT or API key
pub async fn update_me(
    State(state): State<AppState>,
    AuthUser(mut user): AuthUser,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>> {
    // Update email if provided
    if let Some(email) = req.email {
        let email = email.trim().to_lowercase();
        if !email.is_empty() && !email.contains('@') {
            return Err(AppError::BadRequest("Invalid email format".to_string()));
        }
        user.email = if email.is_empty() { None } else { Some(email) };
    }

    user.updated_at = Utc::now();
    state.storage.update_user(&user).await?;

    Ok(Json(user.into()))
}
