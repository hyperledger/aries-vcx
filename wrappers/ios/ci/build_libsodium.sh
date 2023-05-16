#!/bin/sh

#
# A shell script to download and build libsodium for iOS
# Inspired from https://github.com/evernym/libsodium-ios
#

set -ex

PKG_VER="1.0.14"
LIBNAME="libsodium.a"
PKG_NAME="libsodium-${PKG_VER}"

OUTPUT_DIR="$1"
ARCH="$2"
LIB_OUTPUT_DIR="$3"

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

IOS_SDK_VERSION=`xcodebuild -showsdks | grep iphoneos | sed -E "s/.*iphoneos([0-9]+\.[0-9]+).*/\1/"`
IOS_VERSION_MIN="9.0"

DEVELOPER=`xcode-select -print-path`

OTHER_CFLAGS="-Os -Qunused-arguments -fembed-bitcode"

setup() {
    rm -rf ${BUILD_DIR}

    # Don't download the package again if it's already present
    if [ ! -d ${PKG_NAME} ]; then
        PKG_ARCHIVE="${PKG_NAME}.tar.gz"

        pushd ${OUTPUT_DIR}
            curl -O -L https://github.com/jedisct1/libsodium/releases/download/${PKG_VER}/${PKG_ARCHIVE}
            tar xzf ${PKG_ARCHIVE}
            rm ${PKG_ARCHIVE}
        popd
    fi
}

prep_arm64_build() {
    PLATFORM_NAME="iPhoneOS"
    HOST="arm-apple-darwin"
    export ARCH_BUILD_DIR=${BUILD_DIR}/arm64
    export BASE_DIR="${DEVELOPER}/Platforms/${PLATFORM_NAME}.platform/Developer"
    export ISDKROOT="${BASE_DIR}/SDKs/${PLATFORM_NAME}${IOS_SDK_VERSION}.sdk"
    export CFLAGS="-arch arm64 -isysroot ${ISDKROOT} -mios-version-min=${IOS_VERSION_MIN} ${OTHER_CFLAGS}"
    export LDFLAGS="-mthumb -arch arm64 -isysroot ${ISDKROOT}"
}

prep_x86_64_build() {
    PLATFORM_NAME="iPhoneSimulator"
    HOST="x86_64-apple-darwin"
    export ARCH_BUILD_DIR=${BUILD_DIR}/x86_64
    export BASE_DIR="${DEVELOPER}/Platforms/${PLATFORM_NAME}.platform/Developer"
    export ISDKROOT="${BASE_DIR}/SDKs/${PLATFORM_NAME}${IOS_SDK_VERSION}.sdk"
    export CFLAGS="-arch x86_64 -isysroot ${ISDKROOT} -mios-version-min=${IOS_VERSION_MIN} ${OTHER_CFLAGS}"
    export LDFLAGS="-mthumb -arch x86_64 -isysroot ${ISDKROOT}"
}

build() {
    pushd ${LIB_DIR}
        ./configure --prefix=${ARCH_BUILD_DIR} --disable-shared --enable-static --host=${HOST}

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
