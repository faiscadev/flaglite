# FlagLite Documentation

**Open source feature flags. Single binary. Works everywhere.**

---

## Getting Started

New to FlagLite? Start here:

1. **[Self-Hosting Guide](self-hosting.md)** — Deploy FlagLite in under 10 minutes
2. **[Configuration Reference](configuration.md)** — All settings and options
3. **[API Documentation](https://docs.flaglite.dev/api)** — REST API reference

---

## Deployment Options

| Method | Best For | Time to Deploy |
|--------|----------|----------------|
| [Docker Compose](self-hosting.md#option-1-docker-compose-recommended) | Production, easy setup | 5 minutes |
| [Kubernetes + Helm](self-hosting.md#option-2-kubernetes-with-helm) | Existing K8s clusters | 10 minutes |
| [Manual Kubernetes](self-hosting.md#option-3-manual-kubernetes) | Full control | 10 minutes |
| [Single Binary](self-hosting.md#option-4-single-binary) | VMs, bare metal | 2 minutes |

---

## Documentation Index

### Deployment & Operations

- **[Self-Hosting Guide](self-hosting.md)** — Complete deployment instructions
  - Docker Compose setup
  - Kubernetes with Helm
  - Manual Kubernetes manifests
  - Single binary installation
  - Production checklist

- **[Configuration Reference](configuration.md)** — All configuration options
  - Environment variables
  - Database setup (SQLite vs PostgreSQL)
  - Helm values reference
  - Backup configuration

### SDKs

| Language | Documentation | Repository |
|----------|---------------|------------|
| JavaScript | [SDK Docs](https://docs.flaglite.dev/sdks/javascript) | [flaglite-js](https://github.com/faiscadev/flaglite-js) |
| Python | [SDK Docs](https://docs.flaglite.dev/sdks/python) | [flaglite-py](https://github.com/faiscadev/flaglite-py) |
| Go | [SDK Docs](https://docs.flaglite.dev/sdks/go) | [flaglite-go](https://github.com/faiscadev/flaglite-go) |
| Rust | [SDK Docs](https://docs.flaglite.dev/sdks/rust) | [flaglite-rs](https://github.com/faiscadev/flaglite-rs) |

### API Reference

- **[REST API](https://docs.flaglite.dev/api)** — Full API documentation
- **[OpenAPI Spec](../openapi/)** — Machine-readable API specification

### CLI

- **[CLI Reference](https://docs.flaglite.dev/cli)** — Command-line interface docs

---

## Quick Links

- **GitHub:** [github.com/faiscadev/flaglite](https://github.com/faiscadev/flaglite)
- **Issues:** [Report bugs or request features](https://github.com/faiscadev/flaglite/issues)
- **Discord:** [Join the community](https://discord.gg/flaglite)
- **FlagLite Cloud:** [flaglite.dev](https://flaglite.dev) — Managed hosting

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                       Your Application                       │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐        │
│  │   JS    │  │ Python  │  │   Go    │  │  Rust   │        │
│  │   SDK   │  │   SDK   │  │   SDK   │  │   SDK   │        │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘        │
│       │            │            │            │              │
└───────┼────────────┼────────────┼────────────┼──────────────┘
        │            │            │            │
        └────────────┴─────┬──────┴────────────┘
                           │
                    ┌──────▼──────┐
                    │  FlagLite   │
                    │  REST API   │
                    └──────┬──────┘
                           │
              ┌────────────┴────────────┐
              │                         │
       ┌──────▼──────┐          ┌──────▼──────┐
       │   SQLite    │    OR    │ PostgreSQL  │
       │   (dev)     │          │   (prod)    │
       └─────────────┘          └─────────────┘
```

---

## Support

- **Documentation issues:** [Open an issue](https://github.com/faiscadev/flaglite/issues/new?labels=documentation)
- **Bug reports:** [Bug report template](https://github.com/faiscadev/flaglite/issues/new?labels=bug)
- **Feature requests:** [Feature request template](https://github.com/faiscadev/flaglite/issues/new?labels=enhancement)
- **Community help:** [Discord](https://discord.gg/flaglite)
