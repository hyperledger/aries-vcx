# A short history background
LibVCX was library donated into IndySDK by Evernym around 2018. At that point of time Aries did not yet
exist, and the library contained custom proprietary communication protocols of Evernym. In early 2020,
Evernym has contributed implementation of the main Aries protocols. Later on in summer 2020, Absa has 
decided to fork the library, deleted legacy code and worked on improving code quality. In October 2020, 
Absa's forked version was brought back under Hyperledger umbrella and rebranded to "AriesVCX".

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
- ✅ `aries-vcx` - the "glue" between Aries state machines, `libindy` and mediator agent.  
- ✅ `libvcx` - adds memory management and C bindings on top of `aries-vcx` - making it consumable
   on Android, iOS and any programming language.

# Phase 2 - Done ✅
- ✅ Migration from `libindy` to its fork [vdr-tools](https://gitlab.com/evernym/verity/vdr-tools).
- ✅ Removal of blocking calls to `vdr-tools`
- ✅ Support for public DID-based connection invitations
- ✅ Support for [out-of-band protocol](https://github.com/hyperledger/aries-rfcs/tree/master/features/0434-outofband)
- ✅ Implement testing backchannel for aries-vcx. [Test runs](https://github.com/hyperledger/aries-agent-test-harness/actions)
- ✅ Removal of global state from `aries-vcx`
- ✅ Support wallet multi-tenancy on aries-vcx level

# Phase 2 - In progress 🚧 / Planned
- ✅ Extract `aries-vcx::messages` module into `aries-messages` crate
- ✅ Implement simple aries agent on top aries-vcx
- ✅ Implement `aries-vcx - AFJ` AATH back-channel pair
- 🚧 Remove FFI layer between `aries-vcx --- vdr-tools`
- 🚧 Concise `aries-vcx` public crate API
- Update public agents' API to enable for receiving messages directly, without a mediator intermediary.
- Update connection protocol implementation to make use of mediator optional
- Extract `aries-vcx::protocols` module into `aries-protocols` crate
- Extract `aries-vcx::did_doc` module into `did-doc` crate
- Publish crates to crates.io
- Implement `did-exchange` protocol
- Add support for full did format, such as `did:sov`, `did:peer`
- Unify approach for Aries FSM interface across protocols
- Abstract away concept "wallet handle" and "pool handle" from `aries-vcx` codebase

# Phase 3
- Use `indy-vdr` to facilitate ledger calls

# Future vision architecture diagram
# <img alt="AriesVCX architecture diagram" src="docs/architecture/ariesvcx_architecture_future_150922.png"/>
