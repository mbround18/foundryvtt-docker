FROM node:lts

# Patreon Link
ENV PATREON_LINK ""

# CHANGE ME
ENV APPLICATION_HOST "foundry.vtt"
ENV APPLICATION_PORT "4444"

# Directory setup
ENV APPLICATION_DIR "/foundryvtt"
ENV DATA_DIR "/foundrydata"

# Whether or not you are putting this behind SSL. 
ENV SSL_PROXY "true"                


RUN npm install pm2 -g
RUN mkdir -p /scripts
COPY scripts/run.sh /home/node/run.sh
COPY scripts/user.sh /home/node/user.sh

RUN apt-get update &&\
    apt-get install -y unzip

EXPOSE ${APPLICATION_PORT}

WORKDIR ${DATA_DIR}

ENTRYPOINT ["/home/node/run.sh"]