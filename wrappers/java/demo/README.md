# Running the VCX Java Demo

### Alice/Faber demo
The alice/faber demo is widely used in the indy-sdk demo. The description of the VCX node demo explains it well, 
including the operation of the cloud agent. 
[Description here](https://github.com/hyperledger/indy-sdk/tree/master/vcx/wrappers/node#run-demo).

### Pre-requirements
#### Libraries
Before you'll be able to run demo, you need to make sure you've compiled 
- [`libindy`](https://github.com/hyperledger/indy-sdk/tree/master/libindy)
- [`libvcx`](https://github.com/hyperledger/indy-sdk/tree/master/vcx)

Library binaries must be located `/usr/local/lib` on OSX, `/usr/lib` on Linux. 

#### Java wrapper for LibVCX
This demo uses the pre-built Java wrapper from maven repository. See dependencies section in `build.gradle`.

#### Indy pool
You'll also have to run pool of Indy nodes on your machine. You can achieve by simply running a docker container
which encapsulates multiple interconnected Indy nodes. 
[Instructions here](https://github.com/hyperledger/indy-sdk#how-to-start-local-nodes-pool-with-docker).

### Steps to run demo
- Start [Node agency](https://github.com/AbsaOSS/vcxagencynode) or [Dummy Cloud Agent](https://github.com/hyperledger/indy-sdk/tree/master/vcx/dummy-cloud-agent)
- Run Faber agent, representing an institution
```
./gradlew faber
```
- Give it a few seconds, then run Alice's agent which will connect with Faber's agent
```
./gradlew alice
```
