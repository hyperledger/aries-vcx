ARG ALPINE_CORE_IMAGE
FROM ${ALPINE_CORE_IMAGE} as builder
USER indy
WORKDIR /home/indy

COPY --chown=indy  ./ ./

USER indy
RUN cargo build --release --manifest-path=/home/indy/Cargo.toml

USER root
RUN mv /home/indy/target/release/libvcx.so .

FROM alpine:3.12

ARG UID=1000
ARG GID=1000

RUN addgroup -g $GID node && adduser -u $UID -D -G node node

COPY --from=builder /usr/lib/libindy.so /home/indy/lib*.so /usr/lib/

WORKDIR /home/node
COPY --chown=node ./libvcx ./libvcx
COPY --chown=node ./agency_client ./agency_client
COPY --chown=node ./wrappers/node ./wrappers/node
COPY --chown=node ./agents/node ./agents/node

RUN echo '@alpine38 http://dl-cdn.alpinelinux.org/alpine/v3.8/main' >> /etc/apk/repositories

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
        python2 \
        zeromq-dev

ARG RUST_VER="1.52.1"
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain $RUST_VER

USER node
