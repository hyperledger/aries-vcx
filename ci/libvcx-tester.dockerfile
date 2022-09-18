ARG ALPINE_CORE_IMAGE
FROM ${ALPINE_CORE_IMAGE} as builder
USER indy
WORKDIR /home/indy

COPY --chown=indy  ./ ./

USER indy
ENV X86_64_UNKNOWN_LINUX_MUSL_OPENSSL_NO_VENDOR "true"
RUN cargo build --release --manifest-path=/home/indy/Cargo.toml

USER root
RUN mv /home/indy/target/release/libvcx.so .

FROM alpine:3.15.4

ARG UID=1000
ARG GID=1000

RUN addgroup -g $GID node && adduser -u $UID -D -G node node

COPY --from=builder /usr/lib/libvdrtools.so /home/indy/lib*.so /usr/lib/

WORKDIR /home/node
COPY --chown=node ./libvcx ./libvcx
COPY --chown=node ./agency_client ./agency_client
COPY --chown=node ./aries_vcx ./aries_vcx
COPY --chown=node ./wrappers/node ./wrappers/node
COPY --chown=node ./agents/node ./agents/node

RUN apk update && apk upgrade
RUN apk add --no-cache \
        bash \
        g++ \
        gcc \
        git \
        curl \
        libsodium-dev \
        libzmq \
        npm \
        make \
        openssl-dev \
        python3 \
        zeromq-dev
RUN npm install -g npm@8.7.0

RUN echo 'https://dl-cdn.alpinelinux.org/alpine/v3.12/main' >> /etc/apk/repositories
RUN apk add --no-cache nodejs=12.22.12-r0

USER node

ARG RUST_VER="1.62.1"
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain $RUST_VER --default-host x86_64-unknown-linux-musl
ENV PATH="/home/node/.cargo/bin:$PATH" RUSTFLAGS="-C target-feature=-crt-static"

# copy cargo caches - this way we don't have to redownload dependencies on subsequent builds
RUN mkdir -p /home/node/.cargo/registry
COPY --from=builder /home/indy/.cargo/registry /home/node/.cargo/registry
RUN chown -R node:node /home/node/.cargo/registry
