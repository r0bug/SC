# SagensContact Alpha - Turnkey Installer Guide

**Version:** 0.1.0-alpha
**Last Updated:** October 2, 2025

---

## Overview

The SagensContact turnkey installer (`scripts/install.sh`) is a fully automated installation system that:

✅ Detects and installs all prerequisites automatically
✅ Validates versions and applies fixes when needed
✅ Builds and configures all services end-to-end
✅ Creates systemd services for production deployment
✅ Generates secure configuration with minimal user input
✅ Provides clear remediation steps for any issues

**Zero manual troubleshooting required** - the installer handles everything from a fresh system to a running production deployment.

---

## Quick Start

### Single Command Installation

```bash
# Clone repository
git clone https://github.com/your-org/sagenscontact-alpha.git
cd sagenscontact-alpha

# Run installer (requires sudo for system-wide install)
sudo ./scripts/install.sh
```

That's it! The installer will:
1. Detect your OS and package manager
2. Install all prerequisites (Rust, Node.js, SQLite, etc.)
3. Build all binaries and web UI
4. Configure databases and services
5. Set up systemd services
6. Generate secure secrets
7. Verify the installation

**Installation time:** 10-15 minutes on a fresh system

---

## Installation Options

### Standard Installation (Recommended)
```bash
sudo ./scripts/install.sh
```
- Installs to `/opt/sagenscontact`
- Creates systemd services
- Builds from source
- Requires sudo/root

### User-Local Installation
```bash
./scripts/install.sh --no-services
```
- No sudo required
- Manual service management
- Installs to custom directory with `INSTALL_DIR=/path/to/install`

### Use Pre-Built Binaries
```bash
sudo ./scripts/install.sh --skip-build
```
- Skips compilation (use existing `target/release/` binaries)
- Faster installation
- Good for CI/CD or testing

### Custom Installation Directory
```bash
export INSTALL_DIR=/home/user/sagenscontact
export DATA_DIR=/home/user/sagenscontact-data
./scripts/install.sh --no-services
```

---

## What Gets Installed

### Prerequisites Checked and Auto-Installed

#### Rust Toolchain
- **Minimum Version:** 1.83
- **Auto-Install:** Yes (via rustup)
- **Detection:** `rustc --version`
- **Remediation:** Installs rustup, updates to stable

#### Node.js
- **Minimum Version:** 18.x
- **Auto-Install:** Yes (via NodeSource)
- **Detection:** `node --version`
- **Remediation:** Adds NodeSource repo, installs Node.js

#### pnpm
- **Minimum Version:** 8.x
- **Auto-Install:** Yes (via npm)
- **Detection:** `pnpm --version`
- **Remediation:** Runs `npm install -g pnpm`

#### SQLite
- **Minimum Version:** 3.x
- **Auto-Install:** Yes (via package manager)
- **Detection:** `sqlite3 --version`
- **Remediation:** Installs sqlite3 and development libraries

#### Build Tools
- **Components:** gcc, make, pkg-config, libssl-dev
- **Auto-Install:** Yes (via package manager)
- **Detection:** Individual command checks
- **Remediation:** Installs build-essential or equivalent

#### OpenSSL
- **Minimum Version:** 1.1.x
- **Auto-Install:** Yes (via package manager)
- **Detection:** `openssl version`
- **Remediation:** Installs openssl and development libraries

#### ClamAV (Optional)
- **Purpose:** Virus scanning for attachments
- **Auto-Install:** No (warns if missing)
- **Detection:** `clamscan --version`
- **Remediation:** Provides install command for user's OS

#### Certbot (Optional)
- **Purpose:** Let's Encrypt TLS certificates
- **Auto-Install:** No (warns if missing)
- **Detection:** `certbot --version`
- **Remediation:** Provides install command for user's OS

---

## Supported Operating Systems

### Tested & Fully Supported

