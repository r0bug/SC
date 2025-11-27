# SagensContact Alpha

Portable contact manager for macOS/Linux desktops with responsive web interface.

## Features

- Contact, Tag, Project, Calendar Event, and Note management
- CSV/vCard/SMS import with social media support (LinkedIn, Twitter, Facebook, Instagram)
- **Attachment Management** - Upload, scan (ClamAV), and manage files across all entities
- **AI-powered Suggestions** - Segmind integration with caching & retry
- **Communication Queue** - Email (SMTP) and SMS (Twilio) with background worker
- **Search History & Suggestions** - Track searches with privacy mode
- Sharing with per-entity ACL and audit logging
- CLI, Desktop (Tauri), and Web (Svelte) interfaces
- SQLite or PostgreSQL storage with Redis caching option
- S3/MinIO attachment storage option
- WebSocket real-time notifications

## Service Configuration

All external services work out of the box with mock/fallback mode, or can be configured for production:

| Service | Default | Production Config |
|---------|---------|-------------------|
| **Email** | Mock (logs only) | `SMTP_HOST`, `SMTP_USER`, `SMTP_PASSWORD`, `SMTP_FROM` |
| **SMS** | Mock (logs only) | `TWILIO_ACCOUNT_SID`, `TWILIO_AUTH_TOKEN`, `TWILIO_PHONE_NUMBER` |
| **AI** | Mock suggestions | `SEGMIND_API_KEY` |
| **Virus Scan** | Basic check | `VIRUS_SCANNER_ENABLED=true`, `CLAMAV_SOCKET_PATH` |
| **Database** | SQLite | `DATABASE_URL=postgres://...` + `--features postgres` |
| **Cache** | In-memory | `REDIS_URL=redis://...` |
| **Attachments** | Local filesystem | `S3_ENDPOINT_URL`, `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY`, `S3_BUCKET` |

## Prerequisites

