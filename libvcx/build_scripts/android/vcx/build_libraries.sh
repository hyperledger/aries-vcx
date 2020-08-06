#!/bin/bash

REPO_DIR=$(cd "$(dirname "$0")" ; pwd -P)
OUTDIR=prebuilt_deps
LIB_DIR=${OUTDIR}/libs
VCX_DIR=${OUTDIR}/vcx

LIBINDY_VER=${1:-1.15.0}
LIBVCX_VER=${2:-0.8.0}

mkdir -p ${OUTDIR}/libs
mkdir -p ${VCX_DIR}/libvcx_x86
mkdir -p ${VCX_DIR}/libvcx_arm64
mkdir -p ${VCX_DIR}/libvcx_armv7

download_prebuilt_libs(){
    pushd ${LIB_DIR}
    wget -nc https://repo.sovrin.org/android/libindy/deps-libc%2B%2B/openssl/openssl_x86.zip
    unzip -u openssl_x86.zip

    wget -nc https://repo.sovrin.org/android/libindy/deps-libc%2B%2B/openssl/openssl_arm64.zip
    unzip -u openssl_arm64.zip

    wget -nc https://repo.sovrin.org/android/libindy/deps-libc%2B%2B/openssl/openssl_armv7.zip
    unzip -u openssl_armv7.zip

    wget -nc https://repo.sovrin.org/android/libindy/deps-libc%2B%2B/sodium/libsodium_x86.zip
    unzip -u libsodium_x86.zip

    wget -nc https://repo.sovrin.org/android/libindy/deps-libc%2B%2B/sodium/libsodium_arm64.zip
    unzip -u libsodium_arm64.zip

    wget -nc https://repo.sovrin.org/android/libindy/deps-libc%2B%2B/sodium/libsodium_armv7.zip
    unzip -u libsodium_armv7.zip

    wget -nc https://repo.sovrin.org/android/libindy/deps-libc%2B%2B/zmq/libzmq_x86.zip
    unzip -u libzmq_x86.zip

    wget -nc https://repo.sovrin.org/android/libindy/deps-libc%2B%2B/zmq/libzmq_arm64.zip
    unzip -u libzmq_arm64.zip

    wget -nc https://repo.sovrin.org/android/libindy/deps-libc%2B%2B/zmq/libzmq_armv7.zip
    unzip -u libzmq_armv7.zip

    wget -nc "https://repo.sovrin.org/android/libindy/stable/${LIBINDY_VER}/libindy_android_x86_${LIBINDY_VER}.zip"
    unzip -u libindy_android_x86_${LIBINDY_VER}.zip

    wget -nc "https://repo.sovrin.org/android/libindy/stable/${LIBINDY_VER}/libindy_android_arm64_${LIBINDY_VER}.zip"
    unzip -u libindy_android_arm64_${LIBINDY_VER}.zip

    wget -nc "https://repo.sovrin.org/android/libindy/stable/${LIBINDY_VER}/libindy_android_armv7_${LIBINDY_VER}.zip"
    unzip -u libindy_android_armv7_${LIBINDY_VER}.zip
    popd
}

deploy_library(){
    ARCH=$1
    mv libvcx.so ${vcxdir}/libvcx_${ARCH}
    pushd ${VCX_DIR}
    zip -r libvcx_${ARCH}_${LIBVCX_VER}.zip libvcx_${ARCH}
    #Place your deployment script below
    #curl -v --user 'id:pw' --upload-file ./libvcx_${ARCH}_${LIBVCX_VER}.zip http://13.125.219.189/repository/libraries/android/libvcx_${ARCH}_${LIBVCX_VER}.zip
    popd
}

download_prebuilt_libs

$REPO_DIR/libvcx/build_scripts/android/vcx/build.sh x86 24 i686-linux-android $LIB_DIR/openssl_x86 $LIB_DIR/libsodium_x86 $LIB_DIR/libzmq_x86 $LIB_DIR/libindy_x86/lib
deploy_library x86

$REPO_DIR/libvcx/build_scripts/android/vcx/build.sh arm 24 arm-linux-androideabi $LIB_DIR/openssl_armv7 $LIB_DIR/libsodium_armv7 $LIB_DIR/libzmq_armv7 $LIB_DIR/libindy_armv7/lib
deploy_library armv7

$REPO_DIR/libvcx/build_scripts/android/vcx/build.sh arm64 24 aarch64-linux-android $LIB_DIR/openssl_arm64 $LIB_DIR/libsodium_arm64 $LIB_DIR/libzmq_arm64 $LIB_DIR/libindy_arm64/lib
deploy_library arm64
