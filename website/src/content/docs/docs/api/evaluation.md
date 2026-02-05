---
title: Evaluation API
description: Evaluate flags for users
---

Evaluate whether a flag is enabled for a specific user.

## Evaluate Flag

```http
GET /v1/flags/{id}/evaluate?user_id={user_id}
```

### Query Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `user_id` | string | Yes | The user identifier |

### Response

```json
{
  "data": {
    "flag": "dark-mode",
    "enabled": true,
    "user_id": "user-123"
  }
}
```

## How Evaluation Works

1. **Flag disabled?** → Returns `false`
2. **Flag enabled, 100% rollout?** → Returns `true`
3. **Flag enabled, partial rollout?** → Consistent hashing based on `user_id`

### Percentage Rollouts

For partial rollouts (e.g., 30%), FlagLite uses consistent hashing:

```
hash(flag_name + user_id) % 100 < rollout_percentage
```

This ensures:
- Same user always gets the same result
- Distribution is statistically even
- No database storage needed per user

## Batch Evaluation

Evaluate multiple flags at once:

```http
POST /v1/evaluate
```

### Request Body

```json
{
  "user_id": "user-123",
  "flags": ["dark-mode", "new-checkout", "beta-features"]
}
```

### Response

```json
{
  "data": {
    "user_id": "user-123",
    "flags": {
      "dark-mode": true,
      "new-checkout": false,
      "beta-features": true
    }
  }
}
```

## SDK Usage

While you can use the REST API directly, we recommend using our SDKs:

- [JavaScript SDK](/docs/sdks/javascript)
- [Python SDK](/docs/sdks/python)
- [Go SDK](/docs/sdks/go)
- [Rust SDK](/docs/sdks/rust)

SDKs handle caching, retries, and error handling automatically.
