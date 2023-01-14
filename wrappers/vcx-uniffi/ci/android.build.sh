#!/usr/bin/env bash
# should be run from root workspace!

set -ex

ANDROID_BUILD_FOLDER=/tmp/android_build
mkdir -p $ANDROID_BUILD_FOLDER

SCRIPT_DIR="$( cd "$(dirname "$0")" ; pwd -P )"
VCX_DIR=$PWD

source ${SCRIPT_DIR}/setup.android.env.sh

# arm64
prepare_dependencies arm64
generate_arch_flags arm64
setup_dependencies_env_vars arm64
set_env_vars

build_vcx_uniffi ${VCX_DIR}