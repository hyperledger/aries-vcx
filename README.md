# <img alt="Hyperledger Aries logo" src="docs/aries-logo.png" width="45px" /> aries-vcx

![CI build](https://github.com/hyperledger/aries-vcx/workflows/CI/badge.svg)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Join the chat at https://chat.hyperledger.org/channel/aries](https://img.shields.io/badge/Chat%20on-Hyperledger%20Chat-blue)](https://chat.hyperledger.org/channel/aries)

The repository contains a set of crates to build [Aries](https://github.com/hyperledger/aries-rfcs/) / [DIDComm](https://didcomm.org/) applications in Rust.

## Aries components
  - [`aries_vcx`](aries_vcx) - Library implementing DIDComm protocols, with focus on verifiable credential issuance and verification.
  - [`messages`](messages) - Library for building and parsing Aries messages.
  - `aries_vcx_core` - Interfaces for interaction with ledgers, wallets and credentials.
  - [`agents`](aries/agents/rust) - Aries agents built on top of `aries_vcx`.
  
## General components
  - `did_parser` - Building and parsing [DIDs](https://w3c.github.io/did-core/).
  - `did_doc` - Building and parsing DID Documents.
  - `did_peer`, `did_sov`, `did_web`, `did_key` - DID resolvers for different [DID methods](https://w3c.github.io/did-spec-registries/#did-methods). 

## Mobile 📱
  - [`uniffi_aries_vcx`](./uniffi_aries_vcx) - UniFFI wrapper for `aries_vcx` and sample mobile app
  - [`simple_message_relay`](./tools/simple_message_relay) - simple implementation of message relay service for development / testing purposes

# Reach out 👋
- Ask a question on [discord](https://discord.com/channels/905194001349627914/955480822675308604)
- Talk to us on public community call every Thursday @ 09:00 am UTC via Zoom, see [details](https://wiki.hyperledger.org/display/ARIES/Community+calls)
- See high level 2023 roadmap at [ROADMAP_2023.md](docs/ROADMAP_2023.md)
- We welcome new contributors! Connect with us via the channels above and take a look at [CONTRIBUTING.md](CONTRIBUTING.md)

## Versioning & releases
- All releases have currently major version `0` 
  - We bump minor version on releases containing new features, significant refactors or breaking changes. 
  - We bump patch version if release only contains fixes or smaller refactoring.
- See [releases](https://github.com/hyperledger/aries-vcx/releases) page.
