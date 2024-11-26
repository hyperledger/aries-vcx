# Aries VCX Backchannel for Aries Agent Test Harness
This crate contains a Rust server implementing an "Aries Agent Test Harness Backchannel", using [aries-vcx](../../aries_vcx/README.md) as the core implementation.
The implementation is an actix server, with HTTP APIs compliant with the [AATH backchannel](https://github.com/hyperledger/aries-agent-test-harness).

This directory also contains a [dockerfile](Dockerfile.aries-vcx), which when ran as a container, will run the server on port 9020, compatible with integration 
into the AATH setup. Aries-VCX CI also builds and publishes this image, where it is then [consumed in the AATH](https://github.com/hyperledger/aries-agent-test-harness/tree/main/aries-backchannels/aries-vcx).

The AATH scripts will spin up multiple of these backchannel simulatenously using the images, and test them against one another; (vcx<->vcx, vcx<->acapy, etc).

# Mid-Development Testing
When making changes to this AATH backchannel, it is a bit inconvenient to have to have to build the docker image to test it against the AATH testsuite. 
For sake of better DX, the following steps can be used as a starting point for performing local AATH testing.

## VCX AATH to VCX AATH Testing
A general rule of thumb, is to make sure the AATH backchannel works against itself before testing against other agents (e.g. acapy).

Use the following to run 2 VCX AATH instances and test them with specific test suites in the AATH:
1. clone the [AATH repo](https://github.com/hyperledger/aries-agent-test-harness/tree/main)
2. in the root of the AATH repo, start the standard AATH services (ledger, DID resolver, tails server): `./manage start`. If this fails due to von service, you may have to build von seperately first: `./manage service build von-network`
3. from within this directory (aries/agent/aath-backchannel), run the server twice, on port 9020 and 9030, with config to use the AATH components (in two different terminals, leave them running):
   1. `LEDGER_URL=http://localhost:9000 GENESIS_FILE=resource/indypool.txn cargo run -- -p 9020`,
   2. `LEDGER_URL=http://localhost:9000 GENESIS_FILE=resource/indypool.txn cargo run -- -p 9030`
4. cd into `aries-test-harness`, create a python venv (e.g. `python3 -m venv venv`) and enter it (e.g. `source venv/bin/activate`)
5. install deps: `pip install -r requirements.txt` (if step 6 fails, also install `aiohttp`: `pip3 install aiohttp`, and perhaps `setuptools`: `pip3 install setuptools`)
6. run specific tests between the two agents, using the `behave` CLI with it's tagging system. e.g. `behave -D Faber=http://0.0.0.0:9020 -D Acme=http://0.0.0.0:9020 -D Bob=http://0.0.0.0:9030 -t @T001-RFC0160` to run the first RFC0160 (connection) test. Check behave docs for more details.
   1. e.g. run a test with ledger operations: `behave -D Faber=http://0.0.0.0:9020 -D Acme=http://0.0.0.0:9020 -D Bob=http://0.0.0.0:9030 -t @T001-RFC0036`
   2. e.g. to simulate the ariesvcx-ariesvcx "runset" defined in the aath test suite `behave -D Faber=http://0.0.0.0:9020 -D Acme=http://0.0.0.0:9020 -D Bob=http://0.0.0.0:9030 -t @RFC0036,@RFC0037,@RFC0160,@RFC0023,@RFC0793 -t ~@wip -t ~@RFC0434 -t ~@RFC0453 -t ~@RFC0211 -t ~@DIDExchangeConnection -t ~@Transport_Ws`. See the `TEST_SCOPE` of [test-harness-ariesvcx-ariesvcx.yml](https://github.com/hyperledger/aries-agent-test-harness/blob/main/.github/workflows/test-harness-ariesvcx-ariesvcx.yml) for the latest.

## VCX AATH to ACAPy AATH Testing
To test the a VCX AATH instance against another agent, such as ACApy, the following modified steps can be followed:
1. clone the [AATH repo](https://github.com/hyperledger/aries-agent-test-harness/tree/main)
2. in the root of the AATH repo, start the standard AATH services (ledger, DID resolver, tails server) AND an ACApy agent on port 9030 (Bob agent): `AGENT_PUBLIC_ENDPOINT=http://localhost:9032 ./manage start -b acapy-main`. If this fails, you may have to build acapy-main seperately first `./manage build -a acapy-main`
3. from within this directory (aries/agent/aath-backchannel), run the server on port 9020, with config to use the AATH components:
   1. `DOCKERHOST=host.docker.internal LEDGER_URL=http://localhost:9000 GENESIS_FILE=resource/indypool.txn cargo run -- -p 9020`
   2. may need to replace `DOCKERHOST` with your appropriate host dependent on OS. (check `./manage dockerhost`)
4. follow steps 4+ [here](#vcx-aath-to-vcx-aath-testing) for performing tests with the two agents
