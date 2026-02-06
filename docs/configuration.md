# Configuration Reference

Complete reference for all FlagLite configuration options.

## Quick Navigation

- [Environment Variables](#environment-variables)
- [Database Configuration](#database-configuration)
- [Helm Values Reference](#helm-values-reference)
- [Backups](#backups)

---

## Environment Variables

These environment variables configure the FlagLite API server.

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `DATABASE_URL` | Database connection string | `sqlite:flaglite.db?mode=rwc` | No |
| `JWT_SECRET` | Secret for signing JWTs (min 32 chars) | — | Yes |
| `RUST_LOG` | Log level: `debug`, `info`, `warn`, `error` | `info` | No |

### CLI Options

The API server also accepts command-line arguments:

```bash
flaglite serve --port 3000 --host 0.0.0.0
```

| Flag | Description | Default |
|------|-------------|---------|
| `--port`, `-p` | HTTP port to listen on | `3000` |
| `--host` | Host to bind to | `0.0.0.0` |

### DATABASE_URL

Connection string format depends on your database:

**SQLite:**
```bash
DATABASE_URL=sqlite:/path/to/flaglite.db
DATABASE_URL=sqlite:/data/flaglite.db  # Docker/K8s
```

**PostgreSQL:**
```bash
DATABASE_URL=postgres://user:password@host:5432/database
DATABASE_URL=postgres://flaglite:secret@localhost:5432/flaglite
```

**PostgreSQL with SSL:**
```bash
DATABASE_URL=postgres://user:password@host:5432/database?sslmode=require
```

### JWT_SECRET

Used to sign and verify authentication tokens. **Must be at least 32 characters.**

Generate a secure secret:
```bash
openssl rand -hex 32
# Output: a1b2c3d4e5f6...64 hex characters
```

> ⚠️ **Security:** Never commit JWT_SECRET to version control. Use environment variables or secrets management.

### RUST_LOG

Controls logging verbosity:

| Level | Description |
|-------|-------------|
| `error` | Only errors |
| `warn` | Errors and warnings |
| `info` | Standard operational logs (recommended for production) |
| `debug` | Detailed debugging information |

Examples:
```bash
RUST_LOG=info                           # Recommended
RUST_LOG=debug                          # Troubleshooting
RUST_LOG=flaglite_api=debug,info        # Debug only FlagLite, info for dependencies
```

---

## Database Configuration

### SQLite

Best for: Development, single-instance deployments, testing.

**Pros:**
- Zero setup
- No external dependencies
- Fast for small workloads

**Cons:**
- Single-writer only (no horizontal scaling)
- Limited concurrent connections
- Not recommended for production HA

**Configuration:**
```bash
DATABASE_URL=sqlite:/data/flaglite.db
```

The database file is created automatically on first run.

### PostgreSQL

Best for: Production, high availability, multiple replicas.

**Pros:**
- Full ACID compliance
- Horizontal read scaling
- Supports multiple API replicas
- Better for high-traffic workloads

**Cons:**
- Requires separate database setup
- More operational overhead

**Configuration:**
```bash
DATABASE_URL=postgres://user:password@host:5432/flaglite
```

**Connection pooling:**
For high-traffic deployments, consider using PgBouncer:
```bash
DATABASE_URL=postgres://user:password@pgbouncer:6432/flaglite
```

### Database Migrations

Migrations run automatically on startup. No manual intervention required.

To verify migration status:
```bash
curl http://localhost:8080/health
# Returns {"status":"ok"} when database is ready
```

---

## Helm Values Reference

Complete reference for `charts/flaglite/values.yaml`.

### Core Settings

```yaml
# Number of API replicas
# NOTE: Must be 1 for SQLite (ReadWriteOnce PVC limitation)
replicaCount: 1

# Container image
image:
  repository: ghcr.io/faiscadev/flaglite
  tag: "0.1.0"
  pullPolicy: IfNotPresent

# Image pull secrets for private registries
imagePullSecrets: []

# Override chart name
nameOverride: ""
fullnameOverride: ""
```

### Service Account

```yaml
serviceAccount:
  create: false    # Create a dedicated service account
  name: ""         # Name of existing service account to use
```

### Pod Configuration

```yaml
podAnnotations: {}

podSecurityContext:
  fsGroup: 1000

securityContext:
  runAsNonRoot: true
  runAsUser: 1000
```

### Service

```yaml
service:
  type: ClusterIP    # ClusterIP, NodePort, or LoadBalancer
  port: 80           # Service port
  targetPort: 8080   # Container port
```

### Ingress

```yaml
ingress:
  enabled: true
  className: nginx   # Ingress class
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  hosts:
    - host: api.flaglite.dev
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: flaglite-tls
      hosts:
        - api.flaglite.dev
```

### Resources

```yaml
resources:
  requests:
    memory: "64Mi"
    cpu: "100m"
  limits:
    memory: "256Mi"
    cpu: "500m"
```

**Sizing guide:**

| Scale | Memory Request | Memory Limit | CPU Request | CPU Limit |
|-------|----------------|--------------|-------------|-----------|
| Small (< 100 flags) | 64Mi | 256Mi | 100m | 500m |
| Medium (100-1000 flags) | 128Mi | 512Mi | 200m | 1000m |
| Large (1000+ flags) | 256Mi | 1Gi | 500m | 2000m |

### Autoscaling

```yaml
autoscaling:
  enabled: false
  minReplicas: 2
  maxReplicas: 10
  targetCPUUtilizationPercentage: 80
```

### Persistence (SQLite)

```yaml
persistence:
  enabled: true
  storageClass: ""        # Use default if empty
  accessMode: ReadWriteOnce
  size: 1Gi
```

### Database

```yaml
database:
  # "sqlite" or "postgres"
  type: sqlite
  
  # SQLite configuration
  sqlitePath: /data/flaglite.db
  
  # PostgreSQL configuration (when type: postgres)
  postgres:
    host: ""
    port: 5432
    database: flaglite
    existingSecret: ""      # Secret with credentials
    usernameKey: username   # Key in secret for username
    passwordKey: password   # Key in secret for password
```

### Bundled PostgreSQL

```yaml
# Enable bundled PostgreSQL (recommended for production)
postgresql:
  enabled: false
  auth:
    database: flaglite
    username: flaglite
    # password: ""          # Auto-generated if not set
    # existingSecret: ""    # Use existing secret
  primary:
    persistence:
      enabled: true
      size: 10Gi
    resources:
      requests:
        memory: "256Mi"
        cpu: "100m"
      limits:
        memory: "512Mi"
        cpu: "500m"
```

When `postgresql.enabled: true`, the chart automatically:
- Deploys PostgreSQL as a subchart
- Configures `database.type: postgres`
- Sets up the connection string

### JWT

```yaml
jwt:
  existingSecret: ""    # Name of secret containing JWT secret
  secretKey: jwt-secret # Key in the secret
  secret: ""            # Direct value (not recommended)
```

**Recommended:** Create a secret and reference it:
```bash
kubectl create secret generic flaglite-jwt \
  --from-literal=jwt-secret="$(openssl rand -hex 32)"
```
```yaml
jwt:
  existingSecret: flaglite-jwt
```

### Logging

```yaml
logging:
  level: info   # debug, info, warn, error
```

### Health Checks

```yaml
livenessProbe:
  httpGet:
    path: /health
    port: http
  initialDelaySeconds: 5
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /health
    port: http
  initialDelaySeconds: 5
  periodSeconds: 5
```

### Scheduling

```yaml
nodeSelector: {}

tolerations: []

affinity: {}
```

---

## Backups

### PostgreSQL Backups (Helm)

The Helm chart includes optional automated backups:

```yaml
backup:
  enabled: true
  schedule: "0 2 * * *"    # Daily at 2 AM
  retention: 7              # Keep 7 backups
  
  # S3 storage (recommended)
  s3:
    enabled: true
    bucket: my-backups
    endpoint: ""           # Leave empty for AWS S3
    region: us-east-1
    existingSecret: backup-s3-credentials
```

Create the S3 credentials secret:
```bash
kubectl create secret generic backup-s3-credentials \
  --from-literal=access-key-id='AKIAIOSFODNN7EXAMPLE' \
  --from-literal=secret-access-key='wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY'
```

### Manual PostgreSQL Backup

```bash
# Backup
pg_dump -h localhost -U flaglite -d flaglite > backup.sql

# Restore
psql -h localhost -U flaglite -d flaglite < backup.sql
```

### SQLite Backup

Simply copy the database file:
```bash
# Stop writes first (or accept potential inconsistency)
cp /data/flaglite.db /backups/flaglite-$(date +%Y%m%d).db
```

For consistent backups:
```bash
sqlite3 /data/flaglite.db ".backup /backups/flaglite-$(date +%Y%m%d).db"
```

---

## Docker Compose Variables

When using `docker-compose.yml`, set these in your `.env` file:

```bash
# PostgreSQL
POSTGRES_USER=flaglite
POSTGRES_PASSWORD=secure_password_here
POSTGRES_DB=flaglite

# FlagLite API
JWT_SECRET=your_32_char_secret_here
RUST_LOG=info
FLAGLITE_PORT=8080
```

These map to container environment variables as configured in `docker-compose.yml`.

---

## Configuration Examples

### Minimal Production (Docker Compose)

```bash
# .env
POSTGRES_PASSWORD=G3n3r4t3dP4ssw0rd!
JWT_SECRET=a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4
RUST_LOG=info
```

### High-Availability Kubernetes

```yaml
# ha-values.yaml
replicaCount: 3

postgresql:
  enabled: true
  auth:
    database: flaglite
  primary:
    persistence:
      size: 20Gi

jwt:
  existingSecret: flaglite-jwt

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10

resources:
  requests:
    memory: "128Mi"
    cpu: "200m"
  limits:
    memory: "512Mi"
    cpu: "1000m"

backup:
  enabled: true
  s3:
    enabled: true
    bucket: flaglite-backups
    existingSecret: backup-credentials
```

### Development (Local SQLite)

```bash
export DATABASE_URL=sqlite:./dev.db
export JWT_SECRET=dev_secret_not_for_production_use
export RUST_LOG=debug

./flaglite-api serve
```

---

## Next Steps

- [Self-Hosting Guide](self-hosting.md) — Deployment instructions
- [API Documentation](https://flaglite.dev/docs/api) — REST API reference
