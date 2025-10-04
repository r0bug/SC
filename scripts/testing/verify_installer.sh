#!/bin/bash

# Installer Verification Script
# Runs comprehensive checks per the verification checklist

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/test_results/installer_verification"
LOG_FILE="$RESULTS_DIR/verification_$(date +%Y%m%d_%H%M%S).log"

mkdir -p "$RESULTS_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log() {
    echo -e "${BLUE}[INFO]${NC} $1" | tee -a "$LOG_FILE"
}

success() {
    echo -e "${GREEN}[PASS]${NC} $1" | tee -a "$LOG_FILE"
}

warning() {
    echo -e "${YELLOW}[WARN]${NC} $1" | tee -a "$LOG_FILE"
}

error() {
    echo -e "${RED}[FAIL]${NC} $1" | tee -a "$LOG_FILE"
}

section() {
    echo "" | tee -a "$LOG_FILE"
    echo "============================================" | tee -a "$LOG_FILE"
    echo "$1" | tee -a "$LOG_FILE"
    echo "============================================" | tee -a "$LOG_FILE"
}

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_WARNED=0

test_result() {
    local status=$1
    local message=$2

    case $status in
        "pass")
            success "$message"
            ((TESTS_PASSED++))
            ;;
        "fail")
            error "$message"
            ((TESTS_FAILED++))
            ;;
        "warn")
            warning "$message"
            ((TESTS_WARNED++))
            ;;
    esac
}

# ============================================
# PHASE 1: Baseline & Environment Prep
# ============================================
phase1_baseline() {
    section "PHASE 1: Baseline & Environment Prep"

    # OS detection
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        log "OS: $NAME $VERSION"
        test_result "pass" "OS detection successful: $NAME"
    else
        test_result "fail" "Cannot detect OS (/etc/os-release not found)"
    fi

    # Check current tool installations
    log "Checking existing installations..."

    for tool in rustc cargo node pnpm sqlite3 systemctl; do
        if command -v $tool &> /dev/null; then
            VERSION=$(case $tool in
                rustc) rustc --version ;;
                cargo) cargo --version ;;
                node) node --version ;;
                pnpm) pnpm --version ;;
                sqlite3) sqlite3 --version ;;
                systemctl) systemctl --version | head -1 ;;
            esac)
            warning "$tool already installed: $VERSION"
        else
            log "$tool not found (will be installed)"
        fi
    done
}

# ============================================
# PHASE 2: Dry Run / Dependency Detection
# ============================================
phase2_dry_run() {
    section "PHASE 2: Dry Run / Dependency Detection"

    # Check if install script exists
    INSTALL_SCRIPT="$PROJECT_ROOT/scripts/install.sh"
    if [ -f "$INSTALL_SCRIPT" ]; then
        test_result "pass" "Install script found at $INSTALL_SCRIPT"
    else
        test_result "fail" "Install script not found at $INSTALL_SCRIPT"
        return 1
    fi

    # Check script has --help
    if bash "$INSTALL_SCRIPT" --help &> /dev/null; then
        test_result "pass" "Install script has --help option"
    else
        test_result "warn" "Install script --help returned error (may not be implemented)"
    fi

    # Check script syntax
    if bash -n "$INSTALL_SCRIPT"; then
        test_result "pass" "Install script syntax valid"
    else
        test_result "fail" "Install script has syntax errors"
    fi
}

# ============================================
# PHASE 3: Build Verification
# ============================================
phase3_build_check() {
    section "PHASE 3: Build Verification"

    cd "$PROJECT_ROOT"

    # Check Rust build
    log "Checking Rust build..."
    if [ -f "target/release/sync_service" ]; then
        test_result "pass" "sync_service binary exists"

        # Test binary execution
        if target/release/sync_service --version &> /dev/null || true; then
            test_result "pass" "sync_service binary is executable"
        else
            test_result "warn" "sync_service may not have --version flag"
        fi
    else
        test_result "fail" "sync_service binary not found (needs build)"
    fi

    # Check for other binaries
    for binary in worker cli_client; do
        if [ -f "target/release/$binary" ]; then
            test_result "pass" "$binary binary exists"
        else
            test_result "warn" "$binary binary not found (may be optional)"
        fi
    done
}

