---
title: Installation
description: Self-host FlagLite or use our managed cloud
---

Choose your deployment path.

## Self-hosted (Docker)

The fastest way to get started:

```bash
git clone https://github.com/faiscadev/flaglite
cd flaglite
docker compose up -d
```

FlagLite is now running at `http://localhost:8080`.

### Requirements

- Docker & Docker Compose
- 512MB RAM minimum
- 1GB disk space

### Configuration

Create a `.env` file:

```bash
# Required
DATABASE_URL=postgres://flaglite:secret@db:5432/flaglite

# Optional
PORT=8080
LOG_LEVEL=info
JWT_SECRET=your-super-secret-key
```

## Self-hosted (Binary)

Download the 6.9MB binary directly:

```bash
# Linux
curl -L https://github.com/faiscadev/flaglite/releases/latest/download/flaglite-linux-amd64 -o flaglite
chmod +x flaglite
./flaglite

# macOS
curl -L https://github.com/faiscadev/flaglite/releases/latest/download/flaglite-darwin-amd64 -o flaglite
chmod +x flaglite
./flaglite
```

## Cloud (Hosted)

Let us run it. You ship.

1. [Sign up](https://flaglite.dev/signup) for $19/mo
2. Get your API key from the dashboard
3. Start using the SDKs immediately

No setup, no maintenance, 99.9% uptime SLA.
