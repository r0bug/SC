# SagensContact Alpha

Portable contact manager for macOS/Linux desktops with responsive web interface.

## ⚠️ Alpha Release Notice

This is an ALPHA release with the following characteristics:
- Uses placeholder credentials by default (configure the encrypted vault for real secrets)
- Mock implementations for external services (Segmind AI, SMS, email, social)
- Secure vault support is available, but plaintext files remain enabled for development setups
- Limited production readiness

## Features

- Contact, Tag, Project, Calendar Event, and Note management
- CSV/vCard/SMS import
- **Attachment Management** - Upload, scan, and manage files across all entities
- **AI-powered Suggestions** - Configurable Segmind integration with caching & retry
- **Communication Queue** with Email/SMS placeholders (see below)
- **Search History & Suggestions** - Track searches with privacy mode, AI-powered recommendations
- Sharing with per-entity ACL
- CLI, Desktop (Tauri), and Web (Svelte) interfaces
- SQLite local storage with optional Postgres sync
- WebSocket real-time notifications

### 📧📱 Communication Placeholders (Alpha)

**All Email and SMS sends are MOCKED in this release.** The project includes:

- **Web UI**: Explicit placeholder forms at `http://localhost:3001/communications`
  - Email card with recipient/subject/message fields
  - SMS card with phone/message fields
  - Clear "MOCK" badges and warnings throughout
  - Submissions log to database but send nothing

- **CLI**: Enhanced feedback showing mock status
  ```bash
  ./target/release/sagenscontact communicate <contact_id> email "Test"
  # Prints: ⚠️ [MOCK] This is a SIMULATED communication - NO actual EMAIL will be sent!
  ```

- **Purpose**: Test communication workflows without real SMTP/SMS credentials
- **Beta**: Real integrations with SMTP, Twilio, and social platforms

## Prerequisites

**For Backend (Required):**
- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- SQLite 3.44+ (usually pre-installed on macOS/Linux)
- Build tools (gcc/clang)

**For Web UI (Optional):**
- Node.js 20 LTS + pnpm 8+
- Run `cd apps/web && pnpm install && pnpm dev`
- Access at http://localhost:3001

**Future:**
- PostgreSQL 15+ for production sync service
- MinIO for attachment storage in development

## Quick Start

See **[QUICKSTART.md](QUICKSTART.md)** for detailed setup instructions.

### TL;DR

```bash
# 1. Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Build
cd alpha
cargo build --release

# 3. Setup
mkdir -p data
cp config/credentials.toml.example config/credentials.toml

# 4. Import sample data
./target/release/sagenscontact import --csv sample_data/contacts.csv

# 5. Use CLI
./target/release/sagenscontact list
./target/release/sagenscontact search "john"

# 6. (Optional) Run sync service
cargo run --release --bin sync_service
# Then: curl http://localhost:3000/api/contacts

# 7. (Optional) Run web UI (requires sync service)
cd apps/web
pnpm install
pnpm dev
# Then: Visit http://localhost:3001

# 8. (Optional) Run background worker for communication queue
cargo run --release --bin worker
# Processes pending emails/SMS every 30 seconds (mocked)

# 9. Run E2E tests
./scripts/cli_e2e_test.sh
```

### Import Workflow

The CLI import now includes field mapping and preview:

```bash
./target/release/sagenscontact import --csv sample_data/contacts.csv

# Shows:
# - Detected columns and auto-mapped fields
# - Preview of first 3 records
# - Parse errors (if any)
# - Confirmation prompt before writing to database
```

### Web UI (Alpha)

A SvelteKit web UI is included with:
- **Contact list** with search and detail views
- **Contact detail pages** with notes management and AI suggestions
- **Communications screen** with Email/SMS placeholder forms
- **Projects & Notes** placeholder screens
- Live API integration with sync service
- Optimistic updates and error handling
- Clear mock warnings throughout

**Start Web UI:**
```bash
cd apps/web
pnpm install
pnpm dev
# Visit http://localhost:3001
# Requires sync service running on port 3000
```

**Desktop App:** Tauri implementation planned for beta. See `apps/desktop/README.md`

### Desktop Shortcuts (Linux)

Linux desktop integration with launcher scripts and `.desktop` files:

**Install Desktop Shortcuts:**
```bash
# Desktop shortcuts are automatically available in:
# ~/.local/share/applications/

# Three launchers available:
# - SagensContact Web (starts sync service + web UI on port 3001)
# - SagensContact Desktop (starts sync service + Tauri desktop app)
# - SagensContact CLI (opens terminal with CLI)
```

