# FlagLite API Documentation

This directory contains the OpenAPI 3.0 specification for the FlagLite API, along with interactive documentation viewers.

## Files

| File | Description |
|------|-------------|
| `openapi.yaml` | OpenAPI 3.0 specification (source of truth) |
| `index.html` | Redoc-powered documentation (clean, readable) |
| `swagger.html` | Swagger UI documentation (interactive "Try it out") |

## Viewing the Documentation

### Local Development

Serve the files with any static server:

```bash
# Using Python
python3 -m http.server 8080

# Using Node.js (npx)
npx serve .

# Using PHP
php -S localhost:8080
```

Then open:
- **Redoc**: http://localhost:8080/index.html
- **Swagger UI**: http://localhost:8080/swagger.html

### Production

Deploy the entire `openapi/` directory to your static hosting (Vercel, Netlify, S3, etc).

## API Overview

### Authentication

FlagLite uses three types of authentication tokens:

| Token Type | Format | Use Case |
|------------|--------|----------|
| JWT Token | `eyJhbG...` | User authentication (from login/signup) |
| Project API Key | `ffl_proj_xxxxx` | Dashboard operations (manage flags) |
| Environment API Key | `ffl_env_xxx_xxxxx` | SDK flag evaluation |

### Endpoints

#### Auth
- `POST /v1/auth/signup` - Create account
- `POST /v1/auth/login` - Get JWT token
- `GET /v1/auth/me` - Get current user profile

#### Flags
- `GET /v1/flags/:key` - Evaluate flag (SDK endpoint)
- `GET /v1/flags` - List all flags
- `POST /v1/flags` - Create flag
- `PATCH /v1/flags/:key/environments/:env` - Update flag value
- `POST /v1/flags/:key/toggle` - Toggle flag on/off

## SDK Code Generation

The OpenAPI spec can generate client SDKs:

```bash
# Using OpenAPI Generator
npx @openapitools/openapi-generator-cli generate \
  -i openapi.yaml \
  -g typescript-fetch \
  -o ../sdk/generated

# Using Orval (React Query)
npx orval --config orval.config.js
```

## Validation

Validate the spec:

```bash
# Using Redocly CLI
npx @redocly/cli lint openapi.yaml

# Using Swagger CLI
npx swagger-cli validate openapi.yaml
```

## Contributing

When updating the API:

1. Edit `openapi.yaml`
2. Validate: `npx @redocly/cli lint openapi.yaml`
3. Test: Serve locally and check both Redoc and Swagger UI
4. Commit with clear message about API changes
