# SagensContact Desktop Application

A production-ready Tauri desktop application for SagensContact contact management system.

## Version

**v0.1.0-alpha.3** - Beta-ready desktop application

## Overview

The SagensContact Desktop App is a native desktop application built with:
- **Backend**: Tauri + Rust (using existing SagensContact crates)
- **Frontend**: SvelteKit + TypeScript
- **Database**: SQLite (local storage)
- **Platform Support**: macOS and Linux

## Features

### Core Functionality
✅ **Contact Management**
- View, create, edit, and delete contacts
- Full-text search across contacts
- Contact detail pages with notes and metadata
- Tag management

✅ **Native Desktop Integration**
- System tray support (minimize to tray)
- Native file dialogs for CSV import
- Native notifications for events
- OS-specific window management

✅ **Data Management**
- Local SQLite database
- CSV import with native file picker
- Offline-first architecture
- Optional sync with remote server

✅ **Settings & Configuration**
- Sync server configuration
- Notification preferences
- Data directory access
- Online/offline status indicator

### Desktop-Specific Features

#### System Tray
- Minimize to tray instead of closing (Windows/Linux)
- System tray menu with Show/Hide/Quit options
- Left-click to show window

#### Native Dialogs
- File picker for CSV import
- Save dialogs for export
- Native notifications

#### Data Storage
- Database stored in OS-appropriate app data directory:
  - macOS: `~/Library/Application Support/com.sagenscontact.app/`
  - Linux: `~/.local/share/com.sagenscontact.app/`

## Project Structure

```
apps/desktop/
├── src/                          # SvelteKit frontend
│   ├── routes/                   # Page routes
│   │   ├── +layout.svelte        # App layout with navigation
│   │   ├── +page.svelte          # Dashboard
│   │   ├── contacts/             # Contact management
│   │   ├── settings/             # Settings page
│   │   └── import/               # Import functionality
│   ├── lib/                      # Shared code
│   │   ├── api/                  # Tauri API wrapper
│   │   │   ├── tauri-api.ts     # Client API
│   │   │   └── types.ts          # TypeScript types
│   │   └── styles/               # Global styles
│   └── app.css                   # Main stylesheet
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs               # App entry + system tray
│   │   └── commands.rs           # Tauri commands
│   ├── Cargo.toml                # Rust dependencies
│   ├── tauri.conf.json           # Tauri configuration
│   └── icons/                    # App icons (TODO)
├── package.json                  # Node dependencies
└── svelte.config.js              # SvelteKit config
```

## Installation & Setup

### Prerequisites

**System Requirements:**
- Node.js 18+ and pnpm
- Rust 1.83+
- Tauri CLI

**Install Tauri prerequisites:**

```bash
# macOS
brew install gcc

# Ubuntu/Debian
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

### Development Setup

```bash
# 1. Navigate to desktop app
cd alpha/apps/desktop

# 2. Install dependencies
pnpm install

# 3. Run in development mode
pnpm tauri dev
```

### Building for Production

```bash
# Build for current platform
pnpm tauri build

# Output locations:
# macOS DMG: src-tauri/target/release/bundle/dmg/
# Linux DEB: src-tauri/target/release/bundle/deb/
# Linux AppImage: src-tauri/target/release/bundle/appimage/
```

## Architecture

### Frontend (SvelteKit)

The frontend reuses components and patterns from the web UI (`apps/web/`) but adapted for desktop:

**Key Differences from Web UI:**
- Uses Tauri IPC instead of HTTP API
- No authentication flow (single-user desktop app)
- Native file dialogs instead of web file inputs
- System tray integration
- Desktop-specific settings

### Backend (Tauri + Rust)

The Tauri backend provides:
- Direct access to SagensContact Rust crates (`local_store`, `core_domain`, etc.)
- Native OS integration
- System tray management
- File system access for attachments

**Available Tauri Commands:**
- Contact CRUD: `get_contacts`, `create_contact`, `update_contact`, `delete_contact`, `search_contacts`
- Groups: `get_groups`, `create_group`
- Projects: `get_projects`, `create_project`
- Calendar: `get_calendar_events`, `create_event`
- Notes: `get_notes`, `create_note`
- Communications: `queue_communication`
- Attachments: `get_attachments`, `upload_attachment`, `download_attachment`, `delete_attachment`
- Import: `import_csv`
- Dashboard: `get_dashboard`
- Settings: `get_settings`, `update_settings`
- Sync: `sync_with_server`, `check_online`
- Utility: `open_data_directory`

### Database

- **Type**: SQLite
- **Location**: OS app data directory (`sagenscontact.db`)
- **Migrations**: Embedded in `local_store` crate, run automatically on startup
- **Schema**: Same as web version (see `crates/local_store/migrations/`)

## Usage

### Running the App

```bash
# Development mode (hot reload)
pnpm tauri dev

