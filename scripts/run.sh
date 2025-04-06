#!/usr/bin/env bash
set -Eeuo pipefail

echo "──────────────────────────────────────────────────────────"
echo "🎲 FoundryVTT - $(date)"
echo "──────────────────────────────────────────────────────────"

# System Info
echo "🔹 Hostname: $(hostname)"
echo "🔹 Kernel: $(uname -r)"
echo "🔹 OS: $(grep PRETTY_NAME /etc/os-release | cut -d= -f2 | tr -d '\"')"
echo "🔹 CPU: $(lscpu | grep 'Model name' | cut -d: -f2 | sed 's/^ *//')"
echo "🔹 Memory: $(free -h | awk '/^Mem:/ {print $2}')"
echo "🔹 Disk Space: $(df -h / | awk 'NR==2 {print $4}')"
echo "──────────────────────────────────────────────────────────"

# Required env
: "${APPLICATION_DIR?"APPLICATION_DIR is required!"}"
: "${DATA_DIR?"DATA_DIR is required!"}"
: "${APPLICATION_HOST?"APPLICATION_HOST is required!"}"

# Sanity Check
if [[ "${APPLICATION_DIR}" == "${DATA_DIR}" ]]; then
    echo "❌ APPLICATION_DIR and DATA_DIR cannot be the same!"
    echo "   Application: ${APPLICATION_DIR}"
    echo "   Data: ${DATA_DIR}"
    exit 1
fi

# Ownership & permissions
echo "🔄 Ensuring ${DATA_DIR} and ${APPLICATION_DIR} exist and are writable..."
mkdir -p "${DATA_DIR}" "${APPLICATION_DIR}"
echo "👤 Running as: $(whoami) (UID: $(id -u), GID: $(id -g))"
echo "👥 Groups: $(id -Gn)"

echo "🔍 Permissions check:"
ls -ld "${DATA_DIR}" "${APPLICATION_DIR}" || true

echo "🔧 Fixing ownership..."
chown -R "$(id -u):$(id -g)" "${DATA_DIR}" "${APPLICATION_DIR}" 2>/dev/null || true

# Upload fallback
function launchUploader() {
    echo "🚀 Launching upload helper..."
    if [[ -f /usr/local/bin/uploading-tool ]]; then
        STATIC_FILES_DIR=/uploader-tool/frontend \
        SERVER_PORT=${APPLICATION_PORT:-4444} \
        SERVER_HOST=0.0.0.0 \
        TARGET_DIR=$APPLICATION_DIR \
        /usr/local/bin/uploading-tool
        echo "✅ Upload complete." >> "${DATA_DIR}/.uploaded"
    else
        echo "❌ Uploader not found!"
        exit 1
    fi
}

# Uploader if missing
[[ ! -f "${DATA_DIR}/.uploaded" ]] && launchUploader

# Ensure main.js exists
if [[ ! -f "${APPLICATION_DIR}/resources/app/main.js" ]]; then
    echo "❌ main.js not found, re-running uploader."
    launchUploader
fi

# Start Foundry
echo "🛠 Building FoundryVTT args..."
FOUNDRY_VTT_ARGS=(
  "--dataPath=${DATA_DIR}"
  "--port=4444"
  "--hostname=${APPLICATION_HOST}"
  "--noupnp"
)
[[ "${SSL_PROXY,,}" == "true" ]] && FOUNDRY_VTT_ARGS+=("--proxySSL")

echo "🚀 Launching FoundryVTT with: ${FOUNDRY_VTT_ARGS[*]}"
trap 'echo "🧼 Caught signal, cleaning up..."; exit 0' SIGINT SIGTERM

RUN_FILE="${APPLICATION_DIR}/${SCRIPT_PATH:-resources/app/main.js}"
if [[ -f "${RUN_FILE}" ]]; then
    echo "{ \"watch\": false }" > nodemon.json || echo "⚠️ Failed to write nodemon.json"
    npx -y nodemon -C "${RUN_FILE}" -- "${FOUNDRY_VTT_ARGS[@]}"
else
    echo "❌ File not found: ${RUN_FILE}"
    exit 1
fi
