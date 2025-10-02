# SagensContact Alpha - Deployment Package

This is a pre-built deployment package for SagensContact Alpha (Phase 6).

## Quick Start

1. **Verify Installation**
   ```bash
   ./scripts/verify_installation.sh
   ```

2. **Import Sample Data**
   ```bash
   export DATABASE_URL="sqlite:data/sagenscontact.db"
   ./bin/sagenscontact import --csv sample_data/contacts.csv
   ```

3. **Start Sync Service** (Terminal 1)
   ```bash
   ./scripts/start_sync_service.sh
   ```

4. **Start Web UI** (Terminal 2)
   ```bash
   ./scripts/start_web.sh
   ```

5. **Access Application**
   - Web UI: http://localhost:3001
   - API: http://localhost:3000/api

## Optional: Start Background Worker

For processing communication queue (Terminal 3):
```bash
./scripts/start_worker.sh
```

## Directory Structure

```
phase6-build/
├── bin/                     # Compiled binaries
│   ├── sagenscontact       # CLI tool
│   ├── sync_service        # REST API server
│   └── worker              # Background job processor
├── web/                    # Built web application
│   └── build/
├── config/                 # Configuration files
│   └── credentials.toml    # API credentials (MOCK MODE)
├── sample_data/            # Test data
│   └── contacts.csv
├── scripts/                # Convenience scripts
│   ├── verify_installation.sh
│   ├── start_sync_service.sh
│   ├── start_web.sh
│   └── start_worker.sh
├── docs/                   # Documentation
│   ├── README.md
│   ├── TESTING.md
│   └── ARCHITECTURE.md
└── data/                   # Runtime data (created on first run)
    ├── sagenscontact.db
    └── attachments/

```

## Prerequisites

- **Linux or macOS** (Windows via WSL2)
- **Node.js 20+** for web UI (https://nodejs.org/)
- **SQLite 3.44+** (usually pre-installed)

No Rust toolchain needed for this pre-built package.

## Configuration

Edit `config/credentials.toml` to configure:
- Segmind API key (leave empty for mock mode)
- Email/SMS service credentials (mocked in alpha)
- Storage backend settings

## Environment Variables

- `DATABASE_URL` - SQLite database path (default: `sqlite:data/sagenscontact.db`)
- `ATTACHMENT_STORAGE_PATH` - File storage directory (default: `data/attachments`)
- `PORT` - Sync service port (default: 3000)
- `SEGMIND_API_KEY` - AI API key (optional, mock mode if not set)

## Troubleshooting

### Port Already in Use
```bash
# Check what's using port 3000
lsof -i :3000
# Or use different port
PORT=3001 ./scripts/start_sync_service.sh
```

### Database Locked
- Ensure only one sync_service instance is running
- Delete `data/sagenscontact.db-wal` and `data/sagenscontact.db-shm` if needed

### Web UI Not Loading
- Ensure sync service is running first on port 3000
- Check browser console for API connection errors
- Verify `API_URL` environment variable matches sync service port

### Missing Dependencies
```bash
# Install Node.js (if needed)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20

# Install pnpm globally
npm install -g pnpm

# Install sirv for serving built web app
cd web && npm install
```

## Testing

Run the test suite:
```bash
# Navigate to source repository (not this deployment package)
cd /path/to/sagenscontact/alpha
cargo test
```

## Security Notice

⚠️ **This is an ALPHA release with placeholder security:**
- No authentication/authorization
- Plaintext credentials
- Mock external services
- Not production-ready

See `docs/SECURITY_NOTES.md` for details.

## Support

- Documentation: `docs/`
- Issues: https://github.com/anthropics/sagenscontact/issues (hypothetical)
- Architecture: `docs/ARCHITECTURE.md`

## What's New in Phase 6

✅ **Attachment Management**
- Upload/download files with checksum verification
- Virus scanning (mock in alpha)
- S3-compatible storage support (beta)

✅ **AI Integration**
- Configurable Segmind client with caching (1-hour TTL)
- Retry logic with exponential backoff
- Interaction logging with feedback tracking

✅ **Search Enhancement**
- Result tracking with privacy mode
- Recent searches widget
- AI-powered search suggestions

✅ **Comprehensive Testing**
- 39+ unit and integration tests
- Attachment and AI interaction test suites
- Manual testing checklists

See `docs/README.md` for full feature list and roadmap.
