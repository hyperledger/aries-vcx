# Aries-vcx
Crate implementing Hyperledger Aries protocols and building blocks for building Aries agents for
both mobile (typically in role of holder, prover) and server use-cases (issuer and verifier).

# Getting started
Aries-vcx is library, not a framework. We strive to be not too opinionated and simply provide building block for whatever
you want to build. 

Generally, the crate allows you to 
- create encrypted wallet, 
- read/write from/to Indy ledger,
- establish didcomm connections and exchange messages,
- create and process Aries messages to drive Aries protocols. 

Have look at [aries-vcx-agent](../agents/rust/aries-vcx-agent) for inspiration how aries-vcx can be used.

# Message mediation
If you are building mobile agent, you will generally require mediator service, which will receive
messages on device's behalf - sort of like a mail server.

It's possible to opt into integrated message mediator using `MediatedConnection` `impl`, which
speaks the language of [vcxagency-node](https://github.com/AbsaOSS/vcxagencynode) mediator service.

# Verify on your machine
### Stage 1 - unit tests
- First we need to get unit tests working on your machine. These don't require any external services to run. 
```
cargo test --features "general_test" -- --test-threads=1
```
If you run into an errors 
- On OSX, try to install following packages with:
  ```sh
  brew install zmq
  brew install pkg-config
  ```
- On ubuntu, you will likely need following packages:
  ```sh
  sudo apt-get update -y
  sudo apt-get install -y libsodium-dev libssl-dev libzmq3-dev
  ```

### Stage 2 - integration tests
Next up you will need integration tests running. These tests must pointed again some Indy ledger.
You'll get best result by running a pool of Indy nodes on your machine. You can start a pool of 4 nodes
in docker container like this
```sh
docker run --name indylocalhost -p 9701-9708:9701-9708 -d pstas/indypool-localhost:1.15.0-localhost
```
If you are running on arm64, you can specify option `--platform linux/amd64`, as the image above was
originally built for `x86_64` architecture.

Now you should be ready to run integration tests:
```
cargo test  --features "pool_tests" -- --test-threads=1
```

## Implemented Aries protocols
* ✅ Connection Protocol 1.0: [`https://didcomm.org/connections/1.0/*`](https://github.com/hyperledger/aries-rfcs/tree/master/features/0160-connection-protocol)
* ✅ Out of Band 1.0: [`https://didcomm.org/out-of-band/1.1/*`](https://github.com/hyperledger/aries-rfcs/blob/main/features/0434-outofband)
* ✅ Basic Message 1.0: [`https://didcomm.org/basicmessage/1.0/*`](https://github.com/hyperledger/aries-rfcs/tree/master/features/0095-basic-message)
* ✅ Credential Issuance 1.0 [`https://didcomm.org/issue-credential/1.0/*`](https://github.com/hyperledger/aries-rfcs/blob/master/features/0036-issue-credential)
* ✅ Credential Presentation 1.0: [`https://didcomm.org/present-proof/1.0/*`](https://github.com/hyperledger/aries-rfcs/tree/master/features/0037-present-proof)
* ✅ Trust Ping 1.0: [`https://didcomm.org/trust_ping/1.0/*`](https://github.com/hyperledger/aries-rfcs/blob/master/features/0048-trust-ping/README.md)
* ✅ Discover Features 1.0: [`https://didcomm.org/discover-features/1.0/*`](https://github.com/hyperledger/aries-rfcs/tree/master/features/0031-discover-features)
* ✅ Revocation notification 2.0: [`https://didcomm.org/revocation_notification/2.0/*`](https://github.com/hyperledger/aries-rfcs/tree/master/features/0031-discover-features)

## Architecture 

<img alt="AriesVCX architecture diagram" src="../docs/architecture/ariesvcx_architecture_040123.png"/>

