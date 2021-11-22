# Building LibVCX

### 1. Build dependencies
In order to build and call LibVCX code, you need to first build [VdrTools](https://gitlab.com/evernym/verity/vdr-tools).

## 2. Build LibVCX
Lets build LibVCX itself now. Enter into [libvcx directory](../libvcx) of this repo and run
```
cargo build --release
```
The build libraries will be located relatively to build directory in `./target/release`. On OSX, move `.dylib` library
into `/usr/local/lib`. On linux, move them to `/usr/lib`. 

## 3. Run some code
Now you are ready to write code consuming LibVCX API. Pick your language from [list of demos](https://github.com/AbsaOSS/libvcx#get-started)
and follow its instructions.

