# Communications Screen - Desktop Preview

## Overview

The Tauri desktop app will mirror the web UI's Communications screen exactly.
Until the Tauri implementation is complete, test the functionality using the
web UI at http://localhost:3001/communications.

## Planned Desktop Integration

The desktop app will:
1. Embed the same SvelteKit UI as the web app
2. Use Tauri commands for direct database access (no HTTP required)
3. Display the same Email/SMS placeholder cards
4. Show identical mock warnings and feedback

## Component Reuse

The desktop will import from `apps/web/src/routes/communications/+page.svelte`:

```typescript
// Desktop-specific Tauri commands (Beta)
#[tauri::command]
async fn queue_communication(attempt: CommunicationAttempt) -> Result<(), String> {
    // Direct call to local_store, no API needed
    let store = LocalStore::new("sqlite:./data/contacts.db").await?;
    let repo = CommunicationRepository::new(store.pool());
    repo.create(&attempt).await.map_err(|e| e.to_string())?;
    Ok(())
}
```

## Alpha Testing

**Test the Communications UI now:**

```bash
# Terminal 1: Start sync service
cd alpha
cargo run --release --bin sync_service

# Terminal 2: Start web UI
cd alpha/apps/web
pnpm install
pnpm dev

# Browser: Visit http://localhost:3001/communications
```

**What to test:**
- ✅ Select contact from dropdown
- ✅ Email form with recipient/subject/message
- ✅ SMS form with phone/message
- ✅ "MOCK" badges visible on cards and buttons
- ✅ Warning alerts at top of page
- ✅ Detailed mock feedback after submission
- ✅ Info card explaining alpha limitations

## Screenshots (Future)

Once Tauri is implemented, add screenshots showing:
- Desktop window with native chrome
- Communications screen rendered in desktop shell
- Native notifications for queued communications
- System tray integration

## Implementation Plan (Beta)

1. **Week 1**: Initialize Tauri project
2. **Week 2**: Configure SvelteKit adapter for Tauri
3. **Week 3**: Implement Tauri commands for local storage
4. **Week 4**: Test on macOS and Linux
5. **Week 5**: Add desktop-specific features (notifications, tray)

## Differences from Web

| Feature | Web (Alpha) | Desktop (Beta) |
|---------|-------------|----------------|
| Database access | HTTP API | Direct Tauri commands |
| Offline support | Limited | Full offline |
| Native notifications | Browser only | OS notifications |
| System tray | No | Yes |
| Auto-launch | No | Optional |
| Bundle size | N/A | ~5 MB |

## For Now

Use the web UI as a reference for how the desktop Communications screen will
look and behave. All logic, styling, and mock warnings will be identical.