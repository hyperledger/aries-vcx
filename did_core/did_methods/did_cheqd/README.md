# DID Cheqd Resolver
This crate contains a resolver for DIDs of the [did:cheqd](https://docs.cheqd.io/product/architecture/adr-list/adr-001-cheqd-did-method) method. The implementation resolves DIDs via gRPC network requests to the configured nodes. Default nodes for cheqd's `mainnet` & `testnet` can be used, or custom nodes can be opt-in by supplying a different gRPC URL configuration.

The implementations in this crate are largely inspired from cheqd's own typescript [sdk](https://github.com/cheqd/sdk/blob/main/src/modules/did.ts).

This crate uses gRPC types and clients generated using [tonic](https://github.com/hyperium/tonic). The generated rust code is checked-in to this repository for monitoring, [see here](./src/proto/mod.rs). These generated rust files are checked-in alongside the V2 cheqd proto files & dependencies, [here](./cheqd_proto_gen/proto/), which are sourced from [cheqd's Buf registry](https://buf.build/cheqd/proto/docs).

Since the generated code & proto files are not relatively large nor overwhelming in content, they are checked-in rather than pulled and/or generated at build time. The benefit is that the contents of the files can be monitored with each update, making supply-chain attacks obvious. It also reduces the build time complexity for consumers - such as reducing requirements for any 3rd party build tools to be installed (`protobuf`). The drawback is that it introduces some more manual maintainence.

## Crate Maintainence
If there is an update to the `.proto` files, or `tonic` had a breaking update, the checked-in files may be due for a manual update. To do so, update any proto files in the [proto dir](./cheqd_proto_gen/proto/), then re-generate the rust files by using the [cheqd-proto-gen](./cheqd_proto_gen/) binary within this directory:
```
cargo run --bin cheqd-proto-gen
```