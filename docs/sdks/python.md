# Python SDK

## Installation

```bash
pip install flaglite
```

## Quick Start

```python
from flaglite import FlagLite

flags = FlagLite()  # Reads FLAGLITE_API_KEY from env

if await flags.enabled('new-checkout'):
    show_new_checkout()
```

## Percentage Rollouts

```python
if await flags.enabled('new-checkout', user_id='user-123'):
    show_new_checkout()
```

Same user always gets the same result.

## Synchronous Usage

```python
if flags.enabled_sync('new-checkout'):
    show_new_checkout()
```

## Configuration

```python
flags = FlagLite(
    api_key='ffl_env_xxxxx',     # Or use FLAGLITE_API_KEY env var
    base_url='https://...',       # For self-hosted
    cache_ttl=30.0,               # Seconds
    timeout=5.0,                  # HTTP timeout
)
```

## Context Manager

```python
async with FlagLite() as flags:
    if await flags.enabled('feature'):
        do_something()
```

## FastAPI Example

```python
from fastapi import FastAPI
from flaglite import FlagLite

app = FastAPI()
flags = FlagLite()

@app.on_event("shutdown")
async def shutdown():
    await flags.close()

@app.get("/checkout")
async def checkout(user_id: str):
    if await flags.enabled('new-checkout', user_id=user_id):
        return {"version": "new"}
    return {"version": "legacy"}
```

## Behavior

- **Caching:** 30-second TTL (configurable)
- **Fail closed:** Returns `False` on any error
- **Async/sync:** Both supported

## API

| Method | Description |
|--------|-------------|
| `await flags.enabled(key, user_id=None)` | Check flag (async) |
| `flags.enabled_sync(key, user_id=None)` | Check flag (sync) |
| `await flags.close()` | Close HTTP client |
| `await flags.clear_cache()` | Clear cache |

---

Full documentation: [SDK README](https://github.com/faiscadev/flaglite/tree/main/sdks/python)
