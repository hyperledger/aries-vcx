# Testing
You can filter out tests by specifying features they require / use.
```
- general_test       :: unit tests which do not require any external component
- pool_tests         :: requires indypool to be running
- agency_pool_tests  :: requires agency and/or indypool 
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

## Environment variables

- `WARNLOG_MSGS_RECEIVED` - if set to `true` log received E2E connection messages
- `DISALLOW_V1` - if set to `true` process panics whenever one of following is attempted:
  - run legacy V1 onboarding
  - create V1 connection  
  - create legacy issuer credential object