# Testing
You can filter out tests by specifying features they require / use.
```
- general_test :: does not require any external component
- pool_tests   :: requires indypool to be running
- agency_v2    :: requires agency talking libvcx client2agency v2 protocol (nodevcx-agency)
- aries        :: group of quick unit tests related to aries
- agency       :: tests using mocked legacy agency
- pool_legacy_agency_tests :: requires pool and legacy agency
```

Run quick unit tests:
```
cargo test  --features "general_test" -- --test-threads=1
```
Or specific test:
```
cargo test test_init_minimal_with_invalid_agency_config --features "general_test" -- --test-threads=1 -- --exact
```

Run integration tests:
```
TEST_POOL_IP=127.0.0.1 cargo test  --features "pool_tests" -- --test-threads=1
```