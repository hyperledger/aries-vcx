#!/bin/bash
set -e

echo "android.prepare.sh >> starting"

SCRIPT_DIR="$( cd "$(dirname "$0")" ; pwd -P )"
source ${SCRIPT_DIR}/setup.android.env.sh

echo "android.prepare.sh >> download_sdk"
download_sdk
echo "android.prepare.sh >> download_and_setup_toolchain"
download_and_setup_toolchain
echo "android.prepare.sh >> download_emulator"
download_emulator
echo "android.prepare.sh >> prepare_dependencies"
prepare_dependencies "arm"

echo "android.prepare.sh >> finished"