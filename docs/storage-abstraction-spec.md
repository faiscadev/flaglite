# FlagLite Storage Abstraction Spec

## Background

CEO directive: Build a storage abstraction trait to support both SQLite and PostgreSQL via sqlx at runtime (not compile-time feature flags).

## Current State

- Already using sqlx ✅
- Compile-time feature flags for sqlite/postgres in `api/Cargo.toml`
- Separate `run_migrations()` and `create_pool()` functions gated by `#[cfg(feature = ...)]`
- Raw SQL queries scattered throughout handlers (`api/src/handlers/*.rs`)
- SQL uses `?` placeholders (works for both SQLite and Postgres with sqlx)

## Target Architecture

### 1. Storage Trait

Create `api/src/storage/mod.rs` with a `Storage` trait:

```rust
#[async_trait]
pub trait Storage: Send + Sync {
    // Users
    async fn create_user(&self, user: &User) -> Result<()>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn get_user_by_id(&self, id: &str) -> Result<Option<User>>;

    // Projects
    async fn create_project(&self, project: &Project) -> Result<()>;
    async fn get_project_by_id(&self, id: &str) -> Result<Option<Project>>;
    async fn get_project_by_api_key(&self, api_key: &str) -> Result<Option<Project>>;
    async fn list_projects_by_user(&self, user_id: &str) -> Result<Vec<Project>>;

    // Environments
    async fn create_environment(&self, env: &Environment) -> Result<()>;
    async fn get_environment_by_id(&self, id: &str) -> Result<Option<Environment>>;
    async fn get_environment_by_api_key(&self, api_key: &str) -> Result<Option<Environment>>;
    async fn get_environment_by_name(&self, project_id: &str, name: &str) -> Result<Option<Environment>>;
    async fn list_environments_by_project(&self, project_id: &str) -> Result<Vec<Environment>>;

    // Flags
    async fn create_flag(&self, flag: &Flag) -> Result<()>;
    async fn get_flag_by_id(&self, id: &str) -> Result<Option<Flag>>;
    async fn get_flag_by_key(&self, project_id: &str, key: &str) -> Result<Option<Flag>>;
    async fn list_flags_by_project(&self, project_id: &str) -> Result<Vec<Flag>>;

    // Flag Values
    async fn create_flag_value(&self, flag_value: &FlagValue) -> Result<()>;
    async fn get_flag_value(&self, flag_id: &str, environment_id: &str) -> Result<Option<FlagValue>>;
    async fn update_flag_value(&self, flag_value: &FlagValue) -> Result<()>;
    async fn list_flag_values_by_flag_ids(&self, flag_ids: &[String]) -> Result<Vec<FlagValue>>;

    // Migrations
    async fn run_migrations(&self) -> Result<()>;
}
```

### 2. Implementations

Create two implementations:

- `api/src/storage/sqlite.rs` - `SqliteStorage`
- `api/src/storage/postgres.rs` - `PostgresStorage`

Both wrap their respective sqlx pools. Key differences:
- SQLite: `datetime('now')` for timestamps, `INTEGER` for booleans
- PostgreSQL: `NOW()` for timestamps, `BOOLEAN` type, `TIMESTAMP WITH TIME ZONE`

### 3. AppState Changes

```rust
pub struct AppState {
    pub storage: Arc<dyn Storage>,
    pub jwt_secret: String,
}
```

### 4. Handler Refactoring

Replace all direct `sqlx::query` calls with `state.storage.method()` calls.

Example before:
```rust
let flag: Flag = sqlx::query_as("SELECT ... FROM flags WHERE ...")
    .bind(&project_id)
    .fetch_optional(&state.pool)
    .await?;
```

Example after:
```rust
let flag = state.storage.get_flag_by_key(&project_id, &key).await?;
```

### 5. Runtime Selection

In `main.rs`, select storage based on DATABASE_URL:

```rust
let storage: Arc<dyn Storage> = if database_url.starts_with("postgres") {
    Arc::new(PostgresStorage::new(&database_url).await?)
} else {
    Arc::new(SqliteStorage::new(&database_url).await?)
};
```

### 6. Cargo.toml Changes

Enable both sqlite and postgres features by default:

```toml
[features]
default = ["sqlite", "postgres"]
sqlite = ["sqlx/sqlite"]
postgres = ["sqlx/postgres"]
```

Or remove feature flags entirely and always compile both.

## Files to Create/Modify

### Create:
- `api/src/storage/mod.rs` - Storage trait + factory function
- `api/src/storage/sqlite.rs` - SQLite implementation
- `api/src/storage/postgres.rs` - PostgreSQL implementation

### Modify:
- `api/Cargo.toml` - Enable both features by default, add async-trait
- `api/src/main.rs` - Use storage factory, remove feature-gated code
- `api/src/models.rs` - Remove DbPool type alias, use Storage
- `api/src/db.rs` - Can be deleted (absorbed into storage impls)
- `api/src/handlers/auth.rs` - Use storage trait methods
- `api/src/handlers/flags.rs` - Use storage trait methods

## Testing

- Unit tests for each storage implementation
- Integration tests that run against both SQLite and PostgreSQL
- Existing tests should continue to work with SQLite default

## Migration Path

1. Implement storage trait and both backends
2. Keep compile-time feature flags initially for backwards compatibility
3. Default to SQLite if DATABASE_URL not set or starts with `sqlite`
4. PostgreSQL if DATABASE_URL starts with `postgres`

## Dependencies to Add

```toml
async-trait = "0.1"
```
