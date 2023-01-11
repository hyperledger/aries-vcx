FROM ghcr.io/napi-rs/napi-rs/nodejs-rust:lts-alpine
RUN apk update && apk upgrade && apk add openssl-dev zeromq-dev
