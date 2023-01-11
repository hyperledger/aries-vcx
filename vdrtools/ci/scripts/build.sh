#!/bin/bash

if [ $# -ne 1 ]
  then
    echo "ERROR: Incorrect number of arguments"
    echo "Usage:"
    echo "$0 <debug|release>"
    exit 1
fi

BUILD_TYPE=$1

if [ $BUILD_TYPE == 'release' ]
  then
    CARGO_FLAGS='--release'
  else
    CARGO_FLAGS=''
fi

set -eux

pushd libvdrtools

# Due to instability of cheqd implementation feature, building without cheqd first is useful to identify if 
# the build failure is due to cheqd or not. Since most of the code does not involve cheqd, rebuilding with
# the cheqd feature is fairly quick operation meaning that these two build steps only add a little bit of extra
# overhead.
cargo build $CARGO_FLAGS --features "fatal_warnings sodium_static"
cargo build $CARGO_FLAGS --features "fatal_warnings sodium_static cheqd"

popd
