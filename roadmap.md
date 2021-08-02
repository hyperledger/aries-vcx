# A short history background
LibVCX was library donated into IndySDK by Evernym around 2018. At that point of time Aries did not yet
exist, and the library contained custom proprietary communication protocols of Evernym. In early 2020,
Evernym has contributed implementation of the main Aries protocols. Later on in summer 2020, Absa has 
decided to fork the library, deleted legacy code and worked on improving code quality. In October 2020, 
Absa's forked version was brought back under Hyperledger umbrella under name `aries-vcx`.

The further development of the library can be split into a few phases, roughly marking important 
architectural milestones.

# Phase 0 - Done ✅
- We could call the phase from Absa's fork up until return to Hyperledger as Phase 0, the most of the 
code cleanup and code restructuring happened. We have
- ✅ deleted legacy code, 
- ✅ migrated CI to Github Actions (building docker, ios, android artifacts)
- ✅ changed library testing approach (favoring integration testing in language wrappers, removing 
  encrypted mock inputs on rust-level unit testing)
- ✅ thinned language wrappers (wrappers should use functions to access data, rather trying to map out 
  Rust data structures).

# Phase 1 - Done ✅
This phase is all about decoupling parts of the library into independent modules. We have decoupled 
the library into 3 pieces.
- ✅ `mediator agent client` - client for talking to a compatible agencies - the only open source 
  implementation available is [vcxagencynode](https://github.com/AbsaOSS/vcxagencynode).
- ✅ `aries_vcx` - the "glue" between Aries state machines, `libindy` and mediator agent.  
- ✅ `libvcx` - adds memory management and C bindings on top of `aries_vcx` - making it consumable
   on Android, iOS and any programming language.

# Phase 2 - In progress 🚧
- 🚧 Concising `aries-vcx` crate API and start publishing on crates.io
- 🚧 Implement testing backchannel for aries_vcx. [WIP](https://github.com/hyperledger/aries-agent-test-harness/pull/243)
- Support for public DID based connection invitations
- Support for [Out-of-band protocol](https://github.com/hyperledger/aries-rfcs/tree/master/features/0434-outofband)
- Explore possibility migrating from IndySDK `libindy` to [vdr-tools](https://gitlab.com/evernym/verity/vdr-tools) 
  forked version of `libindy`.

# Future work 

### AIP 2.0
- Our current priority is to get satisfying test results 
  in [aries-agent-test-harness](https://github.com/hyperledger/aries-agent-test-harness) tests, however 
  our obvious next steps in future will be support for AIP2.0.

### Multitenancy 
The library was built so that it can power both mobile (usually in the role of holder, prover) and 
institutional agents (usually in the role of issuer, verifier). The current architecture only supports 
single tenant architectures. In order to build scalable institutional agents, we need to enable 
using in multi-tenant context - a single process using `aries-vcx` should be able to 
manage multiple wallets/agents. 

### Migrating to aries-askar
Currently, the main bottleneck of `aries-vcx`, especially in institutional contexts is reliance on 
IndySDK implementation of wallet (see [issue](https://github.com/hyperledger/indy-sdk/issues/2164)). 

There are following solutions under consideration:
1. Use [Aries-askar](https://github.com/andrewwhitehead/aries-askar) as a new implementation for Aries storage 
exposing asynchronous API. Getting this done might be challenging, as either:
- `IndySDK` would have to be updated to reuse `aries-askar`,
- we drop dependency on IndySDK from `aries-vcx` and substitute it with component such as 
  [indy-utils](https://docs.rs/crate/indy-utils/0.3.2), 
  [indy-shared-rs](https://github.com/bcgov/indy-shared-rs), 
  [aries-credx](https://github.com/sovrin-foundation/aries-credx-framework-rs).
2. Swap `libindy` built from Hyperledger by `libindy` built from Evernym's IndySDK fork 
   [vdr-tools](https://gitlab.com/evernym/verity/vdr-tools). It contains further development and in 
   it has in particular refactor storage modules to be asynchronous. 
   
The effort to migrate (and harness power of) `vdr-tools` will be much easier and smoother than `aries-askar`
so it's likely we'll go for the second option.







