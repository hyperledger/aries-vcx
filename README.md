# VCX
LibVCX is Aries c-callable implementation written in Rust with language wrappers currently available in Java, Python,
iOS, NodeJS.  

## Installing the VCX
* VCX requires access to a [Mediator Agency](https://github.com/AbsaOSS/vcxagencynode) 
* VCX requires a payment plugin as an implementation of [libindy payment interface](https://hyperledger-indy.readthedocs.io/projects/sdk/en/latest/docs/design/004-payment-interface/README.html), such as [libnullpay](https://github.com/hyperledger/indy-sdk/tree/master/libnullpay/README.md) for the simplest implementation.

### Building LibVCX on OSX
Instructions can be found [here](./docs/build-osx.md)

# Building LibVCX on mobile
Instructions cane be foun [here](./docs/build-mobile.md)
 
## Wrappers documentation

The following wrappers are tested and complete.

* [Java](wrappers/java/README.md)
* [Python](wrappers/python3/README.md)
* [iOS](wrappers/ios/README.md)
* [NodeJS](wrappers/node/README.md)

## Library initialization
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

## Getting started guide
[The tutorial](docs/getting-started/getting-started.md) which introduces Libvcx and explains how the whole ecosystem works, and how the functions in the SDK can be used to construct rich clients.

### Example use
For the main workflow example check [demo](https://github.com/hyperledger/indy-sdk/tree/master/vcx/wrappers/python3/demo).

## Actors
Libvcx provides APIs for acting as different actors.
The actor states, transitions and messages depend on communication method is used.

There are two communication methods: `proprietary` and `aries`. The default communication method is `proprietary`.
The communication method can be specified as a config option on one of *_init functions.

* Connection:
    * Inviter
        * [API](https://github.com/hyperledger/indy-sdk/tree/master/vcx/libvcx/api/connection.rs) 
        * State diagram
            * [proprietary](docs/states/proprietary/connection-inviter.puml) 
            * [aries](docs/states/aries/connection-inviter.puml) 
    * Invitee
        * [API](https://github.com/hyperledger/indy-sdk/tree/master/vcx/libvcx/api/connection.rs) 
        * State diagram
            * [proprietary](docs/states/proprietary/connection-invitee.puml) 
            * [aries](docs/states/aries/connection-invitee.puml) 

* Credential Issuance:
    * Issuer
        * [API](https://github.com/hyperledger/indy-sdk/tree/master/vcx/libvcx/api/issuer_credential.rs) 
        * State diagram
            * [proprietary](docs/states/proprietary/issuer-credential.puml) 
            * [aries](docs/states/aries/issuer-credential.puml) 
    * Holder
        * [API](https://github.com/hyperledger/indy-sdk/tree/master/vcx/libvcx/api/credential.rs) 
        * State diagram
            * [proprietary](docs/states/proprietary/credential.puml) 
            * [aries](docs/states/aries/credential.puml) 

* Credential Presentation:
    * Verifier
        * [API](https://github.com/hyperledger/indy-sdk/tree/master/vcx/libvcx/api/proof.rs) 
        * State diagram
            * [proprietary](docs/states/proprietary/proof.puml) 
            * [aries](docs/states/aries/proof.puml) 
    * Prover
        * [API](https://github.com/hyperledger/indy-sdk/tree/master/vcx/libvcx/api/disclosed_proof.rs) 
        * State diagram
            * [proprietary](docs/states/proprietary/disclosed-proof.puml) 
            * [aries](docs/states/aries/disclosed-proof.puml) 

## How to migrate
The documents that provide necessary information for Libvcx migrations.
 
* [v0.1.x → v0.2.0](docs/migration-guide-0.1.x-0.2.0.md)
* [v0.2.x → v0.3.0](docs/migration-guide-0.2.x-0.3.0.md)
* [v0.3.x → v0.4.0](docs/migration-guide-0.3.x-0.4.0.md)
* [v0.4.x → v0.5.0](docs/migration-guide-0.4.x-0.5.0.md)
* [v0.5.x → v0.6.0](docs/migration-guide-0.5.x-0.6.0.md)
* [v0.6.x → v0.7.0](docs/migration-guide-0.6.x-0.7.0.md)
* [v0.7.x → v0.8.0](docs/migration-guide-0.7.x-0.8.0.md)
