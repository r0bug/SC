#!/bin/bash
# Build All Components for SagensContact Alpha
# This script compiles all backend binaries and frontend applications

set -e  # Exit on error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/artifacts/phase6-build"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo "=========================================="
echo "SagensContact Alpha - Build All Components"
echo "=========================================="
echo "Project Root: $PROJECT_ROOT"
echo "Build Directory: $BUILD_DIR"
echo "Timestamp: $TIMESTAMP"
echo ""

# Clean previous build
if [ -d "$BUILD_DIR" ]; then
    echo "Cleaning previous build..."
    rm -rf "$BUILD_DIR"
fi

mkdir -p "$BUILD_DIR"/{bin,config,sample_data,scripts,docs,web}

# Step 1: Build Rust binaries
echo ""
echo "=========================================="
echo "Step 1: Building Rust binaries..."
echo "=========================================="
cd "$PROJECT_ROOT"

echo "Building sagenscontact CLI..."
~/.cargo/bin/cargo build --release --bin sagenscontact
cp target/release/sagenscontact "$BUILD_DIR/bin/"

echo "Building sync_service..."
~/.cargo/bin/cargo build --release --bin sync_service
cp target/release/sync_service "$BUILD_DIR/bin/"

echo "Building worker..."
~/.cargo/bin/cargo build --release --bin worker
cp target/release/worker "$BUILD_DIR/bin/"

echo "✓ Rust binaries built successfully"

# Step 2: Build Web UI
echo ""
echo "=========================================="
echo "Step 2: Building Web UI..."
echo "=========================================="
cd "$PROJECT_ROOT/apps/web"

if [ ! -d "node_modules" ]; then
    echo "Installing web dependencies..."
    pnpm install
fi

echo "Building web application..."
pnpm build

cp -r build "$BUILD_DIR/web/"
cp package.json "$BUILD_DIR/web/"
cp -r static "$BUILD_DIR/web/" 2>/dev/null || true

echo "✓ Web UI built successfully"

# Step 3: Copy configuration templates
echo ""
echo "=========================================="
echo "Step 3: Copying configuration files..."
echo "=========================================="
cd "$PROJECT_ROOT"