# Or install the built package
sudo dpkg -i src-tauri/target/release/bundle/deb/*.deb  # Linux
# Or open the DMG and drag to Applications                # macOS
```

### Importing Contacts

1. Navigate to **Import** page
2. Click "Select CSV File"
3. Choose your CSV file (see `alpha/sample_data/contacts.csv` for format)
4. Contacts will be imported automatically

**CSV Format:**
```
first_name,last_name,email,phone,organization,title
John,Doe,john@example.com,555-0100,Acme Corp,CEO
```

### Settings

Access via **Settings** page:
- **Sync URL**: Configure remote sync server endpoint
- **Enable Sync**: Toggle synchronization
- **Notifications**: Enable/disable desktop notifications
- **Data Directory**: Open app data folder

### System Tray

- **Windows/Linux**: Closing the window minimizes to tray
- **macOS**: Closing the window quits the app (standard macOS behavior)
- **Tray Menu**: Right-click tray icon for Show/Hide/Quit

## Configuration

### Tauri Configuration (`src-tauri/tauri.conf.json`)

Key settings:
```json
{
  "bundle": {
    "identifier": "com.sagenscontact.app",
    "targets": ["deb", "appimage", "dmg"]
  },
  "systemTray": {
    "iconPath": "icons/icon.png",
    "menuOnLeftClick": false
  },
  "windows": [{
    "title": "SagensContact",
    "width": 1200,
    "height": 800,
    "minWidth": 900,
    "minHeight": 600
  }]
}
```

### Package Configuration (`package.json`)

Scripts:
- `pnpm dev` - Start Vite dev server
- `pnpm build` - Build for production
- `pnpm tauri dev` - Run Tauri in development mode
- `pnpm tauri build` - Build desktop app bundle

## Known Issues & Limitations

### Alpha Limitations
- [ ] No app icons configured (defaults used)
- [ ] No sync service auto-start (manual start required)
- [ ] No keychain integration for credentials (planned for beta)
- [ ] No auto-update mechanism
- [ ] No WebSocket real-time sync (HTTP-based sync only)

### Platform Notes

**macOS:**
- Minimum version: 10.15 (Catalina)
- First launch may show security warning (unsigned app)
- To run: Right-click → Open → Confirm

**Linux:**
- Tested on Ubuntu 20.04+
- AppIndicator library required for system tray
- May need `libwebkit2gtk-4.0-37` on some distributions

## Testing

### Manual Testing Checklist

✅ **Core Functionality**
- [ ] Launch app successfully
- [ ] View contacts list
- [ ] Create new contact
- [ ] Edit contact
- [ ] Delete contact
- [ ] Search contacts
- [ ] Import CSV file

✅ **Desktop Features**
- [ ] System tray icon appears
- [ ] Minimize to tray works (Windows/Linux)
- [ ] Show from tray works
- [ ] Notifications display
- [ ] Open data directory works
- [ ] Settings persist across restarts

✅ **Navigation**
- [ ] Dashboard loads
- [ ] All sidebar links work
- [ ] Contact detail page loads
- [ ] Settings page loads
- [ ] Import page loads

### Automated Testing

```bash
# Run Rust tests
cd src-tauri
cargo test

# Frontend tests (TODO)
pnpm test
```

## Troubleshooting

### Build Errors

**"cargo metadata command exited with a non zero exit code"**
- Solution: Desktop app excluded from workspace in `alpha/Cargo.toml`

**"vitePreprocess not found"**
- Solution: Import from `@sveltejs/vite-plugin-svelte`, not `@sveltejs/kit/vite`

**"Icon not found"**
- Solution: Icons need to be added to `src-tauri/icons/`
- Generate with: `pnpm tauri icon path/to/icon.png`

### Runtime Issues

**"Database locked"**
- Only one instance can run at a time
- Check for background processes: `ps aux | grep sagenscontact`

**"Sync failed"**
- Ensure sync_service is running: `cd alpha && cargo run --bin sync_service`
- Check sync URL in Settings matches service address

## Future Enhancements (Beta+)

### Planned Features
- [ ] OS keychain integration for secure credential storage
- [ ] Auto-start sync_service on app launch
- [ ] WebSocket support for real-time updates
- [ ] Auto-update mechanism
- [ ] Dark mode support
- [ ] Custom keyboard shortcuts
- [ ] Multi-window support
- [ ] Export functionality (CSV, vCard)
- [ ] Backup/restore database

### Bundle Improvements
- [ ] Proper app icons
- [ ] Code signing (macOS/Windows)
- [ ] Auto-update with Tauri updater
- [ ] Bundle sync_service binary with app
- [ ] Windows installer (MSI)

## Development

### Adding New Tauri Commands

1. **Define command in `src-tauri/src/commands.rs`:**
```rust
#[tauri::command]
pub async fn my_command(arg: String) -> Result<String, String> {
    Ok(format!("Hello {}", arg))
}
```

2. **Register in `src-tauri/src/main.rs`:**
```rust
.invoke_handler(tauri::generate_handler![
    my_command,
    // ... other commands
])
```

3. **Call from frontend:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';
const result = await invoke('my_command', { arg: 'World' });
```

### Adding New Routes

1. Create route file: `src/routes/myroute/+page.svelte`
2. Add navigation link in `src/routes/+layout.svelte`
3. Use `tauriApi` for backend communication

## Contributing

See main `alpha/README.md` for contribution guidelines.

## License

MIT

## Support

- **Issues**: https://github.com/your-org/sagenscontact/issues
- **Docs**: `alpha/ARCHITECTURE.md`, `alpha/TESTING.md`
- **Sample Data**: `alpha/sample_data/`

---

**Status**: ✅ Beta-ready - Core functionality complete, desktop features implemented
