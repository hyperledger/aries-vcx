#!/bin/bash

if [ $# -ne 2 ]
  then
    echo "ERROR: Incorrect number of arguments"
    echo "Usage:"
    echo "$0 <debug|release> <test-pool-ip>"
    exit 1
fi

BUILD_TYPE=$1
export TEST_POOL_IP=$2

if [ $BUILD_TYPE == 'release' ]
  then
    CARGO_FLAGS='--release'
  else
    CARGO_FLAGS=''
fi

function test() {
  MODULE_DIR=$1
  shift
  FEATURE_FLAGS=$*

  pushd $MODULE_DIR
  RUST_BACKTRACE=1 cargo test --no-run ${CARGO_FLAGS} ${FEATURE_FLAGS}
  RUST_BACKTRACE=1 RUST_LOG=indy::=debug,zmq=trace RUST_TEST_THREADS=1 cargo test ${CARGO_FLAGS} ${FEATURE_FLAGS}
  popd
}

set -eux

test libvdrtools --features sodium_static,only_high_cases,cheqd,fatal_warnings
test libvdrtools/indy-api-types
test libvdrtools/indy-utils
test libvdrtools/indy-wallet
