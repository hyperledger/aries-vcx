# Deprecation notice
This is now deprecated and receives limited maitenance support. 

This project provided solution to create bindings and wrappers for other languages, 
primarily Java, iOS. However this was started many years ago (circa 2018) and better approaches
to FFI has emerged since then. One of apparent leaders is `uniffi` developed by Firefox which
handles lots of complexity which had to be done manually in this project.

We encourage new developers to adopt this technology in favor of libvcx - we currently have new 
wrapper for `aries-vcx` in POC stage [uniffi_vcx](https://example.org/___TBD___) therefore you 
will have slower start, perhaps more frequent changes, but much more promising long-term future.

# Libvcx
- Libvcx is library built on top of `aries-vcx`, which provides a particular approach how to 
  build bindings for other languages.
- The library is split in 2 modules:

### `api_vcx` module
Layer on top of `aries-vcx` which provides `u32` "handle" reference API. When you
create a new object, this layer gives you back `u32` reference to that object, which is how you
work with it. 

This was historically feasibly approach for building FFI interfaces. Arguably, nowadays
there's more modern approaches to FFI, for example via [uniffi](https://mozilla.github.io/uniffi-rs/).

### DEPRECATED:`api_c` module & ObjectiveC / Java wrappers
Built on top of `api_vcx`, provides runtime executor (tokio) and FFI interface. Libvcx based Java and 
iOS wrapper are linked to this interface.
If you wish to use iOS or Android wrapper, you may find it useful to have look at this 3rd party demos
* Android [demo](https://github.com/sktston/vcx-demo-android) 
* iOS [demo](https://github.com/sktston/vcx-demo-ios)
* iOS [skeleton project](https://github.com/sktston/vcx-skeleton-ios)

# Testing
Before you try to build/test `libvcx` crate on your machine, make sure yu can buil `aries-vcx` - see 
[aries-vcx README](../aries_vcx/README.md#verify-on-your-machine).

- Run unit tests:
```
cargo test  --features "general_test" -- --test-threads=1
```
- Run integration tests (you need to have Indy pool running)
```
TEST_POOL_IP=127.0.0.1 cargo test  --features "pool_tests" -- --test-threads=1
```

## Architecture

<img alt="Libvcx architecture diagram" src="../docs/architecture/libvcx_architecture_040123.png"/>
