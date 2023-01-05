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

### `api_c` module
Built on top of `api_vcx`, provides runtime executor (tokio) and FFI interface. Java and iOS wrapper
are linked to this interface.

# Get started
If you wish to use iOS or Android wrapper, you may find it useful to have look at this 3rd party demos
* Android [demo](https://github.com/sktston/vcx-demo-android) 
* iOS [demo](https://github.com/sktston/vcx-demo-ios)
* iOS [skeleton project](https://github.com/sktston/vcx-skeleton-ios)
These might be somewhat outdated at the moment, nevertheless they may be a good starting point.

# Testing
- Run unit tests:
```
cargo test  --features "general_test" -- --test-threads=1
```
- Run integration tests (you need to have Indy pool running)
```
TEST_POOL_IP=127.0.0.1 cargo test  --features "pool_tests" -- --test-threads=1
```
