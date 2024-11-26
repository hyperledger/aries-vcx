FROM alpine:3.17.1 AS builder

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

ARG RUST_VER="1.79.0"
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain $RUST_VER --default-host x86_64-unknown-linux-musl

ENV PATH="/home/indy/.cargo/bin:$PATH"
ENV RUST_LOG=warning RUSTFLAGS="-C target-feature=-crt-static"

USER root
RUN apk update && apk upgrade && \
    apk add --no-cache \
        git

USER indy
RUN git clone https://github.com/hyperledger/indy-vdr.git
WORKDIR /home/indy/indy-vdr/indy-vdr-proxy
RUN git checkout c143268
RUN cargo build --release

FROM alpine:3.18
RUN apk update && apk upgrade && \
    apk add --no-cache \
        libstdc++ \
        libgcc

COPY --from=builder /home/indy/indy-vdr/target/release/indy-vdr-proxy indy-vdr-proxy
ENTRYPOINT ["./indy-vdr-proxy"]
