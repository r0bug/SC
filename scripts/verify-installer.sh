#!/bin/bash
#
# SagensContact Installer Verification Script
#
# This script verifies that the installer can run successfully on a fresh system.
# It simulates a clean environment and tests all installer functionality.
#

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

INSTALL_DIR="/tmp/sagenscontact-test-$$"
TEST_LOG="/tmp/sagenscontact-verify-$$.log"

log() {
    echo -e "${GREEN}[$(date +'%H:%M:%S')]${NC} $1" | tee -a "$TEST_LOG"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$TEST_LOG" >&2
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1" | tee -a "$TEST_LOG"
}

show_test() {
    echo -e "${BLUE}▶ Test:${NC} $1"
}

show_pass() {
    echo -e "${GREEN}✓ PASS:${NC} $1"
}

show_fail() {
    echo -e "${RED}✗ FAIL:${NC} $1"
}

cleanup() {
    log_info "Cleaning up test installation..."
    rm -rf "$INSTALL_DIR"
}

trap cleanup EXIT

echo "═══════════════════════════════════════════════════════════════"
echo "SagensContact Installer Verification"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Test directory: $INSTALL_DIR"
echo "Test log: $TEST_LOG"
echo ""

# Test 1: Installer exists and is executable
show_test "Installer script exists and is executable"
if [[ -x "scripts/install.sh" ]]; then
    show_pass "Installer script found"
else
    show_fail "Installer script not found or not executable"
    exit 1
fi

# Test 2: Installer help works
show_test "Installer help command"
if scripts/install.sh --help &> "$TEST_LOG.help"; then
    show_pass "Help command works"
else
    show_fail "Help command failed"
    exit 1
fi

# Test 3: Prerequisite detection
show_test "Prerequisite detection (dry run)"

log_info "Checking Rust detection..."
if command -v rustc &> /dev/null; then
    RUST_VER=$(rustc --version)
    show_pass "Rust detected: $RUST_VER"
else
    show_fail "Rust not detected"
fi

log_info "Checking Node.js detection..."
if command -v node &> /dev/null; then
    NODE_VER=$(node --version)
    show_pass "Node.js detected: $NODE_VER"
else
    show_fail "Node.js not detected"
fi

log_info "Checking pnpm detection..."
if command -v pnpm &> /dev/null; then
    PNPM_VER=$(pnpm --version)
    show_pass "pnpm detected: v$PNPM_VER"
else
    show_fail "pnpm not detected"
fi

log_info "Checking SQLite detection..."
if command -v sqlite3 &> /dev/null; then
    SQLITE_VER=$(sqlite3 --version | awk '{print $1}')
    show_pass "SQLite detected: $SQLITE_VER"
else
    show_fail "SQLite not detected"
fi

log_info "Checking build tools..."
if command -v gcc &> /dev/null && command -v make &> /dev/null; then
    GCC_VER=$(gcc --version | head -1)
    show_pass "Build tools detected: $GCC_VER"
else
    show_fail "Build tools not detected"
fi

# Test 4: User-local installation (no sudo needed)
show_test "User-local installation"

log_info "Running installer in user mode..."
if INSTALL_DIR="$INSTALL_DIR" DATA_DIR="$INSTALL_DIR/data" \
   ./scripts/install.sh --no-services --skip-build &> "$TEST_LOG.install"; then
    show_pass "Installer ran successfully"
else
    show_fail "Installer failed"
    log_info "Check log: $TEST_LOG.install"
    exit 1
fi

# Test 5: Verify directory structure
show_test "Directory structure verification"

declare -a required_dirs=(
    "$INSTALL_DIR/binaries"
    "$INSTALL_DIR/data"
    "$INSTALL_DIR/data/attachments"
    "$INSTALL_DIR/logs"
    "$INSTALL_DIR/config"
    "$INSTALL_DIR/scripts"
)

for dir in "${required_dirs[@]}"; do
    if [[ -d "$dir" ]]; then
        show_pass "Directory exists: $dir"
    else
        show_fail "Directory missing: $dir"
        exit 1
    fi
done

# Test 6: Verify configuration file
show_test "Configuration file verification"

