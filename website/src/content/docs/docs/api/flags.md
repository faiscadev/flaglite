---
title: Flags API
description: Create, read, update, and delete feature flags
---

Manage your feature flags via the REST API.

## List Flags

```http
GET /v1/flags
```

### Response

```json
{
  "data": [
    {
      "id": "flg_abc123",
      "name": "dark-mode",
      "enabled": true,
      "rollout_percentage": 100,
      "created_at": "2024-01-15T10:00:00Z",
      "updated_at": "2024-01-15T10:00:00Z"
    }
  ],
  "meta": {
    "total": 1,
    "page": 1,
    "per_page": 20
  }
}
```

## Create Flag

```http
POST /v1/flags
```

### Request Body

```json
{
  "name": "new-checkout",
  "enabled": false,
  "rollout_percentage": 0
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Unique flag identifier |
| `enabled` | boolean | No | Default: `false` |
| `rollout_percentage` | integer | No | 0-100, default: `100` |

### Response

```json
{
  "data": {
    "id": "flg_xyz789",
    "name": "new-checkout",
    "enabled": false,
    "rollout_percentage": 0,
    "created_at": "2024-01-15T12:00:00Z",
    "updated_at": "2024-01-15T12:00:00Z"
  }
}
```

## Get Flag

```http
GET /v1/flags/{id}
```

## Update Flag

```http
PATCH /v1/flags/{id}
```

### Request Body

```json
{
  "enabled": true,
  "rollout_percentage": 50
}
```

All fields are optional. Only provided fields are updated.

## Delete Flag

```http
DELETE /v1/flags/{id}
```

Returns `204 No Content` on success.

## Toggle Flag

```http
POST /v1/flags/{id}/toggle
```

Toggles the flag's `enabled` state. Returns the updated flag.

## Common Errors

| Status | Code | Description |
|--------|------|-------------|
| 400 | `invalid_request` | Malformed request body |
| 404 | `not_found` | Flag doesn't exist |
| 409 | `conflict` | Flag name already exists |
| 422 | `validation_error` | Invalid field values |
