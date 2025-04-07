#!/usr/bin/env bash
set -Eeuo pipefail

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Logging functions
log_info() { echo -e "${GREEN}[INFO]${NC} $(date +'%Y-%m-%d %H:%M:%S') $*"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $(date +'%Y-%m-%d %H:%M:%S') $*"; }
log_error() { echo -e "${RED}[ERROR]${NC} $(date +'%Y-%m-%d %H:%M:%S') $*"; }
log_debug() { echo -e "${BLUE}[DEBUG]${NC} $(date +'%Y-%m-%d %H:%M:%S') $*"; }

# Error handling
cleanup() {
    log_info "Cleaning up..."
    # Add any cleanup logic here
}

handle_error() {
    local exit_code=$?
    log_error "An error occurred on line $1 with exit code $exit_code"
    cleanup
    exit $exit_code
}

trap 'handle_error $LINENO' ERR
trap cleanup EXIT

# Function to check for required environment variables
check_required_env() {
    local missing=0
    
    for var in "$@"; do
        if [[ -z "${!var:-}" ]]; then
            log_error "$var is required but not set!"
            missing=1
        else
            log_debug "$var = ${!var}"
        fi
    done
    
    if [[ $missing -eq 1 ]]; then
        exit 1
    fi
}

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Print banner
log_info "──────────────────────────────────────────────────────────"
log_info "🎲 FoundryVTT - $(date)"
log_info "──────────────────────────────────────────────────────────"

# System Info
log_info "🔹 Hostname: $(hostname)"
log_info "🔹 Kernel: $(uname -r)"
log_info "🔹 OS: $(grep PRETTY_NAME /etc/os-release | cut -d= -f2 | tr -d '\"')"
log_info "🔹 CPU: $(lscpu | grep 'Model name' | cut -d: -f2 | sed 's/^ *//')"
log_info "🔹 Memory: $(free -h | awk '/^Mem:/ {print $2}')"
log_info "🔹 Disk Space: $(df -h / | awk 'NR==2 {print $4}')"
log_info "🔹 Node Version: $(node --version)"
log_info "🔹 NPM Version: $(npm --version)"
log_info "──────────────────────────────────────────────────────────"

# Check required environment variables
check_required_env "APPLICATION_DIR" "DATA_DIR" "APPLICATION_HOST"

# Optional environment variables with defaults
APPLICATION_PORT="${APPLICATION_PORT:-4444}"
SSL_PROXY="${SSL_PROXY:-false}"

# Validate environment variables
if [[ "${APPLICATION_DIR}" == "${DATA_DIR}" ]]; then
    log_error "APPLICATION_DIR and DATA_DIR cannot be the same!"
    log_error "   Application: ${APPLICATION_DIR}"
    log_error "   Data: ${DATA_DIR}"
    exit 1
fi

if [[ ! "${APPLICATION_PORT}" =~ ^[0-9]+$ ]]; then
    log_error "APPLICATION_PORT must be a number: ${APPLICATION_PORT}"
    exit 1
fi

# Directory creation and permission check
log_info "🔄 Ensuring directories exist and are writable..."
for dir in "${DATA_DIR}" "${APPLICATION_DIR}"; do
    if [[ ! -d "$dir" ]]; then
        log_info "Creating directory: $dir"
        mkdir -p "$dir" || { log_error "Failed to create $dir"; exit 1; }
    fi
    
    if [[ ! -w "$dir" ]]; then
        log_warn "Directory not writable: $dir. Attempting to fix permissions..."
        chown -R "$(id -u):$(id -g)" "$dir" 2>/dev/null || 
            log_warn "Could not change ownership of $dir. This might cause issues."
        chmod -R u+w "$dir" 2>/dev/null || 
            log_warn "Could not change permissions of $dir. This might cause issues."
    fi
done

log_info "👤 Running as: $(whoami) (UID: $(id -u), GID: $(id -g))"
log_info "👥 Groups: $(id -Gn)"

log_debug "🔍 Detailed permissions check:"
for dir in "${DATA_DIR}" "${APPLICATION_DIR}"; do
    log_debug "Directory: $dir"
    log_debug "$(ls -ld "$dir" || echo "Could not check $dir")"
    log_debug "Free space: $(df -h "$dir" | awk 'NR==2 {print $4}')"
done

# Check if foundry-watcher exists
if [[ ! -f /usr/local/bin/foundry-watcher ]]; then
    log_error "❌ Foundry watcher not found at /usr/local/bin/foundry-watcher!"
    exit 1
fi

if [[ ! -d /foundry-watcher/frontend ]]; then
    log_error "❌ Frontend static files not found at /foundry-watcher/frontend!"
    exit 1
fi

# Network check - only run if commands exist
log_debug "🌐 Network configuration:"
if command_exists ip; then
    log_debug "$(ip addr show)"
else
    log_debug "IP command not available"
fi

if command_exists netstat; then
    log_debug "$(netstat -tulpn)"
elif command_exists ss; then
    log_debug "$(ss -tulpn)"
else
    log_debug "Neither netstat nor ss commands are available"
fi

# Launch process
log_info "🚀 Launching Foundry & Tools on port ${APPLICATION_PORT}..."
log_info "Configuration:"
log_info "  - Application directory: ${APPLICATION_DIR}"
log_info "  - Data directory: ${DATA_DIR}"
log_info "  - Host: ${APPLICATION_HOST}"
log_info "  - SSL Proxy: ${SSL_PROXY}"

log_info "Starting foundry watcher. Logs will follow below:"
log_info "──────────────────────────────────────────────────────────"

# Export variables correctly, then exec the command
export STATIC_FILES_DIR="/foundry-watcher/frontend"
export SERVER_PORT="${APPLICATION_PORT}"
export SERVER_HOST="0.0.0.0"
export TARGET_DIR="${APPLICATION_DIR}"
export SSL_PROXY="${SSL_PROXY}"
export APPLICATION_DIR="${APPLICATION_DIR}"

# Change to the DATA_DIR before executing the foundry-watcher
cd "${DATA_DIR}"
exec /usr/local/bin/foundry-watcher
