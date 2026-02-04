use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

use crate::error::Result;
use crate::models::{Environment, Flag, FlagValue, Project, User};
use super::Storage;

pub struct SqliteStorage {
    pool: SqlitePool,
}

impl SqliteStorage {
    pub async fn new(database_url: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl Storage for SqliteStorage {
    // ============ Users ============

    async fn create_user(&self, user: &User) -> Result<()> {
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, created_at) VALUES (?, ?, ?, ?)",
        )
        .bind(&user.id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(user.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as(
            "SELECT id, email, password_hash, created_at FROM users WHERE email = ?",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    async fn get_user_by_id(&self, id: &str) -> Result<Option<User>> {
        let user = sqlx::query_as(
            "SELECT id, email, password_hash, created_at FROM users WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    // ============ Projects ============

    async fn create_project(&self, project: &Project) -> Result<()> {
        sqlx::query(
            "INSERT INTO projects (id, user_id, name, api_key, created_at) VALUES (?, ?, ?, ?, ?)",
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
            "SELECT id, user_id, name, api_key, created_at FROM projects WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(project)
    }

    async fn get_project_by_api_key(&self, api_key: &str) -> Result<Option<Project>> {
        let project = sqlx::query_as(
            "SELECT id, user_id, name, api_key, created_at FROM projects WHERE api_key = ?",
        )
        .bind(api_key)
        .fetch_optional(&self.pool)
        .await?;
        Ok(project)
    }

    async fn list_projects_by_user(&self, user_id: &str) -> Result<Vec<Project>> {
        let projects = sqlx::query_as(
            "SELECT id, user_id, name, api_key, created_at FROM projects WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(projects)
    }

    async fn get_first_project_by_user(&self, user_id: &str) -> Result<Option<Project>> {
        let project = sqlx::query_as(
            "SELECT id, user_id, name, api_key, created_at FROM projects WHERE user_id = ? LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(project)
    }

    // ============ Environments ============

    async fn create_environment(&self, env: &Environment) -> Result<()> {
        sqlx::query(
            "INSERT INTO environments (id, project_id, name, api_key, created_at) VALUES (?, ?, ?, ?, ?)",
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
            "SELECT id, project_id, name, api_key, created_at FROM environments WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(env)
    }

    async fn get_environment_by_api_key(&self, api_key: &str) -> Result<Option<Environment>> {
        let env = sqlx::query_as(
            "SELECT id, project_id, name, api_key, created_at FROM environments WHERE api_key = ?",
        )
        .bind(api_key)
        .fetch_optional(&self.pool)
        .await?;
        Ok(env)
    }

    async fn get_environment_by_name(&self, project_id: &str, name: &str) -> Result<Option<Environment>> {
        let env = sqlx::query_as(
            "SELECT id, project_id, name, api_key, created_at FROM environments WHERE project_id = ? AND name = ?",
        )
        .bind(project_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(env)
    }

    async fn list_environments_by_project(&self, project_id: &str) -> Result<Vec<Environment>> {
        let envs = sqlx::query_as(
            "SELECT id, project_id, name, api_key, created_at FROM environments WHERE project_id = ?",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(envs)
    }

    // ============ Flags ============

    async fn create_flag(&self, flag: &Flag) -> Result<()> {
        sqlx::query(
            "INSERT INTO flags (id, project_id, key, name, description, created_at) VALUES (?, ?, ?, ?, ?, ?)",
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
            "SELECT id, project_id, key, name, description, created_at FROM flags WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(flag)
    }

    async fn get_flag_by_key(&self, project_id: &str, key: &str) -> Result<Option<Flag>> {
        let flag = sqlx::query_as(
            "SELECT id, project_id, key, name, description, created_at FROM flags WHERE project_id = ? AND key = ?",
        )
        .bind(project_id)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;
        Ok(flag)
    }

    async fn list_flags_by_project(&self, project_id: &str) -> Result<Vec<Flag>> {
        let flags = sqlx::query_as(
            "SELECT id, project_id, key, name, description, created_at FROM flags WHERE project_id = ? ORDER BY created_at DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(flags)
    }

    // ============ Flag Values ============

    async fn create_flag_value(&self, flag_value: &FlagValue) -> Result<()> {
        sqlx::query(
            "INSERT INTO flag_values (id, flag_id, environment_id, enabled, rollout_percentage, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
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
            "SELECT id, flag_id, environment_id, enabled, rollout_percentage, updated_at FROM flag_values WHERE flag_id = ? AND environment_id = ?",
        )
        .bind(flag_id)
        .bind(environment_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(fv)
    }

    async fn update_flag_value(&self, flag_value: &FlagValue) -> Result<()> {
        sqlx::query(
            "UPDATE flag_values SET enabled = ?, rollout_percentage = ?, updated_at = ? WHERE id = ?",
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

        let placeholders = flag_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query_str = format!(
            "SELECT id, flag_id, environment_id, enabled, rollout_percentage, updated_at FROM flag_values WHERE flag_id IN ({})",
            placeholders
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
        tracing::info!("Running database migrations (SQLite)...");

        // Create users table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
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
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
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
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
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
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
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
                enabled INTEGER NOT NULL DEFAULT 0,
                rollout_percentage INTEGER NOT NULL DEFAULT 100,
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(flag_id, environment_id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
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
