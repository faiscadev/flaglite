// Storage abstraction module - v2
use crate::error::Result;
use crate::models::{ApiKey, Environment, Flag, FlagValue, Project, User};
use async_trait::async_trait;

pub mod postgres;
pub mod sqlite;

pub use postgres::PostgresStorage;
pub use sqlite::SqliteStorage;

/// Storage trait for FlagLite - abstracts database operations
#[allow(dead_code)]
#[async_trait]
pub trait Storage: Send + Sync {
    // Users
    async fn create_user(&self, user: &User) -> Result<()>;
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn get_user_by_id(&self, id: &str) -> Result<Option<User>>;
    async fn update_user(&self, user: &User) -> Result<()>;
    async fn username_exists(&self, username: &str) -> Result<bool>;

    // API Keys
    async fn create_api_key(&self, api_key: &ApiKey) -> Result<()>;
    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>>;
    async fn list_api_keys_by_user(&self, user_id: &str) -> Result<Vec<ApiKey>>;
    async fn revoke_api_key(&self, id: &str) -> Result<()>;

    // Projects
    async fn create_project(&self, project: &Project) -> Result<()>;
    async fn get_project_by_id(&self, id: &str) -> Result<Option<Project>>;
    async fn get_project_by_api_key(&self, api_key: &str) -> Result<Option<Project>>;
    async fn list_projects_by_user(&self, user_id: &str) -> Result<Vec<Project>>;
    async fn get_first_project_by_user(&self, user_id: &str) -> Result<Option<Project>>;

    // Environments
    async fn create_environment(&self, env: &Environment) -> Result<()>;
    async fn get_environment_by_id(&self, id: &str) -> Result<Option<Environment>>;
    async fn get_environment_by_api_key(&self, api_key: &str) -> Result<Option<Environment>>;
    async fn get_environment_by_name(
        &self,
        project_id: &str,
        name: &str,
    ) -> Result<Option<Environment>>;
    async fn list_environments_by_project(&self, project_id: &str) -> Result<Vec<Environment>>;

    // Flags
    async fn create_flag(&self, flag: &Flag) -> Result<()>;
    async fn get_flag_by_id(&self, id: &str) -> Result<Option<Flag>>;
    async fn get_flag_by_key(&self, project_id: &str, key: &str) -> Result<Option<Flag>>;
    async fn list_flags_by_project(&self, project_id: &str) -> Result<Vec<Flag>>;

    // Flag Values
    async fn create_flag_value(&self, flag_value: &FlagValue) -> Result<()>;
    async fn get_flag_value(
        &self,
        flag_id: &str,
        environment_id: &str,
    ) -> Result<Option<FlagValue>>;
    async fn update_flag_value(&self, flag_value: &FlagValue) -> Result<()>;
    async fn list_flag_values_by_flag_ids(&self, flag_ids: &[String]) -> Result<Vec<FlagValue>>;
    async fn delete_flag(&self, flag_id: &str) -> Result<()>;

    // Migrations
    async fn run_migrations(&self) -> Result<()>;
}

/// Create storage based on DATABASE_URL
pub async fn create_storage(database_url: &str) -> Result<std::sync::Arc<dyn Storage>> {
    if database_url.starts_with("postgres") {
        tracing::info!("Using PostgreSQL storage");
        let storage = PostgresStorage::new(database_url).await?;
        Ok(std::sync::Arc::new(storage))
    } else {
        tracing::info!("Using SQLite storage");
        let storage = SqliteStorage::new(database_url).await?;
        Ok(std::sync::Arc::new(storage))
    }
}
