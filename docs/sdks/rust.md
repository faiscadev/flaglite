# Rust SDK

## Installation

```toml
[dependencies]
flaglite = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Quick Start

```rust
use flaglite::FlagLite;

#[tokio::main]
async fn main() {
    let client = FlagLite::new().expect("FLAGLITE_API_KEY must be set");

    if client.enabled("new-checkout").await {
        show_new_checkout();
    }
}
```

## Percentage Rollouts

```rust
if client.enabled_for_user("new-checkout", "user-123").await {
    show_new_checkout();
}
```

Same user always gets the same result.

## Configuration

```rust
use std::time::Duration;

let client = FlagLite::builder()
    .api_key("ffl_env_xxxxx")
    .base_url("https://api.flaglite.dev/v1")
    .cache_ttl(Duration::from_secs(60))
    .timeout(Duration::from_secs(10))
    .build()?;
```

## Thread Safety

Share across tasks with `Arc`:

```rust
use std::sync::Arc;

let client = Arc::new(FlagLite::new()?);

let client_clone = Arc::clone(&client);
tokio::spawn(async move {
    if client_clone.enabled("feature").await {
        // ...
    }
});
```

## Behavior

- **Caching:** 30-second TTL (configurable)
- **Fail closed:** Returns `false` on any error
- **Thread-safe:** Safe for concurrent use

## API

| Method | Description |
|--------|-------------|
| `client.enabled(key)` | Check if flag is enabled |
| `client.enabled_for_user(key, user_id)` | Check with user ID |
| `client.clear_cache()` | Clear cache |

---

Full documentation: [SDK README](https://github.com/faiscadev/flaglite/tree/main/sdks/rust)
