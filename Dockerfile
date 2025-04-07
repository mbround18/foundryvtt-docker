FROM rust:latest AS builder

WORKDIR /application

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release


FROM node:lts AS runtime

ARG CROC_VERSION=10.2.2
RUN --mount=type=cache,target=/var/cache/apt \
    --mount=type=cache,target=/var/lib/apt \
    apt-get update && apt-get upgrade -y && apt-get install -y --no-install-recommends \
    curl \
    sudo \
    iproute2 \
    net-tools \
    && curl -L https://github.com/schollz/croc/releases/download/v${CROC_VERSION}/croc_v${CROC_VERSION}_Linux-64bit.tar.gz \
    | tar -xz -C /usr/local/bin/ \
    # Remove existing UID 1000 or GID 1000 if they exist
    && if getent passwd 1000 > /dev/null; then userdel -f $(getent passwd 1000 | cut -d: -f1); fi \
    && if getent group 1000 > /dev/null; then groupdel $(getent group 1000 | cut -d: -f1); fi \
    # Create node user and group with UID:GID 1000:1000
    && groupadd -g 1000 node \
    && useradd -m -u 1000 -g 1000 -s /bin/bash node \
    && usermod -aG sudo node \
    && echo "node ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

COPY --from=builder /application/target/release/server /usr/local/bin/foundry-watcher

ENV APPLICATION_HOST="foundry.vtt" \
    APPLICATION_PORT="4444" \
    SSL_PROXY="true"

ENV APPLICATION_DIR="/foundryvtt"
ENV DATA_DIR="/foundrydata"

EXPOSE ${APPLICATION_PORT}

WORKDIR ${DATA_DIR}
COPY scripts/run.sh /home/node/run.sh
COPY ./server/static /foundry-watcher/frontend
RUN mkdir -p /foundryvtt /foundrydata /foundry-watcher/frontend \
    && chown node:node /home/node/run.sh \
    && chmod +x /home/node/run.sh \
    && chown -R node:node /foundry-watcher/frontend \
    && chmod -R 755 /foundry-watcher/frontend \
    && chown -R node:node /foundryvtt \
    && chmod -R 755 /foundryvtt \
    && chown -R node:node /foundrydata \
    && chmod -R 755 /foundrydata \
    && npm install -g npm

USER node

# Set the entrypoint to run the Rust application
ENTRYPOINT ["/home/node/run.sh"]
