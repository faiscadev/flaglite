# API Overview

FlagLite provides a REST API for managing and evaluating feature flags.

## Base URL

| Environment | URL |
|-------------|-----|
| Production | `https://api.flaglite.dev/v1` |
| Staging | `https://api.staging.flaglite.dev/v1` |
| Self-hosted | `http://your-host:3000/v1` |

## Authentication

FlagLite uses Bearer token authentication. Include your token in the `Authorization` header:

```
Authorization: Bearer <token>
```

### Token Types

| Token Type | Format | Use Case |
|------------|--------|----------|
| **Environment API Key** | `ffl_env_xxxxx` | SDK flag evaluation (read-only) |
| **Project API Key** | `ffl_proj_xxxxx` | Dashboard/management operations |
| **JWT Token** | `eyJhbG...` | User authentication (from login/signup) |

**Environment keys** are scoped to a single environment (development, staging, production) and can only evaluate flags.

**Project keys** have full access to create, update, and delete flags across all environments.

## Quick Examples

### Evaluate a Flag (SDK)

```bash
curl -H "Authorization: Bearer ffl_env_xxxxx" \
  https://api.flaglite.dev/v1/flags/new-checkout
```

Response:
```json
{
  "key": "new-checkout",
  "enabled": true
}
```

### Evaluate with User ID (Percentage Rollouts)

```bash
curl -H "Authorization: Bearer ffl_env_xxxxx" \
  "https://api.flaglite.dev/v1/flags/new-checkout?user_id=user-123"
```

The same `user_id` always returns the same result for the same flag (sticky bucketing).

### List All Flags

```bash
curl -H "Authorization: Bearer ffl_proj_xxxxx" \
  https://api.flaglite.dev/v1/flags
```

### Create a Flag

```bash
curl -X POST \
  -H "Authorization: Bearer ffl_proj_xxxxx" \
  -H "Content-Type: application/json" \
  -d '{"key": "dark-mode", "name": "Dark Mode"}' \
  https://api.flaglite.dev/v1/flags
```

### Toggle a Flag

```bash
curl -X POST \
  -H "Authorization: Bearer ffl_proj_xxxxx" \
  "https://api.flaglite.dev/v1/flags/dark-mode/toggle?environment=production"
```

## Rate Limiting

- **Limit:** 1000 requests/minute per API key
- **Headers:** `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`

When rate limited, you'll receive a `429` response with a `Retry-After` header.

## Error Handling

Errors return JSON with `error` and `message` fields:

```json
{
  "error": "not_found",
  "message": "Flag 'my-flag' not found"
}
```

| Status | Error | Description |
|--------|-------|-------------|
| 400 | `bad_request` | Malformed request |
| 401 | `unauthorized` | Invalid or missing API key |
| 404 | `not_found` | Resource not found |
| 409 | `conflict` | Resource already exists |
| 422 | `validation_error` | Validation failed (see `details`) |
| 429 | `rate_limited` | Too many requests |

## OpenAPI Specification

For the complete API specification, see:

- **OpenAPI YAML:** [`../openapi/openapi.yaml`](../../openapi/openapi.yaml)
- **Swagger UI:** [`../openapi/swagger.html`](../../openapi/swagger.html)

The OpenAPI spec includes all endpoints, request/response schemas, and examples.

---

## Next Steps

- [JavaScript SDK](../sdks/javascript.md)
- [Python SDK](../sdks/python.md)
- [Go SDK](../sdks/go.md)
- [Rust SDK](../sdks/rust.md)
