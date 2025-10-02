# SagensContact - Installation Guide

## 🚀 Quick Install (Recommended)

**Single command - online installation:**
```bash
curl -sSL https://raw.githubusercontent.com/r0bug/SC/main/sagenscontact-setup.sh | sudo bash
```

This will:
- Download the source code
- Install all prerequisites automatically
- Build and configure everything
- Set up systemd services
- **Time:** 15-20 minutes

---

## 📦 Alternative Installation Methods

### Method 1: Self-Contained Bundle (Offline)

Download the complete bundle (works without internet):
```bash
# Download from releases
wget https://github.com/r0bug/SC/releases/download/v0.1.0-alpha/sagenscontact-installer-bundle.sh

# Or if you have it locally
sudo ./sagenscontact-installer-bundle.sh
```

**Size:** ~114MB (includes all source code)
**Requirements:** None (completely self-contained)

### Method 2: From Source

```bash
git clone https://github.com/r0bug/SC.git
cd SC
sudo ./scripts/install.sh
```

### Method 3: Skip Build (Use Pre-Compiled)

If you already have binaries built:
```bash
sudo ./scripts/install.sh --skip-build
```

---

## 📋 What Gets Installed

### Prerequisites (Auto-Installed)
- ✅ Rust toolchain (>= 1.83)
- ✅ Node.js (>= v18) + pnpm
- ✅ SQLite 3.x
- ✅ Build tools (gcc, make, pkg-config)
- ✅ OpenSSL

### Application Components
- **CLI Tool:** `sagenscontact` - Import and management commands
- **API Server:** `sync_service` - REST API on port 3002
- **Web UI:** SvelteKit app on port 3001
- **Background Worker:** For communication and scheduled tasks

### System Integration
- **Location:** `/opt/sagenscontact`
- **Data:** `/opt/sagenscontact/data`
- **Services:** 3 systemd units
- **User:** Dedicated `sagenscontact` service account

---

## ⚡ After Installation

### Start Services

```bash
# Start all services
sudo systemctl start sagenscontact-sync sagenscontact-web sagenscontact-worker

# Enable at boot
sudo systemctl enable sagenscontact-{sync,web,worker}

# Check status
sudo systemctl status sagenscontact-sync
```

### Access Points

- **Web UI:** http://localhost:3001
- **API:** http://localhost:3002
- **Health:** http://localhost:3002/health
- **Metrics:** http://localhost:3002/metrics

### Import Data

```bash
# Import SMS backup
sagenscontact import-sms /path/to/backup.xml

# Import CSV contacts
sagenscontact import-csv /path/to/contacts.csv

# Import vCard
sagenscontact import-vcard /path/to/contacts.vcf
```

---

## 🔧 Configuration

Edit `/opt/sagenscontact/config/env`:

```bash
# Database
DATABASE_URL="sqlite:/opt/sagenscontact/data/contacts.db"

# Ports
SYNC_SERVICE_PORT=3002
WEB_UI_PORT=3001

# Security (CHANGE THIS!)
JWT_SECRET="<generated-secret>"

# Logging
LOG_FORMAT=json
LOG_LEVEL=info

# Storage
ATTACHMENT_STORAGE_PATH="/opt/sagenscontact/data/attachments"
```

### Production Checklist

- [ ] Change JWT_SECRET: `openssl rand -base64 32`
- [ ] Set up TLS reverse proxy (nginx/Caddy)
- [ ] Configure firewall
- [ ] Enable log aggregation
- [ ] Set up Prometheus monitoring
- [ ] Configure backups

See `docs/DEPLOYMENT_GUIDE.md` for complete production setup.

---

## 🐛 Troubleshooting

### Services Won't Start

```bash
# Check logs
sudo journalctl -u sagenscontact-sync -n 50 -f

# Or file logs
tail -f /opt/sagenscontact/logs/*.log
```

### Port Already in Use

```bash
# Check what's using the port
sudo lsof -i :3002

# Change port in config
sudo nano /opt/sagenscontact/config/env
# Then restart: sudo systemctl restart sagenscontact-sync
```

### Permission Errors

```bash
# Fix ownership
sudo chown -R sagenscontact:sagenscontact /opt/sagenscontact
```

### Build Errors

Check:
- Disk space (need ~2GB): `df -h`
- Build log: `/tmp/sagenscontact-install-*.log`
- Prerequisites: `rustc --version`, `node --version`

---

## 📚 Documentation

- **INSTALLER_GUIDE.md** - Complete installation reference
- **DEPLOYMENT_GUIDE.md** - Production deployment
- **TLS_HTTPS_SETUP.md** - SSL/TLS configuration
- **QUICKSTART.md** - Getting started guide
- **ARCHITECTURE.md** - System architecture

---

## 🆘 Support

**Issues:** https://github.com/r0bug/SC/issues
**Email:** john@robug.com
**Logs:** `/tmp/sagenscontact-install-*.log`

---

## 🎯 Supported Systems

| OS | Version | Status |
|----|---------|--------|
| Ubuntu | 22.04+ | ✅ Fully tested |
| Debian | 11+ | ✅ Fully tested |
| Fedora | 38+ | ✅ Supported |
| RHEL/Rocky | 8+ | ✅ Supported |
| Arch Linux | Rolling | ✅ Supported |

---

## 🔐 Security Notes

- **Default:** Services bind to localhost only
- **Production:** Use reverse proxy (nginx/Caddy) for TLS
- **JWT Secret:** Auto-generated, change before production
- **Rate Limiting:** Enabled by default
- **Security Headers:** Automatically applied

See `docs/TLS_HTTPS_SETUP.md` for production security setup.

---

## 📦 Uninstall

```bash
# Stop services
sudo systemctl stop sagenscontact-{sync,web,worker}
sudo systemctl disable sagenscontact-{sync,web,worker}

# Remove systemd units
sudo rm /etc/systemd/system/sagenscontact-*.service
sudo systemctl daemon-reload

# Remove installation
sudo rm -rf /opt/sagenscontact

# Remove CLI symlink
sudo rm /usr/local/bin/sagenscontact

# Remove service user
sudo userdel sagenscontact
```

---

**Version:** 0.1.0-alpha
**License:** MIT
**Repository:** https://github.com/r0bug/SC
