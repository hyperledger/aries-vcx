# <img alt="Hyperledger Aries logo" src="docs/aries-logo.png" width="45px" /> aries-vcx

![CI build](https://github.com/hyperledger/aries-vcx/workflows/CI/badge.svg)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Join the chat at https://chat.hyperledger.org/channel/aries](https://img.shields.io/badge/Chat%20on-Hyperledger%20Chat-blue)](https://chat.hyperledger.org/channel/aries)

This is repository of
- Rust library `aries-vcx` implementing Aries protocols
- and collection of supporting projects

## If you are Rust 🦀 developer
You can build your Rust project on top of
- [`aries-vcx`](aries_vcx) - Ready to go Rust library to work with Aries protocols, both from
issuer/verifier side or credential holder/prover.

Additionally, Aries-vcx implementation is built on top of smaller Rust crates:
- [`messages`](messages) - crate for building and parsing Aries messages
- [`diddoc`](diddoc) - crate to work with DIDDocs

## If you are mobile 📱 developer
Aries-vcx is well positioned for native mobile applications. There's to approaches one can take:
- **deprecated**: [`libvcx`](./libvcx) and its Java and Objective-C wrappers is complete but deprecated. It is not actively maintained and will be removed in the future.
- **encouraged**, but in stage of POC: [`uniffi_aries_vcx`](./uniffi_aries_vcx) is next generation approach to enable `aries-vcx` on mobile, providing Swift
and Kotlin wrappers. However, this is yet in POC stage and new adopters `aries-vcx` for mobile
are highly encourage to contribute to its development.

Read more about `libvcx` deprecation and `unifii` benefits [here](https://github.com/hyperledger/libvcx#deprecation-notice)

# Reach out 👋
- Ask question on [discord](https://discord.com/channels/905194001349627914/955480822675308604)
- Talk to us on public community every Thursday @ 09:00 am UTC via Zoom, see [details](https://wiki.hyperledger.org/display/ARIES/Community+calls)
- See high level 2023 roadmap at [ROADMAP_2023.md](docs/ROADMAP_2023.md)
- We welcome new contributors! Connect with us via the channels above and take a look at [CONTRIBUTING.md](CONTRIBUTING.md)

## Versioning & releases
- We are currently not strictly following semantic versioning and all releases are `0.x.x`. 
- We bump minor version for releases with new features, significant refactors, breaking changes. 
We bump patch version if release only contains fixes or smaller refactoring. 
- See more info on [releases](https://github.com/orgs/hyperledger/projects/14)
- See [releases](https://github.com/hyperledger/aries-vcx/releases) page.
