
# Run
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
docker run --name indylocalhost -p 9701-9708:9701-9708 -d ghcr.io/hyperledger/aries-vcx/indy_pool_localhost:1.15.0
```
If you are running on arm64, you can specify option `--platform linux/amd64`, as the image above was
originally built for `x86_64` architecture.

Now you should be ready to run integration tests:
```
cargo test  --features "pool_tests" -- --test-threads=1
```
