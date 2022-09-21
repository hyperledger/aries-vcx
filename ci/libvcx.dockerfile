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

COPY --from=builder /home/indy/lib*.so /usr/lib/

WORKDIR /home/node
RUN apk update && apk upgrade
RUN apk add --no-cache \
        libsodium-dev \
        libzmq \
        openssl-dev \
        zeromq-dev
RUN echo 'https://dl-cdn.alpinelinux.org/alpine/v3.12/main' >> /etc/apk/repositories
RUN apk add --no-cache nodejs=12.22.12-r0

USER node
