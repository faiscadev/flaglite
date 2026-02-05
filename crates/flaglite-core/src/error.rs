//! Error types for FlagLite

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlagLiteError {
    #[error("Authentication required. Run 'flaglite login' first.")]
    NotAuthenticated,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Flag not found: {0}")]
    FlagNotFound(String),

    #[error("Environment not found: {0}")]
    EnvironmentNotFound(String),

    #[error("No project selected. Run 'flaglite projects use <id>' first.")]
    NoProjectSelected,

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Rate limited. Please try again in {retry_after} seconds.")]
    RateLimited { retry_after: u64 },
}
