# did_parser_nom

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
did_parser_nom = { tag = "0.67.0", git = "https://github.com/hyperledger/aries-vcx" }
```
