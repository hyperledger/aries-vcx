# Rust agents

This directory contains some of Rust agents built on top of the `aries_vcx` crate:

- [`aries-vcx-agent`](./aries-vcx-agent) - aries agent library used to build our cross-framework testing [backchannel](https://github.com/hyperledger/aries-agent-test-harness/tree/main/aries-backchannels/aries-vcx). Not intended for production use. A new Aries VCX Framework is in development to provide simple, easy to use, and production ready functions for use.
- [`mediator`](./mediator) - didcomm mediator service
- [`mobile-demo`](./mobile_demo) - android mobile app demo created using UniFFI bindings for aries-vcx library
