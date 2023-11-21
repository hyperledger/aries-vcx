# <img alt="Hyperledger Aries logo" src="docs/aries-logo.png" width="45px" /> Aries components

## Aries components
  - [`aries_vcx`](aries_vcx) - Library implementing DIDComm protocols, with focus on verifiable credential issuance and verification.
  - [`messages`](messages) - Library for building and parsing Aries messages.
  - [`aries_vcx_core`](aries_vcx_core) - Interfaces for interaction with ledgers, wallets and credentials.
  - [`aries-vcx-agent`](agents/rust/aries-vcx-agent) - simple aries agent framework built on top of `aries_vcx` library.
  - [`mediator`](agents/rust/mediator) - Aries message mediator service

## Aries mobile 📱 components
  - [`../tools/uniffi_aries_vcx`](../uniffi_aries_vcx) - UniFFI wrapper for `aries_vcx` and sample mobile app
