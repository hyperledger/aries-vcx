# UniFFI Aries VCX Proof Of Concept

This sub-directory contains the UniFFI wrappers and tools to generate Kotlin & Swift mobile wrappers over the `aries-vcx` crate.

## Core

Core contains a crate which wraps over the `aries-vcx` crate to provide a UniFFI-ready interface for wrapping. This crate's interface is what is used in `demos`.

## Demos

Coming soon..

## Building

This module is distributed in three directories.

1. `core`: The UniFFI bindings for Aries VCX.
2. `demo`: Demo application consuming the UniFFI wrapper code.
3. `scripts`: Contains helper sh scripts to ease the process of setting up, building and generating the UniFFI code.

To set up the Android SDK, NDK and bootstrap the demo application, you can simply run the following scripts in the order:

1. `android.prepare.sh`: This script will prepare the Android SDK and NDK for your system.
2. `android.toolchain.sh`: This script will prepare the required android toolchain.
3. `android.build.sh`: This script will build the UniFFI bindings and bootstrap the demo application for the target architecture.

NB: Before running the demo application you need to generate the language bindings.

# Testing the simple message relay on Android

Aries-VCX supports connection to [tools/simple_message_relay](/tools/simple_message_relay/) on android. You need to setup the agent with instructions [here](/tools/simple_message_relay/README.md#service-setup) and exposing a public endpoint as explained [here](/tools/simple_message_relay/README.md#public-endpoints).

The demo app needs this endpoint to establish communication with the peer.

Update [BASE_RELAY_ENDPOINT](./demo/app/src/main/java/org/hyperledger/ariesvcx/Constants.kt) to reflect the public IP.

```kt
const val BASE_RELAY_ENDPOINT = <your-public-ip-endpoint>;
```

Now you are ready to start communicating. Use the QR scanner in the app to establish connection with the peer.

## Support

Currently the builds have been tested for android `arm64 (aarch64)` on a physical device. In the future we plan to support other architectures.
