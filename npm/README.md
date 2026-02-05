# @faiscadev/flaglite-cli

Official CLI for [FlagLite](https://flaglite.dev) - Feature flags that get out of your way.

## Installation

```bash
npm install -g @faiscadev/flaglite-cli
```

Or use with npx (no install needed):

```bash
npx @faiscadev/flaglite-cli --help
```

## Quick Start

```bash
# Sign up (no email required!)
flaglite signup

# Create a flag
flaglite flags create dark-mode --enabled

# List your flags
flaglite flags list
```

## Commands

- `flaglite signup` - Create an account
- `flaglite login` - Log in to your account
- `flaglite logout` - Log out
- `flaglite whoami` - Show current user
- `flaglite flags list` - List all flags
- `flaglite flags create <name>` - Create a flag
- `flaglite flags toggle <name>` - Toggle a flag
- `flaglite flags delete <name>` - Delete a flag

## Alternative Install Methods

```bash
# macOS/Linux (recommended)
curl -fsSL https://flaglite.dev/install.sh | sh

# Rust
cargo install flaglite
```

## Links

- [Documentation](https://flaglite.dev/docs)
- [GitHub](https://github.com/faiscadev/flaglite)
- [API Reference](https://flaglite.dev/docs/api)

## License

MIT
