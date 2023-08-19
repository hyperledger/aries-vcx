# <img alt="Hyperledger Aries logo" src="docs/aries-logo.png" width="45px" /> aries-vcx

![CI build](https://github.com/hyperledger/aries-vcx/workflows/CI/badge.svg)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Join the chat at https://chat.hyperledger.org/channel/aries](https://img.shields.io/badge/Chat%20on-Hyperledger%20Chat-blue)](https://chat.hyperledger.org/channel/aries)

The repository contains
- Rust library `aries-vcx` implementing Aries protocols,
- collection of supporting projects.

## If you are Rust 🦀 developer
You can build your Rust project on top of
- [`aries-vcx`](aries_vcx) - ready to go Rust library to work with Aries protocols for didcomm, VC issuance and verification.

Additionally, `aries-vcx` is built on top of smaller Rust crates which are part of this repo:
- [`aries_vcx_core`](aries_vcx_core) - foundational APIs to interact with ledger, wallet and anoncreds.
- [`messages`](messages) - crate for building and parsing Aries messages
- [`did_doc`](diddoc) - crate to work with DIDDocs 

## If you are mobile 📱 developer
Aries-vcx can be used to build native mobile applications. You can write part of your mobile backend in Rust on top of
`aries-vcx` crate. Then expose FFI API for iOS/android environments.
- There's POC in progress [`uniffi_aries_vcx`](./uniffi_aries_vcx) using UniFFI library to autogenerate Swift and Kotlin wrappers.

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
