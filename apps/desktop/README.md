# SagensContact Desktop App

## Status: ✅ Implemented (Beta-Ready)

This directory contains the production-ready Tauri desktop application with SvelteKit UI.

**Version:** 0.1.0-alpha.3

## Quick Start

```bash
# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

**For comprehensive documentation, see [DESKTOP_README.md](./DESKTOP_README.md)**

## Architecture

```
apps/desktop/
├── src/                    # SvelteKit frontend
│   ├── routes/
│   │   ├── +page.svelte   # Main contacts list
│   │   ├── contacts/
│   │   ├── projects/
│   │   ├── notes/
│   │   └── settings/
│   └── lib/               # Shared components
├── src-tauri/             # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs        # Tauri commands
│   │   └── commands.rs    # IPC handlers
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── svelte.config.js
```

## Implemented Features

### Core Functionality
- ✅ Contact management (CRUD operations)
- ✅ Full-text search
- ✅ Groups and Projects
- ✅ Calendar events
- ✅ Notes
- ✅ Communication queue
- ✅ Attachments with file upload
- ✅ CSV import with native file picker
- ✅ Dashboard with statistics

### Desktop-Specific Features
- ✅ System tray integration
- ✅ Native file dialogs
- ✅ Desktop notifications
- ✅ Minimize to tray (Windows/Linux)
- ✅ Settings page with sync configuration
- ✅ Offline-first local SQLite database

## Tauri Commands (Implemented)

```rust
// src-tauri/src/commands.rs
#[tauri::command]
async fn list_contacts(limit: i64) -> Result<Vec<Contact>, String> {
    // Call local_store directly
}

#[tauri::command]
async fn import_csv(path: String) -> Result<(), String> {
    // Import CSV using local_store
}

#[tauri::command]
async fn queue_communication(attempt: CommunicationAttempt) -> Result<(), String> {
    // Queue communication
}
```

## UI Features (Planned)

- Contact list with search and filters
- Contact detail view with edit capabilities
- Note creation with attachment upload
- Communication queue viewer
- AI suggestion cards
- Share invite management
- Import wizard (CSV, vCard, SMS)
- Settings panel

## Integration

Desktop app will:
1. Use `local_store` crate directly (no network calls needed)
2. Optionally sync to `sync_service` when online
3. Store data in SQLite in user's app data directory
4. Handle offline-first workflows

## Development Commands (Future)

```bash
# Install dependencies
pnpm install

# Run in dev mode
pnpm tauri dev

# Build for production
pnpm tauri build

# Build for specific platform
pnpm tauri build --target x86_64-apple-darwin
pnpm tauri build --target x86_64-unknown-linux-gnu
```

## Why Tauri?

- Native performance with Rust backend
- Small bundle size (~3-5 MB vs Electron's ~100 MB)
- Direct access to Rust crates (no FFI needed)
- Cross-platform (macOS, Linux, Windows)
- Secure IPC between frontend and backend