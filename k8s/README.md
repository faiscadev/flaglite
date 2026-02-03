# FlagLite Kubernetes Deployment

## Prerequisites

- Kubernetes cluster
- kubectl configured
- nginx-ingress controller
- cert-manager (for TLS)

## Quick Start

1. **Create secrets:**
   ```bash
   kubectl create secret generic flaglite-secrets \
     --from-literal=database-url='sqlite:/data/flaglite.db' \
     --from-literal=jwt-secret='$(openssl rand -hex 32)'
   ```

2. **Deploy:**
   ```bash
   kubectl apply -f deployment.yaml
   ```

3. **Verify:**
   ```bash
   kubectl get pods -l app=flaglite
   kubectl get ingress flaglite-api
   ```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | SQLite or Postgres connection string | `sqlite:/data/flaglite.db` |
| `JWT_SECRET` | Secret for signing JWTs | (required) |
| `RUST_LOG` | Log level | `info` |

### PostgreSQL (Production)

For production, use PostgreSQL instead of SQLite:

```bash
kubectl create secret generic flaglite-secrets \
  --from-literal=database-url='postgres://user:pass@host:5432/flaglite' \
  --from-literal=jwt-secret='$(openssl rand -hex 32)'
```

## Scaling

Default is 2 replicas. Scale with:

```bash
kubectl scale deployment flaglite-api --replicas=3
```

## Resources

Default resource limits:
- Memory: 64Mi request, 256Mi limit
- CPU: 100m request, 500m limit

Adjust in `deployment.yaml` based on load.
