# libvcx_core
This is foundational crate for
- now deprecated [`libvcx`](../libvcx) dynamic library
- NodeJS wrapper [`vcx-napi-rs`](../wrappers/vcx-napi-rs)

# Testing
Before you try to build/test `libvcx` crate on your machine, make sure you can build `aries-vcx` - see
[aries-vcx README](../aries_vcx).

- Run unit tests:
```
cargo test  --features -- --test-threads=1
```
- Run integration tests (you need to have Indy pool running)
```
TEST_POOL_IP=127.0.0.1 cargo test -- --ignored --test-threads=1
```

## Architecture

<img alt="Libvcx architecture diagram" src="../docs/architecture/architecture_230223_libvcx.png"/>