| OS | Version | Package Manager | Status |
|----|---------|-----------------|--------|
| Ubuntu | 22.04 LTS | apt | ✅ Tested |
| Ubuntu | 24.04 LTS | apt | ✅ Tested |
| Debian | 11 (Bullseye) | apt | ✅ Tested |
| Debian | 12 (Bookworm) | apt | ✅ Tested |
| Fedora | 38+ | dnf | ✅ Supported |
| RHEL/Rocky/Alma | 8+ | dnf/yum | ✅ Supported |
| Arch Linux | Rolling | pacman | ✅ Supported |

### Automatic Detection

The installer automatically detects:
- Operating system (`/etc/os-release`)
- Package manager (apt, dnf, yum, pacman)
- System architecture
- Existing installations

### Unsupported Systems

If running on an unsupported OS, the installer will:
1. Clearly identify the unsupported component
2. Provide manual installation instructions
3. Exit gracefully with exit code 1

---

## Installation Process

### Phase 1: Prerequisite Checking

The installer checks and installs (if missing):

```
✓ Detecting operating system
✓ Updating package manager
✓ Checking build tools
✓ Checking Rust toolchain (>= 1.83)
✓ Checking Node.js (>= v18)
✓ Checking pnpm (>= 8)
✓ Checking SQLite
✓ Checking OpenSSL
⚠ Checking ClamAV (optional)
⚠ Checking Certbot (optional)
```

**Automatic Fixes:**
- Installs missing tools
- Updates outdated versions
- Adds required repositories
- Configures environment

**Failure Handling:**
- Provides exact commands to run
- Explains why the requirement exists
- Offers alternative approaches when possible

### Phase 2: Building and Installing

```
✓ Creating installation directories
  /opt/sagenscontact/binaries
  /opt/sagenscontact/web
  /opt/sagenscontact/data/attachments
  /opt/sagenscontact/logs
  /opt/sagenscontact/config

✓ Building SagensContact from source
  - Rust workspace (5-10 minutes)
  - Web UI (2-3 minutes)

✓ Installing binaries
  - sagenscontact (CLI)
  - sync_service (API server)
  - worker (background jobs)

✓ Installing web UI
  - Client bundle
  - Server-side rendering
```

### Phase 3: Configuration

```
✓ Generating JWT secret (32-byte random)
✓ Creating environment configuration
  /opt/sagenscontact/config/env

✓ Running database migrations
  - SQLite database created
  - Schema initialized

✓ Creating startup scripts
  - start-sync-service.sh
  - start-web-ui.sh
  - start-worker.sh
  - start-all.sh

✓ Creating systemd services
  - sagenscontact-sync.service
  - sagenscontact-web.service
  - sagenscontact-worker.service
```

### Phase 4: Verification

```
✓ Verifying binaries exist
✓ Verifying configuration files
✓ Verifying startup scripts
✓ Checking permissions
```

---

## Post-Installation

### Directory Structure

```
/opt/sagenscontact/
├── binaries/
│   ├── sagenscontact       # CLI tool
│   ├── sync_service        # API server
│   └── worker              # Background worker
├── web/
│   ├── client/             # Static assets
│   ├── server/             # SSR bundle
│   └── index.js            # Entry point
├── data/
│   ├── contacts.db         # SQLite database
│   ├── attachments/        # File uploads
│   └── backups/            # Backup storage
├── logs/
│   ├── sync-service.log
│   ├── web-ui.log
│   └── worker.log
├── config/
│   └── env                 # Environment variables
└── scripts/
    ├── start-sync-service.sh
    ├── start-web-ui.sh
    ├── start-worker.sh
    └── start-all.sh
```

### Configuration File

Location: `/opt/sagenscontact/config/env`

```bash
# Database
DATABASE_URL="sqlite:/opt/sagenscontact/data/contacts.db"

# Server ports
SYNC_SERVICE_PORT=3002
WEB_UI_PORT=3001

# Security
JWT_SECRET="<randomly-generated-32-byte-secret>"

# Logging
LOG_FORMAT=json
LOG_LEVEL=info

# Storage
ATTACHMENT_STORAGE_PATH="/opt/sagenscontact/data/attachments"
```

### Starting Services

#### Option A: systemd (Production)

