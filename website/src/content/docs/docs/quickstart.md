---
title: Quickstart
description: Get your first flag running in 30 seconds
---

Three commands. Thirty seconds. Let's go.

## 1. Clone and run

```bash
git clone https://github.com/faiscadev/flaglite && cd flaglite
docker compose up -d
```

## 2. Create your first flag

```bash
curl localhost:8080/v1/flags \
  -H "Content-Type: application/json" \
  -d '{"name": "dark-mode", "enabled": true}'
```

## 3. Use it in your app

import { Tabs, TabItem } from '@astrojs/starlight/components';

<Tabs>
  <TabItem label="JavaScript">
```javascript
import { FlagLite } from '@faiscadev/flaglite';

const client = new FlagLite({ apiKey: 'your-api-key' });

if (await client.evaluate('dark-mode', { userId: 'user-123' })) {
  enableDarkMode();
}
```
  </TabItem>
  <TabItem label="Python">
```python
from flaglite import FlagLite

client = FlagLite(api_key="your-api-key")

if client.evaluate("dark-mode", user_id="user-123"):
    enable_dark_mode()
```
  </TabItem>
  <TabItem label="Go">
```go
import "github.com/faiscadev/flaglite-go"

client := flaglite.New("your-api-key")

if enabled, _ := client.Evaluate("dark-mode", "user-123"); enabled {
    enableDarkMode()
}
```
  </TabItem>
  <TabItem label="Rust">
```rust
use flaglite::FlagLite;

let client = FlagLite::new("your-api-key");

if client.evaluate("dark-mode", "user-123").await? {
    enable_dark_mode();
}
```
  </TabItem>
</Tabs>

## Next steps

- [Installation options](/docs/installation) — Self-host or use cloud
- [SDK reference](/docs/sdks/javascript) — Full API documentation
- [API reference](/docs/api/overview) — REST endpoints
