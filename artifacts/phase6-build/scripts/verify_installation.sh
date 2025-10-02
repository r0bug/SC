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
