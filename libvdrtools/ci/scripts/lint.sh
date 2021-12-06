#!/bin/bash

set -eux

pushd libindy
cargo clippy -- -W clippy::style -W clippy::correctness -W clippy::complexity -W clippy::perf
popd
