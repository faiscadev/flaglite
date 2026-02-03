# FlagLite Helm Chart

Helm chart for deploying FlagLite to Kubernetes.

## Prerequisites

- Kubernetes 1.19+
- Helm 3.0+
- nginx-ingress controller (optional, for ingress)
- cert-manager (optional, for TLS)

## Installation

### Quick Start

```bash
# Add the repo (if published)
helm repo add faiscadev https://charts.faisca.dev
helm repo update

# Or install from local chart
helm install flaglite ./helm/flaglite
```

### Staging

```bash
helm install flaglite-staging ./helm/flaglite \
  -f ./helm/values-staging.yaml \
  -n staging --create-namespace
```

### Production

```bash
# First, create secrets
kubectl create secret generic flaglite-jwt \
  --from-literal=jwt-secret="$(openssl rand -hex 32)" \
  -n production

# Deploy
helm install flaglite ./helm/flaglite \
  -f ./helm/values-production.yaml \
  --set jwt.existingSecret=flaglite-jwt \
  -n production --create-namespace
```

## Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of replicas | `2` |
| `image.repository` | Image repository | `ghcr.io/faiscadev/flaglite` |
| `image.tag` | Image tag | `latest` |
| `ingress.enabled` | Enable ingress | `true` |
| `ingress.hosts[0].host` | Ingress hostname | `api.flaglite.dev` |
| `persistence.enabled` | Enable persistence | `true` |
| `persistence.size` | PVC size | `1Gi` |
| `database.type` | Database type (`sqlite` or `postgres`) | `sqlite` |
| `autoscaling.enabled` | Enable HPA | `false` |

See `values.yaml` for all options.

## Upgrading

```bash
helm upgrade flaglite ./helm/flaglite -f your-values.yaml
```

## Uninstalling

```bash
helm uninstall flaglite
```

**Note:** PVC is not deleted automatically. To remove data:

```bash
kubectl delete pvc flaglite-data
```