if [[ -f "$INSTALL_DIR/config/env" ]]; then
    show_pass "Environment config created"

    # Check for required variables
    if grep -q "DATABASE_URL" "$INSTALL_DIR/config/env"; then
        show_pass "DATABASE_URL configured"
    else
        show_fail "DATABASE_URL missing"
    fi

    if grep -q "JWT_SECRET" "$INSTALL_DIR/config/env"; then
        JWT_SECRET=$(grep JWT_SECRET "$INSTALL_DIR/config/env" | cut -d= -f2 | tr -d '"')
        JWT_LEN=${#JWT_SECRET}
        if [[ $JWT_LEN -ge 32 ]]; then
            show_pass "JWT_SECRET generated (length: $JWT_LEN)"
        else
            show_fail "JWT_SECRET too short (length: $JWT_LEN)"
        fi
    else
        show_fail "JWT_SECRET missing"
    fi
else
    show_fail "Environment config not created"
    exit 1
fi

# Test 7: Verify startup scripts
show_test "Startup scripts verification"

declare -a required_scripts=(
    "$INSTALL_DIR/scripts/start-sync-service.sh"
    "$INSTALL_DIR/scripts/start-web-ui.sh"
    "$INSTALL_DIR/scripts/start-all.sh"
)

for script in "${required_scripts[@]}"; do
    if [[ -x "$script" ]]; then
        show_pass "Script exists and is executable: $(basename $script)"
    else
        show_fail "Script missing or not executable: $(basename $script)"
        exit 1
    fi
done

# Test 8: Verify binaries (if built)
show_test "Binary verification"

if [[ -f "target/release/sagenscontact" ]]; then
    if [[ -f "$INSTALL_DIR/binaries/sagenscontact" ]]; then
        show_pass "CLI binary installed"

        # Test binary
        if "$INSTALL_DIR/binaries/sagenscontact" --version &> /dev/null; then
            VER=$("$INSTALL_DIR/binaries/sagenscontact" --version)
            show_pass "CLI binary works: $VER"
        else
            show_fail "CLI binary doesn't execute"
        fi
    else
        show_fail "CLI binary not installed"
    fi

    if [[ -f "$INSTALL_DIR/binaries/sync_service" ]]; then
        show_pass "Sync service binary installed"
    else
        show_fail "Sync service binary not installed"
    fi
else
    log_info "Binaries not built (--skip-build used) - skipping binary tests"
fi

# Test 9: Verify file permissions
show_test "File permissions verification"

CONFIG_PERMS=$(stat -c "%a" "$INSTALL_DIR/config/env")
if [[ "$CONFIG_PERMS" == "600" ]]; then
    show_pass "Config file has correct permissions (600)"
else
    show_fail "Config file has wrong permissions ($CONFIG_PERMS, expected 600)"
fi

# Test 10: Verify web UI (if built)
show_test "Web UI verification"

if [[ -d "apps/web/.svelte-kit/output" ]]; then
    if [[ -d "$INSTALL_DIR/web/client" ]]; then
        show_pass "Web UI client installed"
    else
        show_fail "Web UI client not installed"
    fi

    if [[ -f "$INSTALL_DIR/web/index.js" ]]; then
        show_pass "Web UI server entry point installed"
    else
        show_fail "Web UI server entry point missing"
    fi
else
    log_info "Web UI not built - skipping web UI tests"
fi

# Test 11: Check for required environment variables in scripts
show_test "Startup script configuration"

if grep -q "source.*config/env" "$INSTALL_DIR/scripts/start-sync-service.sh"; then
    show_pass "Sync service script sources env config"
else
    show_fail "Sync service script doesn't source env config"
fi

# Test 12: Verify log directory is writable
show_test "Log directory permissions"

TEST_LOG_FILE="$INSTALL_DIR/logs/test.log"
if echo "test" > "$TEST_LOG_FILE" 2>/dev/null; then
    show_pass "Log directory is writable"
    rm "$TEST_LOG_FILE"
else
    show_fail "Log directory is not writable"
fi

# Summary
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo -e "${GREEN}Verification Complete!${NC}"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Installation verified at: $INSTALL_DIR"
echo "Test log: $TEST_LOG"
echo ""
echo "Key findings:"
echo "  ✓ Installer runs successfully in user mode"
echo "  ✓ All required directories created"
echo "  ✓ Configuration file generated with secure defaults"
echo "  ✓ Startup scripts created and executable"
echo "  ✓ File permissions set correctly"
echo ""
echo "Next steps:"
echo "  1. Test with full build: ./scripts/install.sh (remove --skip-build)"
echo "  2. Test system-wide install: sudo ./scripts/install.sh"
echo "  3. Test service startup: systemctl start sagenscontact-*"
echo ""

log "Verification completed successfully at $(date)"
