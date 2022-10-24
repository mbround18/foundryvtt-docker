#!/usr/bin/env bash
export TERM=linux

: "${APPLICATION_DIR?"APPLICATION_DIR is a required environment variable!"}"
: "${DATA_DIR?"DATA_DIR is a required environment variable!"}"
: "${APPLICATION_HOST?"APPLICATION_HOST is a required environment variable!"}"

echo "###########################################################################"
echo "# FoundryVTT -  $(date)"
echo "###########################################################################"


# Change working directory to ${DATA_DIR} to allow relative path
mkdir -p "${APPLICATION_DIR}"
mkdir -p "${DATA_DIR}"


function launchUploader() {
    echo "Launching the uploader tool..."
    FRONTEND_DIR=/uploader-tool/frontend PORT=4444 node /uploader-tool/backend/index.js
    [[ -f "/tmp/foundryvtt.zip" ]] && 7z x -aoa -O"${APPLICATION_DIR}" /tmp/foundryvtt.zip
    echo "foundry was uploaded recently" >> "${DATA_DIR}/.uploaded"
}



# EXPIRES=$(date +%s)

# echo -e "Downloading ${VTT_VERION}..."
# wget "https://foundryvtt.s3.amazonaws.com/releases/${VTT_VERSION}/foundryvtt-${VTT_VERSION}.zip?&AWSAccessKeyId=${AWS_ACCESS_KEY_ID}&Signature=${AWS_SIGNATURE}&Expires=${EXPIRES}" -O /tmp/foundryvtt.zip
if [[ ! -f "${DATA_DIR}/.uploaded" ]]; then
    launchUploader
fi

echo "Checking for application executable..."
if [[ ! -f "${APPLICATION_DIR}/resources/app/main.js" ]]; then
    echo "Woahhhhhh!!! Something isnt right! I couldn't find the main.js file in ${APPLICATION_DIR}/resources/app/"
    launchUploader
fi

echo "Building arguments..."
FOUNDRY_VTT_ARGS=("--dataPath=${DATA_DIR}" "--port=4444" "--hostname=${APPLICATION_HOST}" "--noupnp")
# shellcheck disable=SC2154
[[ "${SSL_PROXY,,}" -eq "true" ]] && FOUNDRY_VTT_ARGS+=("--proxySSL")

# shellcheck disable=SC2145
echo "Launching FoundryVTT with: ${FOUNDRY_VTT_ARGS[@]}"
trap stop INT
trap stop TERM
pm2-runtime -i 1 "${APPLICATION_DIR}/${SCRIPT_PATH:-"resources/app/main.js"}" -- "${FOUNDRY_VTT_ARGS[@]}"
