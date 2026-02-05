# Go SDK

## Installation

```bash
go get github.com/faiscadev/flaglite-go
```

## Quick Start

```go
package main

import "github.com/faiscadev/flaglite-go"

func main() {
    client := flaglite.New()  // Reads FLAGLITE_API_KEY from env

    if client.Enabled("new-checkout") {
        showNewCheckout()
    }
}
```

## Percentage Rollouts

```go
if client.Enabled("new-checkout", flaglite.WithUserID(user.ID)) {
    showNewCheckout()
}
```

Same user always gets the same result.

## Configuration

```go
client := flaglite.New(
    flaglite.WithAPIKey("ffl_env_xxxxx"),
    flaglite.WithCacheTTL(1 * time.Minute),
    flaglite.WithTimeout(3 * time.Second),
    flaglite.WithBaseURL("https://my-flaglite.example.com/v1"),
)
```

## Error Handling

Default `Enabled()` fails closed (returns `false`). For explicit errors:

```go
enabled, err := client.EnabledWithError("my-flag")
if err != nil {
    log.Printf("Flag evaluation failed: %v", err)
}
```

## Context Support

```go
ctx, cancel := context.WithTimeout(ctx, 2*time.Second)
defer cancel()

enabled, err := client.EnabledWithContext(ctx, "my-flag")
```

## Behavior

- **Caching:** 30-second TTL (configurable)
- **Fail closed:** Returns `false` on any error
- **Thread-safe:** Safe for concurrent use

## API

| Method | Description |
|--------|-------------|
| `client.Enabled(key, opts...)` | Check flag (fails closed) |
| `client.EnabledWithError(key, opts...)` | Check flag with error |
| `client.EnabledWithContext(ctx, key, opts...)` | Check with context |
| `client.ClearCache()` | Clear cache |

---

Full documentation: [SDK README](https://github.com/faiscadev/flaglite/tree/main/sdks/go)
