# SagensContact Alpha - Deployment Guide

Complete guide for deploying SagensContact Alpha (Phase 6) on a fresh machine.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Building from Source](#building-from-source)
3. [Deploying Pre-built Package](#deploying-pre-built-package)
4. [Configuration](#configuration)
5. [Starting Services](#starting-services)
6. [Verification](#verification)
7. [Troubleshooting](#troubleshooting)
8. [Production Considerations](#production-considerations)

## Prerequisites

### Required

- **Operating System**: Linux or macOS (Windows via WSL2)
- **SQLite 3.44+**: Usually pre-installed on Linux/macOS
  ```bash
  sqlite3 --version
  ```

### For Pre-built Package

- **Node.js 20 LTS+**: Required for web UI
  ```bash
  # Install via nvm (recommended)
  curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
  nvm install 20
  nvm use 20
  ```

- **pnpm 8+**: Package manager for web UI
  ```bash
  npm install -g pnpm
  ```

### For Building from Source

- **Rust 1.83+**: Required for compiling binaries
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source ~/.cargo/env
  rustc --version
  ```

- **Build tools**: gcc/clang (usually pre-installed)
  ```bash
  # Ubuntu/Debian
  sudo apt-get install build-essential pkg-config libssl-dev

  # macOS (via Xcode Command Line Tools)
  xcode-select --install
  ```

- **Node.js 20 LTS+ and pnpm**: (same as above)

### Optional (Beta Features)

- **PostgreSQL 15+**: For production sync service
- **MinIO/S3**: For attachment storage backend
- **ClamAV**: For virus scanning (set `VIRUS_SCANNER_ENABLED=true`)
- **Redis**: For distributed caching and job queue

## Building from Source

### Step 1: Clone Repository

```bash
git clone https://github.com/your-org/sagenscontact.git
cd sagenscontact/alpha
```

### Step 2: Run Build Script

```bash
./scripts/deployment/build_all.sh
```

This script will:
1. Build all Rust binaries (CLI, sync_service, worker)
2. Build web UI with optimizations
3. Copy configuration templates
4. Create deployment scripts
5. Package documentation
6. Create distributable archive in `artifacts/`

**Build Output**: `artifacts/sagenscontact-alpha-phase6-YYYYMMDD_HHMMSS.tar.gz`

**Build Time**: ~5-10 minutes on modern hardware

### Step 3: Verify Build

```bash
cd artifacts/phase6-build
./scripts/verify_installation.sh
```

## Deploying Pre-built Package

### Step 1: Download Package

```bash
# Download from release
wget https://releases.sagenscontact.com/sagenscontact-alpha-phase6-latest.tar.gz

# Verify checksum
sha256sum -c sagenscontact-alpha-phase6-latest.tar.gz.sha256
```

### Step 2: Extract

```bash
tar -xzf sagenscontact-alpha-phase6-latest.tar.gz
cd phase6-build
```

### Step 3: Install Dependencies

```bash
# Install Node.js (if not already installed)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc
nvm install 20

# Install pnpm
npm install -g pnpm

# Install web UI dependencies
cd web
pnpm install
cd ..
```

### Step 4: Verify Installation

```bash
./scripts/verify_installation.sh
```

Expected output:
```
==========================================
SagensContact Installation Verification
==========================================

Checking binaries...
  ✓ sagenscontact
  ✓ sync_service
  ✓ worker

Checking web build...
  ✓ Web build exists

Checking configuration...
  ✓ credentials.toml exists

Checking directories...
  ✓ data/attachments created

Testing CLI...
  ✓ CLI executable works

==========================================
✓ Installation verification complete!
==========================================
```

## Configuration

### Database Setup

The default SQLite database is created automatically on first run.

**Location**: `data/sagenscontact.db`

**Custom location**:
```bash
export DATABASE_URL="sqlite:/path/to/custom/database.db"
```

**PostgreSQL (Beta)**:
```bash
export DATABASE_URL="postgresql://user:password@localhost:5432/sagenscontact"
```

### Credentials Configuration

Edit `config/credentials.toml`:

```toml
# Segmind AI Configuration
[segmind]
api_key = ""  # Leave empty for mock mode
api_base_url = "https://api.segmind.com/v1"
model = "llama-3.1-8b-instruct"

# Email Configuration (set real values for SMTP sending)
[email]
smtp_host = "smtp.example.com"
smtp_port = 587
smtp_user = "user@example.com"
smtp_password = "your-password"

# SMS Configuration (set real values for Twilio sending)
[sms]
twilio_account_sid = "your-sid"
twilio_auth_token = "your-token"
twilio_phone_number = "+1234567890"

# Attachment Storage
[storage]
backend = "local"  # "local" or "s3"
local_path = "./data/attachments"
max_size_mb = 100
enable_scan = true  # Set VIRUS_SCANNER_ENABLED=true for ClamAV

# S3 Configuration (Beta)
[storage.s3]
bucket = "sagenscontact-attachments"
region = "us-east-1"
access_key_id = ""
secret_access_key = ""
```

### Environment Variables

Create `.env` file in deployment directory:

```bash
# Database
DATABASE_URL=sqlite:data/sagenscontact.db

# Sync Service
PORT=3000
ATTACHMENT_STORAGE_PATH=./data/attachments
ATTACHMENT_MAX_SIZE_MB=100
ATTACHMENT_ENABLE_SCAN=true

# AI Configuration
SEGMIND_API_KEY=  # Optional, mock mode if empty

# Web UI
WEB_PORT=3001
API_URL=http://localhost:3000

# Logging
RUST_LOG=info
```

Load environment:
```bash
source .env
```

## Starting Services

### Development Mode (All Services)

Use the provided convenience scripts in separate terminals:

**Terminal 1 - Sync Service**:
```bash
./scripts/start_sync_service.sh
```

**Terminal 2 - Web UI**:
```bash
./scripts/start_web.sh
```

**Terminal 3 - Background Worker** (Optional):
```bash
./scripts/start_worker.sh
```

### Manual Start

**Sync Service**:
```bash
export DATABASE_URL="sqlite:data/sagenscontact.db"
export PORT=3000
./bin/sync_service
```

**Web UI**:
```bash
cd web
PORT=3001 npx sirv build --host 0.0.0.0
```

**Background Worker**:
```bash
export DATABASE_URL="sqlite:data/sagenscontact.db"
./bin/worker
```

### Production Mode with systemd

Create service files in `/etc/systemd/system/`:

**sagenscontact-sync.service**:
```ini
[Unit]
Description=SagensContact Sync Service
After=network.target

[Service]
Type=simple
User=sagenscontact
Group=sagenscontact
WorkingDirectory=/opt/sagenscontact
Environment="DATABASE_URL=sqlite:/opt/sagenscontact/data/sagenscontact.db"
Environment="PORT=3000"
Environment="RUST_LOG=info"
ExecStart=/opt/sagenscontact/bin/sync_service
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

**sagenscontact-web.service**:
```ini
[Unit]
Description=SagensContact Web UI
After=network.target sagenscontact-sync.service

[Service]
Type=simple
User=sagenscontact
Group=sagenscontact
WorkingDirectory=/opt/sagenscontact/web
Environment="PORT=3001"
Environment="API_URL=http://localhost:3000"
ExecStart=/usr/bin/npx sirv build --host 0.0.0.0 --port 3001
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable sagenscontact-sync
sudo systemctl enable sagenscontact-web
sudo systemctl start sagenscontact-sync
sudo systemctl start sagenscontact-web
```

### Docker Deployment (Beta)

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  sync_service:
    image: sagenscontact/sync-service:alpha
    ports:
      - "3000:3000"
    volumes:
      - ./data:/app/data
      - ./config:/app/config
    environment:
      - DATABASE_URL=sqlite:/app/data/sagenscontact.db
      - ATTACHMENT_STORAGE_PATH=/app/data/attachments
    restart: unless-stopped

  web:
    image: sagenscontact/web:alpha
    ports:
      - "3001:3001"
    environment:
      - API_URL=http://sync_service:3000
    depends_on:
      - sync_service
    restart: unless-stopped

  worker:
    image: sagenscontact/worker:alpha
    volumes:
      - ./data:/app/data
      - ./config:/app/config
    environment:
      - DATABASE_URL=sqlite:/app/data/sagenscontact.db
    depends_on:
      - sync_service
    restart: unless-stopped
```

Start:
```bash
docker-compose up -d
```

## Verification

### 1. Health Checks

**Sync Service**:
```bash
curl http://localhost:3000/health
# Expected: "OK"
```

**API Endpoints**:
```bash
# List contacts
curl http://localhost:3000/api/contacts

# Get tags
curl http://localhost:3000/api/tags
```

### 2. Web UI Access

Open browser: http://localhost:3001

Expected:
- Contact list page loads
- Search functionality works
- AI suggestions appear
- Attachment upload works

### 3. CLI Functionality

**Import sample data**:
```bash
./bin/sagenscontact import --csv sample_data/contacts.csv
```

**List contacts**:
```bash
./bin/sagenscontact list
```

**Search**:
```bash
./bin/sagenscontact search "john"
```

**Create note**:
```bash
./bin/sagenscontact note <contact_id> "Meeting Notes" "Discussed project timeline"
```

### 4. Run Test Suite

**Backend tests**:
```bash
# Navigate to source repository
cd /path/to/sagenscontact/alpha
cargo test
```

Expected: 39+ tests passing

**E2E CLI tests**:
```bash
./scripts/cli_e2e_test.sh
```

### 5. Attachment Upload Test

1. Navigate to http://localhost:3001/contacts/[contact-id]
2. Click "Upload Attachment"
3. Select file < 100MB
4. Verify:
   - Progress bar appears
   - File appears in attachment list
   - Scan status shows "Clean" (mock)
   - Download works

### 6. AI Suggestion Test

1. Navigate to contact detail page
2. Scroll to "AI Suggestions" section
3. Click "Get Suggestions"
4. Verify:
   - Suggestions appear with confidence scores
   - Feedback buttons (👍 👎) work
   - "Apply" button marks suggestion as applied

## Troubleshooting

### Port Already in Use

**Problem**: `Error: address already in use`

**Solution**:
```bash
# Check what's using the port
lsof -i :3000
# Or use netstat
netstat -tuln | grep 3000

# Kill the process
kill -9 <PID>

# Or use different port
PORT=3001 ./scripts/start_sync_service.sh
```

### Database Locked

**Problem**: `database is locked`

**Cause**: Multiple processes accessing SQLite simultaneously

**Solution**:
```bash
# Ensure only one sync_service is running
ps aux | grep sync_service

# Remove WAL files if service crashed
rm data/sagenscontact.db-wal data/sagenscontact.db-shm

# Restart service
./scripts/start_sync_service.sh
```

### Web UI Not Loading

**Problem**: Blank page or "Cannot connect to API"

**Solutions**:

1. **Ensure sync service is running**:
   ```bash
   curl http://localhost:3000/health
   ```

2. **Check browser console** (F12):
   - Look for CORS errors
   - Verify API_URL in network requests

3. **Verify environment variables**:
   ```bash
   echo $API_URL  # Should be http://localhost:3000
   ```

4. **Check ports**:
   ```bash
   lsof -i :3000  # Sync service
   lsof -i :3001  # Web UI
   ```

### Missing Dependencies

**Problem**: `command not found: node`, `cargo: command not found`

**Solutions**:

**Node.js**:
```bash
# Install via nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc
nvm install 20
```

**Rust** (only for building from source):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

**pnpm**:
```bash
npm install -g pnpm
```

### Attachment Upload Fails

**Problem**: Upload returns error or file not saved

**Solutions**:

1. **Check storage directory permissions**:
   ```bash
   ls -la data/attachments
   chmod 755 data/attachments
   ```

2. **Verify file size**:
   ```bash
   # Default max is 100MB
   export ATTACHMENT_MAX_SIZE_MB=200
   ```

3. **Check disk space**:
   ```bash
   df -h
   ```

### AI Suggestions Not Appearing

**Problem**: "Get Suggestions" button does nothing or errors

**Solutions**:

1. **Verify mock mode** (no API key required in alpha):
   ```bash
   # API key should be empty for mock mode
   grep api_key config/credentials.toml
   ```

2. **Check logs**:
   ```bash
   # Sync service logs should show AI requests
   RUST_LOG=debug ./scripts/start_sync_service.sh
   ```

3. **Test AI endpoint directly**:
   ```bash
   curl -X POST http://localhost:3000/api/ai/suggest \
     -H "Content-Type: application/json" \
     -d '{"prompt": "Suggest tags", "entity_type": "Contact"}'
   ```

### Tests Failing

**Problem**: `cargo test` shows failures

**Solutions**:

1. **Clean and rebuild**:
   ```bash
   cargo clean
   cargo build --release
   cargo test
   ```

2. **Check specific test**:
   ```bash
   cargo test <test_name> -- --nocapture
   ```

3. **Verify database schema**:
   ```bash
   sqlite3 data/sagenscontact.db ".schema"
   # Should include ai_interactions, attachments, search_history tables
   ```

### Permission Denied Errors

**Problem**: Scripts won't execute

**Solution**:
```bash
# Make scripts executable
chmod +x scripts/*.sh
chmod +x bin/*
```

### CORS Errors in Browser

**Problem**: Browser console shows CORS policy errors

**Solution**: Sync service has CORS enabled by default. If issues persist:

1. Check API_URL matches sync service address
2. Ensure no proxy/firewall blocking
3. Try accessing from same host (localhost)

## Production Considerations

### Security

⚠️ **Alpha release is NOT production-ready**

**Required for production**:
- [ ] Enable authentication (JWT, OAuth2)
- [ ] Use PostgreSQL instead of SQLite
- [ ] Enable TLS/HTTPS
- [ ] Implement rate limiting
- [ ] Use secure credential vault (not plaintext files)
- [ ] Enable real virus scanning (ClamAV)
- [ ] Set up proper backup strategy
- [ ] Configure firewall rules
- [ ] Enable audit logging
- [ ] Implement RBAC for sharing

See `SECURITY_NOTES.md` for detailed security discussion.

### Performance

**For production loads**:

1. **Use PostgreSQL**:
   ```bash
   export DATABASE_URL="postgresql://user:pass@localhost/sagenscontact"
   ```

2. **Enable connection pooling**:
   ```rust
   // In sync_service
   let pool = PgPoolOptions::new()
       .max_connections(20)
       .connect(&database_url).await?;
   ```

3. **Use Redis for caching**:
   ```bash
   export REDIS_URL="redis://localhost:6379"
   ```

4. **Scale horizontally**:
   - Run multiple sync_service instances behind load balancer
   - Use shared PostgreSQL database
   - Use S3 for attachment storage (not local filesystem)

### Monitoring

**Add health check endpoints**:
```bash
# Expose metrics
curl http://localhost:3000/metrics  # Prometheus format

# Check service status
curl http://localhost:3000/health
```

**Log aggregation**:
```bash
# Structured JSON logging
export RUST_LOG=info
export LOG_FORMAT=json

# Ship to ELK/Loki
./bin/sync_service | filebeat
```

### Backup Strategy

**Database backup**:
```bash
# SQLite
sqlite3 data/sagenscontact.db ".backup backup/sagenscontact-$(date +%Y%m%d).db"

# PostgreSQL
pg_dump sagenscontact > backup/sagenscontact-$(date +%Y%m%d).sql
```

**Attachment backup**:
```bash
# Sync to S3
aws s3 sync data/attachments s3://sagenscontact-backups/attachments/

# Or rsync to backup server
rsync -avz data/attachments/ backup-server:/backups/sagenscontact/
```

**Automated backups**:
```bash
# Add to crontab
0 2 * * * /opt/sagenscontact/scripts/backup.sh
```

### Updates and Migrations

**Update process**:

1. **Stop services**:
   ```bash
   sudo systemctl stop sagenscontact-web
   sudo systemctl stop sagenscontact-sync
   ```

2. **Backup database**:
   ```bash
   sqlite3 data/sagenscontact.db ".backup backup/pre-update.db"
   ```

3. **Extract new version**:
   ```bash
   tar -xzf sagenscontact-alpha-phaseN-latest.tar.gz
   ```

4. **Run migrations** (automatic on startup)

5. **Restart services**:
   ```bash
   sudo systemctl start sagenscontact-sync
   sudo systemctl start sagenscontact-web
   ```

## Support and Resources

- **Documentation**: See `docs/` directory
- **Testing Guide**: `TESTING.md`
- **Architecture**: `ARCHITECTURE.md`
- **Security**: `SECURITY_NOTES.md`
- **Issues**: https://github.com/your-org/sagenscontact/issues

## Next Steps

After successful deployment:

1. Import your contact data
2. Configure AI API key (if using real Segmind)
3. Set up regular backups
4. Configure monitoring/alerts
5. Review security checklist
6. Run E2E test suite
7. Train users on web UI

**Phase 7 Roadmap**:
- Observability and metrics
- Security hardening
- Production deployment guides
- Performance optimization
- User management and auth
