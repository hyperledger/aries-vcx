#!/bin/sh

#
# A shell script to download and build libzmq for iOS
# Inspired from https://github.com/evernym/libzmq-ios
#

set -ex

PKG_VER="4.2.5"
LIBNAME="libzmq.a"
PKG_NAME="zeromq-${PKG_VER}"

SCRIPT_DIR="$( cd "$(dirname "$0")" ; pwd -P )"
OUTPUT_DIR="$1"
ARCH="$2"
LIB_OUTPUT_DIR="$3"
LIBSODIUM_BUILD_DIR="$4"

BUILD_DIR=${OUTPUT_DIR}/build
LIB_DIR=${OUTPUT_DIR}/${PKG_NAME}

if [ -z $OUTPUT_DIR ]; then
    echo "An output directory must be provided as a first argument"
    exit 1
fi

if [ -z $LIB_OUTPUT_DIR ]; then
    echo "An output path for the library symlink is required"
    exit 1
fi

if [ -z $LIBSODIUM_BUILD_DIR ]; then
    echo "The libsodium build directory is required"
    exit 1
fi

IOS_SDK_VERSION=`xcodebuild -showsdks | grep iphoneos | sed -E "s/.*iphoneos([0-9]+\.[0-9]+).*/\1/"`
IOS_VERSION_MIN="9.0"

DEVELOPER=`xcode-select -print-path`

OTHER_CXXFLAGS="-Os"
OTHER_CPPFLAGS="-Os -fembed-bitcode"

setup() {
    rm -rf ${BUILD_DIR}

    # Don't download the package again if it's already present
    if [ ! -d ${PKG_NAME} ]; then
        PKG_ARCHIVE="${PKG_NAME}.tar.gz"

        pushd ${OUTPUT_DIR}
            curl -O -L https://github.com/zeromq/libzmq/releases/download/v${PKG_VER}/${PKG_ARCHIVE}
            tar xzf ${PKG_ARCHIVE}
            rm ${PKG_ARCHIVE}
        popd
    fi
}


prep_arm64_build() {
    PLATFORM_NAME="iPhoneOS"
    HOST="arm-apple-darwin"
    export LIBSODIUM_ARCH_DIR="$LIBSODIUM_BUILD_DIR/arm64"
    export ARCH_BUILD_DIR=${BUILD_DIR}/arm64
    export BASE_DIR="${DEVELOPER}/Platforms/${PLATFORM_NAME}.platform/Developer"
    export ISDKROOT="${BASE_DIR}/SDKs/${PLATFORM_NAME}${IOS_SDK_VERSION}.sdk"
    export CXXFLAGS=$OTHER_CXXFLAGS
    export CPPFLAGS="-arch arm64 -isysroot ${ISDKROOT} -mios-version-min=${IOS_VERSION_MIN} ${OTHER_CPPFLAGS} -I${LIBSODIUM_ARCH_DIR}/include"
    export LDFLAGS="-mthumb -arch arm64 -isysroot ${ISDKROOT}"
}

prep_x86_64_build() {
    PLATFORM_NAME="iPhoneSimulator"
    HOST="x86_64-apple-darwin"
    export LIBSODIUM_ARCH_DIR="$LIBSODIUM_BUILD_DIR/x86_64"
    export ARCH_BUILD_DIR=${BUILD_DIR}/x86_64
    export BASE_DIR="${DEVELOPER}/Platforms/${PLATFORM_NAME}.platform/Developer"
    export ISDKROOT="${BASE_DIR}/SDKs/${PLATFORM_NAME}${IOS_SDK_VERSION}.sdk"
    export CXXFLAGS=$OTHER_CXXFLAGS
    export CPPFLAGS="-arch x86_64 -isysroot ${ISDKROOT} -mios-version-min=${IOS_VERSION_MIN} ${OTHER_CPPFLAGS} -I${LIBSODIUM_ARCH_DIR}/include"
    export LDFLAGS="-mthumb -arch x86_64 -isysroot ${ISDKROOT}"
}

build() {
    pushd ${LIB_DIR}
        ./configure --prefix=${ARCH_BUILD_DIR} --disable-shared --enable-static --host=${HOST} --disable-perf --disable-curve-keygen --enable-drafts=no --with-libsodium=${LIBSODIUM_ARCH_DIR}

        # Wrokaround to disable clock_gettime since it is only available on iOS 10+
        cp ${SCRIPT_DIR}/platform-patched.hpp ./src/platform.hpp

        make clean 
        make -j`sysctl -n hw.ncpu` V=0
        make install

        rm -f ${LIB_OUTPUT_DIR}/${LIBNAME}
        ln -s ${ARCH_BUILD_DIR}/lib/${LIBNAME} ${LIB_OUTPUT_DIR}/${LIBNAME}
    popd
}

setup

if [ $ARCH == "arm64" ]; then
    prep_arm64_build
elif [ $ARCH == "x86_64" ]; then
    prep_x86_64_build
else
    echo "Unknown arch provided"
    exit 1
fi

build