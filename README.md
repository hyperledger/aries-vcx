# <img alt="Hyperledger Aries logo" src="docs/aries-logo.png" width="45px" /> Aries Framework Rust

![CI build](https://github.com/hyperledger/aries-vcx/workflows/CI/badge.svg)
[![codecov](https://codecov.io/gh/hyperledger/aries-vcx/branch/master/graph/badge.svg)](https://codecov.io/gh/hyperledger/aries-vcx)
[![Chat](https://raw.githubusercontent.com/hyperledger/chat-assets/master/aries-vcx.svg)](https://chat.hyperledger.org/channel/aries-vcx)


- Aries VCX is C-callable implementation written in Rust with language wrappers available in:
  - Java (+Android)
  - iOS, 
  - NodeJS
  - Python (looking for a maintainer)  
- Aries Framework Rust requires [mediator agency](https://github.com/hyperledger/aries-rfcs/blob/master/concepts/0046-mediators-and-relays/README.md).
  Currently, the only such agency is [NodeVCX Agency](https://github.com/AbsaOSS/vcxagencynode/).  
  
# Work in progress
- This is spin off what has previously been known as LibVCX library. 
- There's still outstanding work to cleanup code, restructure library so small breaking changes occurs with almost every release.
- The project currently does not follow semantic versioning. Fow now we are releasing versions `0.x.x`. 
- See our [roadmap](./roadmap.md).

# Get started
The best way to get your hands on.  
* NodeJS [demo](https://github.com/hyperledger/aries-vcx/tree/master/wrappers/node)
* Android [demo](https://github.com/sktston/vcx-demo-android)  (3rd party demo)
* iOS [demo](https://github.com/sktston/vcx-demo-ios) (3rd party demo)
* iOS [skeleton project](https://github.com/sktston/vcx-skeleton-ios) (3rd party demo)

#### ::Important::
However before you'll be able to pick one of these demos and run them locally, you need to build binary library which
all these demos depends on.  
- [Building aries-vcx on OSX, Linux](./docs/build-general.md)

# Artifacts
Number of artifacts are built for every CI run (unless it's coming from a forked repository due to limitations of Github Actions). 
Artifacts tied with particular release can be found on 
 [release page](https://github.com/hyperledger/aries-vcx/releases).
 
Artifacts produced:
- Alpine based docker image with precompiled Aries Rust Framework
- iOS wrapper
- Android wrapper
- NodeJS wrapper

When looking for artifcats for a particular CI run:
- NodeJS wrappers are published on [npmjs](https://www.npmjs.com/package/@hyperledger/node-vcx-wrapper)
- Docker images are in [Github Packages](https://github.com/hyperledger/aries-vcx/packages)
- Mobile artifacts are attached to [CI runs](https://github.com/hyperledger/aries-vcx/actions) (click on particular CI run to
  see the artifacts)
