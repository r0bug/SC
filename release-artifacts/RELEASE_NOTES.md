# SagensContact Alpha v0.1.0

## What's New

### Features
- ✅ Complete contact management system
- ✅ Import from multiple formats (CSV, vCard, Google, Apple, Social)
- ✅ Dashboard with entity statistics
- ✅ Automatic update system
- ✅ RESTful API with rate limiting
- ✅ Web UI (SvelteKit)

### New in this Release
- Dashboard API endpoint (`/api/dashboard`)
- Import history endpoint (`/api/import/history`)
- Automatic update system with GitHub releases integration
- Improved CI/CD with modern GitHub Actions

## Installation

### Quick Start

```bash
# Download for your platform
wget https://github.com/r0bug/SC/releases/download/v0.1.0/sync_service-ubuntu-latest
wget https://github.com/r0bug/SC/releases/download/v0.1.0/sagenscontact-ubuntu-latest

# Make executable
chmod +x sync_service-ubuntu-latest sagenscontact-ubuntu-latest

# Rename
mv sync_service-ubuntu-latest sync_service
mv sagenscontact-ubuntu-latest sagenscontact

# Run
./sync_service
```

### Environment Setup

```bash
export DATABASE_URL="sqlite:./data/contacts.db"
export PORT=3002
mkdir -p data/attachments
./sync_service
```

## API Endpoints

- `GET /api/dashboard` - Dashboard summary
- `GET /api/import/history` - Import history
- `GET /api/system/version` - Current version
- `GET /api/system/updates/check` - Check for updates

Full API documentation: https://github.com/r0bug/SC/blob/main/docs/

## Requirements

- SQLite 3.x
- Linux, macOS, or Windows
- 50MB disk space minimum

## Checksum Verification

```bash
sha256sum -c SHA256SUMS
```

## Support

- Report issues: https://github.com/r0bug/SC/issues
- Documentation: https://github.com/r0bug/SC/blob/main/README.md
