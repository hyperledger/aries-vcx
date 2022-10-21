FROM ubuntu:20.04 as BASE

ARG UID=1000
ARG DEBIAN_FRONTEND=noninteractive
ARG RUST_VER=nightly-2022-10-14

# Install dependencies
RUN apt-get update -qq && \
    apt-get install -y --no-install-recommends \
      build-essential \
      ca-certificates \
      cmake \
      curl \
      git \
      libssl-dev \
      libzmq3-dev \
      libsodium-dev \
      pkg-config

RUN useradd -ms /bin/bash -u $UID indy

USER indy
WORKDIR /home/indy

# Install Rust toolchain
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain ${RUST_VER}
ENV PATH /home/indy/.cargo/bin:$PATH

RUN cargo install grcov --version 0.8.9

WORKDIR /home/indy/aries-vcx
COPY --chown=indy ./ ./

RUN cargo test -p messages --no-run -F general_test
RUN cargo test -p agency_client --no-run -F general_test
RUN cargo test -p libvcx --no-run -F general_test
