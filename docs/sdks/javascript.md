# JavaScript SDK

## Installation

```bash
npm install flaglite
```

## Quick Start

```javascript
import flags from 'flaglite'

// Set FLAGLITE_API_KEY environment variable, then:
if (await flags.enabled('new-checkout')) {
  showNewCheckout()
}
```

## Explicit Initialization

```javascript
import { FeatureFlags } from 'flaglite'

const flags = new FeatureFlags('ffl_env_xxxxx')

if (await flags.enabled('new-checkout')) {
  showNewCheckout()
}
```

## Percentage Rollouts

Pass a user ID for consistent results:

```javascript
if (await flags.enabled('new-checkout', { userId: user.id })) {
  showNewCheckout()
}
```

Same user always gets the same result.

## Configuration

```javascript
const flags = new FeatureFlags({
  apiKey: 'ffl_env_xxxxx',
  baseUrl: 'https://api.flaglite.dev/v1',
  cacheTtl: 30000,  // 30 seconds
  timeout: 5000,    // 5 seconds
})
```

## Browser (CDN)

```html
<script src="https://cdn.flaglite.dev/sdk.min.js"></script>
<script>
  const flags = new FlagLite('ffl_env_xxxxx')
  
  flags.enabled('new-checkout').then(enabled => {
    if (enabled) showNewCheckout()
  })
</script>
```

## Behavior

- **Caching:** 30-second TTL (configurable)
- **Fail closed:** Returns `false` on any error
- **Prefetch:** All flags fetched on init

## API

| Method | Description |
|--------|-------------|
| `flags.enabled(key, options?)` | Check if flag is enabled |
| `flags.clearCache()` | Clear local cache |
| `flags.waitForInit()` | Wait for prefetch |

---

Full documentation: [SDK README](https://github.com/faiscadev/flaglite/tree/main/sdks/js)
