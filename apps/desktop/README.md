# Desktop App (Tauri + SvelteKit)

## Status: Planned (Beta)

This directory will contain the Tauri desktop application shell with SvelteKit UI.

**For Alpha:** The web UI at `apps/web/` serves as the reference implementation.
The Tauri desktop wrapper will reuse the same SvelteKit components, including
the Communications screen with Email/SMS placeholder cards.

## Planned Setup

```bash
# Install Tauri CLI
cargo install tauri-cli

# Initialize Tauri project
npm create tauri-app@latest

# Choose:
# - Framework: SvelteKit
# - TypeScript: Yes
# - Package manager: pnpm
```

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

## Tauri Commands (Planned)

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