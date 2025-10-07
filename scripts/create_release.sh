#!/bin/bash
set -euo pipefail

VERSION="${1:-v0.1.0}"
RELEASE_DIR="release-artifacts"

echo "Creating release $VERSION..."

# Clean and create release directory
rm -rf "$RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

# Build release binaries
echo "Building release binaries..."
~/.cargo/bin/cargo build --release --bin sync_service
~/.cargo/bin/cargo build --release --bin sagenscontact

# Determine platform
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    PLATFORM="ubuntu-latest"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    PLATFORM="macos-latest"
else
    PLATFORM="unknown"
fi

# Copy binaries with platform suffix
echo "Packaging binaries for $PLATFORM..."
cp target/release/sync_service "$RELEASE_DIR/sync_service-$PLATFORM"
cp target/release/sagenscontact "$RELEASE_DIR/sagenscontact-$PLATFORM"

# Create checksums
cd "$RELEASE_DIR"
sha256sum * > SHA256SUMS
cd ..

echo ""
echo "Release artifacts created in $RELEASE_DIR/"
echo ""
ls -lh "$RELEASE_DIR/"
echo ""

# Create release notes
cat > "$RELEASE_DIR/RELEASE_NOTES.md" <<EOF
# SagensContact Alpha $VERSION

## What's New

### Features
- ✅ Complete contact management system
- ✅ Import from multiple formats (CSV, vCard, Google, Apple, Social)
- ✅ Dashboard with entity statistics
- ✅ Automatic update system
- ✅ RESTful API with rate limiting
- ✅ Web UI (SvelteKit)

### New in this Release
- Dashboard API endpoint (\`/api/dashboard\`)
- Import history endpoint (\`/api/import/history\`)
- Automatic update system with GitHub releases integration
- Improved CI/CD with modern GitHub Actions

## Installation

### Quick Start

\`\`\`bash
# Download for your platform
wget https://github.com/r0bug/SC/releases/download/$VERSION/sync_service-$PLATFORM
wget https://github.com/r0bug/SC/releases/download/$VERSION/sagenscontact-$PLATFORM

# Make executable
chmod +x sync_service-$PLATFORM sagenscontact-$PLATFORM

# Rename
mv sync_service-$PLATFORM sync_service
mv sagenscontact-$PLATFORM sagenscontact

# Run
./sync_service
\`\`\`

### Environment Setup

\`\`\`bash
export DATABASE_URL="sqlite:./data/contacts.db"
export PORT=3002
mkdir -p data/attachments
./sync_service
\`\`\`

## API Endpoints

- \`GET /api/dashboard\` - Dashboard summary
- \`GET /api/import/history\` - Import history
- \`GET /api/system/version\` - Current version
- \`GET /api/system/updates/check\` - Check for updates

Full API documentation: https://github.com/r0bug/SC/blob/main/docs/

## Requirements

- SQLite 3.x
- Linux, macOS, or Windows
- 50MB disk space minimum

## Checksum Verification

\`\`\`bash
sha256sum -c SHA256SUMS
\`\`\`

## Support

- Report issues: https://github.com/r0bug/SC/issues
- Documentation: https://github.com/r0bug/SC/blob/main/README.md
EOF

echo "Release notes created at $RELEASE_DIR/RELEASE_NOTES.md"
echo ""
echo "Next steps:"
echo "1. Review release notes in $RELEASE_DIR/RELEASE_NOTES.md"
echo "2. Create git tag: git tag -a $VERSION -m \"Release $VERSION\""
echo "3. Push tag: git push origin $VERSION"
echo "4. Create GitHub release with: gh release create $VERSION --title \"$VERSION\" --notes-file $RELEASE_DIR/RELEASE_NOTES.md $RELEASE_DIR/*"
echo ""
echo "Or manually upload files to: https://github.com/r0bug/SC/releases/new?tag=$VERSION"