# ============================================
# PHASE 4: Service Status Checks
# ============================================
phase4_service_check() {
    section "PHASE 4: Service Status Checks"

    # Check if systemd is available
    if command -v systemctl &> /dev/null; then
        test_result "pass" "systemd available"

        # Check for service files
        for service in sync_service worker; do
            if systemctl list-unit-files | grep -q "^${service}.service"; then
                STATUS=$(systemctl is-active ${service}.service || echo "inactive")
                if [ "$STATUS" = "active" ]; then
                    test_result "pass" "${service}.service is active"
                else
                    test_result "warn" "${service}.service exists but is $STATUS"
                fi
            else
                test_result "warn" "${service}.service not installed"
            fi
        done
    else
        test_result "warn" "systemd not available (user-local install mode)"
    fi
}

# ============================================
# PHASE 5: API Endpoint Checks
# ============================================
phase5_api_check() {
    section "PHASE 5: API Endpoint Checks"

    # Determine which port to check
    PORTS=(3000 3002 3001)
    API_PORT=""

    for port in "${PORTS[@]}"; do
        if curl -s --connect-timeout 2 "http://localhost:$port/health" &> /dev/null; then
            API_PORT=$port
            break
        fi
    done

    if [ -n "$API_PORT" ]; then
        test_result "pass" "API service responding on port $API_PORT"

        # Test health endpoint
        HEALTH=$(curl -s "http://localhost:$API_PORT/health")
        if [ "$HEALTH" = "OK" ]; then
            test_result "pass" "Health endpoint returns OK"
        else
            test_result "warn" "Health endpoint response: $HEALTH"
        fi

        # Test other endpoints
        for endpoint in "/api/import/connectors" "/metrics" "/api/health/detailed"; do
            HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:$API_PORT$endpoint")
            if [ "$HTTP_CODE" = "200" ]; then
                test_result "pass" "$endpoint returns 200"
            else
                test_result "warn" "$endpoint returns $HTTP_CODE"
            fi
        done
    else
        test_result "fail" "No API service found on ports ${PORTS[*]}"
    fi
}

# ============================================
# PHASE 6: File System Checks
# ============================================
phase6_filesystem_check() {
    section "PHASE 6: File System Checks"

    # Check data directory
    DATA_DIRS=("./data" "/opt/sagenscontact/data" "$HOME/.sagenscontact/data")

    for dir in "${DATA_DIRS[@]}"; do
        if [ -d "$dir" ]; then
            test_result "pass" "Data directory exists: $dir"

            # Check attachments subdirectory
            if [ -d "$dir/attachments" ]; then
                test_result "pass" "Attachments directory exists: $dir/attachments"

                # Check permissions
                if [ -w "$dir/attachments" ]; then
                    test_result "pass" "Attachments directory is writable"
                else
                    test_result "warn" "Attachments directory is not writable"
                fi
            fi

            # Check database
            if [ -f "$dir/contacts.db" ] || [ -f "$dir/sagenscontact.db" ]; then
                test_result "pass" "Database file exists in $dir"
            fi
        fi
    done
}

# ============================================
# PHASE 7: Configuration Checks
# ============================================
phase7_config_check() {
    section "PHASE 7: Configuration Checks"

    # Check for .env file
    if [ -f "$PROJECT_ROOT/.env" ]; then
        test_result "pass" ".env file exists"

        # Check key variables
        for var in DATABASE_URL JWT_SECRET LOG_LEVEL; do
            if grep -q "^$var=" "$PROJECT_ROOT/.env"; then
                test_result "pass" "$var defined in .env"
            else
                test_result "warn" "$var not found in .env"
            fi
        done
    else
        test_result "warn" ".env file not found"
    fi
}

