#!/bin/bash
#
# SagensContact Alpha - Turnkey Installer
#
# This script automatically detects, installs, and configures all prerequisites
# for running SagensContact Alpha on a fresh system.
#
# Supports both fresh installations and upgrades from previous versions.
#
# Usage: ./install.sh [--skip-build] [--no-services] [--upgrade] [--help]
#

set -e  # Exit on error
set -u  # Exit on undefined variable

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Version tracking
INSTALLER_VERSION="0.1.0-alpha.3"
VERSION_FILE="VERSION"

# Configuration
INSTALL_DIR="${INSTALL_DIR:-/opt/sagenscontact}"
DATA_DIR="${DATA_DIR:-$INSTALL_DIR/data}"
LOG_FILE="${LOG_FILE:-/tmp/sagenscontact-install-$$.log}"
SKIP_BUILD=false
NO_SERVICES=false
UPGRADE_MODE=false
EXISTING_INSTALLATION=false

# Ensure log file is writable
touch "$LOG_FILE" 2>/dev/null || LOG_FILE="/dev/null"
chmod 666 "$LOG_FILE" 2>/dev/null
MIN_RUST_VERSION="1.83"
MIN_NODE_VERSION="18"
MIN_PNPM_VERSION="8"

# Log function
log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE" >&2
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" | tee -a "$LOG_FILE"
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1" | tee -a "$LOG_FILE"
}

# Progress indicator
show_progress() {
    local task="$1"
    echo -e "${BLUE}▶${NC} $task..."
}

show_success() {
    local task="$1"
    echo -e "${GREEN}✓${NC} $task"
}

show_failure() {
    local task="$1"
    echo -e "${RED}✗${NC} $task"
}

# Parse arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-build)
                SKIP_BUILD=true
                shift
                ;;
            --no-services)
                NO_SERVICES=true
                shift
                ;;
            --upgrade)
                UPGRADE_MODE=true
                shift
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

show_help() {
    cat << EOF
SagensContact Alpha - Turnkey Installer

Usage: $0 [OPTIONS]

OPTIONS:
    --skip-build     Skip building from source (use pre-built binaries)
    --no-services    Don't set up systemd services (manual start only)
    --upgrade        Force upgrade mode (auto-detected if existing install found)
    --help           Show this help message

ENVIRONMENT VARIABLES:
    INSTALL_DIR      Installation directory (default: /opt/sagenscontact)
    DATA_DIR         Data directory (default: \$INSTALL_DIR/data)
    LOG_FILE         Installation log (default: /tmp/sagenscontact-install.log)

EXAMPLES:
    # Full installation
    sudo ./install.sh

    # Install without systemd services
    sudo ./install.sh --no-services

    # Use pre-built binaries
    sudo ./install.sh --skip-build

    # Upgrade existing installation
    sudo ./install.sh --upgrade

UPGRADES:
    The installer automatically detects existing installations and:
    - Backs up your data before upgrading
    - Preserves your configuration (config/env)
    - Stops services, upgrades, then restarts
    - Runs database migrations automatically

EOF
}

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]] && [[ "$NO_SERVICES" == false ]]; then
        log_error "This script must be run as root for system-wide installation"
        log_info "Run with sudo, or use --no-services for user-local install"
        exit 1
    fi
}

# Get installed version
get_installed_version() {
    if [[ -f "$INSTALL_DIR/$VERSION_FILE" ]]; then
        cat "$INSTALL_DIR/$VERSION_FILE"
    else
        echo "unknown"
    fi
}

# Check for existing installation
check_existing_installation() {
    show_progress "Checking for existing installation"

    if [[ -d "$INSTALL_DIR" ]] && [[ -f "$INSTALL_DIR/binaries/sagenscontact" || -f "$INSTALL_DIR/binaries/sync_service" ]]; then
        EXISTING_INSTALLATION=true
        local installed_version
        installed_version=$(get_installed_version)

        if [[ "$UPGRADE_MODE" == true ]]; then
            show_success "Existing installation found (v$installed_version) - upgrade mode"
        else
            log_warn "Existing installation found at $INSTALL_DIR (v$installed_version)"
            echo ""
            echo -e "${YELLOW}An existing installation was detected.${NC}"
            echo "Current version: $installed_version"
            echo "New version: $INSTALLER_VERSION"
            echo ""
            read -p "Upgrade existing installation? (yes/no): " confirm
            if [[ "$confirm" == "yes" ]]; then
                UPGRADE_MODE=true
                show_success "Proceeding with upgrade"
            else
                log_error "Installation cancelled. Use a different INSTALL_DIR or remove existing installation."
                exit 1
            fi
        fi
    else
        show_success "No existing installation found - fresh install"
    fi
}

