#!/bin/bash
set -e

SCRIPT_DIR="$( cd "$(dirname "$0")" ; pwd -P )"

source ${SCRIPT_DIR}/setup.android.env.sh

archs=("arm" "armv7" "x86" "arm64" "x86_64")

download_sdk
download_and_setup_toolchain
download_emulator

for arch in "${archs[@]}"
do
  prepare_dependencies "${arch}"
done
