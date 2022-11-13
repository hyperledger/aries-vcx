# <img alt="Hyperledger Aries logo" src="docs/aries-logo.png" width="45px" /> AriesVCX

![CI build](https://github.com/hyperledger/aries-vcx/workflows/CI/badge.svg)
[![codecov](https://codecov.io/gh/hyperledger/aries-vcx/branch/main/graph/badge.svg)](https://codecov.io/gh/hyperledger/aries-vcx)
[![Chat](https://raw.githubusercontent.com/hyperledger/chat-assets/master/aries-vcx.svg)](https://discord.com/channels/905194001349627914/955480822675308604)

**Aries-vcx** is Rust library implementing Aries protocols. It can be used to build Aries agents. 

AriesVCX currently requires instance of [mediator agency](https://github.com/hyperledger/aries-rfcs/blob/master/concepts/0046-mediators-and-relays/README.md) - in 
particular [NodeVCX Agency](https://github.com/AbsaOSS/vcxagencynode/).
To get your started with aries-vcx quickly, you can use our deployment at
`https://ariesvcx.agency.staging.absa.id/agency`

# C-Bindings 
- **libvcx** is library, which provides C-interface to interact with AriesVCX. C-bindings exists for:
  - Java (+Android)
  - iOS, 
  - NodeJS


# Get started
The best way to get your hands on.
* Simple Rust [Agent](./agents/rust/aries-vcx-agent)
* Simple NodeJS [Agent](./agents/node/vcxagent-core)
* Android [demo](https://github.com/sktston/vcx-demo-android) (3rd party demo)
* iOS [demo](https://github.com/sktston/vcx-demo-ios) (3rd party demo)
* iOS [skeleton project](https://github.com/sktston/vcx-skeleton-ios) (3rd party demo)

# Implemented Aries protocols
* ✅ Connection Protocol 1.0: [`https://didcomm.org/connections/1.0/*`](https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol)
* ✅ Out of Band 1.0: [`https://didcomm.org/out-of-band/1.1/*`](https://github.com/hyperledger/aries-rfcs/blob/main/features/0434-outofband)
* ✅ Basic Message 1.0: [`https://didcomm.org/basicmessage/1.0/*`](https://github.com/hyperledger/aries-rfcs/tree/master/features/0095-basic-message)
* ✅ Credential Issuance 1.0 [`https://didcomm.org/issue-credential/1.0/*`](https://github.com/hyperledger/aries-rfcs/blob/master/features/0036-issue-credential)
* ✅ Credential Presentation 1.0: [`https://didcomm.org/present-proof/1.0/*`](https://github.com/hyperledger/aries-rfcs/tree/master/features/0037-present-proof)
* ✅ Trust Ping 1.0: [`https://didcomm.org/trust_ping/1.0/*`](https://github.com/hyperledger/aries-rfcs/blob/master/features/0048-trust-ping/README.md)
* ✅ Discover Features 1.0: [`https://didcomm.org/discover-features/1.0/*`](https://github.com/hyperledger/aries-rfcs/tree/master/features/0031-discover-features)
* ✅ Revocation notification 2.0: [`https://didcomm.org/revocation_notification/2.0/*`](https://github.com/hyperledger/aries-rfcs/tree/master/features/0031-discover-features)

# Versioning
- The project currently does not follow semantic versioning. Fow now we are releasing versions `0.x.x`.
- Although the API is mostly stable, breaking changes still occur in our releases. See changelogs at
  [releases](https://github.com/hyperledger/aries-vcx/releases) page.
- See our [roadmap](./ROADMAP.md) for what's coming.

# Project architecture
The architecture is evolving - you can compare the diagram below with diagram under [roadmap](./roadmap.md).

# <img alt="AriesVCX architecture diagram" src="docs/architecture/ariesvcx_architecture_now_150922.png"/>

# Artifacts
Number of artifacts are built for every CI run (unless it's coming from a forked repository due to limitations of Github Actions). 
Artifacts tied with particular release can be found on 
 [release page](https://github.com/hyperledger/aries-vcx/releases).
 
## Artifacts produced:
- Alpine based docker image with precompiled `libvcx.so`
- iOS wrapper
- Android wrapper
- NodeJS wrapper

#### When looking for artifacts for a particular CI run:
- NodeJS wrapper is published at [npmjs](https://www.npmjs.com/package/@hyperledger/node-vcx-wrapper)
- NodeJS agent is published at [npmjs](https://www.npmjs.com/package/@hyperledger/vcxagent-core)
- Docker images are in [Github Packages](https://github.com/hyperledger/aries-vcx/packages)
- Mobile artifacts are attached to [CI runs](https://github.com/hyperledger/aries-vcx/actions) (click on particular CI run to
  see the artifacts)
