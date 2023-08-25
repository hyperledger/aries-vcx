#!/bin/bash
set -ex

SCRIPT_DIR="$( cd "$(dirname "$0")" ; pwd -P )"

# Required env vars
ANDROID_BUILD_DIR=~/android_build

source ${SCRIPT_DIR}/android.utils.sh

create_standalone_toolchain_and_rust_target "arm64"
create_cargo_config
