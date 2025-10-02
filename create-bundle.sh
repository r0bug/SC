#!/bin/bash
# Creates a single-file installer with embedded source code

OUTPUT="sagenscontact-installer-bundle.sh"

cat > "$OUTPUT" << 'HEADER'
#!/bin/bash
#
# SagensContact Alpha - Complete Installer Bundle
# This file contains everything needed to install SagensContact
#
# Usage: sudo ./sagenscontact-installer-bundle.sh
#

set -e

INSTALL_DIR="${INSTALL_DIR:-/opt/sagenscontact}"
TEMP_DIR="/tmp/sagenscontact-install-$$"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "═══════════════════════════════════════════════════════════════"
echo "SagensContact Alpha - Bundle Installer"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Extract embedded tarball
echo -e "${BLUE}▶${NC} Extracting source code..."
ARCHIVE_LINE=$(awk '/^__ARCHIVE_BELOW__/ {print NR + 1; exit 0; }' "$0")
mkdir -p "$TEMP_DIR"
tail -n +$ARCHIVE_LINE "$0" | tar xz -C "$TEMP_DIR"

# Run installer
echo -e "${BLUE}▶${NC} Running installer..."
cd "$TEMP_DIR"
chmod +x scripts/install.sh
./scripts/install.sh "$@"

# Cleanup
cd /
rm -rf "$TEMP_DIR"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo -e "${GREEN}Installation Complete!${NC}"
echo "═══════════════════════════════════════════════════════════════"

exit 0

__ARCHIVE_BELOW__
HEADER

# Create tarball and append
tar czf - \
  --exclude='.git' \
  --exclude='target' \
  --exclude='node_modules' \
  --exclude='*.log' \
  --exclude='.svelte-kit' \
  . >> "$OUTPUT"

chmod +x "$OUTPUT"

SIZE=$(ls -lh "$OUTPUT" | awk '{print $5}')
echo "Created: $OUTPUT ($SIZE)"
echo ""
echo "Usage:"
echo "  sudo ./$OUTPUT"
echo ""
echo "Or distribute via:"
echo "  curl -sSL https://your-server.com/$OUTPUT | sudo bash"
