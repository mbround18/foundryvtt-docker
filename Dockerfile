FROM node:lts

# Install and run the uploader tool
RUN mkdir -p /uploader-tool
RUN mkdir -p /uploader-tool/dist
COPY package.json yarn.lock .yarnclean /uploader-tool/
WORKDIR /uploader-tool/

RUN yarn install --prod
RUN yarn autoclean --force
COPY dist/ dist/


# Back to node directory
WORKDIR /home/node/

RUN yarn global add pm2 --prefix /usr/local
RUN mkdir -p /scripts
COPY scripts/run.sh /home/node/run.sh

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

ENTRYPOINT ["/home/node/run.sh"]