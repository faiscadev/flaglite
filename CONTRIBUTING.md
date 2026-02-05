# Contributing to FlagLite

Thank you for considering contributing to FlagLite! This guide will help you get started.

## Development Setup

### Prerequisites

- Rust 1.80+ (`rustup update stable`)
- rustfmt and clippy (`rustup component add rustfmt clippy`)

### Getting Started

```bash
# Clone the repository
git clone https://github.com/faiscadev/flaglite.git
cd flaglite

# Build the workspace
cargo build --workspace

# Run all checks
cargo xtask check
```

## Development Workflow

FlagLite uses `xtask` for common development tasks. All commands are run via `cargo xtask <command>`.

### Available Commands

| Command | Description |
|---------|-------------|
| `cargo xtask check` | Run all checks: fmt, clippy, and tests |
| `cargo xtask fmt` | Format all code |
| `cargo xtask fmt --check` | Check formatting without modifying files |
| `cargo xtask lint` | Run clippy lints |
| `cargo xtask test` | Run all tests |
| `cargo xtask test -p <package>` | Run tests for a specific package |
| `cargo xtask coverage` | Run tests with coverage report |

### Before Submitting a PR

Always run the full check before pushing:

```bash
cargo xtask check
```

This ensures:
- ✅ Code is formatted correctly
- ✅ No clippy warnings
- ✅ All tests pass

## Project Structure

```
flaglite/
├── apps/
│   ├── flaglite-api/       # API server (Axum)
│   └── flaglite-cli/       # CLI application
├── crates/
│   ├── flaglite-core/      # Core types and errors
│   └── flaglite-client/    # HTTP client library
├── xtask/                  # Dev task automation
├── docs/                   # Documentation
└── Cargo.toml              # Workspace root
```

## Code Style

- Run `cargo xtask fmt` to format code
- Follow Rust idioms and conventions
- Use meaningful variable and function names
- Add doc comments for public APIs

## Testing

- Write unit tests for new functionality
- Place tests in a `tests` submodule or `tests/` directory
- Run `cargo xtask test` to verify all tests pass

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/my-feature`)
3. Make your changes
4. Run `cargo xtask check`
5. Commit with a meaningful message (`git commit -m "feat: add my feature"`)
6. Push to your fork
7. Open a Pull Request

## Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` — New feature
- `fix:` — Bug fix
- `docs:` — Documentation changes
- `refactor:` — Code refactoring
- `test:` — Adding or updating tests
- `chore:` — Maintenance tasks

## Questions?

- Open an issue for bugs or feature requests
- Join our [Discord](https://discord.gg/flaglite) for discussions

Thank you for contributing! 🚀
