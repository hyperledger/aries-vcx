FROM alpine:3.15.4 AS builder

ARG UID=1000
ARG GID=1000

RUN addgroup -g $GID indy && adduser -u $UID -D -G indy indy

# zeromq-dev depends on libsodium-dev and pkg-config

RUN apk update && apk upgrade && \
    apk add --no-cache \
        build-base \
        curl \
        openssl-dev \
        zeromq-dev \
        cmake

USER indy
WORKDIR /home/indy

ARG RUST_VER="1.64.0"
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain $RUST_VER --default-host x86_64-unknown-linux-musl

ENV PATH="/home/indy/.cargo/bin:$PATH" RUSTFLAGS="-C target-feature=-crt-static"
ENV RUST_LOG=warning
