# SagensContact Turnkey Installer - Summary

**Date:** October 2, 2025
**Version:** 0.1.0-alpha
**Status:** ✅ Ready for Testing

---

## Overview

A fully automated, zero-configuration installer that handles everything from detecting prerequisites to running services. Designed for fresh systems with minimal manual intervention.

**Single command installation:**
```bash
sudo ./scripts/install.sh
```

---

## Key Features

### ✅ Comprehensive Prerequisite Management

**Automatically detects and installs:**
- Rust toolchain (>= 1.83)
- Node.js (>= 18.x) via NodeSource
- pnpm (>= 8.x)
- SQLite 3.x with development libraries
- Build tools (gcc, make, pkg-config, libssl-dev)
- OpenSSL

**Provides clear remediation for:**
- ClamAV (optional virus scanning)
- Certbot (optional Let's Encrypt)
- Version mismatches
- Missing dependencies

### ✅ Smart Version Detection

- Compares installed versions against minimums
- Auto-upgrades when possible
- Provides exact install commands when manual action needed
- Sources Rust environment automatically after installation

### ✅ Multi-OS Support

**Supported package managers:**
- apt (Ubuntu, Debian)
- dnf (Fedora, RHEL 9+)
- yum (RHEL 8, CentOS)
- pacman (Arch Linux)

**Auto-detects:**
- OS distribution (`/etc/os-release`)
- Available package manager
- System architecture

### ✅ Secure by Default

**Automatically generates:**
- JWT_SECRET (32-byte cryptographically random)
- Environment configuration with sane defaults
- Proper file permissions (config: 600, binaries: 755)

**Creates dedicated service user:**
- Limited privileges
- No shell access
- Owns only necessary directories

### ✅ Complete Service Setup

**Installs:**
- sagenscontact CLI (with `/usr/local/bin` symlink)
- sync_service API server
- worker background processor
- Web UI (SvelteKit SSR build)

**Creates systemd services:**
- `sagenscontact-sync.service`
- `sagenscontact-web.service`
- `sagenscontact-worker.service`
- Auto-restart on failure
- Centralized logging

**Creates startup scripts:**
- Individual service scripts
- Combined `start-all.sh` for development
- Environment sourcing built-in

### ✅ Flexible Installation Modes

| Mode | Command | Use Case |
|------|---------|----------|
| **Full (recommended)** | `sudo ./scripts/install.sh` | Production deployment |
| **User-local** | `./scripts/install.sh --no-services` | Development/testing |
| **Skip build** | `sudo ./scripts/install.sh --skip-build` | Use pre-compiled binaries |
| **Custom directory** | `INSTALL_DIR=/path ./scripts/install.sh` | Non-standard layout |

### ✅ Comprehensive Verification

**Installer verifies:**
- All binaries exist and are executable
- Configuration file created with required variables
- Database can be accessed
- Scripts have correct permissions
- Directory structure complete

**Exit codes:**
- 0 = Success
- 1 = Prerequisite check failed
- 2 = Build failed
- 3 = Installation failed
- 4 = Verification failed

---

## Installation Process

### Phase 1: Prerequisite Checking (2-5 minutes)

```
✓ Detecting operating system
✓ Updating package manager
✓ Checking build tools
✓ Checking Rust toolchain
✓ Checking Node.js
✓ Checking pnpm
✓ Checking SQLite
✓ Checking TLS tools
⚠ Checking ClamAV (optional)
```

### Phase 2: Building and Installing (10-15 minutes)

```
✓ Creating installation directories
✓ Building SagensContact from source
  - Rust workspace compilation
  - Web UI build
✓ Installing binaries
✓ Installing web UI
```

### Phase 3: Configuration (< 1 minute)

```
✓ Generating JWT secret
✓ Creating environment configuration
✓ Running database migrations
✓ Creating startup scripts
✓ Creating systemd services
```

### Phase 4: Verification (< 10 seconds)

```
✓ Verifying binaries
✓ Verifying configuration
✓ Verifying startup scripts
✓ Checking permissions
```

**Total time:** ~15-20 minutes on a fresh system

---

## What Gets Installed

### Directory Structure

```
/opt/sagenscontact/
├── binaries/
│   ├── sagenscontact       (CLI - ~62MB)
│   ├── sync_service        (API - ~71MB)
│   └── worker              (Worker - ~68MB)
├── web/
│   ├── client/             (Static assets)
│   ├── server/             (SSR bundle)
│   └── index.js
├── data/
│   ├── contacts.db         (SQLite database)
│   ├── attachments/        (File storage)
│   └── backups/
├── logs/
│   ├── sync-service.log
│   ├── web-ui.log
│   └── worker.log
├── config/
│   └── env                 (Environment variables)
└── scripts/
    ├── start-sync-service.sh
    ├── start-web-ui.sh
    ├── start-worker.sh
    └── start-all.sh
```

### Configuration Generated

`/opt/sagenscontact/config/env`:
```bash
DATABASE_URL="sqlite:/opt/sagenscontact/data/contacts.db"
SYNC_SERVICE_PORT=3002
WEB_UI_PORT=3001
JWT_SECRET="<generated-32-byte-secret>"
LOG_FORMAT=json
LOG_LEVEL=info
ATTACHMENT_STORAGE_PATH="/opt/sagenscontact/data/attachments"
```

### System Integration

- **Symlink:** `/usr/local/bin/sagenscontact` → CLI
- **Services:** 3 systemd units in `/etc/systemd/system/`
- **User:** `sagenscontact` system user (no login)
- **Permissions:** Config files 600, binaries 755, data 700

---

## Usage After Installation

### Quick Start

```bash
# Start all services
sudo systemctl start sagenscontact-sync sagenscontact-web sagenscontact-worker

# Enable at boot
sudo systemctl enable sagenscontact-{sync,web,worker}

# Check status
sudo systemctl status sagenscontact-sync

# View logs
sudo journalctl -u sagenscontact-sync -f
```

### Access Points

- **Web UI:** http://localhost:3001
- **API:** http://localhost:3002
- **Health:** http://localhost:3002/health
- **Metrics:** http://localhost:3002/metrics

### CLI Usage

```bash
# Import SMS backup
sagenscontact import-sms /path/to/backup.xml

# Import CSV contacts
sagenscontact import-csv /path/to/contacts.csv

# Check version
sagenscontact --version
```

---

## Installer Behavior

### Automatic Fixes

The installer automatically handles:

1. **Missing Rust** → Installs rustup, configures stable toolchain
2. **Outdated Node** → Adds NodeSource repo, upgrades to v20
3. **Missing pnpm** → Runs `npm install -g pnpm`
4. **Missing build tools** → Installs build-essential/equivalent
5. **Wrong permissions** → Sets correct file modes
6. **Missing directories** → Creates full structure
7. **No JWT secret** → Generates cryptographically secure token

### Manual Intervention Required

The installer **cannot** automatically fix:

1. **Unsupported OS** → Provides manual instructions
2. **Network issues** → Retry or check connectivity
3. **Disk space** → Free up space (needs ~2GB)
4. **Conflicting services** → Stop services on ports 3001/3002
5. **ClamAV/Certbot** → Optional, provides install commands

### Idempotency

The installer is **safe to re-run:**
- Skips already-installed prerequisites
- Overwrites binaries and configs (preserves data)
- Does NOT reset JWT_SECRET (manual change only)
- Updates systemd services
- Preserves database and attachments

---

## Testing and Verification

### Verification Script

Included: `scripts/verify-installer.sh`

**Tests:**
- Installer help works
- Prerequisites detected correctly
- User-local installation succeeds
- Directory structure created
- Configuration file generated
- JWT secret has proper length
- Startup scripts executable
- Binaries installed and work
- File permissions correct

**Run:**
```bash
./scripts/verify-installer.sh
```

### Manual Verification

```bash
# Test health endpoint
curl http://localhost:3002/health

# Test detailed health
curl http://localhost:3002/api/health/detailed

# Test web UI
curl -I http://localhost:3001

# Test CLI
sagenscontact --version

# Test rate limiting (15 rapid requests should hit 429)
for i in {1..15}; do
  curl -w "%{http_code}\n" -o /dev/null -s \
    -X POST http://localhost:3002/api/auth/login \
    -H "Content-Type: application/json" \
    -d '{"email":"test@test.com","password":"test"}'
done

# Test security headers
curl -I http://localhost:3002/health | grep -E "(Strict-Transport|Content-Security)"
```

---

## Fresh System Requirements

### Minimum Requirements

- **OS:** Ubuntu 22.04+, Debian 11+, Fedora 38+, RHEL 8+, Arch Linux
- **RAM:** 2GB minimum, 4GB recommended
- **Disk:** 2GB free space for build + binaries
- **Network:** Internet connection for downloading dependencies
- **Permissions:** sudo/root for system-wide install

### What Doesn't Need Pre-Installation

❌ Rust (installer handles it)
❌ Node.js (installer handles it)
❌ pnpm (installer handles it)
❌ SQLite (installer handles it)
❌ Build tools (installer handles it)

### What You Should Have

✅ Supported Linux distribution
✅ sudo/root access (for system install)
✅ Internet connectivity
✅ Basic shell (bash)
✅ curl or wget (for Rust installation)

---

## Comparison to Manual Installation

### Manual Process (Old Way)

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 2. Install Node.js
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo bash -
sudo apt-get install -y nodejs

# 3. Install pnpm
npm install -g pnpm

# 4. Install dependencies
sudo apt-get install -y sqlite3 libsqlite3-dev build-essential pkg-config libssl-dev

# 5. Clone and build
git clone https://github.com/your-org/sagenscontact.git
cd sagenscontact
cargo build --release
cd apps/web && pnpm install && pnpm build && cd ../..

# 6. Create directories
sudo mkdir -p /opt/sagenscontact/{binaries,web,data,logs,config}

# 7. Copy files
sudo cp target/release/* /opt/sagenscontact/binaries/
sudo cp -r apps/web/.svelte-kit/output/* /opt/sagenscontact/web/

# 8. Create config
sudo tee /opt/sagenscontact/config/env << EOF
DATABASE_URL=sqlite:/opt/sagenscontact/data/contacts.db
JWT_SECRET=$(openssl rand -base64 32)
EOF

# 9. Create systemd services (3 separate files)
...

# 10. Start services
sudo systemctl daemon-reload
sudo systemctl start sagenscontact-sync
```

**Time:** 30-45 minutes + troubleshooting

### Turnkey Installer (New Way)

```bash
sudo ./scripts/install.sh
```

**Time:** 15-20 minutes (fully automated)

---

## Troubleshooting

### Common Issues

**"Not running as root"**
- Solution: Run with `sudo` or use `--no-services` for user-local install

**"cargo: command not found" after Rust install**
- Solution: Source cargo env: `source $HOME/.cargo/env`
- Installer does this automatically, but may need manual source in some shells

**"Port already in use"**
- Solution: Stop conflicting services or change ports in `/opt/sagenscontact/config/env`

**"Permission denied" on /opt**
- Solution: Run with sudo for system install, or use `INSTALL_DIR=$HOME/sagenscontact`

**"Build failed" errors**
- Check: Disk space (need ~2GB)
- Check: Build log at `/tmp/sagenscontact-install.log`
- Try: Re-run installer (it's idempotent)

---

## Documentation

### Complete Guides

1. **INSTALLER_GUIDE.md** - Comprehensive installer documentation
   - All installation options
   - OS-specific instructions
   - Troubleshooting guide
   - Advanced configuration

2. **DEPLOYMENT_GUIDE.md** - Production deployment
   - TLS/HTTPS setup
   - Monitoring with Prometheus
   - Backup procedures
   - Scaling considerations

3. **TLS_HTTPS_SETUP.md** - Certificate management
   - Nginx configuration
   - Caddy configuration
   - Let's Encrypt setup
   - Certificate renewal

---

## Security Notes

### Generated Secrets

- **JWT_SECRET:** 32-byte random, base64-encoded
- **Stored:** `/opt/sagenscontact/config/env` (mode 600)
- **Rotation:** Manual (change before production)

### Service User

- **Username:** `sagenscontact`
- **Shell:** `/bin/false` (no login)
- **Ownership:** `/opt/sagenscontact` only
- **Purpose:** Least-privilege service execution

### Network Security

- **Default:** Services bind to localhost
- **Production:** Use reverse proxy (nginx/Caddy)
- **TLS:** Not built-in, handled by proxy
- **Rate Limiting:** Enabled by default

---

## Conclusion

The SagensContact turnkey installer provides:

✅ **Zero-configuration setup** - Works on fresh systems
✅ **Automatic dependency resolution** - Installs everything needed
✅ **Clear error messages** - Exact remediation steps
✅ **Secure defaults** - Generated secrets, proper permissions
✅ **Production-ready services** - systemd integration
✅ **Comprehensive verification** - Tests before completion

**No manual troubleshooting required** for supported operating systems.

---

**Installation Support:**
- Review: `docs/INSTALLER_GUIDE.md`
- Logs: `/tmp/sagenscontact-install.log`
- Verify: `./scripts/verify-installer.sh`
- Test: Run installer in Docker/VM first

**Ready for QA testing on fresh Ubuntu 24.04 LTS systems.**
