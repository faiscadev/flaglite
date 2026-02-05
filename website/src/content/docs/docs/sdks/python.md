---
title: Python SDK
description: Integrate FlagLite with Python applications
---

The official FlagLite SDK for Python.

## Installation

```bash
pip install flaglite
```

## Quick Start

```python
from flaglite import FlagLite

client = FlagLite(api_key="your-api-key")

# Evaluate a flag
if client.evaluate("dark-mode", user_id="user-123"):
    enable_dark_mode()
```

## Configuration

```python
from flaglite import FlagLite

client = FlagLite(
    api_key="your-api-key",
    base_url="https://api.flaglite.dev",  # Optional
    timeout=5.0,  # seconds
    cache_enabled=True
)
```

## Methods

### `evaluate(flag_name, **context)`

```python
enabled = client.evaluate(
    "new-checkout",
    user_id="user-123",
    email="user@example.com",
    plan="pro"
)
```

### `get_all_flags(**context)`

```python
flags = client.get_all_flags(user_id="user-123")
# {'dark-mode': True, 'new-checkout': False, ...}
```

## Async Support

```python
from flaglite import AsyncFlagLite

client = AsyncFlagLite(api_key="your-api-key")

enabled = await client.evaluate("dark-mode", user_id="user-123")
```

## Django Integration

```python
# settings.py
FLAGLITE_API_KEY = "your-api-key"

# views.py
from flaglite.django import get_flag

def my_view(request):
    if get_flag("new-feature", request.user):
        return render(request, "new_feature.html")
    return render(request, "old_feature.html")
```
