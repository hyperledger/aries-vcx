# <img alt="Hyperledger Aries logo" src="docs/aries-logo.png" width="45px" /> Aries components

## Aries components

- [`aries_vcx`](aries_vcx) - Library implementing DIDComm protocols, with focus on verifiable credential issuance and verification.
- [`messages`](messages) - Library for building and parsing Aries messages.
- [`aries_vcx_core`](aries_vcx_core) - Interfaces for interaction with ledgers, wallets and credentials.
- [`aries-vcx-agent`](agents/rust/aries-vcx-agent) - simple aries agent framework built on top of `aries_vcx` library. Not intended for production use. A new Aries VCX Framework is in development to provide simple, easy to use, and production ready functions for use.
- [`mediator`](agents/rust/mediator) - Aries message mediator service

## Aries mobile 📱 components

- [`uniffi-aries-vcx`](wrappers/uniffi-aries-vcx) - UniFFI wrapper for `aries_vcx`.
- [`android`](agents/android/) - Sample Android App using UniFFI wrapper.
- [`ios`](agents/ios/) - Sample Android App using UniFFI wrapper.
