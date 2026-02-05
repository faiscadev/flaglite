use async_trait::async_trait;
use chrono::Utc;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::error::Result;
use crate::models::{ApiKey, Environment, Flag, FlagValue, Project, User};
use super::Storage;

pub struct PostgresStorage {
    pool: PgPool,
}

impl PostgresStorage {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl Storage for PostgresStorage {
    // ============ Users ============

    async fn create_user(&self, user: &User) -> Result<()> {
        sqlx::query(
            "INSERT INTO users (id, username, password_hash, email, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&user.id)
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(&user.email)
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let user = sqlx::query_as(
            "SELECT id, username, password_hash, email, created_at, updated_at FROM users WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    async fn get_user_by_id(&self, id: &str) -> Result<Option<User>> {
        let user = sqlx::query_as(
            "SELECT id, username, password_hash, email, created_at, updated_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    async fn update_user(&self, user: &User) -> Result<()> {
        sqlx::query(
            "UPDATE users SET email = $1, updated_at = $2 WHERE id = $3",
        )
        .bind(&user.email)
        .bind(user.updated_at)
        .bind(&user.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn username_exists(&self, username: &str) -> Result<bool> {
        let result: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users WHERE username = $1",
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await?;
        Ok(result.0 > 0)
    }

    // ============ API Keys ============

    async fn create_api_key(&self, api_key: &ApiKey) -> Result<()> {
        sqlx::query(
            "INSERT INTO api_keys (id, user_id, key_hash, key_prefix, name, created_at, revoked_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(&api_key.id)
        .bind(&api_key.user_id)
        .bind(&api_key.key_hash)
        .bind(&api_key.key_prefix)
        .bind(&api_key.name)
        .bind(api_key.created_at)
        .bind(api_key.revoked_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        let api_key = sqlx::query_as(
            "SELECT id, user_id, key_hash, key_prefix, name, created_at, revoked_at FROM api_keys WHERE key_hash = $1 AND revoked_at IS NULL",
        )
        .bind(key_hash)
        .fetch_optional(&self.pool)
        .await?;
        Ok(api_key)
    }

    async fn list_api_keys_by_user(&self, user_id: &str) -> Result<Vec<ApiKey>> {
        let keys = sqlx::query_as(
            "SELECT id, user_id, key_hash, key_prefix, name, created_at, revoked_at FROM api_keys WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(keys)
    }

    async fn revoke_api_key(&self, id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE api_keys SET revoked_at = $1 WHERE id = $2",
        )
        .bind(Utc::now())
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ============ Projects ============

    async fn create_project(&self, project: &Project) -> Result<()> {
        sqlx::query(
            "INSERT INTO projects (id, user_id, name, api_key, created_at) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&project.id)
        .bind(&project.user_id)
        .bind(&project.name)
        .bind(&project.api_key)
        .bind(project.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_project_by_id(&self, id: &str) -> Result<Option<Project>> {
        let project = sqlx::query_as(
            "SELECT id, user_id, name, api_key, created_at FROM projects WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(project)
    }

    async fn get_project_by_api_key(&self, api_key: &str) -> Result<Option<Project>> {
        let project = sqlx::query_as(
            "SELECT id, user_id, name, api_key, created_at FROM projects WHERE api_key = $1",
        )
        .bind(api_key)
        .fetch_optional(&self.pool)
        .await?;
        Ok(project)
    }

    async fn list_projects_by_user(&self, user_id: &str) -> Result<Vec<Project>> {
        let projects = sqlx::query_as(
            "SELECT id, user_id, name, api_key, created_at FROM projects WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(projects)
    }

    async fn get_first_project_by_user(&self, user_id: &str) -> Result<Option<Project>> {
        let project = sqlx::query_as(
            "SELECT id, user_id, name, api_key, created_at FROM projects WHERE user_id = $1 LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(project)
    }

    // ============ Environments ============

    async fn create_environment(&self, env: &Environment) -> Result<()> {
        sqlx::query(
            "INSERT INTO environments (id, project_id, name, api_key, created_at) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(&env.id)
        .bind(&env.project_id)
        .bind(&env.name)
        .bind(&env.api_key)
        .bind(env.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_environment_by_id(&self, id: &str) -> Result<Option<Environment>> {
        let env = sqlx::query_as(
            "SELECT id, project_id, name, api_key, created_at FROM environments WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(env)
    }

    async fn get_environment_by_api_key(&self, api_key: &str) -> Result<Option<Environment>> {
        let env = sqlx::query_as(
            "SELECT id, project_id, name, api_key, created_at FROM environments WHERE api_key = $1",
        )
        .bind(api_key)
        .fetch_optional(&self.pool)
        .await?;
        Ok(env)
    }

    async fn get_environment_by_name(&self, project_id: &str, name: &str) -> Result<Option<Environment>> {
        let env = sqlx::query_as(
            "SELECT id, project_id, name, api_key, created_at FROM environments WHERE project_id = $1 AND name = $2",
        )
        .bind(project_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(env)
    }

    async fn list_environments_by_project(&self, project_id: &str) -> Result<Vec<Environment>> {
        let envs = sqlx::query_as(
            "SELECT id, project_id, name, api_key, created_at FROM environments WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(envs)
    }

    // ============ Flags ============

    async fn create_flag(&self, flag: &Flag) -> Result<()> {
        sqlx::query(
            "INSERT INTO flags (id, project_id, key, name, description, created_at) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&flag.id)
        .bind(&flag.project_id)
        .bind(&flag.key)
        .bind(&flag.name)
        .bind(&flag.description)
        .bind(flag.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_flag_by_id(&self, id: &str) -> Result<Option<Flag>> {
        let flag = sqlx::query_as(
            "SELECT id, project_id, key, name, description, created_at FROM flags WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(flag)
    }

    async fn get_flag_by_key(&self, project_id: &str, key: &str) -> Result<Option<Flag>> {
        let flag = sqlx::query_as(
            "SELECT id, project_id, key, name, description, created_at FROM flags WHERE project_id = $1 AND key = $2",
        )
        .bind(project_id)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;
        Ok(flag)
    }

    async fn list_flags_by_project(&self, project_id: &str) -> Result<Vec<Flag>> {
        let flags = sqlx::query_as(
            "SELECT id, project_id, key, name, description, created_at FROM flags WHERE project_id = $1 ORDER BY created_at DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(flags)
    }

    // ============ Flag Values ============

    async fn create_flag_value(&self, flag_value: &FlagValue) -> Result<()> {
        sqlx::query(
            "INSERT INTO flag_values (id, flag_id, environment_id, enabled, rollout_percentage, updated_at) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(&flag_value.id)
        .bind(&flag_value.flag_id)
        .bind(&flag_value.environment_id)
        .bind(flag_value.enabled)
        .bind(flag_value.rollout_percentage)
        .bind(flag_value.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_flag_value(&self, flag_id: &str, environment_id: &str) -> Result<Option<FlagValue>> {
        let fv = sqlx::query_as(
            "SELECT id, flag_id, environment_id, enabled, rollout_percentage, updated_at FROM flag_values WHERE flag_id = $1 AND environment_id = $2",
        )
        .bind(flag_id)
        .bind(environment_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(fv)
    }

    async fn update_flag_value(&self, flag_value: &FlagValue) -> Result<()> {
        sqlx::query(
            "UPDATE flag_values SET enabled = $1, rollout_percentage = $2, updated_at = $3 WHERE id = $4",
        )
        .bind(flag_value.enabled)
        .bind(flag_value.rollout_percentage)
        .bind(flag_value.updated_at)
        .bind(&flag_value.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_flag_values_by_flag_ids(&self, flag_ids: &[String]) -> Result<Vec<FlagValue>> {
        if flag_ids.is_empty() {
            return Ok(vec![]);
        }

        // Build parameterized query for PostgreSQL
        let placeholders: Vec<String> = flag_ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("${}", i + 1))
            .collect();
        let query_str = format!(
            "SELECT id, flag_id, environment_id, enabled, rollout_percentage, updated_at FROM flag_values WHERE flag_id IN ({})",
            placeholders.join(",")
        );

        let mut query = sqlx::query_as(&query_str);
        for id in flag_ids {
            query = query.bind(id);
        }

        let flag_values = query.fetch_all(&self.pool).await?;
        Ok(flag_values)
    }

    // ============ Migrations ============

    async fn run_migrations(&self) -> Result<()> {
        tracing::info!("Running database migrations (PostgreSQL)...");

        // Create users table with username-based auth
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                email TEXT,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create api_keys table for user API keys
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS api_keys (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                key_hash TEXT NOT NULL,
                key_prefix TEXT NOT NULL,
                name TEXT,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                revoked_at TIMESTAMP WITH TIME ZONE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create projects table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                api_key TEXT UNIQUE NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create environments table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS environments (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                api_key TEXT UNIQUE NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                UNIQUE(project_id, name)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create flags table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS flags (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                key TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                UNIQUE(project_id, key)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create flag_values table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS flag_values (
                id TEXT PRIMARY KEY,
                flag_id TEXT NOT NULL REFERENCES flags(id) ON DELETE CASCADE,
                environment_id TEXT NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
                enabled BOOLEAN NOT NULL DEFAULT FALSE,
                rollout_percentage INTEGER NOT NULL DEFAULT 100,
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                UNIQUE(flag_id, environment_id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_api_keys_user ON api_keys(user_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON api_keys(key_hash)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_projects_user ON projects(user_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_projects_api_key ON projects(api_key)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_environments_project ON environments(project_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_environments_api_key ON environments(api_key)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_flags_project ON flags(project_id)")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_flag_values_flag ON flag_values(flag_id)")
            .execute(&self.pool)
            .await?;

        tracing::info!("Migrations completed");
        Ok(())
    }
}