**Launcher Scripts:**
- `start-web.sh` - Starts sync service and web UI, opens browser to http://localhost:3001
- `start-desktop.sh` - Starts sync service and Tauri desktop app
- Desktop shortcuts created during installation reference these scripts

**Manual Launch:**
```bash
# Web UI
./start-web.sh

# Desktop app (when available)
./start-desktop.sh

# Or use the documented start.sh:
./start.sh  # Runs sync_service, worker, and web UI
```

## Project Structure

```
alpha/
├── crates/
│   ├── core_domain/              # Domain entities and types
│   ├── local_store/               # SQLite repository implementations
│   ├── sync_service/              # Axum REST API + WebSocket server
│   ├── communication_queue/       # Email/SMS/social adapters (mocked)
│   │   └── src/bin/worker.rs     # Background worker binary
│   ├── ai_middleware/             # Mock Segmind AI client
│   └── cli_client/                # Command-line interface with import
├── apps/
│   ├── desktop/                   # Tauri + SvelteKit (planned)
│   └── web/                       # SvelteKit web UI (alpha)
│       ├── src/lib/api/client.ts  # Type-safe API client
│       └── src/routes/            # Contact list, detail, communications
├── config/                        # Placeholder credentials
├── sample_data/                   # Test CSV/vCard/SMS data
└── scripts/                       # Dev helpers
```

## Development

### Running Tests

```bash
cargo test
```

### Linting

```bash
cargo clippy -- -D warnings
cargo fmt --check
```

### Database Migrations

Migrations are embedded and run automatically on startup. Schema is defined in `crates/local_store/migrations/`.

## Configuration

See `config/README.md` for detailed configuration instructions.

### Secure Credential Vault

All services (sync, worker, CLI, and desktop) can now read secrets from an encrypted vault instead of plaintext files.

1. Populate `config/credentials.env` using `config/credentials.env.example`.
2. Encrypt it:

   ```bash
   cargo run -p secure_vault --features cli --bin vault_tool -- \
     encrypt --input config/credentials.env --output config/credentials.vault \
     --key "choose-a-strong-master-key"
   ```

3. Export the vault location/key before starting any component:

   ```bash
   export SAGENSCONTACT_VAULT_FILE="$PWD/config/credentials.vault"
   export SAGENSCONTACT_VAULT_KEY="choose-a-strong-master-key"
   ```

The loader decrypts the file at runtime and injects the secrets as environment variables, so existing configuration code continues to work.

**Security Warning**: Alpha uses plaintext credential files. Do NOT use in production.

### Attachment Storage

**Local Storage (Default)**
```bash
# Attachments stored in ./data/attachments/
mkdir -p data/attachments

# Configuration (in sync_service or desktop app)
ATTACHMENT_STORAGE_PATH=./data/attachments
ATTACHMENT_MAX_SIZE_MB=100
ATTACHMENT_ENABLE_SCAN=true  # Uses mock scanner in alpha
```

**S3-Compatible Storage (Beta)**
```bash
# Requires aws-sdk-s3 feature flag
cargo build --features s3-storage

# Environment variables
AWS_S3_BUCKET=my-attachments
AWS_S3_REGION=us-east-1
AWS_ACCESS_KEY_ID=your-key
AWS_SECRET_ACCESS_KEY=your-secret
```

**Virus Scanning**
- **Development**: Set `VIRUS_SCANNER_ENABLED=false` to use the mock scanner.
- **Beta**: Enable ClamAV scanning with `VIRUS_SCANNER_ENABLED=true`, `CLAMAV_SOCKET_PATH=/var/run/clamav/clamd.ctl`, and `VIRUS_SCANNER_STRICT=true`. Uploads stream through the ClamAV `INSTREAM` protocol and fail if the daemon is unavailable.
- **Scan statuses**: `Pending` → `Clean` | `Infected` | `Error`
- Infected files are automatically rejected and deleted

**File Upload Limits**
- Default max size: 100MB
- Allowed types: Configurable via `allowed_content_types` array
- Checksum verification: SHA-256 on upload and download
- Encryption flag: Tracks if file is encrypted at rest

### AI Configuration

