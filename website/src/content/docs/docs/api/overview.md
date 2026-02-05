---
title: API Overview
description: FlagLite REST API reference
---

The FlagLite API is REST-based. All endpoints return JSON.

## Base URL

**Cloud:** `https://api.flaglite.dev/v1`  
**Self-hosted:** `http://your-server:8080/v1`

## Authentication

All API requests require an API key in the `Authorization` header:

```bash
curl https://api.flaglite.dev/v1/flags \
  -H "Authorization: Bearer your-api-key"
```

See [Authentication](/docs/api/authentication) for details.

## Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/auth/signup` | Create account |
| `POST` | `/auth/login` | Get JWT token |
| `GET` | `/auth/me` | Get current user |
| `GET` | `/flags` | List all flags |
| `POST` | `/flags` | Create flag |
| `GET` | `/flags/{id}` | Get flag |
| `PATCH` | `/flags/{id}` | Update flag |
| `DELETE` | `/flags/{id}` | Delete flag |
| `POST` | `/flags/{id}/toggle` | Toggle flag |
| `GET` | `/flags/{id}/evaluate` | Evaluate flag |

## Response Format

All responses follow this structure:

```json
{
  "data": { ... },
  "meta": {
    "request_id": "req_abc123"
  }
}
```

Errors:

```json
{
  "error": {
    "code": "not_found",
    "message": "Flag not found"
  }
}
```

## Rate Limits

| Plan | Requests/min |
|------|--------------|
| Free (self-hosted) | Unlimited |
| Hosted | 10,000 |

Rate limit headers are included in every response:

```
X-RateLimit-Limit: 10000
X-RateLimit-Remaining: 9999
X-RateLimit-Reset: 1699999999
```
