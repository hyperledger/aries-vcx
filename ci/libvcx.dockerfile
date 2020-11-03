FROM alpine:3.12 AS builder

ARG UID=1000
ARG GID=1000

ARG INDYSDK_PATH=/home/indy/indy-sdk
ARG INDYSDK_REPO=https://github.com/hyperledger/indy-sdk.git
ARG INDYSDK_REVISION=v1.15.0

ENV RUST_LOG=warning
ARG RUST_VER="1.45.2"

RUN addgroup -g $GID indy && adduser -u $UID -D -G indy indy

RUN apk update && apk upgrade && \
    apk add --no-cache \
        build-base \
        cargo \
        git \
        libsodium-dev \
        libzmq \
        openssl-dev \
        zeromq-dev

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain $RUST_VER

USER indy
WORKDIR /home/indy

COPY --chown=indy  ./ ./

RUN git clone $INDYSDK_REPO && \
    cd indy-sdk && git checkout $INDYSDK_REVISION

RUN cargo build --release --manifest-path=$INDYSDK_PATH/libindy/Cargo.toml

USER root
RUN mv $INDYSDK_PATH/libindy/target/release/libindy.so /usr/lib

USER indy
RUN cargo build --release --manifest-path=/home/indy/libvcx/Cargo.toml
RUN cargo build --release --manifest-path=$INDYSDK_PATH/libnullpay/Cargo.toml
RUN cargo build --release --manifest-path=$INDYSDK_PATH/experimental/plugins/postgres_storage/Cargo.toml

USER root
RUN mv /home/indy/libvcx/target/release/libvcx.so .
RUN mv $INDYSDK_PATH/libnullpay/target/release/libnullpay.so .
RUN mv $INDYSDK_PATH/experimental/plugins/postgres_storage/target/release/libindystrgpostgres.so .

FROM alpine:3.12

ARG UID=1000
ARG GID=1000

RUN addgroup -g $GID node && adduser -u $UID -D -G node node

COPY --from=builder /usr/lib/libindy.so /home/indy/lib*.so /usr/lib/

WORKDIR /home/node
COPY --chown=node ./libvcx ./libvcx
COPY --chown=node ./wrappers/node ./wrappers/node
COPY --chown=node ./agents/node ./agents/node

RUN apk update && apk upgrade
RUN apk add --no-cache \
        bash \
        cargo \
        g++ \
        gcc \
        git \
        libsodium-dev \
        libzmq \
        nodejs \
        npm \
        make \
        openssl-dev \
        rust \
        python2 \
        zeromq-dev

USER node
