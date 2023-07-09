#!/bin/bash
set -ex

SCRIPT_DIR="$( cd "$(dirname "$0")" ; pwd -P )"

# Required env vars
ARIES_VCX_ROOT=$(dirname $(dirname $SCRIPT_DIR))
ANDROID_BUILD_DIR=~/android_build
BUILD_MODE="debug"
LANGUAGE="kotlin"

source ${SCRIPT_DIR}/android.utils.sh

# set_android_env
# download_and_setup_toolchain
generate_arch_flags "arm"
set_dependencies_env_vars
# set_android_arch_env
build_uniffi
