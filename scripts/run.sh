#!/usr/bin/env bash

: ${PATREON_LINK?"PATREON_LINK is a required environment variable!"}
: ${APPLICATION_DIR?"APPLICATION_DIR is a required environment variable!"}
: ${DATA_DIR?"DATA_DIR is a required environment variable!"}
: ${APPLICATION_HOST?"APPLICATION_HOST is a required environment variable!"}

echo "###########################################################################"
echo "# FoundryVTT - " `date`
echo "###########################################################################"


# Change working directory to ${DATA_DIR} to allow relative path
mkdir -p ${APPLICATION_DIR}
mkdir -p ${DATA_DIR}

[ -p /tmp/FIFO ] && rm /tmp/FIFO
mkfifo /tmp/FIFO

export TERM=linux

wget -q  https://foundryvtt.s3-us-west-2.amazonaws.com/releases/${PATREON_LINK} -O /tmp/foundryvtt.zip
unzip -q /tmp/foundryvtt.zip -d ${APPLICATION_DIR}

FOUNDRYVTT_ARGS=("--dataPath=${DATA_DIR}" "--port=4444" "--hostname=${APPLICATION_HOST}" "--noupnp")
["${SSL_PROXY,,}" = "true"] && FOUNDRYVTT_ARGS+=("--proxySSL")

echo "Launching FoundryVTT with: ${FOUNDRYVTT_ARGS[@]}"
trap stop INT
trap stop TERM
pm2-runtime -i 1 ${APPLICATION_DIR}/resources/app/main.js -- "${FOUNDRYVTT_ARGS[@]}"

# # Stop server in case of signal INT or TERM
# echo "Waiting..."

# read < /tmp/FIFO &
# wait