cp config/credentials.toml.example "$BUILD_DIR/config/credentials.toml"
cp -r sample_data/*.csv "$BUILD_DIR/sample_data/" 2>/dev/null || true
cp -r sample_data/*.vcf "$BUILD_DIR/sample_data/" 2>/dev/null || true
cp -r sample_data/*.json "$BUILD_DIR/sample_data/" 2>/dev/null || true

echo "✓ Configuration files copied"

# Step 4: Copy scripts
echo ""
echo "=========================================="
echo "Step 4: Copying deployment scripts..."
echo "=========================================="

cat > "$BUILD_DIR/scripts/start_sync_service.sh" << 'EOF'
#!/bin/bash
# Start SagensContact Sync Service
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

export DATABASE_URL="${DATABASE_URL:-sqlite:$APP_ROOT/data/sagenscontact.db}"
export ATTACHMENT_STORAGE_PATH="${ATTACHMENT_STORAGE_PATH:-$APP_ROOT/data/attachments}"
export PORT="${PORT:-3000}"

mkdir -p "$APP_ROOT/data/attachments"

echo "Starting SagensContact Sync Service..."
echo "Database: $DATABASE_URL"
echo "Attachments: $ATTACHMENT_STORAGE_PATH"
echo "Port: $PORT"
echo ""

cd "$APP_ROOT"
exec "$APP_ROOT/bin/sync_service"
EOF

cat > "$BUILD_DIR/scripts/start_web.sh" << 'EOF'
#!/bin/bash
# Start SagensContact Web UI
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

export PORT="${PORT:-3001}"
export API_URL="${API_URL:-http://localhost:3000}"

cd "$APP_ROOT/web"

echo "Starting SagensContact Web UI..."
echo "Port: $PORT"
echo "API URL: $API_URL"
echo ""

# Check if node is available
if ! command -v node &> /dev/null; then
    echo "Error: Node.js is not installed"
    echo "Install Node.js 20+ from https://nodejs.org/"
    exit 1
fi

# Serve the built application
npx sirv build --port $PORT --host 0.0.0.0
EOF

cat > "$BUILD_DIR/scripts/start_worker.sh" << 'EOF'
#!/bin/bash
# Start SagensContact Background Worker
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

export DATABASE_URL="${DATABASE_URL:-sqlite:$APP_ROOT/data/sagenscontact.db}"

echo "Starting SagensContact Background Worker..."
echo "Database: $DATABASE_URL"
echo ""

cd "$APP_ROOT"
exec "$APP_ROOT/bin/worker"
EOF

cat > "$BUILD_DIR/scripts/verify_installation.sh" << 'EOF'
#!/bin/bash
# Verify SagensContact Installation
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=========================================="
echo "SagensContact Installation Verification"
echo "=========================================="
echo ""

# Check binaries
echo "Checking binaries..."
BINARIES=("sagenscontact" "sync_service" "worker")
for bin in "${BINARIES[@]}"; do
    if [ -f "$APP_ROOT/bin/$bin" ] && [ -x "$APP_ROOT/bin/$bin" ]; then
        echo "  ✓ $bin"
    else
        echo "  ✗ $bin (missing or not executable)"
        exit 1
    fi
done

# Check web build
echo ""
echo "Checking web build..."
if [ -d "$APP_ROOT/web/build" ]; then
    echo "  ✓ Web build exists"
else
    echo "  ✗ Web build missing"
    exit 1
fi

# Check configuration
echo ""
echo "Checking configuration..."
if [ -f "$APP_ROOT/config/credentials.toml" ]; then
    echo "  ✓ credentials.toml exists"
else
    echo "  ✗ credentials.toml missing"
    exit 1
fi

# Check directories
echo ""
echo "Checking directories..."
mkdir -p "$APP_ROOT/data/attachments"
echo "  ✓ data/attachments created"

# Test CLI
echo ""
echo "Testing CLI..."
if "$APP_ROOT/bin/sagenscontact" --help &> /dev/null; then
    echo "  ✓ CLI executable works"
else
    echo "  ✗ CLI failed to execute"
    exit 1
fi

echo ""
echo "=========================================="
echo "✓ Installation verification complete!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "1. Review config/credentials.toml"
echo "2. Import sample data: ./bin/sagenscontact import --csv sample_data/contacts.csv"
echo "3. Start sync service: ./scripts/start_sync_service.sh"
echo "4. Start web UI: ./scripts/start_web.sh"
echo "5. Access at http://localhost:3001"
EOF

chmod +x "$BUILD_DIR/scripts"/*.sh

echo "✓ Deployment scripts created"

# Step 5: Copy documentation
echo ""
echo "=========================================="
echo "Step 5: Copying documentation..."
echo "=========================================="

cp "$PROJECT_ROOT/README.md" "$BUILD_DIR/docs/"
cp "$PROJECT_ROOT/QUICKSTART.md" "$BUILD_DIR/docs/" 2>/dev/null || true
cp "$PROJECT_ROOT/TESTING.md" "$BUILD_DIR/docs/"
cp "$PROJECT_ROOT/ARCHITECTURE.md" "$BUILD_DIR/docs/"
cp "$PROJECT_ROOT/SECURITY_NOTES.md" "$BUILD_DIR/docs/" 2>/dev/null || true

echo "✓ Documentation copied"

# Step 6: Create README for deployment
cat > "$BUILD_DIR/README.md" << 'EOF'
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
EOF

echo "✓ Deployment README created"

# Step 7: Create archive
echo ""
echo "=========================================="
echo "Step 7: Creating distribution archive..."
echo "=========================================="

cd "$PROJECT_ROOT/artifacts"
ARCHIVE_NAME="sagenscontact-alpha-phase6-$TIMESTAMP.tar.gz"

tar -czf "$ARCHIVE_NAME" phase6-build/

ARCHIVE_SIZE=$(du -h "$ARCHIVE_NAME" | cut -f1)

echo "✓ Archive created: $ARCHIVE_NAME ($ARCHIVE_SIZE)"

# Create latest symlink
ln -sf "$ARCHIVE_NAME" sagenscontact-alpha-phase6-latest.tar.gz

# Generate checksums
sha256sum "$ARCHIVE_NAME" > "$ARCHIVE_NAME.sha256"

echo ""
echo "=========================================="
echo "✓ Build Complete!"
echo "=========================================="
echo ""
echo "Distribution package: artifacts/$ARCHIVE_NAME"
echo "Size: $ARCHIVE_SIZE"
echo "Checksum: $(cat "$ARCHIVE_NAME.sha256")"
echo ""
echo "To deploy on another machine:"
echo "  1. Copy artifacts/$ARCHIVE_NAME to target machine"
echo "  2. tar -xzf $ARCHIVE_NAME"
echo "  3. cd phase6-build"
echo "  4. ./scripts/verify_installation.sh"
echo "  5. Follow instructions in README.md"
echo ""