# Create pre-upgrade backup
create_upgrade_backup() {
    if [[ "$UPGRADE_MODE" != true ]] || [[ "$EXISTING_INSTALLATION" != true ]]; then
        return 0
    fi

    show_progress "Creating pre-upgrade backup"

    local backup_dir="$INSTALL_DIR/backups"
    local timestamp
    timestamp=$(date +%Y%m%d_%H%M%S)
    local backup_file="$backup_dir/pre-upgrade-$timestamp.tar.gz"

    mkdir -p "$backup_dir"

    # Backup data, config, and version info
    local items_to_backup=()
    [[ -d "$DATA_DIR" ]] && items_to_backup+=("$DATA_DIR")
    [[ -f "$INSTALL_DIR/config/env" ]] && items_to_backup+=("$INSTALL_DIR/config/env")
    [[ -f "$INSTALL_DIR/$VERSION_FILE" ]] && items_to_backup+=("$INSTALL_DIR/$VERSION_FILE")

    if [[ ${#items_to_backup[@]} -gt 0 ]]; then
        tar -czf "$backup_file" "${items_to_backup[@]}" 2>/dev/null || true
        show_success "Backup created: $backup_file"
        log_info "Backup size: $(du -h "$backup_file" | cut -f1)"
    else
        log_warn "No data to backup"
    fi

    # Keep only last 5 pre-upgrade backups
    if [[ -d "$backup_dir" ]]; then
        cd "$backup_dir"
        ls -t pre-upgrade-*.tar.gz 2>/dev/null | tail -n +6 | xargs rm -f 2>/dev/null || true
        cd - > /dev/null
    fi
}

# Stop existing services before upgrade
stop_existing_services() {
    if [[ "$UPGRADE_MODE" != true ]] || [[ "$EXISTING_INSTALLATION" != true ]]; then
        return 0
    fi

    show_progress "Stopping existing services"

    # Try systemd first
    if systemctl is-active sagenscontact-sync &>/dev/null; then
        systemctl stop sagenscontact-sync 2>/dev/null || true
        log_info "Stopped sagenscontact-sync service"
    fi
    if systemctl is-active sagenscontact-web &>/dev/null; then
        systemctl stop sagenscontact-web 2>/dev/null || true
        log_info "Stopped sagenscontact-web service"
    fi
    if systemctl is-active sagenscontact-worker &>/dev/null; then
        systemctl stop sagenscontact-worker 2>/dev/null || true
        log_info "Stopped sagenscontact-worker service"
    fi

    # Also try pkill for non-systemd installations
    pkill -f "$INSTALL_DIR/binaries/sync_service" 2>/dev/null || true
    pkill -f "$INSTALL_DIR/binaries/worker" 2>/dev/null || true

    # Wait for services to stop
    sleep 2

    show_success "Existing services stopped"
}

# Preserve existing configuration
preserve_config() {
    if [[ "$UPGRADE_MODE" != true ]] || [[ "$EXISTING_INSTALLATION" != true ]]; then
        return 0
    fi

    show_progress "Preserving existing configuration"

    # Save existing env file to temp location
    if [[ -f "$INSTALL_DIR/config/env" ]]; then
        cp "$INSTALL_DIR/config/env" "/tmp/sagenscontact-env-backup-$$.tmp"
        PRESERVED_ENV="/tmp/sagenscontact-env-backup-$$.tmp"
        show_success "Configuration preserved"
    else
        PRESERVED_ENV=""
        log_warn "No existing configuration to preserve"
    fi
}

# Restore preserved configuration
restore_config() {
    if [[ -z "${PRESERVED_ENV:-}" ]] || [[ ! -f "${PRESERVED_ENV:-}" ]]; then
        return 0
    fi

    show_progress "Restoring preserved configuration"

    # Restore the preserved env file
    cp "$PRESERVED_ENV" "$INSTALL_DIR/config/env"
    rm -f "$PRESERVED_ENV"

    show_success "Configuration restored"
}

# Write version file
write_version_file() {
    show_progress "Writing version information"

    cat > "$INSTALL_DIR/$VERSION_FILE" << EOF
$INSTALLER_VERSION
Installed: $(date -Iseconds)
Installer: scripts/install.sh
EOF

    show_success "Version $INSTALLER_VERSION recorded"
}

# Restart services after upgrade
restart_services() {
    if [[ "$UPGRADE_MODE" != true ]] || [[ "$NO_SERVICES" == true ]]; then
        return 0
    fi

    show_progress "Restarting services after upgrade"

    # Restart systemd services if they exist
    if [[ -f /etc/systemd/system/sagenscontact-sync.service ]]; then
        systemctl daemon-reload
        systemctl start sagenscontact-sync 2>/dev/null || true
        log_info "Started sagenscontact-sync service"
    fi
    if [[ -f /etc/systemd/system/sagenscontact-web.service ]]; then
        systemctl start sagenscontact-web 2>/dev/null || true
        log_info "Started sagenscontact-web service"
    fi
    if [[ -f /etc/systemd/system/sagenscontact-worker.service ]]; then
        systemctl start sagenscontact-worker 2>/dev/null || true
        log_info "Started sagenscontact-worker service"
    fi

    # Wait for services to start
    sleep 3

    # Verify services are running
    if systemctl is-active sagenscontact-sync &>/dev/null; then
        show_success "Services restarted successfully"
    else
        log_warn "Services may not have started. Check: systemctl status sagenscontact-sync"
    fi
}

# Detect OS and package manager
detect_os() {
    show_progress "Detecting operating system"

    if [[ -f /etc/os-release ]]; then
        . /etc/os-release
        OS_ID=$ID
        OS_VERSION=$VERSION_ID
        OS_NAME=$PRETTY_NAME
    else
        log_error "Cannot detect OS. /etc/os-release not found."
        exit 1
    fi

    # Detect package manager
    if command -v apt-get &> /dev/null; then
        PKG_MANAGER="apt"
        PKG_UPDATE="apt-get update"
        PKG_INSTALL="apt-get install -y"
    elif command -v dnf &> /dev/null; then
        PKG_MANAGER="dnf"
        PKG_UPDATE="dnf check-update || true"
        PKG_INSTALL="dnf install -y"
    elif command -v yum &> /dev/null; then
        PKG_MANAGER="yum"
        PKG_UPDATE="yum check-update || true"
        PKG_INSTALL="yum install -y"
    elif command -v pacman &> /dev/null; then
        PKG_MANAGER="pacman"
        PKG_UPDATE="pacman -Sy"
        PKG_INSTALL="pacman -S --noconfirm"
    else
        log_error "No supported package manager found (apt, dnf, yum, pacman)"
        exit 1
    fi

    show_success "Detected $OS_NAME using $PKG_MANAGER"
}

# Version comparison
version_ge() {
    # Returns 0 if $1 >= $2
    printf '%s\n%s\n' "$2" "$1" | sort -V -C
}

# Check and install Rust
check_rust() {
    show_progress "Checking Rust toolchain"

    if command -v rustc &> /dev/null; then
        RUST_VERSION=$(rustc --version | awk '{print $2}')
        if version_ge "$RUST_VERSION" "$MIN_RUST_VERSION"; then
            show_success "Rust $RUST_VERSION installed (>= $MIN_RUST_VERSION required)"
            return 0
        else
            log_warn "Rust $RUST_VERSION found, but $MIN_RUST_VERSION required"
        fi
    fi

    log_info "Installing Rust toolchain..."

    # Install rustup
    if ! command -v rustup &> /dev/null; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable

        # Source cargo env
        if [[ -f "$HOME/.cargo/env" ]]; then
            source "$HOME/.cargo/env"
        fi
    fi

    # Update to latest stable
    rustup update stable
    rustup default stable

    RUST_VERSION=$(rustc --version | awk '{print $2}')
    show_success "Rust $RUST_VERSION installed"
}

# Check and install Node.js
check_node() {
    show_progress "Checking Node.js"

    if command -v node &> /dev/null; then
        NODE_VERSION=$(node --version | sed 's/v//' | cut -d. -f1)
        if [[ $NODE_VERSION -ge $MIN_NODE_VERSION ]]; then
            show_success "Node.js $(node --version) installed (>= v$MIN_NODE_VERSION required)"
            return 0
        else
            log_warn "Node.js v$NODE_VERSION found, but v$MIN_NODE_VERSION required"
        fi
    fi

    log_info "Installing Node.js..."

    # Install using NodeSource repository
    if [[ "$PKG_MANAGER" == "apt" ]]; then
        curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
        $PKG_INSTALL nodejs
    elif [[ "$PKG_MANAGER" == "dnf" ]] || [[ "$PKG_MANAGER" == "yum" ]]; then
        curl -fsSL https://rpm.nodesource.com/setup_20.x | bash -
        $PKG_INSTALL nodejs
    elif [[ "$PKG_MANAGER" == "pacman" ]]; then
        $PKG_INSTALL nodejs npm
    else
        log_error "Cannot auto-install Node.js on this system"
        log_info "Please install Node.js v$MIN_NODE_VERSION or higher manually from https://nodejs.org"
        exit 1
    fi

    show_success "Node.js $(node --version) installed"
}

# Check and install pnpm
check_pnpm() {
    show_progress "Checking pnpm"

    if command -v pnpm &> /dev/null; then
        PNPM_VERSION=$(pnpm --version | cut -d. -f1)
        if [[ $PNPM_VERSION -ge $MIN_PNPM_VERSION ]]; then
            show_success "pnpm $(pnpm --version) installed (>= $MIN_PNPM_VERSION required)"
            return 0
        fi
    fi

    log_info "Installing pnpm..."
    npm install -g pnpm@latest

    show_success "pnpm $(pnpm --version) installed"
}

# Check and install SQLite
check_sqlite() {
    show_progress "Checking SQLite"

    if command -v sqlite3 &> /dev/null; then
        SQLITE_VERSION=$(sqlite3 --version | awk '{print $1}')
        show_success "SQLite $SQLITE_VERSION installed"
        return 0
    fi

    if [[ "$EUID" -ne 0 ]]; then
        log_warn "SQLite not found and cannot install without root"
        log_info "Install SQLite manually:"
        if [[ "$PKG_MANAGER" == "apt" ]]; then
            log_info "  sudo apt-get install sqlite3 libsqlite3-dev"
        elif [[ "$PKG_MANAGER" == "dnf" ]] || [[ "$PKG_MANAGER" == "yum" ]]; then
            log_info "  sudo $PKG_MANAGER install sqlite sqlite-devel"
        elif [[ "$PKG_MANAGER" == "pacman" ]]; then
            log_info "  sudo pacman -S sqlite"
        fi
        log_info "Then re-run this installer"
        return 1
    fi

    log_info "Installing SQLite..."

    if [[ "$PKG_MANAGER" == "apt" ]]; then
        $PKG_INSTALL sqlite3 libsqlite3-dev
    elif [[ "$PKG_MANAGER" == "dnf" ]] || [[ "$PKG_MANAGER" == "yum" ]]; then
        $PKG_INSTALL sqlite sqlite-devel
    elif [[ "$PKG_MANAGER" == "pacman" ]]; then
        $PKG_INSTALL sqlite
    fi

    show_success "SQLite installed"
}

# Check and install build essentials
check_build_tools() {
    show_progress "Checking build tools"

    local missing=()

    if ! command -v gcc &> /dev/null; then
        missing+=("gcc")
    fi

    if ! command -v make &> /dev/null; then
        missing+=("make")
    fi

    if ! command -v pkg-config &> /dev/null; then
        missing+=("pkg-config")
    fi

    if [[ ${#missing[@]} -eq 0 ]]; then
        show_success "Build tools installed"
        return 0
    fi

    log_info "Installing build tools: ${missing[*]}"

    if [[ "$PKG_MANAGER" == "apt" ]]; then
        $PKG_INSTALL build-essential pkg-config libssl-dev
    elif [[ "$PKG_MANAGER" == "dnf" ]] || [[ "$PKG_MANAGER" == "yum" ]]; then
        $PKG_INSTALL gcc gcc-c++ make pkg-config openssl-devel
    elif [[ "$PKG_MANAGER" == "pacman" ]]; then
        $PKG_INSTALL base-devel openssl
    fi

    show_success "Build tools installed"
}

# Check and install ClamAV (optional)
check_clamav() {
    show_progress "Checking ClamAV (optional)"

    if command -v clamscan &> /dev/null; then
        show_success "ClamAV installed"
        return 0
    fi

    log_warn "ClamAV not found - virus scanning disabled"
    log_info "To enable virus scanning, install ClamAV:"
    if [[ "$PKG_MANAGER" == "apt" ]]; then
        log_info "  sudo apt-get install clamav clamav-daemon"
    elif [[ "$PKG_MANAGER" == "dnf" ]] || [[ "$PKG_MANAGER" == "yum" ]]; then
        log_info "  sudo $PKG_MANAGER install clamav clamav-update"
    elif [[ "$PKG_MANAGER" == "pacman" ]]; then
        log_info "  sudo pacman -S clamav"
    fi
}

# Check and install TLS tools
check_tls_tools() {
    show_progress "Checking TLS tools"

    if command -v openssl &> /dev/null; then
        show_success "OpenSSL $(openssl version | awk '{print $2}') installed"
    else
        log_info "Installing OpenSSL..."
        if [[ "$PKG_MANAGER" == "apt" ]]; then
            $PKG_INSTALL openssl
        elif [[ "$PKG_MANAGER" == "dnf" ]] || [[ "$PKG_MANAGER" == "yum" ]]; then
            $PKG_INSTALL openssl
        elif [[ "$PKG_MANAGER" == "pacman" ]]; then
            $PKG_INSTALL openssl
        fi
        show_success "OpenSSL installed"
    fi

    # Check for certbot (optional)
    if command -v certbot &> /dev/null; then
        show_success "Certbot installed"
    else
        log_warn "Certbot not found - manual TLS certificate management required"
        log_info "To enable Let's Encrypt, install certbot:"
        if [[ "$PKG_MANAGER" == "apt" ]]; then
            log_info "  sudo apt-get install certbot"
        elif [[ "$PKG_MANAGER" == "dnf" ]] || [[ "$PKG_MANAGER" == "yum" ]]; then
            log_info "  sudo $PKG_MANAGER install certbot"
        elif [[ "$PKG_MANAGER" == "pacman" ]]; then
            log_info "  sudo pacman -S certbot"
        fi
    fi
}

# Update package manager
update_package_manager() {
    show_progress "Updating package manager"
    $PKG_UPDATE 2>&1 | tee -a "$LOG_FILE" > /dev/null
    show_success "Package manager updated"
}

# Create installation directories
create_directories() {
    show_progress "Creating installation directories"

    mkdir -p "$INSTALL_DIR"/{binaries,web,data/attachments,logs,config}
    mkdir -p "$DATA_DIR"/{attachments,backups}

    show_success "Directories created at $INSTALL_DIR"
}

# Build from source
build_from_source() {
    if [[ "$SKIP_BUILD" == true ]]; then
        log_info "Skipping build (--skip-build specified)"
        return 0
    fi

    show_progress "Building SagensContact from source"

    # Detect if we're in the source directory
    if [[ ! -f "Cargo.toml" ]]; then
        log_error "Not in source directory. Please run from the project root."
        exit 1
    fi

    log_info "Building Rust workspace (this may take 5-10 minutes)..."
    cargo build --release --workspace 2>&1 | tee -a "$LOG_FILE"

    log_info "Building web UI..."
    cd apps/web
    pnpm install 2>&1 | tee -a "$LOG_FILE"
    pnpm build 2>&1 | tee -a "$LOG_FILE"
    cd ../..

    show_success "Build completed"
}

# Install binaries
install_binaries() {
    show_progress "Installing binaries"

    if [[ -f "target/release/sagenscontact" ]]; then
        cp target/release/sagenscontact "$INSTALL_DIR/binaries/"
        chmod +x "$INSTALL_DIR/binaries/sagenscontact"
        show_success "Installed CLI: sagenscontact"
    fi

    if [[ -f "target/release/sync_service" ]]; then
        cp target/release/sync_service "$INSTALL_DIR/binaries/"
        chmod +x "$INSTALL_DIR/binaries/sync_service"
        show_success "Installed API server: sync_service"
    fi

    if [[ -f "target/release/worker" ]]; then
        cp target/release/worker "$INSTALL_DIR/binaries/"
        chmod +x "$INSTALL_DIR/binaries/worker"
        show_success "Installed background worker"
    fi

    # Create symlinks
    if [[ "$EUID" -eq 0 ]]; then
        ln -sf "$INSTALL_DIR/binaries/sagenscontact" /usr/local/bin/sagenscontact
        show_success "Created symlink: /usr/local/bin/sagenscontact"
    fi
}

# Install web UI
install_web() {
    show_progress "Installing web UI"

    if [[ -d "apps/web/.svelte-kit/output" ]]; then
        cp -r apps/web/.svelte-kit/output/* "$INSTALL_DIR/web/"
        show_success "Web UI installed"
    else
        log_warn "Web UI build not found. Run 'cd apps/web && pnpm build' first."
    fi
}

# Generate JWT secret
generate_jwt_secret() {
    show_progress "Generating JWT secret"

    JWT_SECRET=$(openssl rand -base64 32)
    show_success "JWT secret generated"
}

# Create environment file
create_env_file() {
    show_progress "Creating environment configuration"

    cat > "$INSTALL_DIR/config/env" << EOF
# SagensContact Environment Configuration
# Generated on $(date)

# Database
DATABASE_URL="sqlite:$DATA_DIR/contacts.db"

# Server ports
SYNC_SERVICE_PORT=3002
WEB_UI_PORT=3001

# Security
JWT_SECRET="$JWT_SECRET"

# Logging
LOG_FORMAT=json
LOG_LEVEL=info

# Storage
ATTACHMENT_STORAGE_PATH="$DATA_DIR/attachments"

# Optional: PostgreSQL (comment out DATABASE_URL above to use)
# DATABASE_URL="postgresql://sagenscontact:PASSWORD@localhost/sagenscontact"

# Optional: Email/SMS (configure real providers for production)
# SMTP_HOST=smtp.example.com
# SMTP_PORT=587
# SMTP_USER=user@example.com
# SMTP_PASSWORD=password
# SMS_API_KEY=your-api-key

EOF

    chmod 600 "$INSTALL_DIR/config/env"
    show_success "Environment file created at $INSTALL_DIR/config/env"
}

# Run database migrations
run_migrations() {
    show_progress "Running database migrations"

    export DATABASE_URL="sqlite:$DATA_DIR/contacts.db"

    # SQLite will auto-create on first connection
    # Migrations are embedded in the binary
    if [[ -f "$INSTALL_DIR/binaries/sagenscontact" ]]; then
        log_info "Database will be initialized on first start"
        show_success "Migration setup complete"
    fi
}

# Create startup scripts
create_startup_scripts() {
    show_progress "Creating startup scripts"

    # Sync service startup script
    cat > "$INSTALL_DIR/scripts/start-sync-service.sh" << 'EOF'
#!/bin/bash
set -a
source "$(dirname "$0")/../config/env"
set +a

cd "$(dirname "$0")/.."
exec ./binaries/sync_service
EOF

    # Web UI startup script
    cat > "$INSTALL_DIR/scripts/start-web-ui.sh" << 'EOF'
#!/bin/bash
set -a
source "$(dirname "$0")/../config/env"
set +a

cd "$(dirname "$0")/../web"
PORT=$WEB_UI_PORT exec node index.js
EOF

    # Worker startup script
    cat > "$INSTALL_DIR/scripts/start-worker.sh" << 'EOF'
#!/bin/bash
set -a
source "$(dirname "$0")/../config/env"
set +a

cd "$(dirname "$0")/.."
exec ./binaries/worker
EOF

    # Combined startup script
    cat > "$INSTALL_DIR/scripts/start-all.sh" << 'EOF'
#!/bin/bash
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "Starting SagensContact services..."

# Start sync service
"$SCRIPT_DIR/start-sync-service.sh" > "$SCRIPT_DIR/../logs/sync-service.log" 2>&1 &
echo "Sync service started (PID: $!)"

# Wait for sync service
sleep 2

# Start web UI
"$SCRIPT_DIR/start-web-ui.sh" > "$SCRIPT_DIR/../logs/web-ui.log" 2>&1 &
echo "Web UI started (PID: $!)"

# Start worker
"$SCRIPT_DIR/start-worker.sh" > "$SCRIPT_DIR/../logs/worker.log" 2>&1 &
echo "Worker started (PID: $!)"

echo ""
echo "Services started!"
echo "  Web UI: http://localhost:3001"
echo "  API: http://localhost:3002"
echo "  Health: http://localhost:3002/health"
echo ""
echo "Logs: $SCRIPT_DIR/../logs/"
EOF

    mkdir -p "$INSTALL_DIR/scripts"
    chmod +x "$INSTALL_DIR"/scripts/*.sh

    show_success "Startup scripts created"
}

# Create systemd services
create_systemd_services() {
    if [[ "$NO_SERVICES" == true ]] || [[ "$EUID" -ne 0 ]]; then
        log_info "Skipping systemd service creation"
        return 0
    fi

    show_progress "Creating systemd services"

    # Sync service
    cat > /etc/systemd/system/sagenscontact-sync.service << EOF
[Unit]
Description=SagensContact Sync Service
After=network.target

[Service]
Type=simple
User=sagenscontact
Group=sagenscontact
WorkingDirectory=$INSTALL_DIR
EnvironmentFile=$INSTALL_DIR/config/env
ExecStart=$INSTALL_DIR/binaries/sync_service
Restart=on-failure
RestartSec=5s
StandardOutput=append:$INSTALL_DIR/logs/sync-service.log
StandardError=append:$INSTALL_DIR/logs/sync-service.log

[Install]
WantedBy=multi-user.target
EOF

    # Web UI service
    cat > /etc/systemd/system/sagenscontact-web.service << EOF
[Unit]
Description=SagensContact Web UI
After=network.target sagenscontact-sync.service

[Service]
Type=simple
User=sagenscontact
Group=sagenscontact
WorkingDirectory=$INSTALL_DIR/web
EnvironmentFile=$INSTALL_DIR/config/env
Environment="PORT=3001"
ExecStart=/usr/bin/node $INSTALL_DIR/web/index.js
Restart=on-failure
RestartSec=5s
StandardOutput=append:$INSTALL_DIR/logs/web-ui.log
StandardError=append:$INSTALL_DIR/logs/web-ui.log

[Install]
WantedBy=multi-user.target
EOF

    # Worker service
    cat > /etc/systemd/system/sagenscontact-worker.service << EOF
[Unit]
Description=SagensContact Background Worker
After=network.target sagenscontact-sync.service

[Service]
Type=simple
User=sagenscontact
Group=sagenscontact
WorkingDirectory=$INSTALL_DIR
EnvironmentFile=$INSTALL_DIR/config/env
ExecStart=$INSTALL_DIR/binaries/worker
Restart=on-failure
RestartSec=5s
StandardOutput=append:$INSTALL_DIR/logs/worker.log
StandardError=append:$INSTALL_DIR/logs/worker.log

[Install]
WantedBy=multi-user.target
EOF

    # Create service user
    if ! id "sagenscontact" &>/dev/null; then
        useradd -r -s /bin/false -d "$INSTALL_DIR" sagenscontact
        show_success "Created service user: sagenscontact"
    fi

    # Set permissions
    chown -R sagenscontact:sagenscontact "$INSTALL_DIR"

    # Reload systemd
    systemctl daemon-reload

    show_success "Systemd services created"
}

# Verify installation
verify_installation() {
    show_progress "Verifying installation"

    local errors=0

    # Check binaries
    if [[ ! -f "$INSTALL_DIR/binaries/sagenscontact" ]]; then
        log_error "Binary not found: sagenscontact"
        ((errors++))
    fi

    if [[ ! -f "$INSTALL_DIR/binaries/sync_service" ]]; then
        log_error "Binary not found: sync_service"
        ((errors++))
    fi

    # Check config
    if [[ ! -f "$INSTALL_DIR/config/env" ]]; then
        log_error "Environment config not found"
        ((errors++))
    fi

    # Check scripts
    if [[ ! -f "$INSTALL_DIR/scripts/start-all.sh" ]]; then
        log_error "Startup script not found"
        ((errors++))
    fi

    if [[ $errors -gt 0 ]]; then
        show_failure "Verification failed with $errors errors"
        return 1
    fi

    show_success "Installation verified"
}

# Print summary
print_summary() {
    echo ""
    echo "═══════════════════════════════════════════════════════════════"
    if [[ "$UPGRADE_MODE" == true ]]; then
        echo -e "${GREEN}SagensContact Alpha Upgrade Complete!${NC}"
    else
        echo -e "${GREEN}SagensContact Alpha Installation Complete!${NC}"
    fi
    echo "═══════════════════════════════════════════════════════════════"
    echo ""
    echo "Version: $INSTALLER_VERSION"
    echo "Installation Directory: $INSTALL_DIR"
    echo "Data Directory: $DATA_DIR"
    echo "Configuration: $INSTALL_DIR/config/env"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Quick Start:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    if [[ "$NO_SERVICES" == false ]] && [[ "$EUID" -eq 0 ]]; then
        echo "Start services:"
        echo "  sudo systemctl start sagenscontact-sync"
        echo "  sudo systemctl start sagenscontact-web"
        echo "  sudo systemctl start sagenscontact-worker"
        echo ""
        echo "Enable at boot:"
        echo "  sudo systemctl enable sagenscontact-sync sagenscontact-web sagenscontact-worker"
        echo ""
        echo "Check status:"
        echo "  sudo systemctl status sagenscontact-sync"
        echo ""
    else
        echo "Start services manually:"
        echo "  $INSTALL_DIR/scripts/start-all.sh"
        echo ""
    fi

    echo "Access:"
    echo "  Web UI: http://localhost:3001"
    echo "  API: http://localhost:3002"
    echo "  Health check: curl http://localhost:3002/health"
    echo ""
    echo "CLI usage:"
    if [[ "$EUID" -eq 0 ]]; then
        echo "  sagenscontact import-sms /path/to/backup.xml"
    else
        echo "  $INSTALL_DIR/binaries/sagenscontact import-sms /path/to/backup.xml"
    fi
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Next Steps:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "1. Review configuration: $INSTALL_DIR/config/env"
    echo "2. Set up TLS reverse proxy (nginx/Caddy) - see docs/TLS_HTTPS_SETUP.md"
    echo "3. Configure monitoring (Prometheus) - metrics at /metrics endpoint"
    echo "4. Set up backups for: $DATA_DIR"
    echo ""
    echo "Documentation: $INSTALL_DIR/docs/"
    echo "Logs: $INSTALL_DIR/logs/"
    echo ""
    echo "Installation log: $LOG_FILE"
    echo ""
    echo "═══════════════════════════════════════════════════════════════"
}

# Main installation flow
main() {
    echo "═══════════════════════════════════════════════════════════════"
    echo "SagensContact Alpha - Turnkey Installer (v$INSTALLER_VERSION)"
    echo "═══════════════════════════════════════════════════════════════"
    echo ""

    parse_args "$@"

    log "Starting installation at $(date)"
    log "Installation directory: $INSTALL_DIR"
    log "Data directory: $DATA_DIR"

    check_root
    detect_os

    # Check for existing installation (must be early)
    check_existing_installation

    # If upgrading, backup and stop services first
    if [[ "$UPGRADE_MODE" == true ]]; then
        echo ""
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo "Upgrade: Preparing for upgrade"
        echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
        echo ""

        create_upgrade_backup
        stop_existing_services
        preserve_config
    fi

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Phase 1: Checking Prerequisites"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    if [[ "$EUID" -eq 0 ]]; then
        update_package_manager
    fi

    check_build_tools
    check_rust
    check_node
    check_pnpm
    check_sqlite
    check_tls_tools
    check_clamav

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Phase 2: Building and Installing"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    create_directories
    build_from_source
    install_binaries
    install_web

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Phase 3: Configuration"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    generate_jwt_secret

    # For upgrades, restore config instead of creating new
    if [[ "$UPGRADE_MODE" == true ]] && [[ -n "${PRESERVED_ENV:-}" ]]; then
        restore_config
    else
        create_env_file
    fi

    run_migrations
    create_startup_scripts
    create_systemd_services
    write_version_file

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Phase 4: Verification"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    verify_installation

    # Restart services after upgrade
    restart_services

    print_summary

    if [[ "$UPGRADE_MODE" == true ]]; then
        log "Upgrade completed successfully at $(date)"
    else
        log "Installation completed successfully at $(date)"
    fi
}

# Run main
main "$@"
