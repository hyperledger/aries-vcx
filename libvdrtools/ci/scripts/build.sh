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
# Build without cheqd feature enabled first
cargo build $CARGO_FLAGS --features "fatal_warnings sodium_static"
cargo build $CARGO_FLAGS --features "fatal_warnings sodium_static cheqd"
popd
