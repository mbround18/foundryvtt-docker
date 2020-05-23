#!/usr/bin/env bash
export TERM=linux

: ${APPLICATION_DIR?"APPLICATION_DIR is a required environment variable!"}
: ${DATA_DIR?"DATA_DIR is a required environment variable!"}
: ${APPLICATION_HOST?"APPLICATION_HOST is a required environment variable!"}

echo "###########################################################################"
echo "# FoundryVTT - " `date`
echo "###########################################################################"


# Change working directory to ${DATA_DIR} to allow relative path
mkdir -p ${APPLICATION_DIR}
mkdir -p ${DATA_DIR}


function launchUploader() {
    echo "Launching the uploader tool..."
    node /uploader-tool/dist/server
    [[ -f "/tmp/foundryvtt.zip" ]] && unzip -q /tmp/foundryvtt.zip -d ${APPLICATION_DIR}
    echo "foundary was uploaded recently" >> "${DATA_DIR}/.uploaded"
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
FOUNDRYVTT_ARGS=("--dataPath=${DATA_DIR}" "--port=4444" "--hostname=${APPLICATION_HOST}" "--noupnp")
["${SSL_PROXY,,}" = "true"] && FOUNDRYVTT_ARGS+=("--proxySSL")

echo "Launching FoundryVTT with: ${FOUNDRYVTT_ARGS[@]}"
trap stop INT
trap stop TERM
pm2-runtime -i 1 ${APPLICATION_DIR}/resources/app/main.js -- "${FOUNDRYVTT_ARGS[@]}"
