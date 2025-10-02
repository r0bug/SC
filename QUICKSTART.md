# Quick Start Guide

## Prerequisites Installation

### 1. Install Rust (Required)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup default 1.75.0
```

Verify installation:
```bash
rustc --version  # Should show 1.75.0 or higher
cargo --version
```

### 2. Install Node.js (For future web/desktop apps)

**macOS:**
```bash
brew install node@20 pnpm
```

**Linux:**
```bash
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs
npm install -g pnpm
```

### 3. Install System Dependencies

**macOS:**
```bash
brew install sqlite
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get install -y libsqlite3-dev pkg-config libssl-dev
```

---

## Build and Run (5 minutes)

### Step 1: Navigate to Project

```bash
cd alpha
```

### Step 2: Build All Crates

```bash
cargo build --release
```

This will take 5-10 minutes on first build (downloads dependencies and compiles).

### Step 3: Create Data Directory

```bash
mkdir -p data
cp config/credentials.toml.example config/credentials.toml
```

### Step 4: Import Sample Data

```bash
./target/release/sagenscontact import --csv sample_data/contacts.csv
```

The import will show:
- Detected CSV columns and field mappings
- Preview of first 3 records
- Any parsing errors
- Confirmation prompt before writing to database

Type `y` when prompted to confirm the import.

### Step 5: List Contacts

```bash
./target/release/sagenscontact list
```

You should see 5 imported contacts.

---

## Run E2E Test

```bash
./scripts/cli_e2e_test.sh
```

This will run a full end-to-end test of the CLI functionality.

---

## Start Sync Service (Optional)

In a separate terminal:

```bash
cargo run --release --bin sync_service
```

Then test the API:

```bash
curl http://localhost:3000/health
curl http://localhost:3000/api/contacts
```

Should return: `OK` and contact list JSON.

## Start Web UI (Optional)

Requires sync service running on port 3000.

In a separate terminal:

```bash
cd apps/web
pnpm install
pnpm dev
```

Then visit: http://localhost:3001

Features:
- Contact list with search
- Contact detail pages with notes
- Communication forms (Email/SMS mock)
- AI suggestions (mock)

## Start Background Worker (Optional)

Processes queued communications every 30 seconds (mocked).

In a separate terminal:

```bash
cargo run --release --bin worker
```

---

## Common Commands

### Import Data
```bash
# CSV (with field mapping preview)
./target/release/sagenscontact import --csv sample_data/contacts.csv

# vCard (placeholder - not yet implemented)
./target/release/sagenscontact import --vcard sample_data/contacts.vcf

# SMS (placeholder - not yet implemented)
./target/release/sagenscontact import --sms sample_data/sms_export.json
```

### Manage Contacts
```bash
# List all contacts
./target/release/sagenscontact list

# Search contacts
./target/release/sagenscontact search "john"

# Add new contact
./target/release/sagenscontact add "Alice" "Wonder" --email alice@example.com
```

### Create Notes
```bash
# Get contact ID from list command, then:
./target/release/sagenscontact note <CONTACT_ID> "Meeting Notes" "Discussed project timeline"
```

### Queue Communications (Mock)
```bash
# Email
./target/release/sagenscontact communicate <CONTACT_ID> email "Hello from CLI"

# SMS
./target/release/sagenscontact communicate <CONTACT_ID> sms "Quick message"
```

### Share Entities
```bash
./target/release/sagenscontact share contact <CONTACT_ID> recipient@example.com
```

### AI Suggestions (Mock)
```bash
./target/release/sagenscontact suggest <CONTACT_ID>
```

---

## Troubleshooting

### "cargo: command not found"

Rust is not installed. Follow step 1 above.

### "error: linker 'cc' not found"

Install build tools:

**macOS:**
```bash
xcode-select --install
```

**Linux:**
```bash
sudo apt-get install build-essential
```

### "cannot find -lsqlite3"

Install SQLite development headers (see Prerequisites step 3).

### "No such file or directory: data/contacts.db"

Create the data directory:
```bash
mkdir -p data
```

The database will be created automatically on first run.

### Build takes too long / runs out of memory

Use a smaller build target:
```bash
cargo build --release --bin sagenscontact
```

Or build without optimizations (faster but slower runtime):
```bash
cargo build --bin sagenscontact
./target/debug/sagenscontact list
```

---

## Next Steps

1. ✅ Complete Quick Start above
2. 📖 Read [README.md](README.md) for full feature overview
3. 🏗️ Review [ARCHITECTURE.md](ARCHITECTURE.md) for system design
4. 🔒 Review [SECURITY_NOTES.md](SECURITY_NOTES.md) for alpha limitations
5. 🧪 Read [TESTING.md](TESTING.md) for test strategy
6. 🎬 Follow [WORKFLOW_DEMOS.md](WORKFLOW_DEMOS.md) for full demo scenario

---

## Development Mode

For development (faster rebuilds):

```bash
# Use debug builds
cargo build --bin sagenscontact
cargo run --bin sync_service

# Watch for changes and auto-rebuild (install cargo-watch first)
cargo install cargo-watch
cargo watch -x 'build --bin sagenscontact'
```

---

## Project Status

✅ **Completed (Alpha):**
- Core domain entities
- SQLite local storage with foreign key enforcement
- CLI with CSV import (field mapping + preview), search, CRUD
- Mock AI suggestions (Segmind)
- Mock communication adapters (Email/SMS/Social)
- Background worker binary for queue processing
- Sync service API with live database integration
- Web UI with contacts, notes, and communication screens
- Type-safe API client library
- Sample data (CSV/vCard/SMS)
- Documentation (README, Architecture, Security, Testing, Workflow)

⏳ **TODO (Beta):**
- Desktop app (Tauri + SvelteKit)
- Complete web UI (projects detail, sharing, settings)
- vCard/SMS import parsing
- Secure credential vault
- Real external service integrations
- Authentication/authorization
- Offline-first sync with conflict resolution
- E2E tests with Playwright
- Production deployment guides

---

## Support

For issues or questions:
- Check [TESTING.md](TESTING.md) for known issues
- Review [ARCHITECTURE.md](ARCHITECTURE.md) for design decisions
- Read [SECURITY_NOTES.md](SECURITY_NOTES.md) for alpha limitations

This is an alpha release. Expect rough edges and missing features!