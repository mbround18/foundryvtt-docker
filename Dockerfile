FROM node:lts


# Install and run the uploader tool
RUN mkdir -p /uploader-tool
RUN mkdir -p /uploader-tool/dist
COPY package.json yarn.lock .yarnclean /uploader-tool/
WORKDIR /uploader-tool/

RUN yarn install
RUN ls -l
COPY . .
RUN yarn build:prod
RUN rm -rf node_modules/
RUN yarn install --prod
RUN yarn autoclean --force
RUN rm -rf src/ scripts/ Dockerfile public/

# Back to node directory
WORKDIR /home/node/
RUN mkdir -p /scripts
COPY scripts/run.sh /home/node/run.sh
RUN chown node:node /home/node/run.sh
RUN chmod +x /home/node/run.sh

RUN apt-get update &&\
    apt-get install -y unzip

# CHANGE ME
ENV APPLICATION_HOST "foundry.vtt"
ENV APPLICATION_PORT "4444"

# Whether or not you are putting this behind SSL.
ENV SSL_PROXY "true"

# Directory setup
ENV APPLICATION_DIR "/foundryvtt"
ENV DATA_DIR "/foundrydata"

ENV FOUNDRYVTT_TMP_PATH "/tmp/foundryvtt.zip"

EXPOSE ${APPLICATION_PORT}

WORKDIR ${DATA_DIR}

# Install pm2 for later use.
RUN yarn global add pm2 --prefix /usr/local
ENTRYPOINT ["/home/node/run.sh"]