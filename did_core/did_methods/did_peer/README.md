# did_peer

## Overview

Rust crate for creation, parsing, validation, and resolution of [Peer DIDs](https://identity.foundation/peer-did-method-spec).
Peer DIDs are a special type of decentralized identifiers designed for direct peer-to-peer interactions, without the
need for a blockchain or other centralized registry.

## Features

- **Numalgo Support**: The library implements various version of did:peer. The different versions are referred to as "numalgos".
  Currently supports numalgo 1, 2, 3, and 4.
- **DID Parsing**: Capability to parse `did:peer` strings, ensuring they comply with the Peer DID specifications.
- **DID Creation from DIDDoc**: Functionality to create `did:peer` identifiers from DID documents.
- **Numalgo Conversion**: Ability to convert between different numalgos, specifically from Numalgo 2 to Numalgo 3.
- **Validation**: Verification that DIDs adhere to the required specifications and format.

## Getting Started

### Installation

Add the Peer DID library as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
peer_did = { tag = "0.67.0", git = "https://github.com/hyperledger/aries-vcx" }
```

## Demo

To get you off the ground, have a look at the [demo](examples/demo.rs). It demonstrates how to create, parse. You can
run the demo with the following command:

```bash
cargo run --example demo
```