```bash
# Start services
sudo systemctl start sagenscontact-sync
sudo systemctl start sagenscontact-web
sudo systemctl start sagenscontact-worker

# Enable at boot
sudo systemctl enable sagenscontact-sync sagenscontact-web sagenscontact-worker

# Check status
sudo systemctl status sagenscontact-sync

# View logs
sudo journalctl -u sagenscontact-sync -f
```

#### Option B: Manual (Development/Testing)

```bash
# Start all services at once
/opt/sagenscontact/scripts/start-all.sh

# Or start individually
/opt/sagenscontact/scripts/start-sync-service.sh &
/opt/sagenscontact/scripts/start-web-ui.sh &
/opt/sagenscontact/scripts/start-worker.sh &

# View logs
tail -f /opt/sagenscontact/logs/*.log
```

### Verification

```bash
# Health check
curl http://localhost:3002/health
# Expected: OK

# Detailed health
curl http://localhost:3002/api/health/detailed
# Expected: JSON with database status

# Web UI
curl http://localhost:3001
# Expected: HTML page

# CLI tool
sagenscontact --version
# Expected: sagenscontact 0.1.0-alpha
```

---

## Troubleshooting

### Installation Fails at Prerequisite Check

**Symptom:** Missing build tools or outdated versions

**Solution:** Installer provides exact fix commands:
```bash
# Example for Ubuntu
sudo apt-get install build-essential pkg-config libssl-dev
```

### Build Fails with "cargo: command not found"

**Symptom:** Rust not in PATH after installation

**Solution:**
```bash
source $HOME/.cargo/env
./scripts/install.sh
```

### Node.js Version Too Old

**Symptom:** Node < v18 detected

**Solution:** Installer automatically adds NodeSource repo and upgrades

### Permission Denied Errors

**Symptom:** Cannot write to /opt/sagenscontact

**Solution:**
```bash
# Run with sudo for system install
sudo ./scripts/install.sh

# OR use user-local install
export INSTALL_DIR=$HOME/sagenscontact
./scripts/install.sh --no-services
```

### Services Won't Start

**Check logs:**
```bash
# systemd
sudo journalctl -u sagenscontact-sync -n 50

# Manual logs
tail -50 /opt/sagenscontact/logs/sync-service.log
```

**Common issues:**
1. Port already in use → Change `SYNC_SERVICE_PORT` in config/env
2. Database locked → Stop all services, restart one at a time
3. Missing environment → Source `/opt/sagenscontact/config/env`

### Web UI Can't Connect to API

**Check:**
1. Sync service is running: `curl http://localhost:3002/health`
2. Firewall allows connections
3. CORS configured correctly (default: permissive)

---

## Advanced Configuration

### Using PostgreSQL Instead of SQLite

**1. Install PostgreSQL:**
```bash
sudo apt-get install postgresql postgresql-contrib
```

**2. Create database:**
```bash
sudo -u postgres createdb sagenscontact
sudo -u postgres createuser sagenscontact -P
```

**3. Update config:**
Edit `/opt/sagenscontact/config/env`:
```bash
DATABASE_URL="postgresql://sagenscontact:PASSWORD@localhost/sagenscontact"
```

**4. Restart services:**
```bash
sudo systemctl restart sagenscontact-sync
```

### Setting Up TLS/HTTPS

See `docs/TLS_HTTPS_SETUP.md` for complete guide.

**Quick Caddy setup:**
```bash
# Install Caddy
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
sudo apt update && sudo apt install caddy

# Configure Caddyfile
sudo tee /etc/caddy/Caddyfile << 'EOF'
contacts.example.com {
    reverse_proxy localhost:3001
    handle /api/* {
        reverse_proxy localhost:3002
    }
}
EOF

# Reload
sudo systemctl reload caddy
```

### Monitoring with Prometheus

**1. Install Prometheus:**
```bash
sudo apt install prometheus
```

**2. Configure scraping:**
Edit `/etc/prometheus/prometheus.yml`:
```yaml
scrape_configs:
  - job_name: 'sagenscontact'
    static_configs:
      - targets: ['localhost:3002']
    metrics_path: '/metrics'
```

**3. Restart:**
```bash
sudo systemctl restart prometheus
```

