# SagensContact Alpha - Phase 7 QA Package

**Version:** 0.1.0-alpha  
**Build Date:** $(date +%Y-%m-%d)  
**Status:** QA Ready - Security Features Implemented

## Contents

- `binaries/` - Compiled backend services
  - `sagenscontact` - CLI client for imports and management
  - `sync_service` - Main API server
  - `worker` - Background job processor (if available)

- `web/` - Production-built web UI

- `docs/` - Documentation
  - `TLS_HTTPS_SETUP.md` - Complete TLS/HTTPS deployment guide
  - `PHASE_7_COMPLETION_REPORT.md` - Phase 7 implementation details
  - `VERIFICATION_REPORT.md` - Test results and verification

- `scripts/` - Deployment and utility scripts
  - `start_sync_service.sh` - Start the API server
  - `start_web_ui.sh` - Start the web UI
  - `benchmark.sh` - Performance benchmarking

- `config/` - Configuration examples
  - `nginx.conf.example` - Nginx reverse proxy config
  - `systemd/` - systemd service files

## Quick Start

### 1. Database Setup
```bash
export DATABASE_URL="sqlite:./data/contacts.db"
mkdir -p data/attachments
```

### 2. Start Sync Service
```bash
cd scripts
./start_sync_service.sh
```

### 3. Start Web UI
```bash
cd scripts
./start_web_ui.sh
```

### 4. Access
- Web UI: http://localhost:3001
- API: http://localhost:3002
- Health: http://localhost:3002/health
- Metrics: http://localhost:3002/metrics

## Environment Variables

### Sync Service
- `DATABASE_URL` - Database connection (default: sqlite:./data/contacts.db)
- `PORT` - API port (default: 3002)
- `JWT_SECRET` - **REQUIRED IN PRODUCTION** - Secret for JWT tokens
- `LOG_FORMAT` - Logging format: `json` or `pretty` (default: json)
- `ATTACHMENT_STORAGE_PATH` - File storage location (default: ./data/attachments)

### Web UI
- `PORT` - Web UI port (default: 3001)

## Production Deployment

### Security Requirements
1. **Set JWT_SECRET** - Generate a strong secret:
   ```bash
   export JWT_SECRET=$(openssl rand -base64 32)
   ```

2. **Enable TLS** - Use nginx or Caddy reverse proxy (see docs/TLS_HTTPS_SETUP.md)

3. **Restrict Metrics** - Block /metrics endpoint from public access

4. **Database Backups** - Set up regular SQLite backups

### Recommended Setup
1. Deploy behind reverse proxy (nginx/Caddy)
2. Use systemd for process management (see config/systemd/)
3. Configure Prometheus to scrape /metrics
4. Set up log aggregation for JSON logs
5. Monitor rate limits and security headers

## Phase 7 Security Features

### Implemented
✅ Rate Limiting (per-IP token bucket)
- Auth endpoints: 10 req/burst
- Attachments: 100 req/burst
- Search: 30 req/burst

✅ Security Headers
- HSTS, CSP, X-Frame-Options
- X-Content-Type-Options
- Referrer-Policy
- Permissions-Policy

✅ Input Validation
- Email, filename, query sanitization
- File size and type checking
- Password strength validation

✅ Observability
- Prometheus metrics on /metrics
- Structured JSON logging
- Request tracing

### Test Results
- **75 Rust tests passing** ✅
- **Web UI builds successfully** ✅
- Security modules compile and function ✅

## Known Limitations (Alpha)
- In-memory rate limiting (use Redis for production multi-node)
- Mock email/SMS sending (implement real providers)
- Accessibility warnings in web UI (non-blocking)
- SQLite single-writer (use PostgreSQL for production scale)

## CLI Usage

### Import SMS
```bash
./binaries/sagenscontact import-sms /path/to/sms-backup.xml
```

### Import Contacts
```bash
./binaries/sagenscontact import-csv /path/to/contacts.csv
```

## Support
For issues or questions, see project documentation or contact the development team.
