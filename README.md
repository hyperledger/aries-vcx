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
  - [`did_parser`](did_core/did_parser_nom) - Building and parsing  [DIDs](https://w3c.github.io/did-core/)
  - [`did_peer`](did_core/did_methods/did_peer) - https://identity.foundation/peer-did-method-spec/
  - [`did_sov`](did_core/did_methods/did_resolver_sov) - https://sovrin-foundation.github.io/sovrin/spec/did-method-spec-template.html
  - [`did_web`](did_core/did_methods/did_resolver_web) - https://w3c-ccg.github.io/did-method-web/
  - [`did_key`](did_core/did_methods/did_key) - https://w3c-ccg.github.io/did-method-key/

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
  - Crates are known to be stable with atleast Rust version 1.79
