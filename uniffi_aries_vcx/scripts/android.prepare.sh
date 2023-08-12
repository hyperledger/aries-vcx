#!/bin/bash
set -ex

SCRIPT_DIR="$( cd "$(dirname "$0")" ; pwd -P )"

# Required env vars
ANDROID_BUILD_DIR=~/android_build

source ${SCRIPT_DIR}/android.utils.sh

set_android_env
download_sdk
prepare_dependencies "arm"
download_and_setup_toolchain
