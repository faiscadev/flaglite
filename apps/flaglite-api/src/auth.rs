use crate::error::{AppError, Result};
use crate::models::{is_user_api_key, AppState, Claims, Environment, Project, User};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sha2::{Digest, Sha256};

const JWT_EXPIRY_DAYS: i64 = 7;

pub fn create_jwt(user: &User, secret: &str) -> Result<String> {
    let now = Utc::now().timestamp();
    let expiry = now + (JWT_EXPIRY_DAYS * 24 * 60 * 60);

    let claims = Claims {
        sub: user.id.clone(),
        username: user.username.clone(),
        iat: now,
        exp: expiry,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

/// Hash a password using Argon2id
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::Internal(format!("Password hash error: {e}")))
}

/// Verify a password against an Argon2 hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Invalid password hash: {e}")))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Hash an API key using SHA256 for storage
pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}

// ============ Extractors ============

/// Extracts the authenticated user from JWT
pub struct AuthUser(pub User);

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        // Check if it's a user API key (flg_ prefix)
        if is_user_api_key(token) {
            let key_hash = hash_api_key(token);
            let api_key = state
                .storage
                .get_api_key_by_hash(&key_hash)
                .await?
                .ok_or(AppError::InvalidApiKey)?;

            let user = state
                .storage
                .get_user_by_id(&api_key.user_id)
                .await?
                .ok_or(AppError::Unauthorized)?;

            return Ok(AuthUser(user));
        }

        // Otherwise treat as JWT
        let claims = verify_jwt(token, &state.jwt_secret)?;

        let user = state
            .storage
            .get_user_by_id(&claims.sub)
            .await?
            .ok_or(AppError::Unauthorized)?;

        Ok(AuthUser(user))
    }
}

/// Extracts project from project API key, user API key, or JWT
pub struct AuthProject(pub Project);

#[async_trait]
impl FromRequestParts<AppState> for AuthProject {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        // Check if it's a project API key
        if token.starts_with("ffl_proj_") {
            let project = state
                .storage
                .get_project_by_api_key(token)
                .await?
                .ok_or(AppError::InvalidApiKey)?;

            return Ok(AuthProject(project));
        }

        // Check if it's a user API key (flg_ prefix)
        if is_user_api_key(token) {
            let key_hash = hash_api_key(token);
            let api_key = state
                .storage
                .get_api_key_by_hash(&key_hash)
                .await?
                .ok_or(AppError::InvalidApiKey)?;

            let project = state
                .storage
                .get_first_project_by_user(&api_key.user_id)
                .await?
                .ok_or(AppError::NotFound("No project found".to_string()))?;

            return Ok(AuthProject(project));
        }

        // Otherwise treat as JWT and get user's first project
        let claims = verify_jwt(token, &state.jwt_secret)?;

        let project = state
            .storage
            .get_first_project_by_user(&claims.sub)
            .await?
            .ok_or(AppError::NotFound("No project found".to_string()))?;

        Ok(AuthProject(project))
    }
}

/// Extracts environment from environment API key
#[allow(dead_code)]
pub struct AuthEnvironment(pub Environment, pub Project);

#[async_trait]
impl FromRequestParts<AppState> for AuthEnvironment {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        // Must be an environment API key
        if !token.starts_with("ffl_env_") {
            return Err(AppError::InvalidApiKey);
        }

        let env = state
            .storage
            .get_environment_by_api_key(token)
            .await?
            .ok_or(AppError::InvalidApiKey)?;

        let project = state
            .storage
            .get_project_by_id(&env.project_id)
            .await?
            .ok_or(AppError::Internal(
                "Project not found for environment".to_string(),
            ))?;

        Ok(AuthEnvironment(env, project))
    }
}

/// Flexible auth - accepts project key, env key, user API key, or JWT
pub enum FlexAuth {
    Project(Project),
    Environment(Environment, Project),
}

#[async_trait]
impl FromRequestParts<AppState> for FlexAuth {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        if token.starts_with("ffl_proj_") {
            let project = state
                .storage
                .get_project_by_api_key(token)
                .await?
                .ok_or(AppError::InvalidApiKey)?;

            return Ok(FlexAuth::Project(project));
        }

        if token.starts_with("ffl_env_") {
            let env = state
                .storage
                .get_environment_by_api_key(token)
                .await?
                .ok_or(AppError::InvalidApiKey)?;

            let project = state
                .storage
                .get_project_by_id(&env.project_id)
                .await?
                .ok_or(AppError::Internal(
                    "Project not found for environment".to_string(),
                ))?;

            return Ok(FlexAuth::Environment(env, project));
        }

        // Check if it's a user API key (flg_ prefix)
        if is_user_api_key(token) {
            let key_hash = hash_api_key(token);
            let api_key = state
                .storage
                .get_api_key_by_hash(&key_hash)
                .await?
                .ok_or(AppError::InvalidApiKey)?;

            let project = state
                .storage
                .get_first_project_by_user(&api_key.user_id)
                .await?
                .ok_or(AppError::NotFound("No project found".to_string()))?;

            return Ok(FlexAuth::Project(project));
        }

        // JWT auth
        let claims = verify_jwt(token, &state.jwt_secret)?;

        let project = state
            .storage
            .get_first_project_by_user(&claims.sub)
            .await?
            .ok_or(AppError::NotFound("No project found".to_string()))?;

        Ok(FlexAuth::Project(project))
    }
}
