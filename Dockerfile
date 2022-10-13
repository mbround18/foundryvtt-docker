FROM node:lts as Builder

WORKDIR /uploader-tool/

COPY package.json yarn.lock .yarnclean .yarnrc.yml /uploader-tool/
COPY ./.yarn/ /uploader-tool/.yarn
RUN yarn install --immutable


COPY . .
RUN NODE_ENV=production yarn build        \
    && rm -rf src/ scripts/ Dockerfile public/


FROM node:lts as Runtime

# Back to node directory
WORKDIR /home/node/
RUN mkdir -p /scripts
COPY scripts/run.sh /home/node/run.sh
RUN chown node:node /home/node/run.sh \
    && chmod +x /home/node/run.sh

RUN apt-get update                     \
    && apt-get install -y p7zip-full        \
    && rm -rf /var/lib/apt/lists/*

COPY --from=Builder /uploader-tool/dist /uploader-tool/

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
