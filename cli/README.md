# FlagLite CLI

Command-line interface for managing feature flags with FlagLite.

## Installation

```bash
cargo install --path .
```

## Quick Start

```bash
# Authenticate
flaglite login

# List your projects
flaglite projects list

# Select a project
flaglite projects use my-project

# List flags
flaglite flags list

# Create a flag
flaglite flags create dark-mode --description "Enable dark mode"

# Toggle a flag
flaglite flags toggle dark-mode

# Get flag details
flaglite flags get dark-mode
```

## Commands

### Authentication

```bash
flaglite login              # Authenticate with FlagLite
flaglite logout             # Clear stored authentication
flaglite whoami             # Show current user
```

### Projects

```bash
flaglite projects list      # List all projects
flaglite projects create    # Create new project
flaglite projects use <id>  # Set default project
```

### Flags

```bash
flaglite flags list         # List all flags in current project
flaglite flags create       # Create a flag
flaglite flags get <key>    # Get flag details
flaglite flags toggle <key> # Toggle a flag
flaglite flags delete <key> # Delete a flag
```

### Environments

```bash
flaglite envs list          # List environments
flaglite envs use <name>    # Set default environment
```

### Configuration

```bash
flaglite config             # Show current configuration
flaglite config --path      # Show config file path
```

## Global Options

All commands support these options:

| Option | Environment Variable | Description |
|--------|---------------------|-------------|
| `--format <pretty\|json>` | - | Output format (default: pretty) |
| `--api-url <URL>` | `FLAGLITE_API_URL` | API base URL |
| `-p, --project <ID>` | `FLAGLITE_PROJECT` | Project ID |
| `-e, --env <NAME>` | `FLAGLITE_ENV` | Environment name |

## Configuration File

The CLI stores configuration in:
- **macOS**: `~/Library/Application Support/flaglite/config.toml`
- **Linux**: `~/.config/flaglite/config.toml`
- **Windows**: `%APPDATA%\flaglite\config.toml`

Example config:

```toml
api_url = "https://api.flaglite.dev"
token = "your-auth-token"
project_id = "project-uuid"
environment = "development"
```

## JSON Output

For scripting, use `--format json`:

```bash
flaglite flags list --format json | jq '.[].key'
```

## Examples

### Create a flag with options

```bash
flaglite flags create my-feature \
  --name "My Feature" \
  --description "A cool new feature" \
  --flag-type boolean \
  --enabled
```

### Use with different environments

```bash
# Check flag in production
flaglite flags get dark-mode -e production

# Toggle in staging
flaglite flags toggle dark-mode -e staging
```

### Override API URL

```bash
# Use local development server
flaglite --api-url http://localhost:3000 projects list

# Or via environment variable
export FLAGLITE_API_URL=http://localhost:3000
flaglite projects list
```
