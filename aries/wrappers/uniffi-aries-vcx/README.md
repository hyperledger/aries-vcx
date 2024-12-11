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

To generate the UniFFI bindings, you need to set up [cargo-ndk](https://github.com/bbqsrc/cargo-ndk) on your system. Instructions to set up `cargo-ndk` are available [here](https://github.com/bbqsrc/cargo-ndk#installing), but can be summed up as:

1. Install `cargo-ndk`.

```bash
cargo install cargo-ndk
```

2. Add target architectures you want to use.

```bash
rustup target add <target>
```

eg. for Android ARM64 on host linux platform.

```bash
rustup target add aarch64-linux-android
```

More documentation can be found [here](https://rust-lang.github.io/rustup/cross-compilation.html). Information on supported platforms can be found [here](https://doc.rust-lang.org/stable/rustc/platform-support.html).

3. `cargo-ndk` requires Android NDK to be set up on the host machine. The recommended way of doing this is using Android Studio as described [here](https://developer.android.com/studio/projects/install-ndk#default-version).

4. If `cargo-ndk` cannot find the installed NDK, you may have to configure the `ANDROID_NDK_HOME` environment variable as described [here](https://github.com/bbqsrc/cargo-ndk#usage).

5. Run the helper build script.

```bash
sh aries/wrappers/uniffi-aries-vcx/scripts/android.build.cargo.ndk.sh
```

NB: Before running the demo application you need to generate the language bindings.

# Testing the simple message relay on Android

Aries-VCX supports connection to [tools/simple_message_relay](/misc/simple_message_relay/) on android. You need to setup the agent with instructions [here](/misc/simple_message_relay/README.md#service-setup) and exposing a public endpoint as explained [here](/misc/simple_message_relay/README.md#public-endpoints).

The demo app needs this endpoint to establish communication with the peer.

Update [BASE_RELAY_ENDPOINT](./demo/app/src/main/java/org/hyperledger/ariesvcx/Constants.kt) to reflect the public IP.

```kt
const val BASE_RELAY_ENDPOINT = <your-public-ip-endpoint>;
```

Now you are ready to start communicating. Use the QR scanner in the app to establish connection with the peer.

## Support

Currently the builds have been tested for android `arm64 (aarch64)` on a physical device. In the future we plan to support other architectures.
