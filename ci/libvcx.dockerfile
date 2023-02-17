ARG ALPINE_CORE_IMAGE
FROM ${ALPINE_CORE_IMAGE} as builder
USER indy
WORKDIR /home/indy

COPY --chown=indy  ./ ./

USER indy
RUN cargo build --release --manifest-path=/home/indy/libvcx/Cargo.toml
USER root
RUN mv /home/indy/target/release/libvcx.so .


FROM alpine:3.17.1
ARG UID=1000
ARG GID=1000
RUN addgroup -g $GID node && adduser -u $UID -D -G node node

COPY --from=builder /home/indy/lib*.so /usr/lib/

WORKDIR /home/node
RUN apk update && apk upgrade
RUN apk add --no-cache \
        tzdata \
        openssl-dev \
        zeromq-dev # zeromq-dev depends on libsodium-dev and pkg-config

RUN cp /usr/share/zoneinfo/UTC /etc/localtime && echo UTC > /etc/timezone

ENV TZ=UTC

RUN echo 'https://dl-cdn.alpinelinux.org/alpine/v3.17/main' >> /etc/apk/repositories
RUN apk add --no-cache nodejs=18.14.1-r0

USER node
