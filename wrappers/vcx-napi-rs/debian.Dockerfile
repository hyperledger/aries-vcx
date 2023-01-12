FROM ghcr.io/napi-rs/napi-rs/nodejs-rust:lts-debian
USER root
RUN apt update && apt -y install libsodium-dev libssl-dev libzmq3-dev
