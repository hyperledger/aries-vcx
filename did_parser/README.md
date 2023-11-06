# did_parser

## Overview
Rust crate for parsing [DIDs](https://www.w3.org/TR/did-core/#did-syntax) and [DID URLs](https://www.w3.org/TR/did-core/#did-url-syntax).

## Features
- **DID Parsing**: Capability to parse `did:` strings, ensuring they comply with the DID specifications.
- **DID URL**: Functionality to parse DID URLs.

## Getting Started
### Installation
Add the did_parser library as a dependency in your `Cargo.toml` file:
```toml
[dependencies]
did_parser = { tag = "0.61.0", git = "https://github.com/hyperledger/aries-vcx" }
```

## Demo
To get you off the ground, have a look at the [demo](./examples/demo.rs). It demonstrates a basic functionality. You can run the demo with the following command:
```bash
cargo run --example demo
```

