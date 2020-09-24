# LibVCX AbsaFork

![CI build](https://github.com/AbsaOSS/libvcx/workflows/CI/badge.svg)
[![codecov](https://codecov.io/gh/AbsaOSS/libvcx/branch/master/graph/badge.svg)](https://codecov.io/gh/AbsaOSS/libvcx)

- LibVCX is Aries c-callable implementation written in Rust with language wrappers available in Java (+Android), iOS, 
NodeJS, Python and Rust. 
- This is fork of LibVCX which was previously developed 
within [Hyperledger Indy](https://github.com/hyperledger/indy-sdk). This fork will follow its own 
way (see [changelog](./changelog.md)). The main goal is to turn it into pure Aries implementation. 

# Get started
Best way to get your hands on! Demos are available in multiple language and platforms! Try to build and run some of these:
* NodeJS [demo](https://github.com/AbsaOSS/libvcx/tree/master/wrappers/node).
* Java [demo](https://github.com/AbsaOSS/libvcx/tree/master/demo/java).
* Python [demo](https://github.com/AbsaOSS/libvcx/tree/master/wrappers/python3).
* Android [demo](https://github.com/sktston/vcx-demo-android)
* iOS [demo](https://github.com/sktston/vcx-demo-ios)
* iOS [skeleton project](https://github.com/sktston/vcx-skeleton-ios)
#### ::Important::
However before you'll be able to pick one of these demos and run them, you need to build binary LibVCX library which
all these demos depends on.  
-  [Building LibVCX on OSX, Linux](./docs/build-general.md)
-  [Building LibVCX on mobile](./docs/build-mobile.md)

# Artifacts
- In Github Actions CI, we are producing number of Artifacts:
    - Alpine based docker image with precompiled LibVCX
    - iOS and Android builds of LibVCX  
    - Node wrapper
    
- The artifacts are produced from every PR (unless it's coming from a forked repository due to limitations of Github Actions).

- NodeJS wrappers are published on [npmjs](https://www.npmjs.com/package/@absaoss/node-vcx-wrapper)
- Docker images are in [Github Packages](https://github.com/AbsaOSS/libvcx/packages/332720/versions)
- Mobile artifacts are attached to [CI runs](https://github.com/AbsaOSS/libvcx/actions) (click on particular CI run to
  see the artifacts)