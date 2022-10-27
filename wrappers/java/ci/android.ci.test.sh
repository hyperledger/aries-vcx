#!/bin/bash
set -e

SCRIPT_DIR="$( cd "$(dirname "$0")" ; pwd -P )"

export LIBVCX_VER=0.8.0
export ANDROID_BUILD_FOLDER=/tmp/android_build
export ANDROID_SDK=${ANDROID_BUILD_FOLDER}/sdk
export ANDROID_SDK_ROOT=${ANDROID_SDK}
export ANDROID_HOME=${ANDROID_SDK}
export TOOLCHAIN_PREFIX=${ANDROID_BUILD_FOLDER}/toolchains/linux
export ANDROID_NDK_ROOT=${TOOLCHAIN_PREFIX}/android-ndk-r20
export PATH=${PATH}:${ANDROID_HOME}/platform-tools:${ANDROID_HOME}/tools:${ANDROID_HOME}/tools/bin
export LIBVCX_VERSION=$LIBVCX_VER
export JAVA_HOME=/usr/lib/jvm/adoptopenjdk-8-hotspot-amd64

ls -lah "/usr/lib/jvm/"
ls -lah "$JAVA_HOME"

echo "android.ci.test.sh >> calling android.prepare.sh"
source ${SCRIPT_DIR}/android.prepare.sh
echo "android.ci.test.sh >> android.test.sh armv7"
source ${SCRIPT_DIR}/android.test.sh armv7

echo "android.ci.test.sh >> finished"