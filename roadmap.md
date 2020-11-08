# A short history background
LibVCX was library donated into IndySDK by Evernym around 2018. At that point of time Aries did not yet
exist, and the library contained custom proprietary communication protocols of Evernym. In early 2020,
Evernym has contributed implementation of the main Aries protocols. Later on in summer 2020, Absa has 
decided to fork the library, deleted legacy code and worked on improving code quality. In October 2020, 
Absa's forked version was brought back under Hyperledger umbrella under name `aries-vcx`.

The further development of the library can be split into a few phases, roughly marking important 
architectural milestones.

# Phase 0
- We could call the phase from Absa's fork up until return to Hyperledger as Phase 0, the most of the 
code cleanup and code restructuring happened. We have
- deleted legacy code, 
- migrated CI to Github Actions (building docker, ios, android artifacts)
- changed library testing approach (favoring integration testing in language wrappers, removing 
  encrypted mock inputs on rust-level unit testing)
- thinned language wrappers (wrappers should use functions to access data, rather trying to map out 
  Rust data structures).

# Phase 1
This phase is all about decoupling parts of the library into independent modules. We are currently 
targeting extraction of 2 independent reusable modules:
- mediator agent client
- Aries protocol state machines
Once that is done, `aries-vcx` will essentially become opinionated C-Callable glue of independent rust 
crates, rather than single monolithic opinionated library. Additionally `aries-vcx` itself should 
become crate consumable from other Rust projects.

# Phase 2
### Multitenancy 
The library was built so that it can power both mobile (usually in the role of holder, prover) and 
institutional agents (usually in the role of issuer, verifier). The current architecture only supports 
single tenant architectures. In order to build scalable institutional agents, we need to enable 
using in multi-tenant context - a single process using `aries-vcx` should be able to 
manage multiple wallets/agents. 

### Migrating to aries-askar
Currently, the main bottleneck of `aries-vcx`, especially in institutional contexts is reliance on 
IndySDK implementation of wallet (see [issue](https://github.com/hyperledger/indy-sdk/issues/2164)). 
[Aries-askar](https://github.com/andrewwhitehead/aries-askar) is new implementation of Aries storage 
exposing asynchronous API. Getting this done might be challenging, as either:
- `IndySDK` would have to be updated to reuse `aries-askar`,
- we drop dependency on IndySDK from `aries-vcx` and substitute it with component such as 
  [indy-utils](https://docs.rs/crate/indy-utils/0.3.2), 
  [indy-shared-rs](https://github.com/bcgov/indy-shared-rs), 
  [aries-credx](https://github.com/sovrin-foundation/aries-credx-framework-rs).


# Note
Other tasks might come along the way, for example support for anoncreds 2.0, adding support for more 
Aries protocols, multiledger support, different did methods etc.





