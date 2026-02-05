# FlagLite Helm Chart

Deploy FlagLite to Kubernetes.

## Quick Start

```bash
# Clone the repo
git clone https://github.com/faiscadev/flaglite.git
cd flaglite

# Create your values file
cp helm/flaglite/values.example.yaml my-values.yaml
# Edit my-values.yaml with your settings

# Create secrets
kubectl create secret generic flaglite-db \
  --from-literal=database-url='postgres://user:pass@host:5432/flaglite'

kubectl create secret generic flaglite-jwt \
  --from-literal=jwt-secret="$(openssl rand -hex 32)"

# Deploy
helm install flaglite ./helm/flaglite -f my-values.yaml
```

## Prerequisites

- Kubernetes 1.19+
- Helm 3.0+
- PostgreSQL (recommended) or SQLite
- nginx-ingress controller (for ingress)
- cert-manager (for TLS)

## Configuration

See `values.yaml` for all options. Key settings:

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of replicas | `2` |
| `image.repository` | Image repository | `ghcr.io/faiscadev/flaglite` |
| `image.tag` | Image tag | `latest` |
| `database.type` | `sqlite` or `postgres` | `sqlite` |
| `ingress.enabled` | Enable ingress | `true` |
| `ingress.hosts[0].host` | Your domain | `api.flaglite.dev` |
| `autoscaling.enabled` | Enable HPA | `false` |

## Examples

### Minimal Production

```yaml
# my-values.yaml
ingress:
  hosts:
    - host: flags.mycompany.com
      paths:
        - path: /
          pathType: Prefix

database:
  type: postgres
  postgres:
    existingSecret: my-postgres-secret

jwt:
  existingSecret: my-jwt-secret
```

### Development (SQLite)

```yaml
# dev-values.yaml
replicaCount: 1

database:
  type: sqlite

persistence:
  enabled: true
  size: 512Mi

ingress:
  enabled: false
```

## Upgrading

```bash
helm upgrade flaglite ./helm/flaglite -f my-values.yaml
```

## Uninstalling

```bash
helm uninstall flaglite

# Optional: remove PVC
kubectl delete pvc -l app.kubernetes.io/name=flaglite
```

## Troubleshooting

```bash
# Check pods
kubectl get pods -l app.kubernetes.io/name=flaglite

# View logs
kubectl logs -l app.kubernetes.io/name=flaglite -f

# Check ingress
kubectl get ingress flaglite
```

## Support

- Docs: https://flaglite.dev/docs
- Issues: https://github.com/faiscadev/flaglite/issues
- Discord: https://discord.gg/flaglite
