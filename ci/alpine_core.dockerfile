FROM alpine:3.12 AS builder

ARG UID=1000
ARG GID=1000


ARG INDYSDK_PATH=/home/indy/indy-sdk
ARG INDYSDK_REPO=https://github.com/hyperledger/indy-sdk.git
ARG INDYSDK_REVISION=v1.15.0

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
ARG RUST_VER="1.45.2"
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain $RUST_VER
ENV PATH="/home/indy/.cargo/bin:${PATH}"

# https://github.com/rust-lang/rust/pull/58575
ENV RUSTFLAGS='-C target-feature=-crt-static'
RUN cargo --version
RUN rustc --print cfg
RUN rustup show
RUN rustup target list

WORKDIR /home/indy

RUN git clone $INDYSDK_REPO && cd indy-sdk && git checkout $INDYSDK_REVISION

ENV PATH_LIBINDY=$INDYSDK_PATH/libindy
ENV PATH_LIBNULLPAY=$INDYSDK_PATH/libnullpay
ENV PATH_LIBPGWALLET=$INDYSDK_PATH/experimental/plugins/postgres_storage
RUN cargo build --release --manifest-path=$PATH_LIBINDY/Cargo.toml --target-dir=$PATH_LIBINDY/target

RUN rustc --print cfg
RUN ls -lh
RUN ls -lh $PATH_LIBINDY
RUN ls -lh $PATH_LIBINDY/target
RUN ls -lh $PATH_LIBINDY/target/release

USER root
RUN mv $PATH_LIBINDY/target/release/libindy.so /usr/lib

USER indy
RUN cargo build --release --manifest-path=$PATH_LIBNULLPAY/Cargo.toml --target-dir=$PATH_LIBNULLPAY/target
RUN cargo build --release --manifest-path=$PATH_LIBPGWALLET/Cargo.toml --target-dir=$PATH_LIBPGWALLET/target

USER root
RUN mv $PATH_LIBNULLPAY/target/release/libnullpay.so .
RUN mv $PATH_LIBPGWALLET/target/release/libindystrgpostgres.so .