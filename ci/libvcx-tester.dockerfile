ARG ALPINE_CORE_IMAGE
FROM ${ALPINE_CORE_IMAGE} as builder
USER indy
WORKDIR /home/indy
COPY --chown=indy  ./ ./
RUN cargo build --release --manifest-path=/home/indy/libvcx/Cargo.toml

USER root
RUN mv /home/indy/target/release/libvcx.so .

FROM alpine:3.15.4
ARG UID=1000
ARG GID=1000
RUN addgroup -g $GID node && adduser -u $UID -D -G node node

RUN apk update && apk upgrade && \
    apk add --no-cache \
        bash \
        build-base \
        curl \
        openssl-dev \
        zeromq-dev \
        python3 \
        npm git tzdata

RUN cp /usr/share/zoneinfo/UTC /etc/localtime && echo UTC > /etc/timezone

ENV TZ=UTC

COPY --from=builder /home/indy/lib*.so /usr/lib/

WORKDIR /home/node
COPY --chown=node ./Cargo.toml ./Cargo.lock ./
COPY --chown=node ./libvcx ./libvcx
COPY --chown=node ./agency_client ./agency_client
COPY --chown=node ./messages ./messages
COPY --chown=node ./aries_vcx ./aries_vcx
COPY --chown=node ./wrappers/node ./wrappers/node
COPY --chown=node ./agents ./agents


RUN npm install -g npm@8.7.0

RUN echo 'https://dl-cdn.alpinelinux.org/alpine/v3.12/main' >> /etc/apk/repositories
RUN apk add --no-cache nodejs=12.22.12-r0

USER node

ARG RUST_VER="1.64.0"
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain $RUST_VER --default-host x86_64-unknown-linux-musl
ENV PATH="/home/node/.cargo/bin:$PATH" RUSTFLAGS="-C target-feature=-crt-static"

# copy cargo caches - this way we don't have to redownload dependencies on subsequent builds
RUN mkdir -p /home/node/.cargo/registry
COPY --from=builder /home/indy/.cargo/registry /home/node/.cargo/registry
RUN chown -R node:node /home/node/.cargo/registry
