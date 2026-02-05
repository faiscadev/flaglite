---
title: Rust SDK
description: Integrate FlagLite with Rust applications
---

The official FlagLite SDK for Rust.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
flaglite = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use flaglite::FlagLite;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = FlagLite::new("your-api-key");

    if client.evaluate("dark-mode", "user-123").await? {
        enable_dark_mode();
    }

    Ok(())
}
```

## Configuration

```rust
use flaglite::{FlagLite, Config};
use std::time::Duration;

let client = FlagLite::with_config(Config {
    api_key: "your-api-key".into(),
    base_url: "https://api.flaglite.dev".into(),
    timeout: Duration::from_secs(5),
    cache_enabled: true,
});
```

## Methods

### `evaluate(&self, flag: &str, user_id: &str) -> Result<bool>`

```rust
let enabled = client.evaluate("new-checkout", "user-123").await?;
```

### `evaluate_with_context(&self, flag: &str, ctx: Context) -> Result<bool>`

```rust
use flaglite::Context;

let ctx = Context::new("user-123")
    .with_email("user@example.com")
    .with_attr("plan", "pro");

let enabled = client.evaluate_with_context("new-checkout", ctx).await?;
```

### `get_all_flags(&self, user_id: &str) -> Result<HashMap<String, bool>>`

```rust
let flags = client.get_all_flags("user-123").await?;
// HashMap { "dark-mode" => true, "new-checkout" => false }
```

## Error Handling

```rust
use flaglite::Error;

match client.evaluate("dark-mode", "user-123").await {
    Ok(enabled) => {
        if enabled { enable_dark_mode(); }
    }
    Err(Error::FlagNotFound(_)) => {
        // Use default value
    }
    Err(e) => {
        eprintln!("FlagLite error: {}", e);
    }
}
```
