# Self-Hosting FlagLite

This guide covers everything you need to deploy FlagLite in your own infrastructure. Pick the method that fits your setup — you'll be running in under 10 minutes.

## Quick Navigation

- [Prerequisites](#prerequisites)
- [Option 1: Docker Compose](#option-1-docker-compose-recommended) — Easiest, production-ready
- [Option 2: Kubernetes with Helm](#option-2-kubernetes-with-helm) — For existing K8s clusters
- [Option 3: Single Binary](#option-3-single-binary) — Zero dependencies
- [Production Checklist](#production-checklist)

---

## Prerequisites

### For Docker Compose
- Docker Engine 20.10+
- Docker Compose v2+

### For Kubernetes
- Kubernetes 1.19+
- kubectl configured
- Helm 3.0+ (for Helm deployments)
- nginx-ingress controller (for ingress)
- cert-manager (for automatic TLS)

### For Single Binary
- 64-bit Linux, macOS, or Windows
- No other dependencies!

---

## Option 1: Docker Compose (Recommended)

The fastest path to a production-ready setup with PostgreSQL.

### Quick Start

```bash
# Clone the repo
git clone https://github.com/faiscadev/flaglite.git
cd flaglite

# Create and configure environment
cp .env.example .env

# Generate a secure JWT secret
JWT_SECRET=$(openssl rand -hex 32)
sed -i.bak "s/JWT_SECRET=.*/JWT_SECRET=$JWT_SECRET/" .env

# Start FlagLite
docker compose up -d
```

**Expected output:**
```
[+] Running 3/3
 ✔ Network flaglite_default           Created
 ✔ Container flaglite-postgres        Healthy
 ✔ Container flaglite-api             Started
```

Verify it's working:
```bash
curl http://localhost:8080/health
# {"status":"ok"}
```

### Configuration

Edit `.env` before starting:

```bash
# .env
POSTGRES_USER=flaglite
POSTGRES_PASSWORD=your_secure_password_here    # CHANGE THIS
POSTGRES_DB=flaglite

JWT_SECRET=generate_with_openssl_rand_hex_32   # CHANGE THIS
RUST_LOG=info
FLAGLITE_PORT=8080
```

### Managing the Service

```bash
# View logs
docker compose logs -f api

# Stop
docker compose down

# Stop and remove all data (fresh start)
docker compose down -v

# Upgrade to new version
docker compose pull
docker compose up -d
```

---

## Option 2: Kubernetes with Helm

Best for existing Kubernetes clusters. Includes built-in PostgreSQL option.

### Quick Start

```bash
# Clone the repo
git clone https://github.com/faiscadev/flaglite.git
cd flaglite

# Create namespace
kubectl create namespace flaglite

# Create JWT secret
kubectl create secret generic flaglite-jwt \
  --namespace flaglite \
  --from-literal=jwt-secret="$(openssl rand -hex 32)"

# Install with bundled PostgreSQL
helm install flaglite ./charts/flaglite \
  --namespace flaglite \
  --set postgresql.enabled=true \
  --set jwt.existingSecret=flaglite-jwt \
  --set ingress.hosts[0].host=flags.your-domain.com \
  --set ingress.tls[0].hosts[0]=flags.your-domain.com \
  --set ingress.tls[0].secretName=flaglite-tls
```

**Expected output:**
```
NAME: flaglite
LAST DEPLOYED: ...
NAMESPACE: flaglite
STATUS: deployed
```

Verify deployment:
```bash
kubectl get pods -n flaglite
# NAME                           READY   STATUS    RESTARTS   AGE
# flaglite-api-xxx-yyy           1/1     Running   0          30s
# flaglite-postgresql-0          1/1     Running   0          30s
```

### Using an External PostgreSQL

If you have an existing PostgreSQL database:

```bash
# Create database secret
kubectl create secret generic flaglite-db \
  --namespace flaglite \
  --from-literal=database-url='postgres://user:pass@your-db-host:5432/flaglite'

# Create JWT secret
kubectl create secret generic flaglite-jwt \
  --namespace flaglite \
  --from-literal=jwt-secret="$(openssl rand -hex 32)"

# Install
helm install flaglite ./charts/flaglite \
  --namespace flaglite \
  --set database.type=postgres \
  --set database.postgres.existingSecret=flaglite-db \
  --set jwt.existingSecret=flaglite-jwt \
  --set ingress.hosts[0].host=flags.your-domain.com
```

### Using SQLite (Development Only)

For development or testing:

```bash
helm install flaglite ./charts/flaglite \
  --namespace flaglite \
  --set database.type=sqlite \
  --set replicaCount=1 \
  --set ingress.enabled=false
```

> ⚠️ **Warning:** SQLite only supports a single replica due to ReadWriteOnce PVC limitations. Use PostgreSQL for production.

### Custom Values File

For more control, create a values file:

```yaml
# my-values.yaml
replicaCount: 3

postgresql:
  enabled: true
  auth:
    database: flaglite
    username: flaglite
  primary:
    persistence:
      size: 20Gi

jwt:
  existingSecret: flaglite-jwt

ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  hosts:
    - host: flags.mycompany.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: flaglite-tls
      hosts:
        - flags.mycompany.com

resources:
  requests:
    memory: "128Mi"
    cpu: "200m"
  limits:
    memory: "512Mi"
    cpu: "1000m"

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 10
```

Then install:
```bash
helm install flaglite ./charts/flaglite \
  --namespace flaglite \
  -f my-values.yaml
```

### Upgrading

```bash
helm upgrade flaglite ./charts/flaglite \
  --namespace flaglite \
  -f my-values.yaml
```

### Uninstalling

```bash
helm uninstall flaglite --namespace flaglite

# Remove PVCs if you want to delete data
kubectl delete pvc -l app.kubernetes.io/name=flaglite -n flaglite
```

---

## Option 3: Single Binary

Zero dependencies. Perfect for VMs or bare metal.

### Install

```bash
# Download and install
curl -fsSL https://flaglite.dev/install.sh | sh

# Or download directly
wget https://github.com/faiscadev/flaglite/releases/latest/download/flaglite-linux-amd64
chmod +x flaglite-linux-amd64
mv flaglite-linux-amd64 /usr/local/bin/flaglite-api
```

### Run

```bash
# Set required environment
export JWT_SECRET=$(openssl rand -hex 32)
export DATABASE_URL=sqlite:/var/lib/flaglite/data.db
export RUST_LOG=info

# Start server
./flaglite-api serve
# Server running at http://0.0.0.0:8080
```

### Systemd Service

Create `/etc/systemd/system/flaglite.service`:

```ini
[Unit]
Description=FlagLite Feature Flags
After=network.target

[Service]
Type=simple
User=flaglite
Environment=DATABASE_URL=sqlite:/var/lib/flaglite/data.db
Environment=JWT_SECRET=your_secret_here
Environment=RUST_LOG=info
ExecStart=/usr/local/bin/flaglite-api serve
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable flaglite
sudo systemctl start flaglite
```

---

## Production Checklist

Before going live, ensure you've addressed these items:

### Security

- [ ] **JWT Secret**: Generate a secure 32+ character secret
  ```bash
  openssl rand -hex 32
  ```
- [ ] **Database credentials**: Use strong passwords, store in secrets manager
- [ ] **TLS/HTTPS**: Use nginx, traefik, or cloud load balancer with TLS
- [ ] **Network policies**: Restrict database access to FlagLite pods only

### Database

- [ ] **Use PostgreSQL for production**: SQLite is fine for dev, not for HA
- [ ] **Enable backups**: See [Backup Configuration](configuration.md#backups)
- [ ] **Monitor disk space**: Especially with SQLite

### High Availability

- [ ] **Multiple replicas**: At least 2 API pods (requires PostgreSQL)
- [ ] **Pod disruption budgets**: Ensure availability during updates
- [ ] **Resource limits**: Set appropriate memory/CPU limits
- [ ] **Health checks**: Verify liveness/readiness probes are working

### Monitoring

- [ ] **Log aggregation**: Send logs to your observability stack
- [ ] **Metrics**: FlagLite exposes Prometheus metrics at `/metrics`
- [ ] **Alerting**: Monitor `/health` endpoint, database connectivity

### TLS Configuration

**With nginx ingress + cert-manager:**
```yaml
ingress:
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  tls:
    - secretName: flaglite-tls
      hosts:
        - flags.your-domain.com
```

**With Docker Compose (add nginx):**
```yaml
# Add to docker-compose.yml
services:
  nginx:
    image: nginx:alpine
    ports:
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./certs:/etc/nginx/certs
    depends_on:
      - api
```

---

## Monitoring

FlagLite exposes Prometheus metrics at `/metrics` and the Helm chart includes optional ServiceMonitor and alerting rules for Prometheus Operator.

### Prerequisites

- [Prometheus Operator](https://prometheus-operator.dev/) installed in your cluster
- Prometheus configured to discover ServiceMonitors (check your Prometheus `serviceMonitorSelector`)

### Enable ServiceMonitor

Add to your values file or Helm command:

```yaml
# values.yaml
monitoring:
  enabled: true
  serviceMonitor:
    interval: 30s
    # Add labels if your Prometheus requires specific selectors
    labels:
      release: prometheus
```

Or via CLI:
```bash
helm install flaglite ./charts/flaglite \
  --namespace flaglite \
  --set monitoring.enabled=true \
  --set monitoring.serviceMonitor.labels.release=prometheus
```

### Enable Alerting Rules

The chart includes these alerts:

| Alert | Severity | Description |
|-------|----------|-------------|
| **FlagLitePodDown** | Critical | No healthy pods for 5 minutes |
| **FlagLiteHighErrorRate** | Warning | >5% of requests returning 5xx errors |
| **FlagLiteHighLatency** | Warning | p99 latency exceeds 500ms |
| **FlagLitePodRestarting** | Warning | Pod restarted >3 times in an hour |

Enable alerts:

```yaml
monitoring:
  enabled: true
  alerts:
    enabled: true
    # Add labels if your Prometheus requires specific selectors
    labels:
      release: prometheus
```

### Grafana Dashboard

Import FlagLite metrics into Grafana with these useful queries:

```promql
# Request rate
sum(rate(http_requests_total{job="flaglite"}[5m]))

# Error rate percentage
sum(rate(http_requests_total{job="flaglite", status=~"5.."}[5m])) 
/ sum(rate(http_requests_total{job="flaglite"}[5m])) * 100

# P99 latency
histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket{job="flaglite"}[5m])) by (le))

# Active pods
sum(up{job="flaglite"})
```

### Verifying Setup

Check that ServiceMonitor is created:
```bash
kubectl get servicemonitor -n flaglite
# NAME       AGE
# flaglite   1m
```

Verify Prometheus is scraping:
```bash
# Port-forward to Prometheus
kubectl port-forward -n monitoring svc/prometheus-operated 9090

# Check targets in browser
open http://localhost:9090/targets
# Look for flaglite endpoint with "UP" status
```

---

## Troubleshooting

### Common Issues

**Container won't start:**
```bash
# Check logs
docker compose logs api
# or
kubectl logs -l app=flaglite -n flaglite
```

**Database connection failed:**
```bash
# Verify DATABASE_URL is correct
# For Docker: ensure postgres container is healthy
docker compose ps

# For K8s: check secret exists
kubectl get secret flaglite-secrets -n flaglite -o yaml
```

**Health check failing:**
```bash
# Test directly
curl http://localhost:8080/health

# Check if port is exposed
netstat -tlnp | grep 8080
```

**Permission denied (SQLite):**
```bash
# Ensure data directory is writable
chmod 755 /data
chown 1000:1000 /data
```

---

## Next Steps

- [Configuration Reference](configuration.md) — All environment variables and Helm values
- [API Documentation](https://docs.flaglite.dev/api) — REST API reference
- [SDK Setup](https://docs.flaglite.dev/sdks) — Connect your application