**4. Access metrics:**
- Prometheus UI: http://localhost:9090
- Raw metrics: http://localhost:3002/metrics

---

## Uninstallation

### Remove Everything

```bash
# Stop services
sudo systemctl stop sagenscontact-sync sagenscontact-web sagenscontact-worker
sudo systemctl disable sagenscontact-sync sagenscontact-web sagenscontact-worker

# Remove systemd services
sudo rm /etc/systemd/system/sagenscontact-*.service
sudo systemctl daemon-reload

# Remove installation
sudo rm -rf /opt/sagenscontact

# Remove symlinks
sudo rm /usr/local/bin/sagenscontact

# Remove service user (optional)
sudo userdel sagenscontact
```

### Keep Data, Remove Software

```bash
# Stop services
sudo systemctl stop sagenscontact-*

# Backup data
sudo cp -r /opt/sagenscontact/data /backup/location/

# Remove only binaries and web
sudo rm -rf /opt/sagenscontact/binaries
sudo rm -rf /opt/sagenscontact/web
```

---

## CI/CD Integration

### Docker Build

```bash
# Use installer in Dockerfile
FROM ubuntu:24.04

RUN apt-get update && apt-get install -y curl git
COPY . /app
WORKDIR /app
RUN ./scripts/install.sh --no-services --skip-build

CMD ["/opt/sagenscontact/scripts/start-all.sh"]
```

### GitHub Actions

```yaml
name: Build and Test

on: [push]

jobs:
  install-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run installer
        run: |
          sudo ./scripts/install.sh

      - name: Verify installation
        run: |
          sagenscontact --version
          curl http://localhost:3002/health
```

---

## Security Considerations

### Generated Secrets

The installer generates:
- **JWT_SECRET:** 32-byte random string (base64 encoded)
- Stored in: `/opt/sagenscontact/config/env`
- **Permissions:** 600 (owner read/write only)

**Important:** Change the JWT_SECRET before production deployment:
```bash
# Generate new secret
NEW_SECRET=$(openssl rand -base64 32)

# Update config
sudo sed -i "s/JWT_SECRET=.*/JWT_SECRET=\"$NEW_SECRET\"/" /opt/sagenscontact/config/env

# Restart services
sudo systemctl restart sagenscontact-sync
```

### File Permissions

The installer sets:
- Config files: 600 (owner only)
- Binaries: 755 (executable)
- Data directory: 700 (owner only)
- Service user: `sagenscontact` (limited privileges)

### Network Security

**Default configuration:**
- Services bind to localhost only
- Use reverse proxy (nginx/Caddy) for external access
- Rate limiting enabled by default
- Security headers applied automatically

**For production:** See `docs/TLS_HTTPS_SETUP.md`

---

## Installer Behavior Reference

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Prerequisite check failed |
| 2 | Build failed |
| 3 | Installation failed |
| 4 | Verification failed |

### Log Files

- **Installation log:** `/tmp/sagenscontact-install.log`
- **Service logs:** `/opt/sagenscontact/logs/`
- **systemd logs:** `journalctl -u sagenscontact-*`

### Environment Variables

The installer respects:
- `INSTALL_DIR` - Installation location
- `DATA_DIR` - Data storage location
- `LOG_FILE` - Installation log path

### Idempotency

The installer is **partially idempotent:**
- ✅ Can be re-run safely
- ✅ Skips already-installed prerequisites
- ✅ Overwrites binaries and configs
- ⚠️ Does NOT delete existing data
- ⚠️ Does NOT reset JWT_SECRET (manual change only)

---

## Support

### Getting Help

1. Check logs: `/opt/sagenscontact/logs/`
2. Review installer log: `/tmp/sagenscontact-install.log`
3. Verify prerequisites: `./scripts/install.sh --help`
4. Consult docs: `/opt/sagenscontact/docs/`

### Reporting Issues

Include in bug reports:
- OS and version (`cat /etc/os-release`)
- Installer log (`/tmp/sagenscontact-install.log`)
- Error message and exit code
- Installation command used

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 0.1.0-alpha | 2025-10-02 | Initial turnkey installer |

---

## License

Same as SagensContact project (MIT)
