FROM rust:1.84.1 as builder
RUN apt update && apt install -y libssl-dev libzmq3-dev cmake 

WORKDIR /usr/src/aries-vcx
COPY . .
RUN ls -lahF
RUN --mount=type=cache,target=./target \
    --mount=type=cache,target=${CARGO_HOME}/git \
    --mount=type=cache,target=${CARGO_HOME}/registry \
    cargo install --no-default-features --path=./aries/agents/mediator/ --bin mediator --locked

FROM debian:bookworm-slim as mediator
RUN apt update && apt install -y libsodium23 libzmq5 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/mediator /usr/local/bin/mediator
CMD ["./usr/local/bin/mediator"]