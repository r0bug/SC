# SagensContact Alpha - Installation Guide

**Version:** v0.1.0-alpha
**Last Updated:** 2025-10-04
**Tested On:** Ubuntu 24.04 LTS, Fedora 40, macOS 14

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start](#quick-start)
3. [System Installation](#system-installation)
4. [User-Local Installation](#user-local-installation)
5. [Configuration](#configuration)
6. [Verification](#verification)
7. [Troubleshooting](#troubleshooting)
8. [Next Steps](#next-steps)

---

## Prerequisites

### Required Tools

SagensContact requires the following tools to be installed:

- **Rust** (1.75.0 or later)
- **Node.js** (18.0.0 or later)
- **pnpm** (8.0.0 or later)
- **SQLite** (3.35.0 or later)
- **Git** (for cloning the repository)

### System Requirements

- **OS:** Linux (Ubuntu, Fedora, Arch) or macOS
- **RAM:** 4GB minimum, 8GB recommended
- **Disk:** 2GB for source code and dependencies
- **Network:** Internet connection for downloading dependencies

---

## Quick Start

For experienced users who already have all prerequisites installed:

```bash
# Clone the repository
git clone https://github.com/r0bug/SC.git sagenscontact
cd sagenscontact

# Install dependencies and build
pnpm install
cargo build --release

# Configure
cp .env.example .env
export DATABASE_URL="sqlite:data/contacts.db"

# Run the sync service
./target/release/sync_service
```

---

## System Installation

### Option 1: Automated Install Script

The easiest way to install SagensContact system-wide:

```bash
# Download and run the installer
sudo ./scripts/install.sh

# This will:
# - Install system dependencies (Rust, Node, pnpm, SQLite)
# - Build release binaries
# - Set up systemd services
# - Configure database and directories
```

**Installation Locations:**
- Binaries: `/opt/sagenscontact/bin/`
- Data: `/opt/sagenscontact/data/`
- Config: `/opt/sagenscontact/.env`
- Logs: `/var/log/sagenscontact/` or journalctl

### Option 2: Manual System Installation

**Step 1: Install System Dependencies**

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y curl build-essential pkg-config libssl-dev sqlite3
```

**Fedora/RHEL:**
```bash
sudo dnf install -y curl gcc openssl-devel sqlite sqlite-devel
```

**Arch Linux:**
```bash
sudo pacman -S curl base-devel openssl sqlite
```

**Step 2: Install Rust**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version  # Verify installation
```

**Step 3: Install Node.js and pnpm**
```bash
# Install Node.js (using nvm recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
source ~/.bashrc
nvm install 20
node --version  # Verify

# Install pnpm
npm install -g pnpm
pnpm --version  # Verify
```

**Step 4: Clone and Build**
```bash
# Clone repository
git clone https://github.com/r0bug/SC.git sagenscontact
cd sagenscontact

# Install Node dependencies
pnpm install

# Build Rust binaries (release mode)
cargo build --release

# Verify binaries
ls -lh target/release/sync_service
```

**Step 5: Configure**
```bash
# Create data directory
sudo mkdir -p /opt/sagenscontact/data/attachments
sudo chown -R $USER:$USER /opt/sagenscontact

# Set up environment
cat > /opt/sagenscontact/.env <<EOF
DATABASE_URL=sqlite:/opt/sagenscontact/data/contacts.db
JWT_SECRET=$(openssl rand -base64 32)
LOG_LEVEL=info
LOG_FORMAT=json
PORT=3000
EOF
```

**Step 6: Set Up Systemd Service**
```bash
# Create service file
sudo tee /etc/systemd/system/sagenscontact.service > /dev/null <<EOF
[Unit]
Description=SagensContact Sync Service
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=/opt/sagenscontact
EnvironmentFile=/opt/sagenscontact/.env
ExecStart=/opt/sagenscontact/bin/sync_service
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable sagenscontact
sudo systemctl start sagenscontact
sudo systemctl status sagenscontact
```

---

## User-Local Installation

For development or single-user setups without system-wide installation:

**Step 1: Install User Dependencies**

Install Rust and Node.js using the same commands as above, but without sudo.

**Step 2: Clone and Build**
```bash
# Clone to your home directory
cd ~
git clone https://github.com/r0bug/SC.git sagenscontact
cd sagenscontact

# Install and build
pnpm install
cargo build --release
```

**Step 3: Configure for Local Use**
```bash
# Create local data directory
mkdir -p ~/.sagenscontact/data/attachments

# Set up environment
cat > ~/.sagenscontact/.env <<EOF
DATABASE_URL=sqlite:$HOME/.sagenscontact/data/contacts.db
JWT_SECRET=$(openssl rand -base64 32)
LOG_LEVEL=info
PORT=3000
EOF

# Export for current session
export $(cat ~/.sagenscontact/.env | xargs)
```

**Step 4: Run Manually**
```bash
# Start the sync service
./target/release/sync_service

# Or in the background
nohup ./target/release/sync_service > ~/.sagenscontact/logs/sync.log 2>&1 &
```

**Optional: Add to Shell Profile**
```bash
# Add to ~/.bashrc or ~/.zshrc
export SAGENSCONTACT_HOME="$HOME/.sagenscontact"
export DATABASE_URL="sqlite:$SAGENSCONTACT_HOME/data/contacts.db"
export PORT=3000
alias sagenscontact="$HOME/sagenscontact/target/release/sync_service"
```

---

## Configuration

### Environment Variables

Create a `.env` file with the following variables:

```bash
# Database
DATABASE_URL=sqlite:data/contacts.db

# Security
JWT_SECRET=your-secret-key-here  # Generate with: openssl rand -base64 32
JWT_EXPIRATION=86400              # 24 hours in seconds

# Server
PORT=3000
HOST=127.0.0.1

# Logging
LOG_LEVEL=info                    # debug, info, warn, error
LOG_FORMAT=json                   # json or pretty

# Features
ENABLE_METRICS=true
ENABLE_TRACING=true
```

### Database Setup

The database is automatically created on first run. To manually initialize:

```bash
# Set database path
export DATABASE_URL="sqlite:data/contacts.db"

# Run migrations (automatic on first start)
./target/release/sync_service --migrate

# Or use SQLite directly
sqlite3 data/contacts.db < crates/sync_service/migrations/001_initial.sql
```

### Web UI Setup (Optional)

To run the web interface:

```bash
# Navigate to web app
cd apps/web

# Install dependencies
pnpm install

# Development mode
pnpm dev

# Production build
pnpm build
pnpm preview
```

**Web UI will be available at:** `http://localhost:3001`

---

## Verification

### Step 1: Check Service Status

**Systemd Installation:**
```bash
sudo systemctl status sagenscontact
journalctl -u sagenscontact -f  # View logs
```

**Manual Installation:**
```bash
ps aux | grep sync_service
curl http://localhost:3000/health
```

### Step 2: Test API Endpoints

```bash
# Health check
curl http://localhost:3000/health
# Expected: "OK"

# Detailed health
curl http://localhost:3000/api/health/detailed | jq
# Expected: JSON with status, version, database info

# Metrics
curl http://localhost:3000/metrics
# Expected: Prometheus metrics

# List import connectors
curl http://localhost:3000/api/import/connectors | jq
# Expected: Array of 9 connector definitions
```

### Step 3: Test Import Functionality

```bash
# Download sample data
curl -O https://example.com/contacts_sample.csv

# Test import preview
curl -X POST http://localhost:3000/api/import/preview?limit=5 \
  -F "file=@contacts_sample.csv"

# Execute import
curl -X POST http://localhost:3000/api/import/execute \
  -F "file=@contacts_sample.csv" \
  -F 'config={"connector_id":"generic_csv","dedupe_strategy":"skip"}'

# Check job status
curl http://localhost:3000/api/import/jobs/{job_id}
```

### Step 4: Run Test Suite (Optional)

```bash
# Unit tests
cargo test --workspace

# Integration tests
cargo test --test basic_integration

# Performance baseline
./test_simple_baseline.sh

# Security audit
./scripts/testing/security_audit.sh

# Full verification
./scripts/testing/verify_installer.sh
```

---

## Troubleshooting

### Common Issues

#### 1. Service Won't Start

**Symptom:** `systemctl status sagenscontact` shows failed
**Solution:**
```bash
# Check logs
journalctl -u sagenscontact -n 50

# Common causes:
# - Port already in use: Change PORT in .env
# - Database permissions: Check data/ directory ownership
# - Missing dependencies: Reinstall with install script
```

#### 2. Database Connection Error

**Symptom:** "Cannot connect to database"
**Solution:**
```bash
# Verify DATABASE_URL
echo $DATABASE_URL

# Check database file exists
ls -l data/contacts.db

# Check permissions
chmod 644 data/contacts.db
chmod 755 data/

# Recreate database
rm data/contacts.db
./target/release/sync_service  # Will auto-create
```

#### 3. Import Fails with Large Files

**Symptom:** "Error parsing multipart/form-data request"
**Solution:**
- Ensure you're running v0.1.0-alpha or later (has 50MB body limit)
- Check file size: `ls -lh your_file.csv`
- For files >50MB, split into smaller chunks

#### 4. Build Errors

**Symptom:** `cargo build` fails
**Solution:**
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release

# Check Rust version (need 1.75+)
rustc --version
```

#### 5. Port Already in Use

**Symptom:** "Address already in use (os error 98)"
**Solution:**
```bash
# Find process using port 3000
sudo lsof -i :3000

# Kill process or change port
export PORT=3002
```

### Getting Help

- **Documentation:** See `docs/` directory
- **Issues:** https://github.com/r0bug/SC/issues
- **Logs:** Check systemd journal or app logs
- **Debug Mode:** Set `LOG_LEVEL=debug` in .env

---

## Next Steps

After successful installation:

### 1. Import Your Data

```bash
# Use the import API
curl -X POST http://localhost:3000/api/import/execute \
  -F "file=@your_contacts.csv" \
  -F 'config={"connector_id":"generic_csv"}'
```

### 2. Explore the API

- Browse API documentation: `docs/API.md`
- Test endpoints: Use Postman or curl
- Check metrics: `http://localhost:3000/metrics`

### 3. Set Up Web UI

```bash
cd apps/web
pnpm install
pnpm dev  # Development mode
```

Access at: `http://localhost:3001`

### 4. Configure Production

For production deployments:

- Set up HTTPS/TLS
- Configure reverse proxy (nginx/caddy)
- Set up backup schedules
- Enable monitoring (Prometheus/Grafana)
- Review security settings

### 5. Read Documentation

- **Sprint Summary:** `docs/SPRINT_SUMMARY.md`
- **Integration Report:** `docs/WEEK_5_INTEGRATION_REPORT.md`
- **Testing Guide:** `scripts/testing/README.md`
- **Verification Report:** `docs/INSTALLER_VERIFICATION.md`

---

## Quick Reference

### Essential Commands

```bash
# Start service
sudo systemctl start sagenscontact

# Stop service
sudo systemctl stop sagenscontact

# View logs
journalctl -u sagenscontact -f

# Rebuild
cargo build --release

# Run tests
cargo test --workspace

# Check health
curl http://localhost:3000/health
```

### File Locations

| Item | System Install | User Install |
|------|---------------|--------------|
| Binaries | `/opt/sagenscontact/bin/` | `~/sagenscontact/target/release/` |
| Data | `/opt/sagenscontact/data/` | `~/.sagenscontact/data/` |
| Config | `/opt/sagenscontact/.env` | `~/.sagenscontact/.env` |
| Logs | `/var/log/sagenscontact/` | `~/.sagenscontact/logs/` |

### Performance Baselines

Tested on Ubuntu 24.04 with i7 CPU, 16GB RAM:

| Dataset | Records | Processing Time | Throughput |
|---------|---------|----------------|------------|
| 10k | 10,000 | 0.017s | 596k/sec |
| 50k (with dupes) | 50,000 | 0.093s | 539k/sec |
| 100k | 100,000 | 0.168s | 595k/sec |

---

## Support

**For installation issues:**
1. Check this guide's troubleshooting section
2. Review logs: `journalctl -u sagenscontact -n 100`
3. Run verification: `./scripts/testing/verify_installer.sh`
4. Create issue: https://github.com/r0bug/SC/issues

**For development:**
- See `docs/DEVELOPMENT.md`
- Run tests: `cargo test`
- Code style: `cargo fmt --check`

---

**Installation Complete!** 🎉

Your SagensContact instance should now be running. Visit `http://localhost:3000/health` to verify.
