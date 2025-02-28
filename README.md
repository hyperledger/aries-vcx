# aries-vcx

![CI build](https://github.com/hyperledger/aries-vcx/workflows/CI/badge.svg)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Join the chat at https://chat.hyperledger.org/channel/aries](https://img.shields.io/badge/Chat%20on-Hyperledger%20Chat-blue)](https://chat.hyperledger.org/channel/aries)

The repository contains Rust crates to build

- [Aries](https://github.com/hyperledger/aries-rfcs/) based applications (mobile, server, anything, ...),
- [DIDComm](https://didcomm.org/) related components.

## Aries implementation

- [`aries_vcx`](aries/aries_vcx) - Library implementing DIDComm protocols, with focus on verifiable credential issuance and verification.
- [`messages`](aries/messages) - Library for building and parsing Aries (DIDComm v1) messages.
- [`aries_vcx_anoncreds`](aries/aries_vcx_anoncreds) - Interfaces for interaction with credentials.
- [`aries_vcx_ledger`](aries/aries_vcx_ledger) - Interfaces for interaction with ledgers.
- [`aries_vcx_wallet`](aries/aries_vcx_wallet) - Interfaces for interaction with wallets.
- [`agents`](aries/agents) - Aries agents built on top of `aries_vcx`.

## Did document implementation

- [`did_doc`](did_core/did_doc) - Building and parsing [DID Documents](https://w3c.github.io/did-core/)

## Did methods implementation

- [`did_parser`](did_core/did_parser_nom) - Building and parsing [DIDs](https://w3c.github.io/did-core/)
- [`did_peer`](did_core/did_methods/did_peer) - https://identity.foundation/peer-did-method-spec/
- [`did_sov`](did_core/did_methods/did_resolver_sov) - https://sovrin-foundation.github.io/sovrin/spec/did-method-spec-template.html
- [`did_cheqd`](did_core/did_methods/did_cheqd) - https://docs.cheqd.io/product/architecture/adr-list/adr-001-cheqd-did-method
- [`did_web`](did_core/did_methods/did_resolver_web) - https://w3c-ccg.github.io/did-method-web/
- [`did_key`](did_core/did_methods/did_key) - https://w3c-ccg.github.io/did-method-key/
- [`did_jwk`](did_core/did_methods/did_jwk) - https://github.com/quartzjer/did-jwk/blob/main/spec.md

# Contact

Do you have a question ❓Are you considering using our components? 🚀 We'll be excited to hear from you. 👋

There's 2 best way to reach us:

- Leave us message on `aries-vcx` [discord](https://discord.com/channels/905194001349627914/955480822675308604) channel.
- Join our Zoom community calls. Biweekly Tuesdays @ 11:00 pm UTC via Zoom, find more details on [wiki](https://wiki.hyperledger.org/display/ARIES/Community+calls)

## Versioning & releases

- Crates are not yet published on crates.io. You can consume crates as github-type Cargo dependency.
- All releases have currently major version `0`
  - We bump minor version on releases containing new features, significant refactors or breaking changes.
  - We bump patch version if release only contains fixes or smaller refactoring.
- See [releases](https://github.com/hyperledger/aries-vcx/releases) page.
- MSRV 1.81 - Crates are known to be stable with atleast Rust version 1.81

# Contributions

Contributions are very welcome! If you have questions or issues, please let us know on [Discord](https://chat.hyperledger.org/channel/aries) or at our [bi-weekly community call](https://wiki.hyperledger.org/display/ARIES/Community+calls).

## Install

Install Rust: https://www.rust-lang.org/tools/install

We recommend using rustup, as VCX is currently targeting Rust v1.84.1 (this maintains consistency between local and CI environments). 

Anoncreds and Indy require the use of openssl and zmq. These may be vendored by consuming applications, but for development installation is required. 

> [!NOTE]
> For those familiar with the Indy SDK dependencies (which is no longer in use as it has been replaced by [anoncreds-rs](https://github.com/openwallet-foundation/askar), [indy-vdr](https://github.com/hyperledger/indy-vdr/tree/main), and [aries-askar](https://github.com/openwallet-foundation/askar)) note that: 
> - Openssl requirements are no longer restricted to the out of support 1.1 version.
> - Libsodium is no longer required (as it's been replaced by [anoncreds-clsignatures](https://github.com/hyperledger/anoncreds-clsignatures-rs))

### Linux / MacOS:

- ZMQ: https://zeromq.org/download/
- Openssl: https://docs.rs/openssl/latest/openssl/#automatic

### Windows

If you get a VCX development environment running in Windows, we'd love a contribution documenting that process here!

## Formatting / Linting

For contributions, please run `clippy` and format prior to creating a PR. This can be done via `just`:

```
cargo install just
just clippy
just fmt
```

## Signed Commits

We enforce [developer certificate of origin](https://developercertificate.org/) (DCO) commit signing — [guidance](https://github.com/apps/dco) on this is available.

See this [guide](https://hackmd.io/@James-Ebert/HyYOcRAXo) for signing previously unsigned commits.