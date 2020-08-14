#!/bin/bash

TARGET_ARCH=$1
TARGET_API=$2
CROSS_COMPILE=$3

if [ -z "${TARGET_ARCH}" ]; then
    echo STDERR "Missing TARGET_ARCH argument"
    echo STDERR "e.g. x86 or arm"
    exit 1
fi

if [ -z "${TARGET_API}" ]; then
    echo STDERR "Missing TARGET_API argument"
    echo STDERR "e.g. 21"
    exit 1
fi

if [ -z "${CROSS_COMPILE}" ]; then
    echo STDERR "Missing CROSS_COMPILE argument"
    echo STDERR "e.g. i686-linux-android"
    exit 1
fi

if [ -z "${OPENSSL_DIR}" ]; then
    OPENSSL_DIR="openssl_${TARGET_ARCH}"
    if [ -d "${OPENSSL_DIR}" ] ; then
        echo "Found ${OPENSSL_DIR}"
    elif [ -z "$4" ]; then
        echo STDERR "Missing OPENSSL_DIR argument and environment variable"
        echo STDERR "e.g. set OPENSSL_DIR=<path> for environment or openssl_${TARGET_ARCH}"
        exit 1
    else
        OPENSSL_DIR=$4
    fi
fi
# May be potentially necessary due to https://github.com/sfackler/rust-openssl/issues/604
# OPENSSL_INCLUDE_DIR=$OPENSSL_DIR/include
# OPENSSL_LIB_DIR=$OPENSSL_DIR/lib

if [ -z "${SODIUM_DIR}" ] ; then
    SODIUM_DIR="libsodium_${TARGET_ARCH}"
    if [ -d "${SODIUM_DIR}" ] ; then
        echo "Found ${SODIUM_DIR}"
    elif [ -z "$5" ]; then
        echo STDERR "Missing SODIUM_DIR argument and environment variable"
        echo STDERR "e.g. set SODIUM_DIR=<path> for environment or libsodium_${TARGET_ARCH}"
        exit 1
    else
        SODIUM_DIR=$6
    fi
fi

if [ -z "${LIBZMQ_DIR}" ] ; then
    LIBZMQ_DIR="libzmq_${TARGET_ARCH}"
    if [ -d "${LIBZMQ_DIR}" ] ; then
        echo "Found ${LIBZMQ_DIR}"
    elif [ -z "$6" ]; then
        echo STDERR "Missing LIBZMQ_DIR argument and environment variable"
        echo STDERR "e.g. set LIBZMQ_DIR=<path> for environment or libzmq_${TARGET_ARCH}"
        exit 1
    else
        LIBZMQ_DIR=$6
    fi
fi

if [ -z "${LIBINDY_DIR}" ] ; then
    LIBINDY_DIR="libindy_${TARGET_ARCH}"
    if [ -d "${LIBINDY_DIR}" ] ; then
        echo "Found ${LIBINDY_DIR}"
    elif [ -z "$7" ] ; then
        echo STDERR "Missing LIBINDY_DIR argument and environment variable"
        echo STDERR "e.g. set LIBINDY_DIR=<path> for environment or libindy_${TARGET_ARCH}"
        exit 1
    else
        LIBINDY_DIR=$7
    fi
fi

if [ ! -f "android-ndk-r20-linux-x86_64.zip" ] ; then
    echo "Downloading android-ndk-r20-linux-x86_64.zip"
    wget -q https://dl.google.com/android/repository/android-ndk-r20-linux-x86_64.zip
else
    echo "Skipping download android-ndk-r20-linux-x86_64.zip"
fi

rm -f "libvcx/Cargo.lock"

docker build -t libvcx-android:latest -f libvcx/build_scripts/android/vcx/Dockerfile . --build-arg target_arch=${TARGET_ARCH} --build-arg target_api=${TARGET_API} --build-arg cross_compile=${CROSS_COMPILE} --build-arg openssl_dir=${OPENSSL_DIR} --build-arg sodium_dir=${SODIUM_DIR} --build-arg libzmq_dir=${LIBZMQ_DIR} --build-arg libindy_dir=${LIBINDY_DIR} &&
docker run libvcx-android:latest && \
docker_id=$(docker ps -a | grep libvcx-android:latest | grep Exited | tail -n 1 | cut -d ' ' -f 1) && \
docker_image_id=$(docker image ls | grep libvcx-android | perl -pe 's/\s+/ /g' | cut -d ' ' -f 3) && \
docker cp ${docker_id}:/home/vcx_user/libvcx.so . && \
docker cp ${docker_id}:/home/vcx_user/libvcx.a . && \
docker rm ${docker_id} > /dev/null && \
docker rmi ${docker_image_id} > /dev/null
