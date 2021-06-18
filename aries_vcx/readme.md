# Testing
You can filter out tests by specifying features they require / use.
```
- general_test       :: tests which do not require external components
- pool_tests         :: tests which require running indypool
- agency_pool_tests  :: tests which require running indypool and agency 
- agency_tests       :: tests which require running agency
- plugin_test        :: tests which require running a wallet plugin database 
```

## Other features
```
- warnlog_fetched_messages :: if enabled, fetched connection messages will be logged in warn log level. This is useful
  for producing mock data by running integration tests from NodeJS.
```
