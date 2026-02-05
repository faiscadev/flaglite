---
title: Authentication
description: API authentication and API keys
---

FlagLite uses API keys for authentication.

## Getting Your API Key

### Cloud

1. Sign up at [flaglite.dev/signup](https://flaglite.dev/signup)
2. Go to Settings → API Keys
3. Create a new key

### Self-hosted

```bash
curl -X POST http://localhost:8080/v1/auth/signup \
  -H "Content-Type: application/json" \
  -d '{"email": "you@example.com", "password": "secure-password"}'
```

Response:

```json
{
  "data": {
    "user": {
      "id": "usr_abc123",
      "email": "you@example.com"
    },
    "api_key": "fl_live_abc123...",
    "token": "eyJhbGciOiJIUzI1NiIs..."
  }
}
```

## Using Your API Key

Include your API key in the `Authorization` header:

```bash
curl https://api.flaglite.dev/v1/flags \
  -H "Authorization: Bearer fl_live_abc123..."
```

## API Key Types

| Prefix | Type | Use Case |
|--------|------|----------|
| `fl_live_` | Live | Production apps |
| `fl_test_` | Test | Development/testing |

Test keys only access flags in test environments.

## Security Best Practices

1. **Never commit API keys** — Use environment variables
2. **Rotate keys regularly** — Especially if compromised
3. **Use test keys in development** — Keep live keys for production
4. **Server-side only** — Never expose keys in client-side code

## Endpoints

### `POST /v1/auth/signup`

Create a new account.

```json
{
  "email": "you@example.com",
  "password": "secure-password"
}
```

### `POST /v1/auth/login`

Get a JWT token.

```json
{
  "email": "you@example.com",
  "password": "secure-password"
}
```

### `GET /v1/auth/me`

Get the current authenticated user.
