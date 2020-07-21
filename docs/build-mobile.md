# Building LibVCX on mobile
For general overview, see https://dev.to/jakubkoci/how-to-build-a-vcx-ios-library-f32

### TODO: Revise instruction below

### Android
1. Go to `https://repo.sovrin.org/android/libvcx/{release-channel}`.
2. Download the latest version of libvcx.
3. Unzip archives to the directory where you want to save working library.
4. After unzip you will get next structure of files:

* `Your working directory`
    * `include` - contains c-header files which contains all necessary declarations that may be need for your applications.
        * `...`
    * `lib` - contains library binaries (static and dynamic).
        * `libvcx.a`
        * `libvcx.so`
    
Copy `libvcx.so` file to the jniLibs/{abi} folder of your android project
    
{release channel} must be replaced with master, rc or stable to define corresponded release channel.

## How to build VCX from source

### Linux 
1) Install rust and rustup (https://www.rust-lang.org/install.html). 
2) [Install Libindy](../README.md#installing-the-sdk) 
3) Optionally [install Libnullpay](../libnullpay/README.md) to include payment functionality.
3) Clone this repo to your local machine. 
4) From the indy-sdk/vcx/libvcx folder inside this local repository run the following commands to verify everything works: 
    ``` 
    $ cargo build 
    $ cargo test 
    ``` 
5) Currently developers are using intellij for IDE development (https://www.jetbrains.com/idea/download/) with the rust plugin (https://plugins.jetbrains.com/plugin/8182-rust). 

### Android
1) Install rust and rustup (https://www.rust-lang.org/install.html).
2) Clone this repo to your local machine.
3) [Install Libindy](../README.md#installing-the-sdk) 
4) Run `install_toolchains.sh`. You need to run this once to setup toolchains for android
5) Run `android.build.sh aarm64` to build libvcx for aarm64 architecture.(Other architerctures will follow soon)
6) Tests are not working on Android as of now.