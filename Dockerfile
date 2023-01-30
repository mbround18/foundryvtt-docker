FROM node:lts as Runtime

# Back to node directory
WORKDIR /home/node/
RUN mkdir -p /scripts \
    && mkdir -p /uploader-tool/bin \
    && mkdir -p /uploader-tool/frontend

COPY scripts/run.sh /home/node/run.sh

RUN chown node:node /home/node/run.sh \
    && chmod +x /home/node/run.sh

RUN apt-get update && apt-get upgrade -y   \
    && apt-get install -y p7zip-full       \
    && curl https://getcroc.schollz.com | bash \
    && rm -rf /var/lib/apt/lists/*

COPY ./dist/bin/index /uploader-tool/uploader
COPY ./dist/frontend /uploader-tool/frontend

# CHANGE ME
ENV APPLICATION_HOST="foundry.vtt"              \
    APPLICATION_PORT="4444"                     \
    # Whether or not you are putting this behind SSL.
    SSL_PROXY="true"                            \
    # Directory setup
    APPLICATION_DIR="/foundryvtt"               \
    DATA_DIR="/foundrydata"                     \
    FOUNDRYVTT_TMP_PATH="/tmp/foundryvtt.zip"

EXPOSE ${APPLICATION_PORT}

WORKDIR ${DATA_DIR}

# Install pm2 for later use.
RUN yarn global add pm2 --prefix /usr/local
ENTRYPOINT ["/home/node/run.sh"]
