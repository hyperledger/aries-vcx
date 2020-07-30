#!/bin/bash

workdir=output
libdir=${workdir}/libs
vcxdir=${workdir}/vcx

libindy_version=1.15.0
libvcx_version=0.8.2

mkdir -p ${workdir}/libs
mkdir -p ${vcxdir}/libvcx_x86
mkdir -p ${vcxdir}/libvcx_arm64
mkdir -p ${vcxdir}/libvcx_armv7

chmod +x build.sh

download_prebuilt_libs(){
    pushd ${libdir}
    wget https://repo.sovrin.org/android/libindy/deps-libc%2B%2B/openssl/openssl_x86.zip
    unzip openssl_x86.zip

    wget https://repo.sovrin.org/android/libindy/deps-libc%2B%2B/openssl/openssl_arm64.zip
    unzip openssl_arm64.zip

    wget https://repo.sovrin.org/android/libindy/deps-libc%2B%2B/openssl/openssl_armv7.zip
    unzip openssl_armv7.zip

    wget https://repo.sovrin.org/android/libindy/deps-libc%2B%2B/sodium/libsodium_x86.zip
    unzip libsodium_x86.zip

    wget https://repo.sovrin.org/android/libindy/deps-libc%2B%2B/sodium/libsodium_arm64.zip
    unzip libsodium_arm64.zip

    wget https://repo.sovrin.org/android/libindy/deps-libc%2B%2B/sodium/libsodium_armv7.zip
    unzip libsodium_armv7.zip

    wget https://repo.sovrin.org/android/libindy/deps-libc%2B%2B/zmq/libzmq_x86.zip
    unzip libzmq_x86.zip

    wget https://repo.sovrin.org/android/libindy/deps-libc%2B%2B/zmq/libzmq_arm64.zip
    unzip libzmq_arm64.zip

    wget https://repo.sovrin.org/android/libindy/deps-libc%2B%2B/zmq/libzmq_armv7.zip
    unzip libzmq_armv7.zip

    wget "https://repo.sovrin.org/android/libindy/stable/${libindy_version}/libindy_android_x86_${libindy_version}.zip"
    unzip libindy_android_x86_${libindy_version}.zip

    wget "https://repo.sovrin.org/android/libindy/stable/${libindy_version}/libindy_android_arm64_${libindy_version}.zip"
    unzip libindy_android_arm64_${libindy_version}.zip

    wget "https://repo.sovrin.org/android/libindy/stable/${libindy_version}/libindy_android_armv7_${libindy_version}.zip"
    unzip libindy_android_armv7_${libindy_version}.zip
    popd
}

deploy_library(){
    arch=$1
    cp libvcx.so ${vcxdir}/libvcx_${arch}
    pushd ${vcxdir}
    zip -r libvcx_${arch}_${libvcx_version}.zip libvcx_${arch}
    #Place your deployment script below
    #curl -v --user 'id:pw' --upload-file ./libvcx_${arch}_${libvcx_version}.zip http://13.125.219.189/repository/libraries/android/libvcx_${arch}_${libvcx_version}.zip
    popd
}

download_prebuilt_libs

./build.sh x86 21 i686-linux-android skt-develop ./output/libs/openssl_x86 ./output/libs/libsodium_x86 ./output/libs/libzmq_x86 ./output/libs/libindy_x86/lib
deploy_library x86

./build.sh arm 21 arm-linux-androideabi skt-develop ./output/libs/openssl_armv7 ./output/libs/libsodium_armv7 ./output/libs/libzmq_armv7 ./output/libs/libindy_armv7/lib
deploy_library armv7

./build.sh arm64 21 aarch64-linux-android skt-develop ./output/libs/openssl_arm64 ./output/libs/libsodium_arm64 ./output/libs/libzmq_arm64 ./output/libs/libindy_arm64/lib
deploy_library arm64
