# FlagLite API

Lightweight feature flag service built with Rust, Axum, and SQLx.

## Features

- 🚀 Fast and lightweight single binary
- 💾 Dual database support: SQLite (self-host) and PostgreSQL (hosted)
- 🔐 JWT authentication with 7-day expiry
- 🎯 Percentage rollout with sticky bucketing (murmur3 hash)
- 📊 Per-environment flag configuration (dev/staging/production)

## Quick Start

```bash
# Copy and configure environment
cp .env.example .env
# Edit .env and set JWT_SECRET

# Build and run
cargo build --release
./target/release/flaglite serve

# Or with cargo
cargo run -- serve
```

## API Endpoints

### Authentication

```bash
# Sign up (creates user + project + 3 environments)
POST /v1/auth/signup
{
  "email": "user@example.com",
  "password": "securepassword",
  "project_name": "My App"  # optional
}

# Login
POST /v1/auth/login
{
  "email": "user@example.com",
  "password": "securepassword"
}

# Get current user
GET /v1/auth/me
Authorization: Bearer <jwt_token>
```

### Flags

```bash
# Evaluate flag (SDK endpoint - use environment API key)
GET /v1/flags/:key?user_id=123
Authorization: Bearer ffl_env_xxxxx

# List all flags
GET /v1/flags
Authorization: Bearer ffl_proj_xxxxx  # or JWT

# Create flag
POST /v1/flags
Authorization: Bearer ffl_proj_xxxxx
{
  "key": "new-checkout",
  "name": "New Checkout Flow",
  "description": "Redesigned checkout"
}

# Update flag value
PATCH /v1/flags/:key/environments/:env
Authorization: Bearer ffl_proj_xxxxx
{
  "enabled": true,
  "rollout_percentage": 25
}

# Toggle flag
POST /v1/flags/:key/toggle?environment=production
Authorization: Bearer ffl_proj_xxxxx
```

## API Keys

- `ffl_proj_*` - Project API key: full CRUD access to flags
- `ffl_env_*` - Environment API key: read-only flag evaluation

## Database Selection

### SQLite (default, self-hosting)

```bash
cargo build --release --features sqlite
DATABASE_URL=sqlite:flaglite.db?mode=rwc
```

### PostgreSQL (hosted)

```bash
cargo build --release --features postgres --no-default-features
DATABASE_URL=postgres://user:pass@host:5432/flaglite
```

## Percentage Rollout

Uses murmur3 hashing for deterministic, sticky bucketing:

```
hash = murmur3("flag_key:user_id")
bucket = hash % 100
enabled = bucket < rollout_percentage
```

Same user always gets the same result for the same flag.
Increasing rollout % from 10→20 adds users, doesn't reshuffle.

## Development

```bash
# Run with debug logging
RUST_LOG=flaglite=debug cargo run -- serve

# Run migrations only
cargo run -- migrate
```

## License

MIT
