FROM node:lts


# Install and run the uploader tool
RUN mkdir -p /uploader-tool /uploader-tool/dist
COPY package.json yarn.lock .yarnclean /uploader-tool/
WORKDIR /uploader-tool/

RUN yarn install
COPY . .
RUN yarn build:prod             \
    && rm -rf node_modules/     \
    && yarn install --prod      \
    && yarn autoclean --force   \
    && rm -rf src/ scripts/ Dockerfile public/

# Back to node directory
WORKDIR /home/node/
RUN mkdir -p /scripts
COPY scripts/run.sh /home/node/run.sh
RUN chown node:node /home/node/run.sh \
    && chmod +x /home/node/run.sh

RUN apt-get update                     \
    && apt-get install -y unzip        \
    && rm -rf /var/lib/apt/lists/*

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