**Segmind API Setup**
```bash
# Set environment variable for real API (optional)
export SEGMIND_API_KEY=your-api-key

# Without API key, client runs in MOCK MODE
# Mock mode returns deterministic suggestions for testing
```

**Features**
- **Response Caching**: 1-hour TTL to reduce API costs
- **Retry Logic**: 3 attempts with exponential backoff (100ms, 200ms, 400ms)
- **Interaction Logging**: All prompts/responses logged to `ai_interactions` table
- **Feedback Tracking**: Users can mark suggestions as helpful/not helpful
- **Apply Tracking**: Records when AI suggestions are applied

**Configuration Options**
```rust
// In your application code
let client = SegmindClient::new(Some(api_key))
    .with_config(
        Some("https://api.segmind.com/v1".to_string()),
        Some("llama-3.1-8b-instruct".to_string())
    );
```

**Logging AI Interactions**
```rust
use ai_middleware::LoggingSegmindClient;

let logging_client = LoggingSegmindClient::new(client, db_pool);

// All interactions are automatically logged with:
// - Prompt and response
// - Confidence score
// - Model used
// - Entity context (if applicable)
// - Cache hit status
// - Retry attempts
let response = logging_client.generate_suggestion_with_logging(
    "Suggest tags for this contact",
    user_id,
    Some("Contact".to_string()),
    Some(contact_id),
    "TagSuggestion"
).await?;
```

### Search History & Privacy

**Privacy Mode**
Enable privacy mode for searches that shouldn't be logged or suggested:
```typescript
// Web client
await api.searchContacts(query, { privacy_mode: true });
```

**Enriched History**
- `filters`: JSON object of search criteria
- `result_ids`: Array of UUIDs returned in search
- `result_count`: Number of results
- `clicked_result_id`: Which result user clicked
- `privacy_mode`: Whether search should be private
- `metadata`: Extensible JSON for future features

**Recent Searches Widget**
Displays last 10 searches with result count and timestamps, respecting privacy settings.

## Sample Workflow: Military Artifact Sharing

1. Import contacts from sample data
2. Create project "WWII Artifact Authentication"
3. Add contacts (Colonel Johnson, Curator Davis, Archivist Martinez) to project
4. Add note with artifact details and attach photos
5. Share note with Curator Davis (read permission)
6. Queue communication to remind Colonel Johnson for follow-up
7. AI suggestion: "Consider tagging this as 'Historical' and 'Confidential'"
8. Accept AI suggestion, apply tags
9. Sync changes to central server
10. View shared artifact on mobile web interface

## Known Limitations (Alpha)

- **Desktop app:** Architecture documented, implementation planned for beta
- **Web UI:** Core features implemented (contacts, notes, communications), advanced features (projects detail, sharing UI, settings) pending
- **External services:** Email, SMS, social, AI use mock/deterministic responses
- **Security:** Auth still limited to single-user JWT, but secrets can now be sourced from the encrypted vault (see SECURITY_NOTES.md)
- **Sync:** Basic WebSocket support, no conflict resolution yet
- **Attachments:** Local filesystem only (MinIO/S3 in beta)
- **Single-user:** No multi-tenancy or user isolation
- **Import:** CSV with field mapping implemented, vCard/SMS parsing pending

See **[SECURITY_NOTES.md](SECURITY_NOTES.md)** for comprehensive security discussion.

## Roadmap to Beta

**Phase 6 Completed (Attachments & AI Plumbing)**:
- [x] Attachment pipeline with upload/scan/storage across all entities
- [x] Configurable Segmind client with caching, retry, and mock mode
- [x] AI interaction logging with feedback tracking
- [x] Search history enrichment with privacy mode
- [x] Web/desktop UI components for attachments and AI suggestions
- [x] Comprehensive test coverage for attachment and AI systems

**Upcoming**:
- [x] Implement secure credential vault integration (use `secure_vault` + encrypted env file)
- [x] Add real virus scanning (ClamAV `INSTREAM` integration)
- [ ] Implement proper authentication (JWT, OAuth2)
- [ ] Add end-to-end encryption for synced data
- [ ] Complete Tauri desktop application polish
- [ ] Polish web UI responsive design
- [ ] Add Playwright E2E tests
- [ ] Implement conflict resolution for sync
- [ ] S3-compatible storage backend
- [ ] Hosted vs self-host deployment options

## License

MIT

## Contributing

This is an alpha release for testing and feedback. See ARCHITECTURE.md and TESTING.md for development guidelines.
