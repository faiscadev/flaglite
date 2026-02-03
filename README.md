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

## Quick Start (Self-Hosted)

```bash
# Install
curl -fsSL https://flaglite.dev/install.sh | sh

# Start the server
./flaglite serve

# Server running at http://localhost:8080
```

That's it. SQLite database created automatically.

---

## Quick Start (Hosted)

Don't want to self-host? Use FlagLite Cloud.

→ [Sign up at flaglite.dev](https://flaglite.dev/signup)

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

- [Getting Started](https://docs.flaglite.dev/getting-started)
- [API Reference](https://docs.flaglite.dev/api)
- [Self-Hosting Guide](https://docs.flaglite.dev/self-hosting)

---

## Project Structure

```
flaglite/
├── api/        # Rust API server (Axum + SQLx)
├── cli/        # Rust CLI (clap)
├── shared/     # Shared types and utilities
├── dashboard/  # React dashboard (Vite + TypeScript)
└── openapi/    # OpenAPI spec + docs
```

---

## Building from Source

```bash
# Clone
git clone https://github.com/faiscadev/flaglite.git
cd flaglite

# Build API + CLI
cargo build --release

# Binaries at:
# - target/release/flaglite-api (6.9MB)
# - target/release/flaglite (5.5MB CLI)

# Build dashboard
cd dashboard
npm install
npm run build
```

---

## License

AGPL-3.0 — see [LICENSE](LICENSE)
