# FlagLite

[![Build Status](https://img.shields.io/github/actions/workflow/status/faiscadev/flaglite/ci.yml?branch=main)](https://github.com/faiscadev/flaglite/actions)
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)
[![GitHub Stars](https://img.shields.io/github/stars/faiscadev/flaglite)](https://github.com/faiscadev/flaglite)

**Open source feature flags. Single binary. Works everywhere.**

---

## Why FlagLite

- **Single binary, zero dependencies** — One 6.9MB binary. No JVM, no Docker, no runtime required.
- **API-first** — Everything via REST API or CLI. Dashboard is optional.
- **SQLite to Postgres** — Start with SQLite, scale to Postgres when you need it.
- **Production SDKs** — First-class support for JS, Python, Go, and Rust.

---

## Getting Started

Choose your deployment method:

| Method | Best For | Time |
|--------|----------|------|
| [Docker Compose](#quick-start-docker-compose) | Production, easy setup | 5 min |
| [Single Binary](#quick-start-single-binary) | VMs, bare metal | 2 min |
| [Kubernetes](docs/self-hosting.md#option-2-kubernetes-with-helm) | K8s clusters | 10 min |

📖 **Full guide:** [Self-Hosting Documentation](docs/self-hosting.md)

---

## Quick Start (Docker Compose)

The fastest path to production with PostgreSQL:

```bash
# Clone and enter
git clone https://github.com/faiscadev/flaglite.git
cd flaglite

# Configure (generate secure JWT secret)
cp .env.example .env
sed -i.bak "s/JWT_SECRET=.*/JWT_SECRET=$(openssl rand -hex 32)/" .env

# Start
docker compose up -d

# Verify
curl http://localhost:8080/health
# {"status":"ok"}
```

**For production**, also update `POSTGRES_PASSWORD` in `.env`.

📖 [Full Docker Compose guide](docs/self-hosting.md#option-1-docker-compose-recommended)

---

## Quick Start (Single Binary)

Zero dependencies, instant setup:

```bash
# Install
curl -fsSL https://flaglite.dev/install.sh | sh

# Configure and run
export JWT_SECRET=$(openssl rand -hex 32)
./flaglite serve

# Server running at http://localhost:8080
```

📖 [Full binary setup guide](docs/self-hosting.md#option-4-single-binary)

---

## Quick Start (Hosted)

Don't want to self-host? Use FlagLite Cloud.

→ [Get started at flaglite.dev](https://flaglite.dev/#quickstart)

---

## SDKs

| Language | Install | Repo |
|----------|---------|------|
| JavaScript | `npm install flaglite` | [flaglite-js](https://github.com/faiscadev/flaglite-js) |
| Python | `pip install flaglite` | [flaglite-py](https://github.com/faiscadev/flaglite-py) |
| Go | `go get github.com/faiscadev/flaglite-go` | [flaglite-go](https://github.com/faiscadev/flaglite-go) |
| Rust | `cargo add flaglite` | [flaglite-rs](https://github.com/faiscadev/flaglite-rs) |

### JavaScript Example

```javascript
import { FlagLite } from 'flaglite'

const flags = new FlagLite()

if (await flags.enabled('new-checkout')) {
  showNewCheckout()
}
```

---

## CLI

```bash
# Login
flaglite login

# Create a flag
flaglite flags create new-checkout

# Toggle a flag
flaglite flags toggle new-checkout --env production

# List all flags
flaglite flags list
```

Full CLI reference: [docs.flaglite.dev/cli](https://docs.flaglite.dev/cli)

---

## Features

- **Boolean flags** — Simple on/off toggles
- **Percentage rollouts** — Gradual feature releases with sticky bucketing
- **Environments** — Separate dev, staging, production
- **REST API** — Full control programmatically
- **CLI** — Full parity with dashboard
- **Self-hosted** — SQLite by default, Postgres for scale
- **Production SDKs** — JS, Python, Go, Rust

---

## Documentation

- **[Self-Hosting Guide](docs/self-hosting.md)** — Deploy FlagLite (Docker, Kubernetes, binary)
- **[Configuration Reference](docs/configuration.md)** — Environment variables, Helm values, database setup
- **[API Reference](https://docs.flaglite.dev/api)** — REST API documentation
- **[CLI Reference](https://docs.flaglite.dev/cli)** — Command-line interface

---

## Project Structure

```
flaglite/
├── apps/
│   ├── flaglite-api/       # API server (Axum + SQLx)
│   ├── flaglite-cli/       # CLI binary (clap)
│   └── e2e-tests/          # Integration tests
├── crates/
│   ├── flaglite-core/      # Core types, traits, errors
│   └── flaglite-client/    # HTTP client library
├── charts/                 # Helm charts
├── dashboard/              # React dashboard (Vite + TypeScript)
├── docker/                 # Dockerfiles
├── docs/                   # Documentation
├── openapi/                # OpenAPI spec
├── scripts/                # Helper scripts
├── website/                # Marketing site (Astro)
└── xtask/                  # Dev tooling (cargo xtask)
```

---

## Building from Source

```bash
# Clone
git clone https://github.com/faiscadev/flaglite.git
cd flaglite

# Build all crates
cargo build --workspace --release

# Binaries at:
# - target/release/flaglite-api
# - target/release/flaglite-cli

# Build dashboard
cd dashboard
npm install
npm run build
```

---

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and workflow.

Quick start:
```bash
# Run all quality checks
cargo xtask check

# Run tests
cargo test --workspace

# Format code
cargo fmt --all
```

---

## License

AGPL-3.0 — see [LICENSE](LICENSE)
