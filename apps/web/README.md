# Web App (SvelteKit)

## Status: Implemented

Full-featured SvelteKit web application for contact management.

## Quick Start

```bash
# Install dependencies
pnpm install

# Run dev server (requires sync_service on port 3000)
pnpm dev

# Build for production
pnpm build

# Preview production build
pnpm preview

# Run Playwright tests
pnpm test
```

Visit http://localhost:3001 after starting the dev server.

## Architecture

```
apps/web/
├── src/
│   ├── routes/
│   │   ├── +page.svelte           # Home / redirect to dashboard
│   │   ├── +layout.svelte         # App shell with navigation
│   │   ├── auth/
│   │   │   └── login/             # Login page
│   │   ├── dashboard/             # Dashboard with statistics
│   │   ├── contacts/
│   │   │   ├── +page.svelte       # Contact list with search
│   │   │   ├── new/               # Create contact
│   │   │   └── [id]/              # Contact detail
│   │   ├── projects/              # Project management
│   │   ├── notes/                 # Notes list
│   │   ├── communications/        # Communication queue
│   │   ├── import/                # Import wizard
│   │   └── settings/              # User settings
│   ├── lib/
│   │   ├── components/            # UI components
│   │   ├── stores/                # Svelte stores
│   │   ├── api/
│   │   │   └── client.ts          # Type-safe API client
│   │   └── types/                 # TypeScript types
│   └── app.html
├── tests/
│   ├── web-flows.test.ts          # Core workflow tests
│   └── api-integration.test.ts    # API integration tests
├── static/                        # Static assets
├── package.json
└── svelte.config.js
```

## Features

### Implemented

- **Contact Management**: List, search, create, edit, delete contacts
- **Contact Detail**: Full contact info with notes and AI suggestions
- **Import**: CSV, vCard, and social media exports (LinkedIn, Twitter, Facebook, Instagram)
- **Communications**: Queue email and SMS messages
- **Projects**: Group contacts by project
- **Notes**: Attach notes to contacts and projects
- **Dashboard**: Statistics and recent activity
- **Settings**: Configuration options
- **Search**: Full-text search across contacts
- **Responsive**: Mobile, tablet, and desktop layouts

### API Integration

The web app communicates with `sync_service` on port 3000:

```typescript
// src/lib/api/client.ts
const API_BASE = 'http://localhost:3000/api';

export async function listContacts(limit = 50): Promise<Contact[]> {
  const response = await fetch(`${API_BASE}/contacts?limit=${limit}`);
  return response.json();
}

export async function searchContacts(query: string): Promise<Contact[]> {
  const response = await fetch(`${API_BASE}/contacts/search`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ query }),
  });
  return response.json();
}
```

### WebSocket Real-time Updates

```typescript
// src/lib/ws.ts
const ws = new WebSocket('ws://localhost:3000/ws');

ws.onmessage = (event) => {
  const update = JSON.parse(event.data);
  // Update Svelte stores based on message type
};
```

## Playwright E2E Tests

Located in `tests/`:

### web-flows.test.ts
- Authentication flows
- Contact CRUD operations
- Search functionality
- Notes management

### api-integration.test.ts
- API health checks
- Contact import flow
- Communication tabs
- Attachment management
- Performance benchmarks
- Responsive design tests
- Accessibility tests

Run tests:
```bash
pnpm test        # Run all tests
pnpm test:ui     # Interactive mode
```

## Development

### Environment Variables

Create `.env` file:
```bash
PUBLIC_API_URL=http://localhost:3000
```

### Building

```bash
# Development
pnpm dev

# Production build
pnpm build

# Preview production
pnpm preview
```

## Deployment

### Self-hosted (Node adapter)

```bash
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

### Static Hosting

Configure static adapter in `svelte.config.js`:
```javascript
import adapter from '@sveltejs/adapter-static';

export default {
  kit: {
    adapter: adapter({
      pages: 'build',
      assets: 'build',
      fallback: 'index.html'
    })
  }
};
```

## Responsive Design

Mobile-first CSS with breakpoints:
- Mobile: 320px - 768px
- Tablet: 768px - 1024px
- Desktop: 1024px+

## Authentication

JWT-based authentication with httpOnly cookies:
- Login via `/auth/login`
- Protected routes check session
- Logout clears session cookie
