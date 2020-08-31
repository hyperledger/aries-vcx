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