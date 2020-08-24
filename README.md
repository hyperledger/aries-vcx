# LibVCX AbsaFork

![CI build](https://github.com/AbsaOSS/libvcx/workflows/CI/badge.svg)

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

# Library initialization
Libvcx library must be initialized with one of the functions:
* `vcx_init_with_config` -  initializes with <configuration> passed as JSON string. 
* `vcx_init` -  initializes with a path to the file containing <configuration>. 
* `vcx_init_minimal` - initializes with the minimal <configuration> (without any agency configuration).

Each library function will use this <configuration> data after the initialization. 
The list of options can be find [here](../docs/configuration.md#vcx)
An example of <configuration> file can be found [here](../vcx/libvcx/sample_config/config.json)

If the library works with an agency `vcx_agent_provision` function must be called before initialization to populate configuration and wallet for this agent.
The result of this function is <configuration> JSON which can be extended and used for initialization.

To change <configuration> a user must call `vcx_shutdown` and then call initialization function again.

