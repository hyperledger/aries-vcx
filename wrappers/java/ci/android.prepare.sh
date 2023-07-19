#!/bin/bash
set -e

SCRIPT_DIR="$( cd "$(dirname "$0")" ; pwd -P )"

source ${SCRIPT_DIR}/setup.android.env.sh

download_sdk
download_and_setup_toolchain
