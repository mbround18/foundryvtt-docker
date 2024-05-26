FROM rust:latest as builder

WORKDIR /application

COPY ./Cargo.toml ./Cargo.toml ./
RUN cargo new --bin server
COPY ./server/Cargo.toml ./server/Cargo.toml

RUN cargo build --release

COPY ./server/src ./server/src
COPY ./server/static ./server/static

RUN --mount=type=cache,target=/usr/local/cargo/registry <<EOF
  set -e
  touch ./server/src/main.rs
  cargo build --release
EOF


FROM node:lts as runtime
ARG CROC_VERSION=10.04
ADD https://github.com/schollz/croc/releases/download/v${CROC_VERSION}/croc_v${CROC_VERSION}_Linux-64bit.tar.gz /tmp/croc.tar.gz

RUN tar -czvf /tmp/croc.tar.gz /usr/local/bin/

COPY --from=builder /application/target/release/rocket_server /usr/local/bin/rocket_server

ENV APPLICATION_HOST="foundry.vtt" \
    APPLICATION_PORT="4444" \
    SSL_PROXY="true"

ENV APPLICATION_DIR="/foundryvtt"
ENV DATA_DIR="/foundrydata"

EXPOSE ${APPLICATION_PORT}

WORKDIR ${DATA_DIR}
COPY scripts/run.sh /home/node/run.sh
COPY ./server/static /uploader-tool/frontend
RUN chown node:node /home/node/run.sh \
    && chmod +x /home/node/run.sh

# Set the entrypoint to run the Rust application
ENTRYPOINT ["/home/node/run.sh"]
