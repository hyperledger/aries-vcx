FROM alpine:3.15.4 AS builder

ARG UID=1000
ARG GID=1000

ARG INDYSDK_PATH=/home/indy/vdr-tools
ARG INDYSDK_REPO=https://gitlab.com/mirgee/vdr-tools.git
ARG INDYSDK_REVISION=c1390f91

ENV RUST_LOG=warning

RUN addgroup -g $GID indy && adduser -u $UID -D -G indy indy

RUN apk update && apk upgrade && \
    apk add --no-cache \
        build-base \
        git \
        curl \
        libsodium-dev \
        libzmq \
        openssl-dev \
        zeromq-dev

USER indy
WORKDIR /home/indy

ARG RUST_VER="1.62.1"
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain $RUST_VER --default-host x86_64-unknown-linux-musl
ENV PATH="/home/indy/.cargo/bin:$PATH" RUSTFLAGS="-C target-feature=-crt-static"

RUN git clone $INDYSDK_REPO && cd $INDYSDK_PATH && git checkout $INDYSDK_REVISION

RUN cargo build --release --manifest-path=$INDYSDK_PATH/libindy/Cargo.toml

USER root
RUN mv $INDYSDK_PATH/libindy/target/release/libindy.so /usr/lib
