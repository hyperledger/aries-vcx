
## CI artifacts
The following artifacts are built on every CI run and every release:

### Github Actions artifacts
- *(these are to be found at bottom of Summary page for each CI run)*
- `libvcx.so`, `libvcx.dylib` - dynamic library for x86_64 ubuntu, x86_64 darwin)
- ios and java wrapper built on top of `libvcx`

### Images in Github Container Registry
- Alpine-based Docker image with prebuilt `libvcx`; [ghcr.io/hyperledger/aries-vcx/libvcx:version](https://github.com/orgs/hyperledger/packages?repo_name=aries-vcx)

### Packages on npmjs
- NodeJS wrapper - bindings for libvcx; [node-vcx-wrapper](https://www.npmjs.com/package/@hyperledger/node-vcx-wrapper)
- Simple NodeJS aries agent for testing; [vcxagent-core](https://www.npmjs.com/package/@hyperledger/vcxagent-core)