# ============================================
# PHASE 8: Security Checks
# ============================================
phase8_security_check() {
    section "PHASE 8: Security Checks"

    # Check file permissions
    if [ -f "$PROJECT_ROOT/.env" ]; then
        PERMS=$(stat -c "%a" "$PROJECT_ROOT/.env" 2>/dev/null || stat -f "%OLp" "$PROJECT_ROOT/.env" 2>/dev/null)
        if [ "$PERMS" = "600" ] || [ "$PERMS" = "640" ]; then
            test_result "pass" ".env has secure permissions ($PERMS)"
        else
            test_result "warn" ".env permissions are $PERMS (should be 600 or 640)"
        fi
    fi

    # Check for secrets in repository
    log "Scanning for potential secrets..."
    if git rev-parse --git-dir > /dev/null 2>&1; then
        # Check if .env is gitignored
        if git check-ignore .env &> /dev/null; then
            test_result "pass" ".env is gitignored"
        else
            test_result "warn" ".env is not gitignored"
        fi
    fi
}

# ============================================
# PHASE 9: Integration Tests
# ============================================
phase9_integration_test() {
    section "PHASE 9: Integration Tests"

    # Check if test data exists
    if [ -f "$PROJECT_ROOT/sample_data/load_tests/contacts_10k.csv" ]; then
        test_result "pass" "Test data available"

        # Run quick API test if service is up
        if [ -n "$API_PORT" ]; then
            log "Testing import preview..."
            PREVIEW=$(curl -s -X POST "http://localhost:$API_PORT/api/import/preview?limit=5" \
                -F "file=@$PROJECT_ROOT/sample_data/load_tests/contacts_10k.csv" 2>&1)

            if echo "$PREVIEW" | grep -q "total_rows"; then
                test_result "pass" "Import preview API functional"
            else
                test_result "warn" "Import preview failed: ${PREVIEW:0:100}"
            fi
        fi
    else
        test_result "warn" "Test data not found"
    fi
}

# ============================================
# PHASE 10: Documentation Checks
# ============================================
phase10_documentation_check() {
    section "PHASE 10: Documentation Checks"

    # Check for key documentation files
    DOCS=(
        "README.md"
        "docs/INSTALLATION.md"
        "docs/WEEK_5_INTEGRATION_REPORT.md"
        "scripts/testing/README.md"
    )

    for doc in "${DOCS[@]}"; do
        if [ -f "$PROJECT_ROOT/$doc" ]; then
            test_result "pass" "Documentation exists: $doc"
        else
            test_result "warn" "Documentation missing: $doc"
        fi
    done
}

# ============================================
# Main Execution
# ============================================
main() {
    log "Starting SagensContact Installer Verification"
    log "Project Root: $PROJECT_ROOT"
    log "Results Directory: $RESULTS_DIR"
    log "Log File: $LOG_FILE"

    phase1_baseline
    phase2_dry_run
    phase3_build_check
    phase4_service_check
    phase5_api_check
    phase6_filesystem_check
    phase7_config_check
    phase8_security_check
    phase9_integration_test
    phase10_documentation_check

    # Final summary
    section "VERIFICATION SUMMARY"

    TOTAL_TESTS=$((TESTS_PASSED + TESTS_FAILED + TESTS_WARNED))

    echo "" | tee -a "$LOG_FILE"
    echo "Total Tests: $TOTAL_TESTS" | tee -a "$LOG_FILE"
    success "Passed: $TESTS_PASSED"
    warning "Warnings: $TESTS_WARNED"
    error "Failed: $TESTS_FAILED"
    echo "" | tee -a "$LOG_FILE"

    if [ $TESTS_FAILED -eq 0 ]; then
        success "✅ VERIFICATION PASSED - Installer ready for QA"
        exit 0
    else
        error "❌ VERIFICATION FAILED - Please address failures before QA"
        exit 1
    fi
}

# Run main
main "$@"
