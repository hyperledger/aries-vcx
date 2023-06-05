ARG ALPINE_CORE_IMAGE
FROM ${ALPINE_CORE_IMAGE} AS builder

USER root
RUN apk update && apk upgrade && \
    apk add --no-cache \
        git

USER indy
RUN git clone https://github.com/hyperledger/indy-vdr.git
WORKDIR /home/indy/indy-vdr/indy-vdr-proxy
RUN git checkout 86395e3c
RUN cargo build --release

FROM alpine:3.18
RUN apk update && apk upgrade && \
    apk add --no-cache \
        libstdc++ \
        libgcc

COPY --from=builder /home/indy/indy-vdr/target/release/indy-vdr-proxy indy-vdr-proxy
ENTRYPOINT ["./indy-vdr-proxy"]
