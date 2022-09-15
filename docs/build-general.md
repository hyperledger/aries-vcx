# Building libvcx

## 1. Build libvcx
Lets build libvcx itself now. Enter into [libvcx directory](../libvcx) of this repo and run
```
cargo build --release
```
The build libraries will be located relatively to build directory in `./target/release`. On OSX, move `.dylib` library
into `/usr/local/lib`. On linux, move them to `/usr/lib`.

## 2. Run some code
Now you are ready to write code consuming libvcx API. Pick your language from [list of demos](https://github.com/AbsaOSS/libvcx#get-started)
and follow its instructions.
