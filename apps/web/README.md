# Web App (SvelteKit SSR)

## Status: TODO (Beta)

This directory will contain the SvelteKit SSR web application for mobile and browser access.

## Planned Setup

```bash
# Create SvelteKit project
npm create svelte@latest

# Choose:
# - Template: Skeleton project
# - TypeScript: Yes
# - ESLint, Prettier: Yes
# - Playwright: Yes (for E2E tests)
```

## Architecture

```
apps/web/
├── src/
│   ├── routes/
│   │   ├── +page.svelte        # Home / contacts list
│   │   ├── +layout.svelte      # App shell
│   │   ├── contacts/
│   │   │   ├── +page.svelte    # Contact list
│   │   │   └── [id]/
│   │   │       └── +page.svelte # Contact detail
│   │   ├── projects/
│   │   ├── notes/
│   │   ├── share/
│   │   │   └── accept/[token]/ # Accept share invite
│   │   ├── login/
│   │   └── api/                # API routes (proxy to sync_service)
│   ├── lib/
│   │   ├── components/         # UI components
│   │   ├── stores/             # Svelte stores
│   │   ├── api/                # API client
│   │   └── types/              # TypeScript types
│   └── app.html
├── static/                     # Static assets
├── tests/                      # Playwright E2E tests
├── package.json
└── svelte.config.js
```

## API Integration

Web app communicates with `sync_service`:

```typescript
// src/lib/api/contacts.ts
export async function listContacts(limit = 50): Promise<Contact[]> {
  const response = await fetch('http://localhost:3000/api/contacts?limit=' + limit);
  return response.json();
}

export async function searchContacts(query: string): Promise<Contact[]> {
  const response = await fetch('http://localhost:3000/api/contacts/search', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ query }),
  });
  return response.json();
}
```

## WebSocket for Real-time Updates

```typescript
// src/lib/ws.ts
const ws = new WebSocket('ws://localhost:3000/ws');

ws.onmessage = (event) => {
  const update = JSON.parse(event.data);
  // Update Svelte stores based on message type
};
```

## Responsive Design

Mobile-first CSS with breakpoints:
- Mobile: 320px - 768px
- Tablet: 768px - 1024px
- Desktop: 1024px+

## Authentication (Beta)

```typescript
// src/routes/login/+page.svelte
// JWT-based auth with httpOnly cookies
// Protect routes with +page.server.ts guards
```

## Features (Planned)

### Public Routes
- `/login` - Authentication
- `/share/accept/[token]` - Accept share invite (no login required)

### Protected Routes
- `/` - Dashboard
- `/contacts` - Contact list with search
- `/contacts/[id]` - Contact detail
- `/contacts/import` - Import wizard
- `/projects` - Project list
- `/notes` - Notes list
- `/communication` - Communication queue
- `/settings` - User settings

## Development Commands (Future)

```bash
# Install dependencies
pnpm install

# Run dev server
pnpm dev

# Build for production
pnpm build

# Preview production build
pnpm preview

# Run E2E tests
pnpm test:e2e
```

## Deployment

### Self-hosted
```bash
# Build adapter-node
pnpm build
node build/index.js
```

### Docker
```dockerfile
FROM node:20-alpine
WORKDIR /app
COPY build/ ./
EXPOSE 3001
CMD ["node", "index.js"]
```

### Vercel/Netlify
Use respective adapters:
- `@sveltejs/adapter-vercel`
- `@sveltejs/adapter-netlify`

## PWA Support (Future)

Add service worker for offline capabilities:
- Cache static assets
- Queue API calls when offline
- Background sync when online

## Playwright E2E Tests (Planned)

```typescript
// tests/import-workflow.spec.ts
import { test, expect } from '@playwright/test';

test('import and share workflow', async ({ page }) => {
  await page.goto('/login');
  await page.fill('[name="email"]', 'test@example.com');
  await page.fill('[name="password"]', 'password');
  await page.click('button[type="submit"]');

  await page.goto('/contacts/import');
  await page.setInputFiles('[type="file"]', 'sample_data/contacts.csv');
  await page.click('button:has-text("Import")');

  await expect(page.locator('text=John Doe')).toBeVisible();

  // Continue with share workflow...
});
```