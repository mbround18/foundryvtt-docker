ARG RUST_VERSION=1.86
ARG NODE_VERSION=22
FROM rust:${RUST_VERSION}-alpine AS builder

WORKDIR /application

COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml server/

RUN apk add --no-cache \
    musl-dev \
    build-base \
    openssl-dev \
    pkgconfig \
    curl \
    bash \
    cmake


RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo fetch

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    rustup target add x86_64-unknown-linux-musl && \
    cargo build --release --target x86_64-unknown-linux-musl \
    && mv target/x86_64-unknown-linux-musl/release/foundry-watcher target/release/foundry-watcher

FROM node:${NODE_VERSION}-alpine AS runtime

ARG CROC_VERSION=10.2.2
RUN apk add --no-cache \
    curl \
    iproute2 \
    net-tools \
    shadow \
    sudo \
    bash \
    && curl -L https://github.com/schollz/croc/releases/download/v${CROC_VERSION}/croc_v${CROC_VERSION}_Linux-64bit.tar.gz \
    | tar -xz -C /usr/local/bin/ \
    && groupdel $(getent group 1000 | cut -d: -f1) 2>/dev/null || true \
    && userdel -f $(getent passwd 1000 | cut -d: -f1) 2>/dev/null || true \
    && groupadd -g 1001 sudo \
    && groupadd -g 1000 node \
    && useradd -m -u 1000 -g 1000 -s /bin/bash node \
    && usermod -aG sudo node \
    && echo "node ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

# Copy the binary ensuring it exists and is executable
COPY --from=builder --chmod=0755 /application/target/release/foundry-watcher /usr/local/bin/foundry-watcher
# Add better verification that the file exists and is executable
RUN ls -la /usr/local/bin/foundry-watcher && \
    chmod +x /usr/local/bin/foundry-watcher && \
    chown node:node /usr/local/bin/foundry-watcher

ENV APPLICATION_HOST="foundry.vtt" \
    APPLICATION_PORT="4444" \
    SSL_PROXY="true" \
    APPLICATION_DIR="/foundryvtt" \
    DATA_DIR="/foundrydata" \
    STATIC_FILES_DIR="/foundry-watcher/frontend" \
    SERVER_PORT="4444" \
    SERVER_HOST="0.0.0.0" \
    TARGET_DIR="/foundryvtt"

EXPOSE ${APPLICATION_PORT}

WORKDIR ${DATA_DIR}
COPY ./server/static /foundry-watcher/frontend
RUN mkdir -p /foundryvtt /foundrydata /foundry-watcher/frontend \
    && chown -R node:node /foundry-watcher/frontend \
    && chmod -R 755 /foundry-watcher/frontend \
    && chown -R node:node /foundryvtt \
    && chmod -R 755 /foundryvtt \
    && chown -R node:node /foundrydata \
    && chmod -R 755 /foundrydata \
    && npm install -g npm

USER node

RUN ls -la /usr/local/bin/foundry-watcher && \
    chmod +x /usr/local/bin/foundry-watcher && \
    chown node:node /usr/local/bin/foundry-watcher


# Set the entrypoint to run the Rust application directly
ENTRYPOINT ["/usr/local/bin/foundry-watcher"]
