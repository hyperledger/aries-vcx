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
        cargo \
        git \
        libsodium-dev \
        libzmq \
        openssl-dev \
        rust \
        sqlite-dev \
        zeromq-dev

USER indy
WORKDIR /home/indy

RUN git clone $INDYSDK_REPO && \
    cd indy-sdk && git checkout $INDYSDK_REVISION

RUN cargo build --release --manifest-path=$INDYSDK_PATH/libindy/Cargo.toml

USER root
RUN mv $INDYSDK_PATH/libindy/target/release/libindy.so /usr/lib

USER indy
RUN cargo build --release --manifest-path=$INDYSDK_PATH/vcx/libvcx/Cargo.toml
RUN cargo build --release --manifest-path=$INDYSDK_PATH/libnullpay/Cargo.toml
RUN cargo build --release --manifest-path=$INDYSDK_PATH/experimental/plugins/postgres_storage/Cargo.toml

USER root
RUN mv $INDYSDK_PATH/vcx/libvcx/target/release/libvcx.so .
RUN mv $INDYSDK_PATH/libnullpay/target/release/libnullpay.so .
RUN mv $INDYSDK_PATH/experimental/plugins/postgres_storage/target/release/libindystrgpostgres.so .

FROM alpine:3.12

ARG UID=1000
ARG GID=1000

RUN addgroup -g $GID node && adduser -u $UID -D -G node node

COPY --from=builder /usr/lib/libindy.so /home/indy/lib*.so /usr/lib/

RUN echo '@alpine38 http://dl-cdn.alpinelinux.org/alpine/v3.8/main' >> /etc/apk/repositories

RUN apk update && apk upgrade
RUN apk add --no-cache \
        bash \
        g++ \
        gcc \
        git \
        libsodium-dev \
        libzmq \
        make \
        nodejs@alpine38 \
        npm@alpine38 \
        openssl-dev \
        python2 \
        sqlite-dev \
        zeromq-dev

LABEL org.label-schema.schema-version="0.8.0"
LABEL org.label-schema.name="libvcx"
LABEL org.label-schema.version="${INDYSDK_REVISION}"