**For Backend (Required):**
- Rust 1.83+ (install via [rustup](https://rustup.rs/))
- SQLite 3.44+ (usually pre-installed on macOS/Linux)
- Build tools (gcc/clang)

**For Web UI (Optional):**
- Node.js 20 LTS + pnpm 8+
- Run `cd apps/web && pnpm install && pnpm dev`
- Access at http://localhost:3001

**For Production:**
- PostgreSQL 15+ (optional, for sync service)
- Redis 7+ (optional, for distributed caching)
- ClamAV daemon (optional, for virus scanning)
- MinIO or S3 (optional, for attachment storage)

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
# Processes pending emails/SMS every 30 seconds

# 9. Run E2E tests
./scripts/cli_e2e_test.sh
```

### Import Workflow

The CLI import supports multiple formats with field mapping and preview:

```bash
# CSV import
./target/release/sagenscontact import --csv sample_data/contacts.csv

# vCard import
./target/release/sagenscontact import --vcard contacts.vcf

# Social media imports (LinkedIn, Twitter, Facebook, Instagram)
./target/release/sagenscontact import --json linkedin_connections.json
```

### Web UI

A SvelteKit web UI is included with:
- **Contact list** with search and detail views
- **Contact detail pages** with notes management and AI suggestions
- **Communications screen** with Email/SMS forms
- **Import page** with format selection and preview
- **Projects & Notes** management
- Live API integration with sync service
- Optimistic updates and error handling

**Start Web UI:**
```bash
cd apps/web
pnpm install
pnpm dev
# Visit http://localhost:3001
# Requires sync service running on port 3000
```

## Project Structure

```
alpha/
├── crates/
│   ├── core_domain/              # Domain entities and types
│   ├── local_store/              # SQLite/PostgreSQL repository implementations
│   ├── sync_service/             # Axum REST API + WebSocket server
│   ├── communication_queue/      # Email/SMS/social adapters
│   │   └── src/bin/worker.rs     # Background worker binary
│   ├── ai_middleware/            # Segmind AI client with caching
│   ├── import_service/           # CSV/vCard/JSON/social importers
│   ├── attachment_service/       # File storage with S3 support
│   ├── cache_layer/              # Redis/in-memory caching
│   ├── secure_vault/             # Encrypted credential storage
│   └── cli_client/               # Command-line interface
├── apps/
│   ├── desktop/                  # Tauri + SvelteKit desktop app
│   └── web/                      # SvelteKit web UI
├── config/                       # Configuration files
├── sample_data/                  # Test CSV/vCard/SMS data
└── scripts/                      # Dev helpers
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

All services can read secrets from an encrypted vault:

```bash
# 1. Create credentials file
cp config/credentials.env.example config/credentials.env
# Edit with your real credentials

# 2. Encrypt it
cargo run -p secure_vault --features cli --bin vault_tool -- \
  encrypt --input config/credentials.env --output config/credentials.vault \
  --key "choose-a-strong-master-key"

# 3. Export vault location/key
export SAGENSCONTACT_VAULT_FILE="$PWD/config/credentials.vault"
export SAGENSCONTACT_VAULT_KEY="choose-a-strong-master-key"
```

### Attachment Storage

**Local Storage (Default)**
```bash
ATTACHMENT_STORAGE_PATH=./data/attachments
ATTACHMENT_MAX_SIZE_MB=100
```

**S3-Compatible Storage**
```bash
S3_ENDPOINT_URL=http://localhost:9000  # For MinIO
S3_REGION=us-east-1
S3_ACCESS_KEY_ID=your-key
S3_SECRET_ACCESS_KEY=your-secret
S3_BUCKET=attachments
```

**Virus Scanning**
```bash
VIRUS_SCANNER_ENABLED=true
CLAMAV_SOCKET_PATH=/var/run/clamav/clamd.sock
VIRUS_SCANNER_STRICT=true  # Fail if ClamAV unavailable
```

### AI Configuration

```bash
# Set for real AI suggestions
export SEGMIND_API_KEY=your-api-key

# Without API key, returns deterministic mock suggestions
```

Features:
- **Response Caching**: 1-hour TTL to reduce API costs
- **Retry Logic**: 3 attempts with exponential backoff
- **Interaction Logging**: All prompts/responses logged to database
- **Feedback Tracking**: Users can rate suggestions

### Communication Services

**Email (SMTP)**
```bash
SMTP_HOST=smtp.example.com
SMTP_PORT=587
SMTP_USER=user@example.com
SMTP_PASSWORD=password
SMTP_FROM=noreply@example.com
```

**SMS (Twilio)**
```bash
TWILIO_ACCOUNT_SID=your-sid
TWILIO_AUTH_TOKEN=your-token
TWILIO_PHONE_NUMBER=+1234567890
```

## Known Limitations (Alpha)

- **Authentication**: Single-user JWT, no OAuth2 yet
- **TLS/HTTPS**: Sync service runs HTTP only
- **Encryption at Rest**: SQLite not encrypted (SQLCipher planned)
- **Desktop App**: Functional but attachment commands not wired up
- **Social OAuth**: Import from exports only, no live API integration

See **[SECURITY_NOTES.md](SECURITY_NOTES.md)** for security discussion.

## Roadmap to Beta

**Completed**:
- [x] PostgreSQL support with feature flags
- [x] Redis caching layer
- [x] S3/MinIO attachment storage
- [x] ClamAV virus scanning integration
- [x] Social media importers (LinkedIn, Twitter, Facebook, Instagram)
- [x] Secure credential vault
- [x] Audit logging
- [x] ACL enforcement on all routes
- [x] Playwright E2E test suite
- [x] Desktop app icons

**Upcoming**:
- [ ] TLS/HTTPS for sync service
- [ ] SQLCipher encryption at rest
- [ ] OAuth2 authentication
- [ ] Social platform live API integration
- [ ] Conflict resolution for sync
- [ ] Multi-tenancy support

## License

MIT

## Contributing

See ARCHITECTURE.md and TESTING.md for development guidelines.
