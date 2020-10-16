# Aries Framework Rust

![CI build](https://github.com/AbsaOSS/libvcx/workflows/CI/badge.svg)
[![codecov](https://codecov.io/gh/AbsaOSS/libvcx/branch/master/graph/badge.svg)](https://codecov.io/gh/AbsaOSS/libvcx)

- Aries c-callable implementation written in Rust with language wrappers available in:
  - Java (+Android)
  - iOS, 
  - NodeJS
  - Python (looking for maintainer)  
- Aries Framework Rust requires [mediator agency](https://github.com/hyperledger/aries-rfcs/blob/master/concepts/0046-mediators-and-relays/README.md).
  Currently, the only such agency is [NodeVCX Agency](https://github.com/AbsaOSS/vcxagencynode/). 
  
# Work in progress
- This is spin off what has previously been known as LibVCX library. 
- There's still outstanding work to cleanup code, restructure library so small breaking changes occurs with almost every release.
- The project currently does not follow semantic versioning. Fow now we are releasing versions `0.x.x`. 

# Get started
The best way to get your hands on.  
* NodeJS [demo](https://github.com/AbsaOSS/libvcx/tree/master/wrappers/node) - NodeJS wrapper is our first class citizien 
* Android [demo](https://github.com/sktston/vcx-demo-android)  (3rd party demo)
* iOS [demo](https://github.com/sktston/vcx-demo-ios) (3rd party demo)
* iOS [skeleton project](https://github.com/sktston/vcx-skeleton-ios) (3rd party demo)

#### ::Important::
However before you'll be able to pick one of these demos and run them locally, you need to build binary library which
all these demos depends on.  
-  [Building AriesFrameworkRs on OSX, Linux](./docs/build-general.md)

# Artifacts
- In Github Actions CI, we are producing number of Artifacts:
    - Alpine based docker image with precompiled Aries Rust Framework
    - iOS and Android builds of Aries Rust Framework
    - Node wrapper
    
- The artifacts are produced from every PR (unless it's coming from a forked repository due to limitations of Github Actions).

- NodeJS wrappers are published on [npmjs](https://www.npmjs.com/package/@hyperledger/node-vcx-wrapper)
- Docker images are in [Github Packages](https://github.com/AbsaOSS/libvcx/packages/332720/versions)
- Mobile artifacts are attached to [CI runs](https://github.com/AbsaOSS/libvcx/actions) (click on particular CI run to
  see the artifacts)