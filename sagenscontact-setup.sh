#!/bin/bash
#
# SagensContact Alpha - Self-Contained Setup
#
# This single script:
# 1. Downloads source code from repository
# 2. Installs all prerequisites
# 3. Builds and configures everything
# 4. Sets up services
#
# Usage:
#   curl -sSL https://raw.githubusercontent.com/your-org/sagenscontact/main/sagenscontact-setup.sh | sudo bash
#
#   OR download and run:
#   wget https://raw.githubusercontent.com/your-org/sagenscontact/main/sagenscontact-setup.sh
#   chmod +x sagenscontact-setup.sh
#   sudo ./sagenscontact-setup.sh
#

set -e

# Configuration
REPO_URL="${REPO_URL:-https://github.com/your-org/sagenscontact-alpha.git}"
REPO_BRANCH="${REPO_BRANCH:-main}"
INSTALL_DIR="${INSTALL_DIR:-/opt/sagenscontact}"
TEMP_DIR="/tmp/sagenscontact-setup-$$"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log() {
    echo -e "${GREEN}[$(date +'%H:%M:%S')]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

# Header
echo "═══════════════════════════════════════════════════════════════"
echo "SagensContact Alpha - Self-Contained Setup"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    log_error "This script must be run as root"
    log_info "Run with: sudo $0"
    exit 1
fi

# Check for git
if ! command -v git &> /dev/null; then
    log_info "Installing git..."
    if command -v apt-get &> /dev/null; then
        apt-get update && apt-get install -y git
    elif command -v dnf &> /dev/null; then
        dnf install -y git
    elif command -v yum &> /dev/null; then
        yum install -y git
    elif command -v pacman &> /dev/null; then
        pacman -S --noconfirm git
    else
        log_error "Cannot install git automatically. Please install git and try again."
        exit 1
    fi
fi

# Clone repository
log "Cloning SagensContact repository..."
mkdir -p "$TEMP_DIR"
git clone --depth 1 --branch "$REPO_BRANCH" "$REPO_URL" "$TEMP_DIR" 2>&1 | grep -v "Cloning into"

# Change to repo directory
cd "$TEMP_DIR"

# Run the installer
log "Running installer..."
if [[ -f "scripts/install.sh" ]]; then
    chmod +x scripts/install.sh
    ./scripts/install.sh "$@"
else
    log_error "Installer script not found in repository"
    log_info "Expected: scripts/install.sh"
    exit 1
fi

# Cleanup
log "Cleaning up temporary files..."
cd /
rm -rf "$TEMP_DIR"

# Final message
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo -e "${GREEN}Setup Complete!${NC}"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Installation directory: $INSTALL_DIR"
echo ""
echo "Start services:"
echo "  sudo systemctl start sagenscontact-sync sagenscontact-web"
echo ""
echo "Access:"
echo "  Web UI: http://localhost:3001"
echo "  API: http://localhost:3002"
echo ""
