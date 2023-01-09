# <img alt="Hyperledger Aries logo" src="docs/aries-logo.png" width="45px" /> AriesVCX

![CI build](https://github.com/hyperledger/aries-vcx/workflows/CI/badge.svg)
[![codecov](https://codecov.io/gh/hyperledger/aries-vcx/branch/main/graph/badge.svg)](https://codecov.io/gh/hyperledger/aries-vcx)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Join the chat at https://chat.hyperledger.org/channel/aries](https://img.shields.io/badge/Chat%20on-Hyperledger%20Chat-blue)](https://chat.hyperledger.org/channel/aries)

## Core crates:
- [`aries-vcx`](aries_vcx) - **implementation of Hyperledger Aries protocols**
- `messages` - crate encapsulating Aries message models and builders
- `diddoc` - crate for working with DIDDocs
- `agency-client` - client to communicate with [vcx mediator](https://github.com/AbsaOSS/vcxagencynode)   

## Additional crates:
Additionally, you can find here project built on **top of `aries-vcx`**:
- `agents/rust/aries-vcx-agent` - simple agent implementation in rust on top of `aries-vcx` crate
- [`libvcx`](libvcx) - built on top of `aries-vcx`, is a particular approach how to use `aries-vcx` on
  mobile or from other languages.

## Getting started
- Ask question on [discord](https://discord.com/channels/905194001349627914/955480822675308604)
- Talk to us on community call starting every Thursday 09:00am UTC via [zoom](https://zoom.us/j/97759680284?pwd=VytRRlJSd3c5NXJ1V25XbUxNU0Jndz09)
- See high level 2023 roadmap at [ROADMAP_2023.md](ROADMAP_2023.md)
- Find out what's planned in [issues](https://github.com/hyperledger/aries-vcx/issues) 
  and project [board](https://github.com/orgs/hyperledger/projects/14)
- We welcome new contributors! Connect with us via the channels above and take a look at [CONTRIBUTING.md](CONTRIBUTING.md)

## Versioning
- We are currently not following semantic versioning. Version are releasing `0.x.x` versions. 
- Breaking changes to APIs happen occasionally. See full changelogs records at 
  [releases](https://github.com/hyperledger/aries-vcx/releases) page.

## CI artifacts
Following artifacts are build with every CI run and release:

### Github Actions artifacts
  - *(these are to be found at bottom of Summary page for each CI run)*
  - `libvcx.so`, `libvcx.dylib` - dynamic library for x86_64 ubuntu, x86_64 darwin)
  - ios and java wrapper built on top of `libvcx`

### Images in Github Container Registry
  - Alpine based Docker image with prebuilt `libvcx`; [ghcr.io/hyperledger/aries-vcx/libvcx:version](https://github.com/orgs/hyperledger/packages?repo_name=aries-vcx)

### Packages on npmjs 
  - NodeJS wrapper - bindings for libvcx; [node-vcx-wrapper](https://www.npmjs.com/package/@hyperledger/node-vcx-wrapper) 
  - Simple NodeJS aries agent for testing; [vcxagent-core](https://www.npmjs.com/package/@hyperledger/vcxagent-core)
