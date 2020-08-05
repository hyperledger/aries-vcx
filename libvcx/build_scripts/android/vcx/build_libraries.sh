#!/bin/bash

workdir=output
libdir=${workdir}/libs
vcxdir=${workdir}/vcx

libindy_version=${1:-1.15.0}
libvcx_version=${2:-0.8.0}

mkdir -p ${workdir}/libs
mkdir -p ${vcxdir}/libvcx_x86
mkdir -p ${vcxdir}/libvcx_arm64
mkdir -p ${vcxdir}/libvcx_armv7

download_prebuilt_libs(){
    pushd ${libdir}
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

    wget -nc "https://repo.sovrin.org/android/libindy/stable/${libindy_version}/libindy_android_x86_${libindy_version}.zip"
    unzip -u libindy_android_x86_${libindy_version}.zip

    wget -nc "https://repo.sovrin.org/android/libindy/stable/${libindy_version}/libindy_android_arm64_${libindy_version}.zip"
    unzip -u libindy_android_arm64_${libindy_version}.zip

    wget -nc "https://repo.sovrin.org/android/libindy/stable/${libindy_version}/libindy_android_armv7_${libindy_version}.zip"
    unzip -u libindy_android_armv7_${libindy_version}.zip
    popd
}

deploy_library(){
    arch=$1
    mv libvcx.so ${vcxdir}/libvcx_${arch}
    pushd ${vcxdir}
    zip -r libvcx_${arch}_${libvcx_version}.zip libvcx_${arch}
    #Place your deployment script below
    #curl -v --user 'id:pw' --upload-file ./libvcx_${arch}_${libvcx_version}.zip http://13.125.219.189/repository/libraries/android/libvcx_${arch}_${libvcx_version}.zip
    popd
}

download_prebuilt_libs

./libvcx/build_scripts/android/vcx/build.sh x86 21 i686-linux-android $workdir/libs/openssl_x86 $workdir/libs/libsodium_x86 $workdir/libs/libzmq_x86 $workdir/libs/libindy_x86/lib
deploy_library x86

./libvcx/build_scripts/android/vcx/build.sh arm 21 arm-linux-androideabi $workdir/libs/openssl_armv7 $workdir/libs/libsodium_armv7 $workdir/libs/libzmq_armv7 $workdir/libs/libindy_armv7/lib
deploy_library armv7

./libvcx/build_scripts/android/vcx/build.sh arm64 21 aarch64-linux-android $workdir/libs/openssl_arm64 $workdir/libs/libsodium_arm64 $workdir/libs/libzmq_arm64 $workdir/libs/libindy_arm64/lib
deploy_library arm64
