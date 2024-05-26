#!/usr/bin/env bash
export TERM=linux

: "${APPLICATION_DIR?"APPLICATION_DIR is a required environment variable!"}"
: "${DATA_DIR?"DATA_DIR is a required environment variable!"}"
: "${APPLICATION_HOST?"APPLICATION_HOST is a required environment variable!"}"

# If Application dir and data dir are the same exit
if [[ "${APPLICATION_DIR}" == "${DATA_DIR}" ]]; then
    echo "APPLICATION_DIR and DATA_DIR cannot be the same!"
    echo "Application dir: ${APPLICATION_DIR}"
    echo "Data dir: ${DATA_DIR}"
    exit 1
fi

echo "###########################################################################"
echo "# FoundryVTT -  $(date)"
echo "###########################################################################"


# Change working directory to ${DATA_DIR} to allow relative path
mkdir -p "${APPLICATION_DIR}"
mkdir -p "${DATA_DIR}"


function launchUploader() {
    echo "Launching the uploader tool..."
    if [[ -f /usr/local/bin/rocket_server ]]; then
        echo "Launching Rocket server..."
        STATIC_FILES_DIR=/uploader-tool/frontend \
        ROCKET_PORT=4444 \
        ROCKET_ADDRESS=0.0.0.0 \
        TARGET_DIR=$APPLICATION_DIR \
        /usr/local/bin/rocket_server
    else
      echo "Rocket server not found!"
      exit 1
    fi


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
    echo "Woahhh!!! Something isnt right! I couldn't find the main.js file in ${APPLICATION_DIR}/resources/app/"
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

run_file="${APPLICATION_DIR}/${SCRIPT_PATH:-"resources/app/main.js"}"
if [[ -f "${run_file}" ]]; then
    echo "Running ${run_file}"
    echo "{ \"watch\": false }" > nodemon.json
    npx -y nodemon -C "${run_file}" -- "${FOUNDRY_VTT_ARGS[@]}"
else
    echo "File not found: ${run_file}"
    exit 1
fi
