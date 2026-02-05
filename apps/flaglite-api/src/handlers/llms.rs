use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

const LLMS_TXT: &str = r#"# FlagLite

> Feature flags for teams who ship fast — without the enterprise tax.

## Overview

FlagLite is an open-source feature flag service. Self-host or use our hosted version.

- Boolean flags (on/off)
- Percentage rollouts (0-100%)
- Multi-environment (dev/staging/prod)
- API-first with full CLI parity

## Quick Start

1. Run FlagLite:
   ```bash
   git clone https://github.com/faiscadev/flaglite
   cd flaglite && docker compose up -d
   ```

2. Create a flag:
   ```bash
   curl -X POST http://localhost:8080/v1/projects/$PROJECT_ID/flags \
     -H "Authorization: Bearer $JWT_TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"key": "new-feature", "name": "New Feature", "enabled": false}'
   ```

3. Evaluate in your app (see SDK examples below)

## API Endpoints

Base URL: `https://api.flaglite.dev/v1` (or your self-hosted instance)

### Authentication
- `POST /v1/auth/signup` — Create account, returns JWT + API key
- `POST /v1/auth/login` — Get JWT token
- `GET /v1/auth/me` — Get current user

### Projects
- `GET /v1/projects` — List all projects
- `POST /v1/projects` — Create project `{"name": "string"}`

### Environments
- `GET /v1/projects/{project_id}/environments` — List environments (dev/staging/prod)

### Flags
- `GET /v1/projects/{project_id}/flags?environment={env}` — List all flags
- `POST /v1/projects/{project_id}/flags` — Create flag `{"key": "string", "name": "string", "enabled": bool}`
- `GET /v1/projects/{project_id}/flags/{key}?environment={env}` — Get flag with state
- `DELETE /v1/projects/{project_id}/flags/{key}` — Delete flag
- `POST /v1/projects/{project_id}/flags/{key}/toggle?environment={env}` — Toggle flag on/off

## SDKs

### JavaScript/TypeScript
```bash
npm install @faiscadev/flaglite
```
```javascript
import { FlagLite } from '@faiscadev/flaglite';

const client = new FlagLite({ apiKey: 'your-api-key' });
const enabled = await client.evaluate('new-feature', { userId: 'user-123' });
```

### Python
```bash
pip install flaglite
```
```python
from flaglite import FlagLite

client = FlagLite(api_key="your-api-key")
enabled = client.evaluate("new-feature", user_id="user-123")
```

### Go
```bash
go get github.com/faiscadev/flaglite-go
```
```go
client := flaglite.New("your-api-key")
enabled, _ := client.Evaluate("new-feature", "user-123")
```

### Rust
```toml
[dependencies]
flaglite = "0.1"
```
```rust
let client = FlagLite::new("your-api-key");
let enabled = client.evaluate("new-feature", "user-123").await?;
```

## Common Patterns

### Feature rollout
```javascript
// Roll out to 10% of users
if (await client.evaluate('new-checkout', { userId })) {
  showNewCheckout();
} else {
  showOldCheckout();
}
```

### Kill switch
```javascript
// Instantly disable a broken feature
if (await client.evaluate('payments-enabled', { userId })) {
  processPayment();
} else {
  showMaintenanceMessage();
}
```

## Links

- GitHub: https://github.com/faiscadev/flaglite
- Docs: https://flaglite.dev/docs
- API: https://api.flaglite.dev
"#;

/// Handler for /llms.txt - provides LLM-friendly documentation
pub async fn llms_txt() -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        LLMS_TXT,
    )
        .into_response()
}
