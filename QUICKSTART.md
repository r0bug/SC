# Quick Start Guide

## Prerequisites Installation

### 1. Install Rust (Required)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup default stable
```

Verify installation:
```bash
rustc --version  # Should show 1.83.0 or higher
cargo --version
```

### 2. Install Node.js (For Web UI)

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
- Import page with format selection
- Communication forms (Email/SMS)
- AI suggestions

## Start Background Worker (Optional)

Processes queued communications every 30 seconds.

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

# vCard
./target/release/sagenscontact import --vcard contacts.vcf

# Social media exports (LinkedIn, Twitter, Facebook, Instagram)
./target/release/sagenscontact import --json linkedin_connections.json
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

### Queue Communications
```bash
# Email (requires SMTP config for real sending)
./target/release/sagenscontact communicate <CONTACT_ID> email "Hello from CLI"

# SMS (requires Twilio config for real sending)
./target/release/sagenscontact communicate <CONTACT_ID> sms "Quick message"
```

### Share Entities
```bash
./target/release/sagenscontact share contact <CONTACT_ID> recipient@example.com
```

### AI Suggestions
```bash
# Requires SEGMIND_API_KEY for real AI, otherwise returns mock suggestions
./target/release/sagenscontact suggest <CONTACT_ID>
```

---

## Service Configuration

Enable real services by setting environment variables:

```bash
# Email (SMTP)
export SMTP_HOST=smtp.example.com
export SMTP_USER=user@example.com
export SMTP_PASSWORD=password
export SMTP_FROM=noreply@example.com

# SMS (Twilio)
export TWILIO_ACCOUNT_SID=your-sid
export TWILIO_AUTH_TOKEN=your-token
export TWILIO_PHONE_NUMBER=+1234567890

# AI (Segmind)
export SEGMIND_API_KEY=your-api-key

# Virus Scanning (ClamAV)
export VIRUS_SCANNER_ENABLED=true
export CLAMAV_SOCKET_PATH=/var/run/clamav/clamd.sock
```

Without these variables, services run in fallback mode (logging only).

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

1. Read [README.md](README.md) for full feature overview
2. Review [ARCHITECTURE.md](ARCHITECTURE.md) for system design
3. Review [SECURITY_NOTES.md](SECURITY_NOTES.md) for security considerations
4. Read [TESTING.md](TESTING.md) for test strategy
5. Follow [WORKFLOW_DEMOS.md](WORKFLOW_DEMOS.md) for demo scenarios

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

## Support

For issues or questions:
- Check [TESTING.md](TESTING.md) for known issues
- Review [ARCHITECTURE.md](ARCHITECTURE.md) for design decisions
- Read [SECURITY_NOTES.md](SECURITY_NOTES.md) for security considerations
