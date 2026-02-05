---
title: Go SDK
description: Integrate FlagLite with Go applications
---

The official FlagLite SDK for Go.

## Installation

```bash
go get github.com/faiscadev/flaglite-go
```

## Quick Start

```go
package main

import (
    "github.com/faiscadev/flaglite-go"
)

func main() {
    client := flaglite.New("your-api-key")

    enabled, err := client.Evaluate("dark-mode", "user-123")
    if err != nil {
        log.Fatal(err)
    }

    if enabled {
        enableDarkMode()
    }
}
```

## Configuration

```go
client := flaglite.NewWithConfig(flaglite.Config{
    APIKey:   "your-api-key",
    BaseURL:  "https://api.flaglite.dev",
    Timeout:  5 * time.Second,
    Cache:    true,
})
```

## Methods

### `Evaluate(flagName, userID string, opts ...Option) (bool, error)`

```go
enabled, err := client.Evaluate("new-checkout", "user-123",
    flaglite.WithEmail("user@example.com"),
    flaglite.WithAttr("plan", "pro"),
)
```

### `GetAllFlags(userID string) (map[string]bool, error)`

```go
flags, err := client.GetAllFlags("user-123")
// map[string]bool{"dark-mode": true, "new-checkout": false}
```

## Context Support

```go
ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
defer cancel()

enabled, err := client.EvaluateContext(ctx, "dark-mode", "user-123")
```

## Error Handling

```go
enabled, err := client.Evaluate("dark-mode", "user-123")
if errors.Is(err, flaglite.ErrFlagNotFound) {
    // Flag doesn't exist, use default
    enabled = false
} else if err != nil {
    log.Printf("FlagLite error: %v", err)
    enabled = false // Safe default
}
```
