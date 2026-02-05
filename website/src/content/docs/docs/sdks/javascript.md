---
title: JavaScript / TypeScript SDK
description: Integrate FlagLite with JavaScript and TypeScript applications
---

The official FlagLite SDK for JavaScript and TypeScript.

## Installation

```bash
npm install @faiscadev/flaglite
# or
yarn add @faiscadev/flaglite
# or
pnpm add @faiscadev/flaglite
```

## Quick Start

```javascript
import { FlagLite } from '@faiscadev/flaglite';

const client = new FlagLite({ 
  apiKey: 'your-api-key',
  // Optional: self-hosted URL
  baseUrl: 'https://api.flaglite.dev'
});

// Evaluate a flag
const enabled = await client.evaluate('dark-mode', { 
  userId: 'user-123' 
});

if (enabled) {
  enableDarkMode();
}
```

## Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `apiKey` | string | required | Your FlagLite API key |
| `baseUrl` | string | `https://api.flaglite.dev` | API endpoint URL |
| `timeout` | number | `5000` | Request timeout in ms |
| `cache` | boolean | `true` | Enable local caching |

## Methods

### `evaluate(flagName, context)`

Evaluate a feature flag for a user.

```typescript
const enabled = await client.evaluate('new-checkout', {
  userId: 'user-123',
  email: 'user@example.com',
  plan: 'pro'
});
```

### `getAllFlags(context)`

Get all flags for a user context.

```typescript
const flags = await client.getAllFlags({ userId: 'user-123' });
// { 'dark-mode': true, 'new-checkout': false, ... }
```

## TypeScript Support

Full TypeScript support with type definitions included:

```typescript
import { FlagLite, FlagContext, FlagValue } from '@faiscadev/flaglite';

interface MyContext extends FlagContext {
  userId: string;
  plan: 'free' | 'pro' | 'enterprise';
}

const client = new FlagLite<MyContext>({ apiKey: 'your-key' });
```
